use crate::{for_each_op, index::Reg, Instruction, RegSpan};

impl Instruction {
    /// Relinks the result [`Reg`] of `self` to `new_reg` if it was `old_reg`.
    ///
    /// Returns `true` if `self` has been changed.
    pub fn relink_result(&mut self, old_reg: Reg, new_reg: Reg) -> bool {
        <Self as RelinkResult>::relink_result(self, old_reg, new_reg)
    }
}

/// Utility trait to conditionally relink result registers.
pub trait RelinkResult {
    /// Relinks the result register to `new_reg` if it matches `old_reg`.
    fn relink_result(&mut self, old_reg: Reg, new_reg: Reg) -> bool;
}

macro_rules! impl_relink_result {
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
        impl RelinkResult for Instruction {
            fn relink_result(&mut self, old_reg: Reg, new_reg: Reg) -> bool {
                match self {
                    $(
                        Self::$name {
                            $( $( $result_name, )? )?
                            ..
                        } => {
                            false
                            $( $( || $result_name.relink_result(old_reg, new_reg) )? )?
                        }
                    )*
                }
            }
        }
    };
}
for_each_op!(impl_relink_result);

impl RelinkResult for Reg {
    fn relink_result(&mut self, old_reg: Reg, new_reg: Reg) -> bool {
        if *self != old_reg {
            return false;
        }
        *self = new_reg;
        true
    }
}

impl RelinkResult for RegSpan {
    fn relink_result(&mut self, old_reg: Reg, new_reg: Reg) -> bool {
        if self.head() != old_reg {
            return false;
        }
        *self.head_mut() = new_reg;
        true
    }
}
