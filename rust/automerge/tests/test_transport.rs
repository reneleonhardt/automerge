use automerge::{TransportCompression, TransportError};

#[test]
fn uncompressed_transport_copies_and_enforces_limit() {
    let input = b"transport payload";
    assert_eq!(
        TransportCompression::default(),
        TransportCompression::none()
    );
    assert_eq!(TransportCompression::none().compress(input).unwrap(), input);
    assert_eq!(
        automerge::transport::TransportCompression::None
            .decompress(input, input.len())
            .unwrap(),
        input
    );
    assert!(matches!(
        TransportCompression::None.decompress(input, input.len() - 1),
        Err(TransportError::OutputTooLarge { .. })
    ));
}

#[cfg(feature = "zstd")]
#[test]
fn zstd_transport_round_trips_through_the_policy() {
    let input = b"repeated transport payload".repeat(32);
    let policy = TransportCompression::zstd();
    let compressed = policy.compress(&input).unwrap();

    assert_eq!(policy.decompress(&compressed, input.len()).unwrap(), input);
    assert!(matches!(
        policy.decompress(&compressed, input.len() - 1),
        Err(TransportError::Zstd(
            automerge::zstd::Error::OutputTooLarge { .. }
        ))
    ));
    assert!(matches!(
        TransportCompression::Zstd { level: i32::MAX }.compress(&input),
        Err(TransportError::Zstd(
            automerge::zstd::Error::InvalidCompressionLevel { .. }
        ))
    ));
    assert!(matches!(
        policy.decompress(b"not a zstd frame", input.len()),
        Err(TransportError::Zstd(_))
    ));
}
