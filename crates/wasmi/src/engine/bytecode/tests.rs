use super::*;

#[test]
fn size_of_instruction() {
    assert_eq!(core::mem::size_of::<DropKeep>(), 4);
}
