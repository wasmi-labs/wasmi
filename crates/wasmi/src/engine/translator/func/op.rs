use crate::{
    engine::translator::utils::{ToBits, Wrap},
    ir::{index::Memory, Address, Offset16, Op, Slot},
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
    fn store_ii(address: Address, value: Self::Value, memory: Memory) -> Op;
    fn store_mem0_offset16_ss(ptr: Slot, offset: Offset16, value: Slot) -> Op;
    fn store_mem0_offset16_si(ptr: Slot, offset: Offset16, value: Self::Value) -> Op;
}

macro_rules! impl_store_wrap {
    ( $(
        impl StoreOperator for $name:ident {
            type Value = $value_ty:ty;
            type Immediate = $wrapped_ty:ty;

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
                type Immediate = $wrapped_ty;

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

                fn store_ii(address: Address, value: Self::Value, memory: Memory) -> Op {
                    $store_ii(address, value, memory)
                }

                fn store_mem0_offset16_ss(ptr: Slot, offset: Offset16, value: Slot) -> Op {
                    $store_mem0_offset16_ss(ptr, offset, value)
                }

                fn store_mem0_offset16_si(ptr: Slot, offset: Offset16, value: Self::Value) -> Op {
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
        type Immediate = i64;

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
