use crate::engine::bytecode::{Instruction, Reg, RegSpan, VisitRegs};

/// Extension-trait for [`Instruction`] to only visit certain [`Reg`]s via closure.
pub trait VisitInputRegisters {
    /// Calls `f` on all input [`Reg`].
    fn visit_input_registers(&mut self, f: impl FnMut(&mut Reg));
}

/// A [`Reg`] visitor.
pub struct Visitor<F> {
    f: F,
}

impl<F: FnMut(&'_ mut Reg)> VisitRegs for Visitor<F> {
    #[inline(always)]
    fn visit_result_reg(&mut self, _reg: &mut Reg) {}

    #[inline(always)]
    fn visit_result_regs(&mut self, _reg: &mut RegSpan, _len: Option<u16>) {}

    #[inline]
    fn visit_input_reg(&mut self, reg: &mut Reg) {
        (self.f)(reg);
    }

    #[inline]
    fn visit_input_regs(&mut self, regs: &mut RegSpan, _len: Option<u16>) {
        (self.f)(regs.head_mut());
    }
}

impl VisitInputRegisters for Instruction {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Reg)) {
        // Note: for copy instructions that copy register values we also need to visit
        //       their results because preserved registers might be populating them.
        match self {
            | Self::Copy { result, .. } => f(result),
            | Self::Copy2 { results, .. } => f(results.span_mut().head_mut()),
            | Self::CopySpan { results, .. }
            | Self::CopySpanNonOverlapping { results, .. }
            | Self::CopyMany { results, .. }
            | Self::CopyManyNonOverlapping { results, .. } => f(results.head_mut()),
            _ => {}
        }
        self.visit_regs(&mut Visitor { f });
    }
}
