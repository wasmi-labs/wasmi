mod op_code;
pub mod op;
mod op_ty;
mod impls;

pub mod class {
    pub use super::impls::*;
}

pub use self::{
    op_code::OpCode,
    op_ty::Op,
};
