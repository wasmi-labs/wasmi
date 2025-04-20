mod op_code;
pub mod op;
mod op_ty;
mod unary_op;

pub mod class {
    pub use super::unary_op::*;
}

pub use self::{
    op_code::OpCode,
    op_ty::Op,
};
