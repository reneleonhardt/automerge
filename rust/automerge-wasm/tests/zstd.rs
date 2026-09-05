#![cfg(all(feature = "zstd", target_arch = "wasm32"))]

use automerge_wasm::{compress_zstd, decompress_zstd};
use js_sys::Uint8Array;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn wasm_zstd_round_trip_and_limit() {
    let input = Uint8Array::from(&b"repeated transport payload"[..]);
    let compressed = compress_zstd(input.clone(), None).unwrap();
    let restored = decompress_zstd(compressed.clone(), input.length()).unwrap();

    assert_eq!(restored.to_vec(), input.to_vec());
    assert!(decompress_zstd(compressed, input.length() - 1).is_err());
}
