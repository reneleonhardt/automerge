use std::env;

use duct::cmd;

// #[test]
// fn import_stdin() {
//     let bin = env!("CARGO_BIN_EXE_automerge");
//     let initial_state_json = serde_json::json!({
//         "birds": {
//             "wrens": 3.0,
//             "sparrows": 15.0
//         }
//     });
//     let json_bytes = serde_json::to_string_pretty(&initial_state_json).unwrap();

//     let no_pipe_no_file = cmd!(bin, "import").stdin_bytes(json_bytes.clone()).run();

//     assert!(no_pipe_no_file.is_err());

//     let pipe_no_file = cmd!(bin, "import")
//         .stdin_bytes(json_bytes.clone())
//         .stdout_capture()
//         .run();

//     assert!(pipe_no_file.is_ok());

//     let mut temp_file = std::env::temp_dir();
//     temp_file.push("import_test.mpl");
//     let no_pipe_file = cmd!(bin, "import", "--out", &temp_file)
//         .stdin_bytes(json_bytes)
//         .run();

//     assert!(no_pipe_file.is_ok());
//     std::fs::remove_file(temp_file).unwrap();
// }

// #[test]
// fn export_stdout() {
//     let bin = env!("CARGO_BIN_EXE_automerge");
//     let no_pipe_no_file = cmd!(bin, "export").stdout_capture().run();

//     assert!(no_pipe_no_file.is_err());
// }

#[test]
fn import_export_isomorphic() {
    let bin = env!("CARGO_BIN_EXE_automerge");
    let initial_state_json = serde_json::json!({
        "birds": {
            "wrens": 3.0,
            "sparrows": 15.0
        }
    });
    let json_bytes = serde_json::to_string_pretty(&initial_state_json).unwrap();

    let stdout = cmd!(bin, "import")
        .stdin_bytes(json_bytes.clone())
        .pipe(cmd!(bin, "export"))
        .read()
        .unwrap();
    assert_eq!(stdout, json_bytes);
}

#[test]
fn import_and_export_reject_same_path_without_truncating_input() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, ROOT};

    let bin = env!("CARGO_BIN_EXE_automerge");
    let base = env::temp_dir().join(format!("automerge-cli-same-path-{}", std::process::id()));
    std::fs::create_dir(&base).unwrap();

    let mut source = AutoCommit::new();
    source.put(ROOT, "key", "value").unwrap();
    let automerge_path = base.join("document.automerge");
    std::fs::write(&automerge_path, source.save()).unwrap();
    let export_result = cmd!(bin, "export", &automerge_path, "--out", &automerge_path)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();
    assert!(!export_result.status.success());
    assert!(!export_result.stderr.is_empty());
    assert!(automerge::Automerge::load(&std::fs::read(&automerge_path).unwrap()).is_ok());

    #[cfg(unix)]
    {
        let hard_link = base.join("document-hard-link.automerge");
        std::fs::hard_link(&automerge_path, &hard_link).unwrap();
        let hard_link_result = cmd!(bin, "export", &automerge_path, "--out", &hard_link)
            .stderr_capture()
            .unchecked()
            .run()
            .unwrap();
        assert!(!hard_link_result.status.success());
        assert!(automerge::Automerge::load(&std::fs::read(&automerge_path).unwrap()).is_ok());
    }

    let json_path = base.join("document.json");
    let json = r#"{"key":"value"}"#;
    std::fs::write(&json_path, json).unwrap();
    let import_result = cmd!(bin, "import", &json_path, "--out", &json_path)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();
    assert!(!import_result.status.success());
    assert!(!import_result.stderr.is_empty());
    assert_eq!(std::fs::read_to_string(&json_path).unwrap(), json);

    std::fs::remove_dir_all(&base).unwrap();
}

#[test]
fn merge_failure_returns_nonzero_status() {
    let bin = env!("CARGO_BIN_EXE_automerge");
    let base = env::temp_dir().join(format!("automerge-cli-merge-error-{}", std::process::id()));
    std::fs::create_dir(&base).unwrap();
    let input = base.join("invalid.automerge");
    std::fs::write(&input, b"not an automerge document").unwrap();

    let output = cmd!(bin, "merge", &input)
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());

    std::fs::remove_dir_all(&base).unwrap();
}

