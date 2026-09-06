use std::{
    io::{self, Read, Write},
    path::Path,
};

use anyhow::Result;
use automerge::Automerge;

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
