use super::*;

fn table_type(element: ValueType, minimum: u32, maximum: impl Into<Option<u32>>) -> TableType {
    TableType::new(element, minimum, maximum.into())
}

use ValueType::{F64, I32};

#[test]
fn subtyping_works() {
    assert!(!table_type(I32, 0, 1).is_subtype_of(&table_type(F64, 0, 1)));
    assert!(table_type(I32, 0, 1).is_subtype_of(&table_type(I32, 0, 1)));
    assert!(table_type(I32, 0, 1).is_subtype_of(&table_type(I32, 0, 2)));
    assert!(!table_type(I32, 0, 2).is_subtype_of(&table_type(I32, 0, 1)));
    assert!(table_type(I32, 2, None).is_subtype_of(&table_type(I32, 1, None)));
    assert!(table_type(I32, 0, None).is_subtype_of(&table_type(I32, 0, None)));
    assert!(table_type(I32, 0, 1).is_subtype_of(&table_type(I32, 0, None)));
    assert!(!table_type(I32, 0, None).is_subtype_of(&table_type(I32, 0, 1)));
}
