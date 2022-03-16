use super::{
    super::{DedupProviderSlice, DedupProviderSliceArena, Provider},
    ExecInstruction,
};

#[test]
fn size_of_instruction() {
    use core::mem::size_of;
    assert_eq!(size_of::<Provider>(), 4);
    assert_eq!(size_of::<ExecInstruction>(), 16);
}
