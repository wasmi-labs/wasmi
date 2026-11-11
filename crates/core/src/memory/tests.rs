use super::*;

fn memory_type(minimum: u32, maximum: impl Into<Option<u32>>) -> MemoryType {
    let mut b = MemoryType::builder();
    b.min(u64::from(minimum));
    b.max(maximum.into().map(u64::from));
    b.build().unwrap()
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

#[test]
fn memory32_absolute_max_with_1_byte_pages() {
    let build = |min: u64, max: Option<u64>| {
        let mut b = MemoryType::builder();
        b.page_size_log2(0).min(min).max(max);
        b.build()
    };
    assert!(build(u64::from(u32::MAX), None).is_ok());
    assert!(build(0, Some(u64::from(u32::MAX))).is_ok());
    assert!(build(1 << 32, None).is_err());
    assert!(build(0, Some(1 << 32)).is_err());
}
