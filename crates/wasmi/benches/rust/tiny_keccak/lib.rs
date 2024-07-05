use tiny_keccak::{Hasher, Keccak};

pub struct TinyKeccakData {
    data: Box<[u8; 4096]>,
    result: [u8; 32],
}

#[no_mangle]
pub extern "C" fn setup() -> Box<TinyKeccakData> {
    Box::new(TinyKeccakData {
        data: Box::new([254_u8; 4096]),
        result: [0x0_u8; 32],
    })
}

#[no_mangle]
pub extern "C" fn teardown(_: Box<TinyKeccakData>) {}

#[no_mangle]
pub extern "C" fn run(data: &mut TinyKeccakData) {
    let mut keccak = Keccak::v256();
    keccak.update(&data.data[..]);
    keccak.finalize(&mut data.result[..]);
}
