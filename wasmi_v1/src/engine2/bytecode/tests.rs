use super::{
    super::{DedupProviderSlice, DedupProviderSliceArena, ExecProvider},
    ExecInstruction,
};

#[test]
fn size_of_instruction() {
    use core::mem::size_of;
    assert_eq!(size_of::<ExecProvider>(), 4);
    assert_eq!(size_of::<ExecInstruction>(), 16);
}
