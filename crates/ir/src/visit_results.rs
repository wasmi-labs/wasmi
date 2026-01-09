use crate::{index::*, *};

impl Op {
    /// Visit result [`Slot`]s of `self` via the `visitor`.
    pub fn visit_results<V: VisitResults>(&mut self, visitor: &mut V) {
        ResultsVisitor::host_visitor(self, visitor)
    }
}

/// Implemented by [`Slot`] visitors to visit result [`Slot`]s of an [`Op`] via [`Op::visit_results`].
pub trait VisitResults {
    /// Visits a [`Slot`] storing the result of an [`Op`].
    fn visit_result_reg(&mut self, reg: &mut Slot);
    /// Visits a [`SlotSpan`] storing the results of an [`Op`].
    fn visit_result_regs(&mut self, reg: &mut SlotSpan, len: Option<u16>);
}

/// Internal trait used to dispatch to a [`VisitResults`] visitor.
pub trait ResultsVisitor {
    /// Host the [`VisitResults`] visitor in the appropriate way.
    fn host_visitor<V: VisitResults>(self, visitor: &mut V);
}

impl ResultsVisitor for &'_ mut Slot {
    fn host_visitor<V: VisitResults>(self, visitor: &mut V) {
        visitor.visit_result_reg(self);
    }
}

impl ResultsVisitor for &'_ mut [Slot; 2] {
    fn host_visitor<V: VisitResults>(self, visitor: &mut V) {
        visitor.visit_result_reg(&mut self[1]);
        visitor.visit_result_reg(&mut self[0]);
    }
}

impl ResultsVisitor for &'_ mut SlotSpan {
    fn host_visitor<V: VisitResults>(self, visitor: &mut V) {
        visitor.visit_result_regs(self, None);
    }
}

impl ResultsVisitor for &'_ mut BoundedSlotSpan {
    fn host_visitor<V: VisitResults>(self, visitor: &mut V) {
        let len = self.len();
        visitor.visit_result_regs(self.span_mut(), Some(len));
    }
}

impl<const N: u16> ResultsVisitor for &'_ mut FixedSlotSpan<N> {
    fn host_visitor<V: VisitResults>(self, visitor: &mut V) {
        visitor.visit_result_regs(self.span_mut(), Some(N));
    }
}
