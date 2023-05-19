use super::*;
use core::mem::size_of;

#[test]
fn size_of_instruction() {
    assert_eq!(size_of::<Instruction>(), 16);
    assert_eq!(size_of::<DropKeep>(), 3);
    assert_eq!(size_of::<BranchParams>(), 6);
    assert_eq!(size_of::<BranchOffset>(), 3);
}
