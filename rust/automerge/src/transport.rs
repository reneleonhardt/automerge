//! Compression policies for bytes outside the Automerge document format.

/// A transport or archive compression policy.
///
/// This policy is independent of Automerge's document serialization. The
/// `Zstd` variant is available when the crate's `zstd` feature is enabled.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TransportCompression {
    /// Leave the payload uncompressed.
    #[default]
    None,
    /// Wrap the payload in a standard zstd frame at the requested level.
    #[cfg(feature = "zstd")]
    Zstd { level: i32 },
}

impl TransportCompression {
    /// Returns the default uncompressed policy.
    pub const fn none() -> Self {
        Self::None
    }

    /// Returns zstd at its library default compression level.
    #[cfg(feature = "zstd")]
    pub const fn zstd() -> Self {
        Self::Zstd {
            level: crate::zstd::DEFAULT_COMPRESSION_LEVEL,
        }
    }

    /// Compresses a borrowed payload and returns owned bytes.
    pub fn compress(self, input: &[u8]) -> Result<Vec<u8>, TransportError> {
        match self {
            Self::None => Ok(input.to_vec()),
            #[cfg(feature = "zstd")]
            Self::Zstd { level } => crate::zstd::compress(input, level).map_err(Into::into),
        }
    }

    /// Decompresses a borrowed payload without exceeding `max_output_size`.
    pub fn decompress(
        self,
        input: &[u8],
        max_output_size: usize,
    ) -> Result<Vec<u8>, TransportError> {
        match self {
            Self::None => {
                if input.len() > max_output_size {
                    Err(TransportError::OutputTooLarge {
                        limit: max_output_size,
                    })
                } else {
                    Ok(input.to_vec())
                }
            }
            #[cfg(feature = "zstd")]
            Self::Zstd { .. } => {
                crate::zstd::decompress(input, max_output_size).map_err(Into::into)
            }
        }
    }
}

/// Errors returned by a transport compression policy.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// The decompressed result would exceed the caller's limit.
    #[error("transport decompressed output exceeds the {limit}-byte limit")]
    OutputTooLarge { limit: usize },
    /// The selected codec rejected the input or parameters.
    #[cfg(feature = "zstd")]
    #[error(transparent)]
    Zstd(#[from] crate::zstd::Error),
}
