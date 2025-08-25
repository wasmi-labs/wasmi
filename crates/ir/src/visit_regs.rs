use crate::{index::*, *};

impl Instruction {
    /// Visit [`Reg`]s of `self` via the `visitor`.
    pub fn visit_regs<V: VisitRegs>(&mut self, visitor: &mut V) {
        HostVisitor::host_visitor(self, visitor)
    }
}

/// Implemented by [`Reg`] visitors to visit [`Reg`]s of an [`Instruction`] via [`Instruction::visit_regs`].
pub trait VisitRegs {
    /// Visits a [`Reg`] storing the result of an [`Instruction`].
    fn visit_result_reg(&mut self, reg: &mut Reg);
    /// Visits a [`RegSpan`] storing the results of an [`Instruction`].
    fn visit_result_regs(&mut self, reg: &mut RegSpan, len: Option<u16>);
}

/// Internal trait used to dispatch to a [`VisitRegs`] visitor.
pub trait HostVisitor {
    /// Host the [`VisitRegs`] visitor in the appropriate way.
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V);
}

impl HostVisitor for &'_ mut Reg {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_result_reg(self);
    }
}

impl HostVisitor for &'_ mut [Reg; 2] {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_result_reg(&mut self[0]);
        visitor.visit_result_reg(&mut self[1]);
    }
}

impl HostVisitor for &'_ mut RegSpan {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_result_regs(self, None);
    }
}

impl HostVisitor for &'_ mut BoundedRegSpan {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        let len = self.len();
        visitor.visit_result_regs(self.span_mut(), Some(len));
    }
}

impl<const N: u16> HostVisitor for &'_ mut FixedRegSpan<N> {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_result_regs(self.span_mut(), Some(N));
    }
}
