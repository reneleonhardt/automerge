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

    let lexical_alias = base.join("nested").join("..").join("document.automerge");
    std::fs::create_dir(base.join("nested")).unwrap();
    let lexical_alias_result = cmd!(bin, "export", &automerge_path, "--out", &lexical_alias)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();
    assert!(!lexical_alias_result.status.success());
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
fn failed_file_outputs_preserve_existing_bytes() {
    let bin = env!("CARGO_BIN_EXE_automerge");
    let base = env::temp_dir().join(format!(
        "automerge-cli-atomic-failure-{}",
        std::process::id()
    ));
    std::fs::create_dir(&base).unwrap();

    let invalid_json = base.join("invalid.json");
    let import_output = base.join("import.automerge");
    std::fs::write(&invalid_json, b"not json").unwrap();
    std::fs::write(&import_output, b"keep import output").unwrap();
    let import_result = cmd!(bin, "import", &invalid_json, "--out", &import_output)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();
    assert!(!import_result.status.success());
    assert_eq!(
        std::fs::read(&import_output).unwrap(),
        b"keep import output"
    );
    assert!(!std::fs::read_dir(&base)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .any(|name| name
            .to_string_lossy()
            .starts_with(".import.automerge.automerge-")));

    let invalid_document = base.join("invalid.automerge");
    let export_output = base.join("export.json");
    std::fs::write(&invalid_document, b"not an automerge document").unwrap();
    std::fs::write(&export_output, b"keep export output").unwrap();
    let export_result = cmd!(bin, "export", &invalid_document, "--out", &export_output)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();
    assert!(!export_result.status.success());
    assert_eq!(
        std::fs::read(&export_output).unwrap(),
        b"keep export output"
    );
    assert!(!std::fs::read_dir(&base)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .any(|name| name
            .to_string_lossy()
            .starts_with(".export.json.automerge-")));

    std::fs::remove_dir_all(&base).unwrap();
}

#[cfg(target_os = "macos")]
#[test]
fn export_rejects_private_alias_of_temp_path() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, ROOT};
    use std::path::PathBuf;

    let bin = env!("CARGO_BIN_EXE_automerge");
    let mut source = AutoCommit::new();
    source.put(ROOT, "key", "value").unwrap();

    let base = env::temp_dir().join(format!(
        "automerge-cli-macos-private-alias-{}",
        std::process::id()
    ));
    std::fs::create_dir(&base).unwrap();
    let input = base.join("input.automerge");
    std::fs::write(&input, source.save()).unwrap();

    let private_alias = if let Ok(relative) = input.strip_prefix("/private") {
        PathBuf::from("/").join(relative)
    } else {
        PathBuf::from("/private").join(input.strip_prefix("/").unwrap())
    };
    assert_eq!(
        std::fs::canonicalize(&input).unwrap(),
        std::fs::canonicalize(&private_alias).unwrap()
    );

    let result = cmd!(bin, "export", &private_alias, "--out", &input)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();
    assert!(!result.status.success());
    assert!(automerge::Automerge::load(&std::fs::read(&input).unwrap()).is_ok());

    std::fs::remove_dir_all(&base).unwrap();
}

#[cfg(unix)]
#[test]
fn file_outputs_reject_non_regular_files_and_unrelated_hard_links() {
    use automerge::transaction::Transactable;
    use automerge::{AutoCommit, ROOT};

    let bin = env!("CARGO_BIN_EXE_automerge");
    let base = env::temp_dir().join(format!("automerge-cli-atomic-links-{}", std::process::id()));
    std::fs::create_dir(&base).unwrap();
    let mut source = AutoCommit::new();
    source.put(ROOT, "key", "value").unwrap();
    let input = base.join("input.automerge");
    std::fs::write(&input, source.save()).unwrap();

    let target = base.join("target.automerge");
    let symlink = base.join("symlink.automerge");
    std::fs::write(&target, b"keep target").unwrap();
    std::os::unix::fs::symlink(&target, &symlink).unwrap();
    let symlink_result = cmd!(bin, "copy", &input, "--out", &symlink)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();
    assert!(!symlink_result.status.success());
    assert_eq!(std::fs::read(&target).unwrap(), b"keep target");

    let hard_link = base.join("hard-link.automerge");
    std::fs::write(&target, b"keep target").unwrap();
    std::fs::hard_link(&target, &hard_link).unwrap();
    let hard_link_result = cmd!(bin, "copy", &input, "--out", &hard_link)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();
    assert!(!hard_link_result.status.success());
    assert_eq!(std::fs::read(&target).unwrap(), b"keep target");

    for device in ["/dev/null", "/dev/zero", "/dev/stdout", "/dev/fd/1"] {
        if std::path::Path::new(device).exists() {
            let device_result = cmd!(bin, "copy", &input, "--out", device)
                .stderr_capture()
                .unchecked()
                .run()
                .unwrap();
            assert!(
                !device_result.status.success(),
                "accepted special path {device}"
            );
        }
    }

    let socket = base.join("socket.automerge");
    let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    let socket_result = cmd!(bin, "copy", &input, "--out", &socket)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();
    assert!(!socket_result.status.success());

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
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o640)).unwrap();
    }
    cmd!(bin, "merge", &input, "--out", &output).run().unwrap();
    assert_eq!(std::fs::read(&output).unwrap(), source_bytes);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&output).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }
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
