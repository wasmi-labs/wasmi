use crate::{
    ValType,
    ir::{Address, Offset16, Op, Slot, index::Memory},
};

/// Trait implemented by all Wasm operators that can be translated as load extend instructions.
pub trait LoadOp {
    /// The type of the loaded value.
    const LOADED_TY: ValType;

    fn load_ss(result: Slot, ptr: Slot, offset: u64, memory: Memory) -> Op;
    fn load_si(_address: Address, _memory: Memory) -> Option<impl FnOnce(Slot) -> Op> {
        <Option<fn(Slot) -> Op>>::None
    }
    fn load_mem0_offset16_ss(result: Slot, ptr: Slot, offset: Offset16) -> Op;
}

macro_rules! impl_load_extend {
    ( $(
        impl LoadOp for $name:ident {
            const LOADED_TY: ValType = $loaded_ty:expr;

            fn load_ss = $store_ss:expr;
            $( fn load_si = $store_si:expr; )?
            fn load_mem0_offset16_ss = $store_mem0_offset16_ss:expr;
        }
    )* ) => {
        $(
            pub enum $name {}
            impl LoadOp for $name {
                const LOADED_TY: ValType = $loaded_ty;

                fn load_ss(result: Slot, ptr: Slot, offset: u64, memory: Memory) -> Op {
                    $store_ss(result, ptr, offset, memory)
                }

                $(
                    fn load_si(address: Address, memory: Memory) -> Option<impl FnOnce(Slot) -> Op> {
                        Some(move |result| $store_si(result, address, memory))
                    }
                )?

                fn load_mem0_offset16_ss(result: Slot, ptr: Slot, offset: Offset16) -> Op {
                    $store_mem0_offset16_ss(result, ptr, offset)
                }
            }
        )*
    };
}
impl_load_extend! {
    impl LoadOp for I32Load {
        const LOADED_TY: ValType = ValType::I32;

        fn load_ss = Op::u32_load_ss;
        fn load_si = Op::u32_load_si;
        fn load_mem0_offset16_ss = Op::u32_load_mem0_offset16_ss;
    }

    impl LoadOp for I32Load8 {
        const LOADED_TY: ValType = ValType::I32;

        fn load_ss = Op::i32_load_extend8_ss;
        fn load_si = Op::i32_load_extend8_si;
        fn load_mem0_offset16_ss = Op::i32_load_extend8_mem0_offset16_ss;
    }

    impl LoadOp for U32Load8 {
        const LOADED_TY: ValType = ValType::I32;

        fn load_ss = Op::u32_load_extend8_ss;
        fn load_si = Op::u32_load_extend8_si;
        fn load_mem0_offset16_ss = Op::u32_load_extend8_mem0_offset16_ss;
    }

    impl LoadOp for I32Load16 {
        const LOADED_TY: ValType = ValType::I32;

        fn load_ss = Op::i32_load_extend16_ss;
        fn load_si = Op::i32_load_extend16_si;
        fn load_mem0_offset16_ss = Op::i32_load_extend16_mem0_offset16_ss;
    }

    impl LoadOp for U32Load16 {
        const LOADED_TY: ValType = ValType::I32;

        fn load_ss = Op::u32_load_extend16_ss;
        fn load_si = Op::u32_load_extend16_si;
        fn load_mem0_offset16_ss = Op::u32_load_extend16_mem0_offset16_ss;
    }

    impl LoadOp for I64Load {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::u64_load_ss;
        fn load_si = Op::u64_load_si;
        fn load_mem0_offset16_ss = Op::u64_load_mem0_offset16_ss;
    }

    impl LoadOp for I64Load8 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::i64_load_extend8_ss;
        fn load_si = Op::i64_load_extend8_si;
        fn load_mem0_offset16_ss = Op::i64_load_extend8_mem0_offset16_ss;
    }

    impl LoadOp for U64Load8 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::u64_load_extend8_ss;
        fn load_si = Op::u64_load_extend8_si;
        fn load_mem0_offset16_ss = Op::u64_load_extend8_mem0_offset16_ss;
    }

    impl LoadOp for I64Load16 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::i64_load_extend16_ss;
        fn load_si = Op::i64_load_extend16_si;
        fn load_mem0_offset16_ss = Op::i64_load_extend16_mem0_offset16_ss;
    }

    impl LoadOp for U64Load16 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::u64_load_extend16_ss;
        fn load_si = Op::u64_load_extend16_si;
        fn load_mem0_offset16_ss = Op::u64_load_extend16_mem0_offset16_ss;
    }

    impl LoadOp for I64Load32 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::i64_load_extend32_ss;
        fn load_si = Op::i64_load_extend32_si;
        fn load_mem0_offset16_ss = Op::i64_load_extend32_mem0_offset16_ss;
    }

    impl LoadOp for U64Load32 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::u64_load_extend32_ss;
        fn load_si = Op::u64_load_extend32_si;
        fn load_mem0_offset16_ss = Op::u64_load_extend32_mem0_offset16_ss;
    }

    impl LoadOp for F32Load {
        const LOADED_TY: ValType = ValType::F32;

        fn load_ss = Op::u32_load_ss;
        fn load_si = Op::u32_load_si;
        fn load_mem0_offset16_ss = Op::u32_load_mem0_offset16_ss;
    }

    impl LoadOp for F64Load {
        const LOADED_TY: ValType = ValType::F64;

        fn load_ss = Op::u64_load_ss;
        fn load_si = Op::u64_load_si;
        fn load_mem0_offset16_ss = Op::u64_load_mem0_offset16_ss;
    }
}
