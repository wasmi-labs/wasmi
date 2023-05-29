use super::*;
use std::mem::size_of;

#[test]
fn bytecode_size() {
    assert_eq!(size_of::<Register>(), 2);
    assert_eq!(size_of::<UnaryInstr>(), 4);
    assert_eq!(size_of::<BinInstr>(), 6);
    assert_eq!(size_of::<BinInstrImm16>(), 6);
    assert_eq!(size_of::<Instruction>(), 8);
}
