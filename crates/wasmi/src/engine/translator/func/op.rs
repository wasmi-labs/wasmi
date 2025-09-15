use crate::{
    engine::translator::utils::{ToBits, Wrap},
    ir::{index::Memory, Address, Offset16, Op, Slot},
    ValType,
};

/// Trait implemented by all Wasm operators that can be translated as wrapping store instructions.
pub trait StoreOperator {
    /// The type of the value to the stored.
    type Value;
    /// The type of immediate values.
    type Immediate;

    /// Converts the value into the immediate value type.
    ///
    /// # Examples
    ///
    /// - Wrapping for wrapping stores.
    /// - Conversion to bits type or identity for normal stores.
    fn into_immediate(value: Self::Value) -> Self::Immediate;

    fn store_ss(ptr: Slot, offset: u64, value: Slot, memory: Memory) -> Op;
    fn store_si(ptr: Slot, offset: u64, value: Self::Immediate, memory: Memory) -> Op;
    fn store_is(address: Address, value: Slot, memory: Memory) -> Op;
    fn store_ii(address: Address, value: Self::Immediate, memory: Memory) -> Op;
    fn store_mem0_offset16_ss(ptr: Slot, offset: Offset16, value: Slot) -> Op;
    fn store_mem0_offset16_si(ptr: Slot, offset: Offset16, value: Self::Immediate) -> Op;
}

macro_rules! impl_store_wrap {
    ( $(
        impl StoreOperator for $name:ident {
            type Value = $value_ty:ty;
            type Immediate = $immediate_ty:ty;

            fn into_immediate = $apply:expr;

            fn store_ss = $store_ss:expr;
            fn store_si = $store_si:expr;
            fn store_is = $store_is:expr;
            fn store_ii = $store_ii:expr;
            fn store_mem0_offset16_ss = $store_mem0_offset16_ss:expr;
            fn store_mem0_offset16_si = $store_mem0_offset16_si:expr;
        }
    )* ) => {
        $(
            pub enum $name {}
            impl StoreOperator for $name {
                type Value = $value_ty;
                type Immediate = $immediate_ty;

                fn into_immediate(value: Self::Value) -> Self::Immediate {
                    $apply(value)
                }

                fn store_ss(ptr: Slot, offset: u64, value: Slot, memory: Memory) -> Op {
                    $store_ss(ptr, offset, value, memory)
                }

                fn store_si(ptr: Slot, offset: u64, value: Self::Immediate, memory: Memory) -> Op {
                    $store_si(ptr, offset, value, memory)
                }

                fn store_is(address: Address, value: Slot, memory: Memory) -> Op {
                    $store_is(address, value, memory)
                }

                fn store_ii(address: Address, value: Self::Immediate, memory: Memory) -> Op {
                    $store_ii(address, value, memory)
                }

                fn store_mem0_offset16_ss(ptr: Slot, offset: Offset16, value: Slot) -> Op {
                    $store_mem0_offset16_ss(ptr, offset, value)
                }

                fn store_mem0_offset16_si(ptr: Slot, offset: Offset16, value: Self::Immediate) -> Op {
                    $store_mem0_offset16_si(ptr, offset, value)
                }
            }
        )*
    };
}
impl_store_wrap! {
    impl StoreOperator for I32Store {
        type Value = i32;
        type Immediate = u32;

        fn into_immediate = <i32 as ToBits>::to_bits;
        fn store_ss = Op::store32_ss;
        fn store_si = Op::store32_si;
        fn store_is = Op::store32_is;
        fn store_ii = Op::store32_ii;
        fn store_mem0_offset16_ss = Op::store32_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::store32_mem0_offset16_si;
    }

    impl StoreOperator for I64Store {
        type Value = i64;
        type Immediate = u64;

        fn into_immediate = <i64 as ToBits>::to_bits;
        fn store_ss = Op::store64_ss;
        fn store_si = Op::store64_si;
        fn store_is = Op::store64_is;
        fn store_ii = Op::store64_ii;
        fn store_mem0_offset16_ss = Op::store64_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::store64_mem0_offset16_si;
    }

    impl StoreOperator for F32Store {
        type Value = f32;
        type Immediate = u32;

        fn into_immediate = <f32 as ToBits>::to_bits;
        fn store_ss = Op::store32_ss;
        fn store_si = Op::store32_si;
        fn store_is = Op::store32_is;
        fn store_ii = Op::store32_ii;
        fn store_mem0_offset16_ss = Op::store32_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::store32_mem0_offset16_si;
    }

    impl StoreOperator for F64Store {
        type Value = f64;
        type Immediate = u64;

        fn into_immediate = <f64 as ToBits>::to_bits;
        fn store_ss = Op::store64_ss;
        fn store_si = Op::store64_si;
        fn store_is = Op::store64_is;
        fn store_ii = Op::store64_ii;
        fn store_mem0_offset16_ss = Op::store64_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::store64_mem0_offset16_si;
    }

    impl StoreOperator for I32Store8 {
        type Value = i32;
        type Immediate = i8;

        fn into_immediate = <i32 as Wrap<i8>>::wrap;
        fn store_ss = Op::i32_store8_ss;
        fn store_si = Op::i32_store8_si;
        fn store_is = Op::i32_store8_is;
        fn store_ii = Op::i32_store8_ii;
        fn store_mem0_offset16_ss = Op::i32_store8_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i32_store8_mem0_offset16_si;
    }

    impl StoreOperator for I32Store16 {
        type Value = i32;
        type Immediate = i16;

        fn into_immediate = <i32 as Wrap<i16>>::wrap;
        fn store_ss = Op::i32_store16_ss;
        fn store_si = Op::i32_store16_si;
        fn store_is = Op::i32_store16_is;
        fn store_ii = Op::i32_store16_ii;
        fn store_mem0_offset16_ss = Op::i32_store16_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i32_store16_mem0_offset16_si;
    }

    impl StoreOperator for I64Store8 {
        type Value = i64;
        type Immediate = i8;

        fn into_immediate = <i64 as Wrap<i8>>::wrap;
        fn store_ss = Op::i64_store8_ss;
        fn store_si = Op::i64_store8_si;
        fn store_is = Op::i64_store8_is;
        fn store_ii = Op::i64_store8_ii;
        fn store_mem0_offset16_ss = Op::i64_store8_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i64_store8_mem0_offset16_si;
    }

    impl StoreOperator for I64Store16 {
        type Value = i64;
        type Immediate = i16;

        fn into_immediate = <i64 as Wrap<i16>>::wrap;
        fn store_ss = Op::i64_store16_ss;
        fn store_si = Op::i64_store16_si;
        fn store_is = Op::i64_store16_is;
        fn store_ii = Op::i64_store16_ii;
        fn store_mem0_offset16_ss = Op::i64_store16_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i64_store16_mem0_offset16_si;
    }

    impl StoreOperator for I64Store32 {
        type Value = i64;
        type Immediate = i32;

        fn into_immediate = <i64 as Wrap<i32>>::wrap;
        fn store_ss = Op::i64_store32_ss;
        fn store_si = Op::i64_store32_si;
        fn store_is = Op::i64_store32_is;
        fn store_ii = Op::i64_store32_ii;
        fn store_mem0_offset16_ss = Op::i64_store32_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i64_store32_mem0_offset16_si;
    }
}

