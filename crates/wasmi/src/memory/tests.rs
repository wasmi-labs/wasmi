use super::*;

fn memory_type(minimum: u32, maximum: impl Into<Option<u32>>) -> MemoryType {
    MemoryType::new(minimum, maximum.into()).unwrap()
}

#[test]
fn subtyping_works() {
    assert!(memory_type(0, 1).is_subtype_of(&memory_type(0, 1)));
    assert!(memory_type(0, 1).is_subtype_of(&memory_type(0, 2)));
    assert!(!memory_type(0, 2).is_subtype_of(&memory_type(0, 1)));
    assert!(memory_type(2, None).is_subtype_of(&memory_type(1, None)));
    assert!(memory_type(0, None).is_subtype_of(&memory_type(0, None)));
    assert!(memory_type(0, 1).is_subtype_of(&memory_type(0, None)));
    assert!(!memory_type(0, None).is_subtype_of(&memory_type(0, 1)));
}
