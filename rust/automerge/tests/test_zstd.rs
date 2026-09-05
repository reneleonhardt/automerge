#![cfg(feature = "zstd")]

use automerge::transaction::Transactable;
use automerge::{zstd, Automerge, ReadDoc, ROOT};

#[test]
fn compressed_uncompressed_document_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Automerge::new();
    {
        let mut transaction = doc.transaction();
        transaction.put(ROOT, "message", "hello")?;
        transaction.commit();
    }

    let raw = doc.save_nocompress();
    let compressed = zstd::compress(&raw, zstd::DEFAULT_COMPRESSION_LEVEL)?;
    let restored = zstd::decompress(&compressed, raw.len())?;
    let loaded = Automerge::load(&restored)?;

    assert_eq!(loaded.get(ROOT, "message")?.unwrap().0, "hello".into());
    Ok(())
}

#[test]
fn malformed_frame_is_an_error() {
    assert!(zstd::decompress(b"not a zstd frame", 1024).is_err());
}

#[test]
fn empty_payload_and_invalid_level_are_handled() {
    let compressed = zstd::compress(&[], zstd::DEFAULT_COMPRESSION_LEVEL).unwrap();
    assert!(zstd::decompress(&compressed, 0).unwrap().is_empty());
    assert!(matches!(
        zstd::compress(b"data", i32::MAX),
        Err(zstd::Error::InvalidCompressionLevel { .. })
    ));
}
