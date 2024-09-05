use crate::{core::TrapCode, index::*, *};

impl Instruction {
    /// Mutably visits all input [`Reg`]s of `self` with `f`.
    pub fn visit_input_regs(&mut self, f: impl FnMut(&mut Reg)) {
        <Self as VisitInputRegs>::visit_input_regs(self, f)
    }
}

/// Utility trait to visit all input [`Reg`]s of a type.
pub trait VisitInputRegs {
    /// Mutably visits all input [`Reg`]s of `self`.
    fn visit_input_regs(&mut self, f: impl FnMut(&mut Reg));
}

macro_rules! impl_visit_input_regs_fallback {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl VisitInputRegs for $ty {
                #[inline(always)]
                fn visit_input_regs(&mut self, _f: impl FnMut(&mut Reg)) {}
            }
        )*
    };
}
impl_visit_input_regs_fallback!(
    u8,
    i8,
    i16,
    u16,
    u32,
    Sign,
    TrapCode,
    BlockFuel,
    BranchOffset,
    BranchOffset16,
    Reg,
    RegSpan,
    RegSpanIter,
    AnyConst32,
    InternalFunc,
    Elem,
    Data,
    Func,
    Table,
    Global,
    FuncType,
);

impl<T> VisitInputRegs for Const16<T> {
    #[inline(always)]
    fn visit_input_regs(&mut self, _f: impl FnMut(&mut Reg)) {}
}

impl<T> VisitInputRegs for Const32<T> {
    #[inline(always)]
    fn visit_input_regs(&mut self, _f: impl FnMut(&mut Reg)) {}
}

impl<const N: usize> VisitInputRegs for [Reg; N] {
    #[inline(always)]
    fn visit_input_regs(&mut self, _f: impl FnMut(&mut Reg)) {}
}

macro_rules! impl_visit_input_regs {
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
        impl VisitInputRegs for Instruction {
            fn visit_input_regs(&mut self, mut f: impl FnMut(&mut Reg)) {
                match self {
                    $(
                        Self::$name { $( $( $field_name, )* )? .. } => {
                            $( $( $field_name.visit_input_regs(&mut f); )* )?
                        }
                    )*
                }
            }
        }
    };
}
for_each_op!(impl_visit_input_regs);
