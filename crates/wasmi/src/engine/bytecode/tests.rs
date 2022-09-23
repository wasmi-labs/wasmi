use super::*;

#[test]
fn size_of_instruction() {
    assert_eq!(core::mem::size_of::<Instruction>(), 16);
    assert_eq!(core::mem::size_of::<DropKeep>(), 4);
    assert_eq!(core::mem::size_of::<Target>(), 8);
}
