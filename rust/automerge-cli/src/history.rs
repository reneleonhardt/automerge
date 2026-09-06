use std::{
    collections::BTreeSet,
    io::{self, Read, Write},
    path::Path,
};

use anyhow::Result;
use automerge::{Automerge, ChangeHash, ChangeMetadata};
use serde_json::json;

fn load(path: &Path) -> Result<Automerge> {
    Ok(Automerge::load(&std::fs::read(path)?)?)
}

pub(super) fn print_heads(input: Option<std::path::PathBuf>) -> Result<()> {
    let mut input = super::open_file_or_stdin(input)?;
    let mut bytes = Vec::new();
    input.read_to_end(&mut bytes)?;
    let document = Automerge::load(&bytes)?;
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, &document.get_heads())?;
    writeln!(stdout)?;
    Ok(())
}

pub(super) fn print_diff(before: &Path, after: &Path) -> Result<()> {
    let before = load(before)?;
    let after = load(after)?;
    let changes = before
        .get_changes_added(&after)
        .iter()
        .map(|change| change.decode())
        .collect::<Vec<_>>();
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, &changes)?;
    writeln!(stdout)?;
    Ok(())
}

fn metadata_to_json(metadata: ChangeMetadata<'_>) -> serde_json::Value {
    let operation_count = metadata.max_op - metadata.start_op + 1;
    json!({
        "actor": metadata.actor.to_hex_string(),
        "author": metadata.author.map(|author| author.to_hex_string()),
        "seq": metadata.seq,
        "start_op": metadata.start_op,
        "max_op": metadata.max_op,
        "operation_count": operation_count,
        "timestamp": metadata.timestamp,
        "message": metadata.message.map(|message| message.into_owned()),
        "deps": metadata
            .deps
            .into_iter()
            .map(|hash| hash.to_string())
            .collect::<Vec<_>>(),
        "hash": metadata.hash.to_string(),
    })
}

pub(super) fn print_changes(
    input: Option<std::path::PathBuf>,
    after: Option<ChangeHash>,
    hash: Option<ChangeHash>,
    limit: Option<usize>,
) -> Result<()> {
    let mut input = super::open_file_or_stdin(input)?;
    let mut bytes = Vec::new();
    input.read_to_end(&mut bytes)?;
    let document = Automerge::load(&bytes)?;
    let metadata = match hash {
        Some(hash) => document
            .get_change_meta_by_hash(&hash)
            .into_iter()
            .collect::<Vec<_>>(),
        None => document.get_changes_meta(after.as_slice()),
    };
    let metadata = metadata
        .into_iter()
        .take(limit.unwrap_or(usize::MAX))
        .map(metadata_to_json)
        .collect::<Vec<_>>();

    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, &metadata)?;
    writeln!(stdout)?;
    Ok(())
}

pub(super) fn verify(input: Option<std::path::PathBuf>, json_output: bool) -> Result<()> {
    let mut input = super::open_file_or_stdin(input)?;
    let mut bytes = Vec::new();
    input.read_to_end(&mut bytes)?;
    let result = Automerge::load(&bytes);

    let mut stdout = io::stdout().lock();
    let document = match result {
        Ok(document) => document,
        Err(error) => {
            if json_output {
                serde_json::to_writer_pretty(
                    &mut stdout,
                    &json!({"valid": false, "error": error.to_string()}),
                )?;
                writeln!(stdout)?;
            }
            return Err(error.into());
        }
    };

    let metadata = document.get_changes_meta(&[]);
    let actors = metadata
        .iter()
        .map(|change| change.actor.to_hex_string())
        .collect::<BTreeSet<_>>();
    if json_output {
        serde_json::to_writer_pretty(
            &mut stdout,
            &json!({
                "valid": true,
                "heads": document
                    .get_heads()
                    .into_iter()
                    .map(|hash| hash.to_string())
                    .collect::<Vec<_>>(),
                "change_count": metadata.len(),
                "actor_count": actors.len(),
                "actors": actors.into_iter().collect::<Vec<_>>(),
            }),
        )?;
        writeln!(stdout)?;
    } else {
        writeln!(
            stdout,
            "valid: {} change(s), {} actor(s)",
            metadata.len(),
            actors.len()
        )?;
    }
    Ok(())
}

pub(super) fn extract(base: &Path, target: &Path) -> Result<Vec<u8>> {
    let base = load(base)?;
    let target = load(target)?;
    Ok(target.save_after(&base.get_heads()))
}

pub(super) fn apply(base: &Path, incremental: &Path) -> Result<Vec<u8>> {
    let mut base = load(base)?;
    base.load_incremental(&std::fs::read(incremental)?)?;
    Ok(base.save())
}
