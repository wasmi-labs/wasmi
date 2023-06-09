use super::{BinInstr, BinInstrImm16, Const16, Const32, Instruction, Register, UnaryInstr};

macro_rules! constructor_for {
    (
        $(
            fn $fn_name:ident($mode:ident) -> Self::$op_code:ident;
        )* $(,)?
    ) => {
        $( constructor_for! { @impl fn $fn_name($mode) -> Self::$op_code } )*
    };
    ( @impl fn $fn_name:ident(binary) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register, rhs: Register) -> Self {
            Self::$op_code(BinInstr::new(result, lhs, rhs))
        }
    };
    ( @impl fn $fn_name:ident(binary_imm) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register) -> Self {
            Self::$op_code(UnaryInstr::new(result, lhs))
        }
    };
    ( @impl fn $fn_name:ident(binary_imm16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register, rhs: Const16) -> Self {
            Self::$op_code(BinInstrImm16::new(result, lhs, rhs))
        }
    };
}

impl Instruction {
    /// Creates a new [`Instruction::Const32`] from the given `value`.
    pub fn const32(value: impl Into<Const32>) -> Self {
        Self::Const32(value.into())
    }

    constructor_for! {
        fn i32_add(binary) -> Self::I32Add;
        fn i32_add_imm(binary_imm) -> Self::I32AddImm;
        fn i32_add_imm16(binary_imm16) -> Self::I32AddImm16;

        fn i32_mul(binary) -> Self::I32Mul;
        fn i32_mul_imm(binary_imm) -> Self::I32MulImm;
        fn i32_mul_imm16(binary_imm16) -> Self::I32MulImm16;
    }
}
