use crate::{index::*, *};

impl Op {
    /// Visit result [`Reg`]s of `self` via the `visitor`.
    pub fn visit_results<V: VisitResults>(&mut self, visitor: &mut V) {
        ResultsVisitor::host_visitor(self, visitor)
    }
}

/// Implemented by [`Reg`] visitors to visit result [`Reg`]s of an [`Op`] via [`Op::visit_results`].
pub trait VisitResults {
    /// Visits a [`Reg`] storing the result of an [`Op`].
    fn visit_result_reg(&mut self, reg: &mut Reg);
    /// Visits a [`RegSpan`] storing the results of an [`Op`].
    fn visit_result_regs(&mut self, reg: &mut RegSpan, len: Option<u16>);
}

/// Internal trait used to dispatch to a [`VisitResults`] visitor.
pub trait ResultsVisitor {
    /// Host the [`VisitResults`] visitor in the appropriate way.
    fn host_visitor<V: VisitResults>(self, visitor: &mut V);
}

impl ResultsVisitor for &'_ mut Reg {
    fn host_visitor<V: VisitResults>(self, visitor: &mut V) {
        visitor.visit_result_reg(self);
    }
}

impl ResultsVisitor for &'_ mut [Reg; 2] {
    fn host_visitor<V: VisitResults>(self, visitor: &mut V) {
        visitor.visit_result_reg(&mut self[0]);
        visitor.visit_result_reg(&mut self[1]);
    }
}

impl ResultsVisitor for &'_ mut RegSpan {
    fn host_visitor<V: VisitResults>(self, visitor: &mut V) {
        visitor.visit_result_regs(self, None);
    }
}

impl ResultsVisitor for &'_ mut BoundedRegSpan {
    fn host_visitor<V: VisitResults>(self, visitor: &mut V) {
        let len = self.len();
        visitor.visit_result_regs(self.span_mut(), Some(len));
    }
}

impl<const N: u16> ResultsVisitor for &'_ mut FixedRegSpan<N> {
    fn host_visitor<V: VisitResults>(self, visitor: &mut V) {
        visitor.visit_result_regs(self.span_mut(), Some(N));
    }
}
