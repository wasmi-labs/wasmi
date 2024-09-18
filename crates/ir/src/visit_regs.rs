use crate::{core::TrapCode, index::*, *};

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
    /// Visits a [`Reg`] storing an input of an [`Instruction`].
    fn visit_input_reg(&mut self, reg: &mut Reg);
    /// Visits a [`RegSpan`] storing inputs of an [`Instruction`].
    fn visit_input_regs(&mut self, regs: &mut RegSpan, len: Option<u16>);
}

/// Internal trait used to dispatch to a [`VisitRegs`] visitor.
trait HostVisitor {
    /// Host the [`VisitRegs`] visitor in the appropriate way.
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V);
}

impl HostVisitor for &'_ mut Reg {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_input_reg(self);
    }
}

impl<const N: usize> HostVisitor for &'_ mut [Reg; N] {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        for reg in self {
            visitor.visit_input_reg(reg);
        }
    }
}

impl HostVisitor for &'_ mut RegSpan {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_input_regs(self, None);
    }
}

impl HostVisitor for &'_ mut BoundedRegSpan {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        let len = self.len();
        visitor.visit_input_regs(self.span_mut(), Some(len));
    }
}

impl<const N: u16> HostVisitor for &'_ mut FixedRegSpan<N> {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_input_regs(self.span_mut(), Some(N));
    }
}

macro_rules! impl_host_visitor_for {
    ( $( $ty:ident $(<$t:ident>)? ),* $(,)? ) => {
        $(
            impl $(<$t>)? HostVisitor for &'_ mut $ty $(<$t>)? {
                #[inline]
                fn host_visitor<V: VisitRegs>(self, _visitor: &mut V) {}
            }
        )*
    };
}
impl_host_visitor_for!(
    u8,
    i8,
    i16,
    u16,
    u32,
    TrapCode,
    BlockFuel,
    AnyConst16,
    AnyConst32,
    BranchOffset,
    BranchOffset16,
    InternalFunc,
    Func,
    FuncType,
    Global,
    Table,
    Elem,
    Data,
    Sign,
    Const16<T>,
    Const32<T>,
);

/// Type-wrapper to signal that the wrapped [`Reg`], [`RegSpan`] (etc.) is a result.
pub struct Res<T>(T);

impl HostVisitor for Res<&'_ mut Reg> {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_result_reg(self.0);
    }
}

impl HostVisitor for Res<&'_ mut RegSpan> {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_result_regs(self.0, None);
    }
}

impl HostVisitor for Res<&'_ mut BoundedRegSpan> {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        let len = self.0.len();
        visitor.visit_result_regs(self.0.span_mut(), Some(len));
    }
}

impl<const N: u16> HostVisitor for Res<&'_ mut FixedRegSpan<N>> {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_result_regs(self.0.span_mut(), Some(N));
    }
}

macro_rules! impl_host_visitor {
    (
        $(
            $( #[doc = $doc:literal] )*
            #[snake_name($snake_name:ident)]
            $name:ident
            $(
                {
                    $( @ $result_name:ident: $result_ty:ty, )?
                    $(
                        $( #[$field_docs:meta] )*
                        $field_name:ident: $field_ty:ty
                    ),*
                    $(,)?
                }
            )?
        ),* $(,)?
    ) => {
        impl<'a> HostVisitor for &'a mut Instruction {
            fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
                match self {
                    $(
                        Instruction::$name { $( $( $result_name, )? $( $field_name, )* )? } => {
                            $(
                                $( Res($result_name).host_visitor(visitor); )?
                                $( $field_name.host_visitor(visitor); )*
                            )?
                        }
                    )*
                }
            }
        }
    };
}
for_each_op!(impl_host_visitor);
