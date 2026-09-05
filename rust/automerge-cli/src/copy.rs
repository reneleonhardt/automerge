use std::{
    io::{IsTerminal, Read},
    path::PathBuf,
};

use automerge as am;

#[derive(Debug, thiserror::Error)]
pub(super) enum CopyError {
    #[error("Provide a file path or pipe input through stdin")]
    MissingInput,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Automerge(#[from] am::AutomergeError),
}

pub(super) fn copy(input: Option<PathBuf>) -> Result<Vec<u8>, CopyError> {
    let mut bytes = Vec::new();
    match input {
        Some(path) => {
            bytes = std::fs::read(path)?;
        }
        None if std::io::stdin().is_terminal() => return Err(CopyError::MissingInput),
        None => {
            std::io::stdin().read_to_end(&mut bytes)?;
        }
    }
    am::Automerge::load(&bytes)?;
    Ok(bytes)
}
