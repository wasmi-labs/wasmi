use crate::build::Ident;

#[derive(Copy, Clone)]
pub enum Op {
    Binary(BinaryOp),
}

#[derive(Copy, Clone)]
pub struct UnaryOp {
    pub ident: Ident,
    pub result_ty: Ty,
    pub input_ty: Ty,
}

impl UnaryOp {
    pub fn new_conversion(ident: Ident, result_ty: Ty, input_ty: Ty) -> Self {
        Self {
            ident,
            result_ty,
            input_ty,
        }
    }

    pub fn new(ident: Ident, ty: Ty) -> Self {
        Self {
            ident,
            result_ty: ty,
            input_ty: ty,
        }
    }
}

#[derive(Copy, Clone)]
pub struct BinaryOp {
    pub ident: Ident,
    pub result_ty: Ty,
    pub input_ty: Ty,
    pub commutative: bool,
}

impl BinaryOp {
    pub fn new_commutative(ident: Ident, result_ty: Ty, input_ty: Ty) -> Self {
        Self {
            ident,
            result_ty,
            input_ty,
            commutative: true,
        }
    }

    pub fn new(ident: Ident, result_ty: Ty, input_ty: Ty) -> Self {
        Self {
            ident,
            result_ty,
            input_ty,
            commutative: false,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Ty {
    I32,
    I64,
    F32,
    F64,
    Ref,
}

impl From<Ty> for Ident {
    fn from(ty: Ty) -> Self {
        match ty {
            Ty::I32 => Self::I32,
            Ty::I64 => Self::I64,
            Ty::F32 => Self::F32,
            Ty::F64 => Self::F64,
            Ty::Ref => Self::Ref,
        }
    }
}
