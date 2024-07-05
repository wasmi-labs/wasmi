use tiny_keccak::{Hasher, Keccak};

/// The data required to run the `tiny_keccak` benchmark.
pub struct TinyKeccakData {
    /// The input data to the tiny keccak hasher.
    input: [u8; 4096],
    /// The buffer to store the hash result.
    result: [u8; 32],
}

#[no_mangle]
pub extern "C" fn setup() -> Box<TinyKeccakData> {
    Box::new(TinyKeccakData {
        input: [254_u8; 4096],
        result: [0x0_u8; 32],
    })
}

#[no_mangle]
pub extern "C" fn teardown(_: Box<TinyKeccakData>) {}

#[no_mangle]
pub extern "C" fn run(data: &mut TinyKeccakData) {
    let mut keccak = Keccak::v256();
    keccak.update(&data.input[..]);
    keccak.finalize(&mut data.result[..]);
}
