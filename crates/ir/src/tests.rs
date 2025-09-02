use crate::{Slot, SlotSpan, SlotSpanIter};

#[test]
fn has_overlapping_copy_spans_works() {
    fn span(reg: impl Into<Slot>) -> SlotSpan {
        SlotSpan::new(reg.into())
    }

    fn has_overlapping_copy_spans(results: SlotSpan, values: SlotSpan, len: u16) -> bool {
        SlotSpanIter::has_overlapping_copies(results.iter(len), values.iter(len))
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
