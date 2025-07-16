use crate::ir::{Address32, Instruction, Offset16, Offset64Lo, Reg};

/// Trait implemented by all Wasm operators that can be translated as wrapping store instructions.
pub trait StoreWrapOperator {
    /// The type of the value to the stored.
    type Value;
    /// The type of the wrapped value.
    type Wrapped;
    /// The type of the value as (at most) 16-bit encoded instruction parameter.
    type Param;

    fn store(ptr: Reg, offset_lo: Offset64Lo) -> Instruction;
    fn store_imm(ptr: Reg, offset_lo: Offset64Lo) -> Instruction;
    fn store_offset16(ptr: Reg, offset: Offset16, value: Reg) -> Instruction;
    fn store_offset16_imm(ptr: Reg, offset: Offset16, value: Self::Param) -> Instruction;
    fn store_at(value: Reg, address: Address32) -> Instruction;
    fn store_at_imm(value: Self::Param, address: Address32) -> Instruction;
}

macro_rules! impl_store_wrap {
    ( $(
        impl StoreWrapOperator for $name:ident {
            type Value = $value_ty:ty;
            type Wrapped = $wrapped_ty:ty;
            type Param = $param_ty:ty;

            fn store = $store:expr;
            fn store_imm = $store_imm:expr;
            fn store_offset16 = $store_offset16:expr;
            fn store_offset16_imm = $store_offset16_imm:expr;
            fn store_at = $store_at:expr;
            fn store_at_imm = $store_at_imm:expr;
        }
    )* ) => {
        $(
            pub enum $name {}
            impl StoreWrapOperator for $name {
                type Value = $value_ty;
                type Wrapped = $wrapped_ty;
                type Param = $param_ty;

                fn store(ptr: Reg, offset_lo: Offset64Lo) -> Instruction {
                    $store(ptr, offset_lo)
                }

                fn store_imm(ptr: Reg, offset_lo: Offset64Lo) -> Instruction {
                    $store_imm(ptr, offset_lo)
                }

                fn store_offset16(ptr: Reg, offset: Offset16, value: Reg) -> Instruction {
                    $store_offset16(ptr, offset, value)
                }

                fn store_offset16_imm(ptr: Reg, offset: Offset16, value: Self::Param) -> Instruction {
                    $store_offset16_imm(ptr, offset, value)
                }

                fn store_at(value: Reg, address: Address32) -> Instruction {
                    $store_at(value, address)
                }

                fn store_at_imm(value: Self::Param, address: Address32) -> Instruction {
                    $store_at_imm(value, address)
                }
            }
        )*
    };
}
impl_store_wrap! {
    impl StoreWrapOperator for I32Store {
        type Value = i32;
        type Wrapped = i32;
        type Param = i16;

        fn store = Instruction::store32;
        fn store_imm = Instruction::i32_store_imm16;
        fn store_offset16 = Instruction::store32_offset16;
        fn store_offset16_imm = Instruction::i32_store_offset16_imm16;
        fn store_at = Instruction::store32_at;
        fn store_at_imm = Instruction::i32_store_at_imm16;
    }

    impl StoreWrapOperator for I64Store {
        type Value = i64;
        type Wrapped = i64;
        type Param = i16;

        fn store = Instruction::store64;
        fn store_imm = Instruction::i64_store_imm16;
        fn store_offset16 = Instruction::store64_offset16;
        fn store_offset16_imm = Instruction::i64_store_offset16_imm16;
        fn store_at = Instruction::store64_at;
        fn store_at_imm = Instruction::i64_store_at_imm16;
    }

    impl StoreWrapOperator for I32Store8 {
        type Value = i32;
        type Wrapped = i8;
        type Param = i8;

        fn store = Instruction::i32_store8;
        fn store_imm = Instruction::i32_store8_imm;
        fn store_offset16 = Instruction::i32_store8_offset16;
        fn store_offset16_imm = Instruction::i32_store8_offset16_imm;
        fn store_at = Instruction::i32_store8_at;
        fn store_at_imm = Instruction::i32_store8_at_imm;
    }

    impl StoreWrapOperator for I32Store16 {
        type Value = i32;
        type Wrapped = i16;
        type Param = i16;

        fn store = Instruction::i32_store16;
        fn store_imm = Instruction::i32_store16_imm;
        fn store_offset16 = Instruction::i32_store16_offset16;
        fn store_offset16_imm = Instruction::i32_store16_offset16_imm;
        fn store_at = Instruction::i32_store16_at;
        fn store_at_imm = Instruction::i32_store16_at_imm;
    }

    impl StoreWrapOperator for I64Store8 {
        type Value = i64;
        type Wrapped = i8;
        type Param = i8;

        fn store = Instruction::i64_store8;
        fn store_imm = Instruction::i64_store8_imm;
        fn store_offset16 = Instruction::i64_store8_offset16;
        fn store_offset16_imm = Instruction::i64_store8_offset16_imm;
        fn store_at = Instruction::i64_store8_at;
        fn store_at_imm = Instruction::i64_store8_at_imm;
    }

    impl StoreWrapOperator for I64Store16 {
        type Value = i64;
        type Wrapped = i16;
        type Param = i16;

        fn store = Instruction::i64_store16;
        fn store_imm = Instruction::i64_store16_imm;
        fn store_offset16 = Instruction::i64_store16_offset16;
        fn store_offset16_imm = Instruction::i64_store16_offset16_imm;
        fn store_at = Instruction::i64_store16_at;
        fn store_at_imm = Instruction::i64_store16_at_imm;
    }

    impl StoreWrapOperator for I64Store32 {
        type Value = i64;
        type Wrapped = i32;
        type Param = i16;

        fn store = Instruction::i64_store32;
        fn store_imm = Instruction::i64_store32_imm16;
        fn store_offset16 = Instruction::i64_store32_offset16;
        fn store_offset16_imm = Instruction::i64_store32_offset16_imm16;
        fn store_at = Instruction::i64_store32_at;
        fn store_at_imm = Instruction::i64_store32_at_imm16;
    }
}
