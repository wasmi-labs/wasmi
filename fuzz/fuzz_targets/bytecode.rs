#![no_main]

use libfuzzer_sys::{arbitrary::*, fuzz_target};
use wasmi_ir as ir;

fuzz_target!(|data| {
    let _ = fuzz(data);
});

/// Fuzzing procedure:
///
/// 1. Use [`ir::SafeDecoder`] to decode the `data` bytes into an `Op` buffer `ops` until an error is met.
/// 2. Encode `ops` via [`ir::Encoder`].
/// 3. Decode the encoded `ops` from step 2 again safely via [`ir::SafeDecoder`] and store into `ops2`.
/// 4. Decode the encoded `ops` from step 2 unsafely via [`ir::UnsafeDecoder`] and store into `ops3`.
/// 5. Assert that `ops`, `ops2` and `ops3` are all equal.
fn fuzz(data: &[u8]) -> Result<()> {
    let mut decoder = ir::SafeOpDecoder::new(data);
    let mut ops = Vec::new();
    while let Ok(decoded) = decoder.decode() {
        ops.push(decoded);
    }
    let mut encoder = ir::OpEncoder::default();
    for encoded in &ops {
        encoder.push(*encoded);
    }
    let mut safe_decoder = ir::SafeOpDecoder::new(encoder.as_bytes());
    let mut unsafe_decoder = ir::UnsafeOpDecoder::new(encoder.as_bytes().as_ptr());
    let mut ops2 = Vec::new();
    let mut ops3 = Vec::new();
    for _ in 0..ops.len() {
        ops2.push(safe_decoder.decode().unwrap());
        ops3.push(unsafe { unsafe_decoder.decode() });
    }
    assert_eq!(ops, ops2);
    assert_eq!(ops, ops3);
    Ok(())
}
