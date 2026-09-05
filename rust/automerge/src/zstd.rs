//! Optional zstd transport compression for bytes outside the Automerge format.

use std::io::{self, Read};

/// Errors returned by the transport zstd helpers.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The compressed input was invalid, truncated, or the codec failed.
    #[error("zstd codec error: {0}")]
    Codec(#[from] io::Error),
    /// The decompressed result would exceed the caller's limit.
    #[error("zstd decompressed output exceeds the {limit}-byte limit")]
    OutputTooLarge { limit: usize },
    /// The compression level is outside zstd's supported range.
    #[error("zstd compression level {level} is outside the supported range")]
    InvalidCompressionLevel { level: i32 },
}

/// The default zstd compression level.
pub const DEFAULT_COMPRESSION_LEVEL: i32 = ::zstd::DEFAULT_COMPRESSION_LEVEL;

fn window_log_for_output_limit(max_output_size: usize) -> u32 {
    let bits = usize::BITS - max_output_size.saturating_sub(1).leading_zeros();
    bits.clamp(10, 31)
}

/// Compresses bytes into one standard zstd frame.
pub fn compress(input: &[u8], level: i32) -> Result<Vec<u8>, Error> {
    if !::zstd::compression_level_range().contains(&level) {
        return Err(Error::InvalidCompressionLevel { level });
    }
    Ok(::zstd::bulk::compress(input, level)?)
}

/// Decompresses zstd frames without allowing the result to exceed `max_output_size`,
/// while also applying an output-derived cap to decoder window allocation.
pub fn decompress(input: &[u8], max_output_size: usize) -> Result<Vec<u8>, Error> {
    const BUFFER_SIZE: usize = 16 * 1024;

    let mut decoder = ::zstd::stream::read::Decoder::new(input)?;
    decoder.window_log_max(window_log_for_output_limit(max_output_size))?;
    let mut output = Vec::with_capacity(max_output_size.min(BUFFER_SIZE));
    let mut buffer = [0; BUFFER_SIZE];

    while output.len() < max_output_size {
        let available = (max_output_size - output.len()).min(BUFFER_SIZE);
        let read = decoder.read(&mut buffer[..available])?;
        if read == 0 {
            return Ok(output);
        }
        output.extend_from_slice(&buffer[..read]);
    }

    let mut extra = [0; 1];
    if decoder.read(&mut extra)? != 0 {
        return Err(Error::OutputTooLarge {
            limit: max_output_size,
        });
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{compress, decompress, Error, DEFAULT_COMPRESSION_LEVEL};

    #[test]
    fn round_trip_and_output_limit() {
        let input = b"repeated transport payload".repeat(256);
        let compressed = compress(&input, DEFAULT_COMPRESSION_LEVEL).unwrap();

        assert_eq!(decompress(&compressed, input.len()).unwrap(), input);
        assert!(matches!(
            decompress(&compressed, input.len() - 1),
            Err(Error::OutputTooLarge { .. })
        ));
    }

    #[test]
    fn empty_input_round_trips_and_invalid_level_errors() {
        let compressed = compress(&[], DEFAULT_COMPRESSION_LEVEL).unwrap();
        assert!(decompress(&compressed, 0).unwrap().is_empty());
        assert!(compress(b"data", i32::MAX).is_err());
    }

    #[test]
    fn zero_output_limit_rejects_nonempty_payload() {
        let compressed = compress(b"data", DEFAULT_COMPRESSION_LEVEL).unwrap();
        assert!(matches!(
            decompress(&compressed, 0),
            Err(Error::OutputTooLarge { limit: 0 })
        ));
    }
}
