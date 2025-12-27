use crate::{RefType, TableType};

fn table_type(element: RefType, minimum: u32, maximum: impl Into<Option<u32>>) -> TableType {
    TableType::new(element, minimum, maximum.into())
}

#[test]
fn subtyping_works() {
    assert!(!table_type(RefType::Func, 0, 1).is_subtype_of(&table_type(RefType::Extern, 0, 1)));
    assert!(table_type(RefType::Func, 0, 1).is_subtype_of(&table_type(RefType::Func, 0, 1)));
    assert!(table_type(RefType::Func, 0, 1).is_subtype_of(&table_type(RefType::Func, 0, 2)));
    assert!(!table_type(RefType::Func, 0, 2).is_subtype_of(&table_type(RefType::Func, 0, 1)));
    assert!(table_type(RefType::Func, 2, None).is_subtype_of(&table_type(RefType::Func, 1, None)));
    assert!(table_type(RefType::Func, 0, None).is_subtype_of(&table_type(RefType::Func, 0, None)));
    assert!(table_type(RefType::Func, 0, 1).is_subtype_of(&table_type(RefType::Func, 0, None)));
    assert!(!table_type(RefType::Func, 0, None).is_subtype_of(&table_type(RefType::Func, 0, 1)));
}
