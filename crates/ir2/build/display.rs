use indoc::indoc;
use core::fmt::{self, Display};
use crate::build::op::{UnaryOp, BinaryOp, Ty};
use crate::build::token::{CamelCase, SnakeCase, Ident};

pub struct DisplayEnum<T>(T);

impl Display for DisplayEnum<UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ident = CamelCase(self.0.ident);
        let result_ty = self.0.result_ty;
        let input_ty = self.0.input_ty;
        let result_ident = CamelCase(Ident::from(result_ty));
        write!(f,
            "/
            {result_ident}{ident} {{\n
            ",
        )
        // write!(f, indoc! {"
        //     {result_ident}{ident}
        
        //     ",
        //     result_ident = CamelCase(Ident::from(result_ty)),
        // })
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Ty::I32 => "i32",
            Ty::I64 => "i64",
            Ty::F32 => "f32",
            Ty::F64 => "f64",
            Ty::Ref => "ref",
        };
        write!(f, s)
    }
}
