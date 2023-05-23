use super::*;
use core::mem::size_of;

#[test]
fn size_of_instruction() {
    assert_eq!(size_of::<Instruction>(), 8);
    assert_eq!(size_of::<DropKeep>(), 4);
    assert_eq!(size_of::<BranchOffset>(), 4);
    assert_eq!(size_of::<BlockFuel>(), 4);
    assert_eq!(size_of::<BranchTableTargets>(), 4);
    assert_eq!(size_of::<DataSegmentIdx>(), 4);
    assert_eq!(size_of::<ElementSegmentIdx>(), 4);
    assert_eq!(size_of::<FuncIdx>(), 4);
    assert_eq!(size_of::<GlobalIdx>(), 4);
    assert_eq!(size_of::<TableIdx>(), 4);
    assert_eq!(size_of::<SignatureIdx>(), 4);
    assert_eq!(size_of::<LocalDepth>(), 4);
}