/// Trait implemented by all Wasm operators that can be translated as load extend instructions.
pub trait LoadOperator {
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
        impl LoadOperator for $name:ident {
            const LOADED_TY: ValType = $loaded_ty:expr;

            fn load_ss = $store_ss:expr;
            $( fn load_si = $store_si:expr; )?
            fn load_mem0_offset16_ss = $store_mem0_offset16_ss:expr;
        }
    )* ) => {
        $(
            pub enum $name {}
            impl LoadOperator for $name {
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
    impl LoadOperator for I32Load {
        const LOADED_TY: ValType = ValType::I32;

        fn load_ss = Op::load32_ss;
        fn load_si = Op::load32_si;
        fn load_mem0_offset16_ss = Op::load32_mem0_offset16_ss;
    }

    impl LoadOperator for I32Load8 {
        const LOADED_TY: ValType = ValType::I32;

        fn load_ss = Op::i32_load8_ss;
        fn load_si = Op::i32_load8_si;
        fn load_mem0_offset16_ss = Op::i32_load8_mem0_offset16_ss;
    }

    impl LoadOperator for U32Load8 {
        const LOADED_TY: ValType = ValType::I32;

        fn load_ss = Op::u32_load8_ss;
        fn load_si = Op::u32_load8_si;
        fn load_mem0_offset16_ss = Op::u32_load8_mem0_offset16_ss;
    }

    impl LoadOperator for I32Load16 {
        const LOADED_TY: ValType = ValType::I32;

        fn load_ss = Op::i32_load16_ss;
        fn load_si = Op::i32_load16_si;
        fn load_mem0_offset16_ss = Op::i32_load16_mem0_offset16_ss;
    }

    impl LoadOperator for U32Load16 {
        const LOADED_TY: ValType = ValType::I32;

        fn load_ss = Op::u32_load16_ss;
        fn load_si = Op::u32_load16_si;
        fn load_mem0_offset16_ss = Op::u32_load16_mem0_offset16_ss;
    }

    impl LoadOperator for I64Load {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::load64_ss;
        fn load_si = Op::load64_si;
        fn load_mem0_offset16_ss = Op::load64_mem0_offset16_ss;
    }

    impl LoadOperator for I64Load8 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::i64_load8_ss;
        fn load_si = Op::i64_load8_si;
        fn load_mem0_offset16_ss = Op::i64_load8_mem0_offset16_ss;
    }

    impl LoadOperator for U64Load8 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::u64_load8_ss;
        fn load_si = Op::u64_load8_si;
        fn load_mem0_offset16_ss = Op::u64_load8_mem0_offset16_ss;
    }

    impl LoadOperator for I64Load16 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::i64_load16_ss;
        fn load_si = Op::i64_load16_si;
        fn load_mem0_offset16_ss = Op::i64_load16_mem0_offset16_ss;
    }

    impl LoadOperator for U64Load16 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::u64_load16_ss;
        fn load_si = Op::u64_load16_si;
        fn load_mem0_offset16_ss = Op::u64_load16_mem0_offset16_ss;
    }

    impl LoadOperator for I64Load32 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::i64_load32_ss;
        fn load_si = Op::i64_load32_si;
        fn load_mem0_offset16_ss = Op::i64_load32_mem0_offset16_ss;
    }

    impl LoadOperator for U64Load32 {
        const LOADED_TY: ValType = ValType::I64;

        fn load_ss = Op::u64_load32_ss;
        fn load_si = Op::u64_load32_si;
        fn load_mem0_offset16_ss = Op::u64_load32_mem0_offset16_ss;
    }

    impl LoadOperator for F32Load {
        const LOADED_TY: ValType = ValType::F32;

        fn load_ss = Op::load32_ss;
        fn load_si = Op::load32_si;
        fn load_mem0_offset16_ss = Op::load32_mem0_offset16_ss;
    }

    impl LoadOperator for F64Load {
        const LOADED_TY: ValType = ValType::F64;

        fn load_ss = Op::load64_ss;
        fn load_si = Op::load64_si;
        fn load_mem0_offset16_ss = Op::load64_mem0_offset16_ss;
    }
}
