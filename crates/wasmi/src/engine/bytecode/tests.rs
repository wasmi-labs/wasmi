use super::*;
use std::mem::size_of;

#[test]
fn bytecode_size() {
    assert_eq!(size_of::<Register>(), 2);
    assert_eq!(size_of::<UnaryInstr>(), 4);
    assert_eq!(size_of::<BinInstr>(), 6);
    assert_eq!(size_of::<BinInstrImm16<i32>>(), 6);
    assert_eq!(size_of::<BinInstrImm16<i64>>(), 6);
    assert_eq!(size_of::<Instruction>(), 8);
}

#[test]
fn has_overlapping_copy_spans_works() {
    fn span(register: impl Into<Register>) -> RegisterSpan {
        RegisterSpan::new(register.into())
    }

    fn has_overlapping_copy_spans(results: RegisterSpan, values: RegisterSpan, len: u16) -> bool {
        RegisterSpanIter::has_overlapping_copies(results.iter_u16(len), values.iter_u16(len))
    }

    // len == 0
    assert!(!has_overlapping_copy_spans(span(0), span(0), 0));
    assert!(!has_overlapping_copy_spans(span(0), span(1), 0));
    assert!(!has_overlapping_copy_spans(span(1), span(0), 0));
    // len == 1
    assert!(!has_overlapping_copy_spans(span(0), span(0), 1));
    assert!(!has_overlapping_copy_spans(span(0), span(1), 1));
    assert!(!has_overlapping_copy_spans(span(1), span(0), 1));
    assert!(!has_overlapping_copy_spans(span(1), span(1), 1));
    // len == 2
    assert!(!has_overlapping_copy_spans(span(0), span(0), 2));
    assert!(!has_overlapping_copy_spans(span(0), span(1), 2));
    assert!(has_overlapping_copy_spans(span(1), span(0), 2));
    assert!(!has_overlapping_copy_spans(span(1), span(1), 2));
    // len == 3
    assert!(!has_overlapping_copy_spans(span(0), span(0), 3));
    assert!(!has_overlapping_copy_spans(span(0), span(1), 3));
    assert!(has_overlapping_copy_spans(span(1), span(0), 3));
    assert!(!has_overlapping_copy_spans(span(1), span(1), 3));
    // special cases
    assert!(has_overlapping_copy_spans(span(1), span(0), 3));
    assert!(has_overlapping_copy_spans(span(2), span(0), 3));
    assert!(!has_overlapping_copy_spans(span(3), span(0), 3));
    assert!(!has_overlapping_copy_spans(span(4), span(0), 3));
    assert!(!has_overlapping_copy_spans(span(4), span(0), 4));
    assert!(has_overlapping_copy_spans(span(4), span(1), 4));
    assert!(has_overlapping_copy_spans(span(4), span(0), 5));
}
