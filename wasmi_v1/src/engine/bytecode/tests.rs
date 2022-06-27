use super::{
    super::{DedupProviderSliceArena, ExecProvider, ExecProviderSlice},
    ExecInstruction,
};

#[test]
fn size_of_instruction() {
    use core::mem::size_of;
    assert_eq!(size_of::<ExecProvider>(), 4);
    assert_eq!(size_of::<ExecInstruction>(), 24); // TODO: we want this to be 16
}
