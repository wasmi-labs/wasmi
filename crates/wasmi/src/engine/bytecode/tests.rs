use super::*;
use core::mem::size_of;

#[test]
fn size_of_instruction() {
    assert_eq!(size_of::<Instruction>(), 8);
    assert_eq!(size_of::<DropKeep>(), 3);
    assert_eq!(size_of::<BranchOffset>(), 3);
    assert_eq!(size_of::<BlockFuel>(), 3);
    assert_eq!(size_of::<BranchTableTargets>(), 3);
    assert_eq!(size_of::<DataSegmentIdx>(), 3);
    assert_eq!(size_of::<ElementSegmentIdx>(), 3);
    assert_eq!(size_of::<FuncIdx>(), 3);
    assert_eq!(size_of::<GlobalIdx>(), 3);
    assert_eq!(size_of::<TableIdx>(), 3);
    assert_eq!(size_of::<SignatureIdx>(), 3);
    assert_eq!(size_of::<LocalDepth>(), 3);
}
