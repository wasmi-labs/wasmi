use super::*;
use core::mem::size_of;

#[test]
fn size_of_instruction() {
    assert_eq!(size_of::<Instruction>(), 16);
    assert_eq!(size_of::<DropKeep>(), 4);
    assert_eq!(size_of::<BranchParams>(), 8);
    assert_eq!(size_of::<BranchOffset>(), 4);
}
