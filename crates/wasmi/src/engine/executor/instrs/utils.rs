macro_rules! impl_unary_executors {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, input: Reg) {
                self.execute_unary_t(result, input, $op)
            }
        )*
    };
}

macro_rules! impl_binary_executors {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Reg) {
                self.execute_binary_t(result, lhs, rhs, $op)
            }
        )*
    };
}
