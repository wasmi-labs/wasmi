use crate::{
    decode::{Decode, SafeDecoder, UnsafeDecoder},
    encode::{Encode, Encoder},
    *,
};

#[test]
fn encode_decode_reg_slice() {
    let regs = [1, -3, 5, -7, 11, -13, 17]
        .map(Reg::from)
        .map(Unalign::from);
    let reg_slice = Slice::from(&regs[..]);
    let mut enc = Encoder::default();
    reg_slice.encode(&mut enc);
    let encoded_bytes = enc.as_slice();
    let mut decoder = SafeDecoder::new(encoded_bytes);
    let decoded = Slice::decode(&mut decoder).unwrap();
    assert_eq!(reg_slice, decoded);
    let mut decoder = unsafe { UnsafeDecoder::new(encoded_bytes.as_ptr()) };
    let decoded = Slice::decode(&mut decoder).unwrap();
    assert_eq!(reg_slice, decoded);
}
