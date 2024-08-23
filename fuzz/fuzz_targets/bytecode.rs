#![no_main]

use ir::Code as _;
use libfuzzer_sys::{arbitrary::*, fuzz_target};
use wasmi_ir as ir;

fuzz_target!(|data| {
    let _ = fuzz(data);
});

/// Fuzzing procedure:
///
/// 1. Use [`ir::CheckedDecoder`] to decode the `data` bytes into an `Op` buffer `ops` until an error is met.
/// 2. Encode `ops` via [`ir::Encoder`].
/// 3. Decode the encoded `ops` from step 2 again safely via [`ir::CheckedDecoder`] and store into `ops2`.
/// 4. Decode the encoded `ops` from step 2 unsafely via [`ir::UnCheckedDecoder`] and store into `ops3`.
/// 5. Assert that `ops`, `ops2` and `ops3` are all equal.
fn fuzz(data: &[u8]) -> Result<()> {
    let mut decoder = ir::CheckedOpDecoder::new(data);
    let mut ops = ir::OpEncoder::default();
    while let Ok(decoded) = decoder.decode() {
        let pos = ops.push(decoded);
        assert_eq!(ops.get(pos), Some(decoded));
        assert_eq!(
            ops.get(pos).map(|op| op.code()),
            ops.get_mut(pos).map(|op| op.code())
        );
    }
    let mut safe_decoder = ir::CheckedOpDecoder::new(ops.as_bytes());
    let mut unsafe_decoder = ir::UncheckedOpDecoder::new(ops.as_bytes().as_ptr());
    let mut ops2 = ir::OpEncoder::default();
    let mut ops3 = ir::OpEncoder::default();
    for _ in 0..ops.len() {
        ops2.push(safe_decoder.decode().unwrap());
        ops3.push(unsafe { unsafe_decoder.decode() });
    }
    assert!(ops.iter().eq(&ops2));
    assert!(ops.iter().eq(&ops3));
    Ok(())
}