#[test]
fn anonymize_scrubs_a_document_from_stdin() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, Automerge, ReadDoc, ROOT};

    let bin = env!("CARGO_BIN_EXE_automerge");
    let mut source = AutoCommit::new();
    source.put(ROOT, "private-key", "private value").unwrap();
    let source_bytes = source.save();

    let output = cmd!(bin, "anonymize")
        .stdin_bytes(source_bytes.clone())
        .stdout_capture()
        .stderr_capture()
        .run()
        .unwrap();
    let anonymized = Automerge::load(&output.stdout).unwrap();

    assert_eq!(anonymized.get_changes(&[]).len(), 1);
    assert!(anonymized.get(ROOT, "private-key").unwrap().is_none());
    assert_ne!(output.stdout, source_bytes);
}

#[cfg(unix)]
#[test]
fn merge_skips_an_unchanged_read_only_output() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, ROOT};
    use std::os::unix::fs::PermissionsExt;

    let bin = env!("CARGO_BIN_EXE_automerge");
    let mut source = AutoCommit::new();
    source.put(ROOT, "key", "value").unwrap();
    let source_bytes = source.save();

    let base = env::temp_dir().join(format!("automerge-cli-merge-noop-{}", std::process::id()));
    std::fs::create_dir(&base).unwrap();
    let input = base.join("input.automerge");
    let output = base.join("output.automerge");
    std::fs::write(&input, &source_bytes).unwrap();
    std::fs::write(&output, &source_bytes).unwrap();
    std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o444)).unwrap();

    let result = cmd!(bin, "merge", &input, "--out", &output).run();

    std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::remove_dir_all(&base).unwrap();
    assert!(result.is_ok(), "merge failed: {result:?}");
}

#[test]
fn merge_writes_missing_or_changed_output() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, ROOT};

    let bin = env!("CARGO_BIN_EXE_automerge");
    let mut source = AutoCommit::new();
    source.put(ROOT, "key", "value").unwrap();
    let source_bytes = source.save();

    let base = env::temp_dir().join(format!("automerge-cli-merge-write-{}", std::process::id()));
    std::fs::create_dir(&base).unwrap();
    let input = base.join("input.automerge");
    let output = base.join("output.automerge");
    std::fs::write(&input, &source_bytes).unwrap();

    cmd!(bin, "merge", &input, "--out", &output).run().unwrap();
    assert_eq!(std::fs::read(&output).unwrap(), source_bytes);

    std::fs::write(&output, b"different").unwrap();
    cmd!(bin, "merge", &input, "--out", &output).run().unwrap();
    assert_eq!(std::fs::read(&output).unwrap(), source_bytes);
    std::fs::remove_dir_all(&base).unwrap();
}

#[test]
fn merge_writes_to_stdout() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, ROOT};

    let bin = env!("CARGO_BIN_EXE_automerge");
    let mut source = AutoCommit::new();
    source.put(ROOT, "key", "value").unwrap();
    let source_bytes = source.save();

    let output = cmd!(bin, "merge")
        .stdin_bytes(source_bytes.clone())
        .stdout_capture()
        .stderr_capture()
        .run()
        .unwrap();
    assert_eq!(output.stdout, source_bytes);
    assert!(output.stderr.is_empty());
}

#[test]
fn copy_preserves_exact_document_bytes() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, ROOT};

    let bin = env!("CARGO_BIN_EXE_automerge");
    let mut source = AutoCommit::new();
    for index in 0..100 {
        source
            .put(ROOT, format!("key-{index}"), "value-value-value")
            .unwrap();
    }
    let source_bytes = source.save_nocompress();
    assert_ne!(source_bytes, source.save());

    let base = env::temp_dir().join(format!("automerge-cli-copy-{}", std::process::id()));
    std::fs::create_dir(&base).unwrap();
    let input = base.join("input.automerge");
    let output = base.join("output.automerge");
    std::fs::write(&input, &source_bytes).unwrap();

    cmd!(bin, "copy", &input, "--out", &output).run().unwrap();
    assert_eq!(std::fs::read(&output).unwrap(), source_bytes);
    std::fs::remove_dir_all(&base).unwrap();
}

