mod constructors;
mod decode;
mod encode;
mod ident;
mod op;
mod op_code;
mod result_mut;
mod utils;

pub use self::{
    constructors::DisplayConstructor,
    decode::DisplayDecode,
    encode::DisplayEncode,
    op::DisplayOp,
    op_code::DisplayOpCode,
    result_mut::DisplayResultMut,
    utils::Indent,
};
use crate::build::{display::ident::DisplayIdent, op::Op};
use core::fmt::{self, Display};

macro_rules! impl_trait_for_op {
    (
        $trait:ident,
        $($variant:ident($op_ty:ty)),* $(,)?
    ) => {
        impl Display for $trait<&'_ Op> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.value {
                    $( Op::$variant(op) => self.map(op).fmt(f), )*
                }
            }
        }
    };
}
apply_macro_for_ops!(impl_trait_for_op, DisplayOp);
apply_macro_for_ops!(impl_trait_for_op, DisplayIdent);
apply_macro_for_ops!(impl_trait_for_op, DisplayConstructor);
apply_macro_for_ops!(impl_trait_for_op, DisplayResultMut);
apply_macro_for_ops!(impl_trait_for_op, DisplayEncode);
apply_macro_for_ops!(impl_trait_for_op, DisplayDecode);