#[cfg(unix)]
#[test]
fn copy_skips_an_unchanged_read_only_output() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, ROOT};
    use std::os::unix::fs::PermissionsExt;

    let bin = env!("CARGO_BIN_EXE_automerge");
    let mut source = AutoCommit::new();
    source.put(ROOT, "key", "value").unwrap();
    let source_bytes = source.save();

    let base = env::temp_dir().join(format!("automerge-cli-copy-noop-{}", std::process::id()));
    std::fs::create_dir(&base).unwrap();
    let input = base.join("input.automerge");
    let output = base.join("output.automerge");
    std::fs::write(&input, &source_bytes).unwrap();
    std::fs::write(&output, &source_bytes).unwrap();
    std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o444)).unwrap();

    let result = cmd!(bin, "copy", &input, "--out", &output).run();

    std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::remove_dir_all(&base).unwrap();
    assert!(result.is_ok(), "copy failed: {result:?}");
}

#[test]
fn copy_rejects_invalid_input_before_writing_output() {
    let bin = env!("CARGO_BIN_EXE_automerge");
    let base = env::temp_dir().join(format!("automerge-cli-copy-invalid-{}", std::process::id()));
    std::fs::create_dir(&base).unwrap();
    let input = base.join("input.automerge");
    let output = base.join("output.automerge");
    std::fs::write(&input, b"not an automerge document").unwrap();
    std::fs::write(&output, b"keep this output").unwrap();

    let result = cmd!(bin, "copy", &input, "--out", &output)
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();

    assert!(
        !result.status.success(),
        "invalid input unexpectedly copied"
    );
    assert!(result.stdout.is_empty());
    assert!(!result.stderr.is_empty());
    assert_eq!(std::fs::read(&output).unwrap(), b"keep this output");
    std::fs::remove_dir_all(&base).unwrap();
}

#[test]
fn copy_rejects_incremental_input() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, ROOT};

    let bin = env!("CARGO_BIN_EXE_automerge");
    let mut source = AutoCommit::new();
    source.put(ROOT, "key", "value").unwrap();
    let heads = source.get_heads();
    source.put(ROOT, "key", "updated").unwrap();
    let incremental = source.save_after(&heads);

    let result = cmd!(bin, "copy")
        .stdin_bytes(incremental)
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run();
    let output = result.unwrap();
    assert!(
        !output.status.success(),
        "incremental input unexpectedly copied"
    );
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn copy_preserves_stdin_bytes_on_stdout() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, ROOT};

    let bin = env!("CARGO_BIN_EXE_automerge");
    let mut source = AutoCommit::new();
    source.put(ROOT, "key", "value").unwrap();
    let source_bytes = source.save();

    let output = cmd!(bin, "copy")
        .stdin_bytes(source_bytes.clone())
        .stdout_capture()
        .stderr_capture()
        .run()
        .unwrap();
    assert_eq!(output.stdout, source_bytes);
    assert!(output.stderr.is_empty());
}

#[test]
fn copy_preserves_bytes_through_a_pipe() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, ROOT};

    let bin = env!("CARGO_BIN_EXE_automerge");
    let mut source = AutoCommit::new();
    source.put(ROOT, "key", "value").unwrap();
    let source_bytes = source.save();

    let output = cmd!(bin, "copy")
        .stdin_bytes(source_bytes.clone())
        .pipe(cmd!(bin, "copy"))
        .stdout_capture()
        .stderr_capture()
        .run()
        .unwrap();
    assert_eq!(output.stdout, source_bytes);
    assert!(output.stderr.is_empty());
}

#[test]
fn copy_accepts_empty_stdin_without_diagnostics() {
    let bin = env!("CARGO_BIN_EXE_automerge");

    let output = cmd!(bin, "copy")
        .stdin_bytes(Vec::<u8>::new())
        .stdout_capture()
        .stderr_capture()
        .run()
        .unwrap();
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/*
#[test]
fn import_change_export() {
    let bin = env!("CARGO_BIN_EXE_automerge");
    let initial_state_json = serde_json::json!({
        "birds": {
            "wrens": 3.0,
            "sparrows": 15.0
        }
    });
    let json_bytes = serde_json::to_string_pretty(&initial_state_json).unwrap();

    let stdout = cmd!(bin, "import")
        .stdin_bytes(json_bytes.clone())
        .pipe(cmd!(bin, "change", "set $[\"birds\"][\"owls\"] 12.0"))
        .stdin_bytes(json_bytes)
        .pipe(cmd!(bin, "export"))
        .read()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(stdout.as_str()).unwrap();
    let expected = serde_json::json!({
        "birds": {
            "wrens": 3.0,
            "sparrows": 15.0,
            "owls": 12.0,
        }
    });
    assert_eq!(result, expected);
}
*/
