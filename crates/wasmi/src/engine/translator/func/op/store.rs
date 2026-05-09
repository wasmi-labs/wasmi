use crate::{
    core::Typed,
    engine::translator::utils::{ToBits, Wrap},
    ir::{Address, Offset16, Op, Slot, index::Memory},
};

/// Trait implemented by all Wasm operators that can be translated as wrapping store instructions.
pub trait StoreOp {
    /// The type of the value to the stored.
    type Value: Typed;
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
        impl StoreOp for $name:ident {
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
            impl StoreOp for $name {
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
    impl StoreOp for I32Store {
        type Value = i32;
        type Immediate = u32;

        fn into_immediate = <i32 as ToBits>::to_bits;
        fn store_ss = Op::u32_store_ss;
        fn store_si = Op::u32_store_si;
        fn store_is = Op::u32_store_is;
        fn store_ii = Op::u32_store_ii;
        fn store_mem0_offset16_ss = Op::u32_store_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::u32_store_mem0_offset16_si;
    }

    impl StoreOp for I64Store {
        type Value = i64;
        type Immediate = u64;

        fn into_immediate = <i64 as ToBits>::to_bits;
        fn store_ss = Op::u64_store_ss;
        fn store_si = Op::u64_store_si;
        fn store_is = Op::u64_store_is;
        fn store_ii = Op::u64_store_ii;
        fn store_mem0_offset16_ss = Op::u64_store_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::u64_store_mem0_offset16_si;
    }

    impl StoreOp for F32Store {
        type Value = f32;
        type Immediate = u32;

        fn into_immediate = <f32 as ToBits>::to_bits;
        fn store_ss = Op::u32_store_ss;
        fn store_si = Op::u32_store_si;
        fn store_is = Op::u32_store_is;
        fn store_ii = Op::u32_store_ii;
        fn store_mem0_offset16_ss = Op::u32_store_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::u32_store_mem0_offset16_si;
    }

    impl StoreOp for F64Store {
        type Value = f64;
        type Immediate = u64;

        fn into_immediate = <f64 as ToBits>::to_bits;
        fn store_ss = Op::u64_store_ss;
        fn store_si = Op::u64_store_si;
        fn store_is = Op::u64_store_is;
        fn store_ii = Op::u64_store_ii;
        fn store_mem0_offset16_ss = Op::u64_store_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::u64_store_mem0_offset16_si;
    }

    impl StoreOp for I32Store8 {
        type Value = i32;
        type Immediate = i8;

        fn into_immediate = <i32 as Wrap<Self::Immediate>>::wrap;
        fn store_ss = Op::i32_store_wrap8_ss;
        fn store_si = Op::i32_store_wrap8_si;
        fn store_is = Op::i32_store_wrap8_is;
        fn store_ii = Op::i32_store_wrap8_ii;
        fn store_mem0_offset16_ss = Op::i32_store_wrap8_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i32_store_wrap8_mem0_offset16_si;
    }

    impl StoreOp for I32Store16 {
        type Value = i32;
        type Immediate = i16;

        fn into_immediate = <i32 as Wrap<Self::Immediate>>::wrap;
        fn store_ss = Op::i32_store_wrap16_ss;
        fn store_si = Op::i32_store_wrap16_si;
        fn store_is = Op::i32_store_wrap16_is;
        fn store_ii = Op::i32_store_wrap16_ii;
        fn store_mem0_offset16_ss = Op::i32_store_wrap16_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i32_store_wrap16_mem0_offset16_si;
    }

    impl StoreOp for I64Store8 {
        type Value = i64;
        type Immediate = i8;

        fn into_immediate = <i64 as Wrap<Self::Immediate>>::wrap;
        fn store_ss = Op::i64_store_wrap8_ss;
        fn store_si = Op::i64_store_wrap8_si;
        fn store_is = Op::i64_store_wrap8_is;
        fn store_ii = Op::i64_store_wrap8_ii;
        fn store_mem0_offset16_ss = Op::i64_store_wrap8_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i64_store_wrap8_mem0_offset16_si;
    }

    impl StoreOp for I64Store16 {
        type Value = i64;
        type Immediate = i16;

        fn into_immediate = <i64 as Wrap<Self::Immediate>>::wrap;
        fn store_ss = Op::i64_store_wrap16_ss;
        fn store_si = Op::i64_store_wrap16_si;
        fn store_is = Op::i64_store_wrap16_is;
        fn store_ii = Op::i64_store_wrap16_ii;
        fn store_mem0_offset16_ss = Op::i64_store_wrap16_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i64_store_wrap16_mem0_offset16_si;
    }

    impl StoreOp for I64Store32 {
        type Value = i64;
        type Immediate = i32;

        fn into_immediate = <i64 as Wrap<Self::Immediate>>::wrap;
        fn store_ss = Op::i64_store_wrap32_ss;
        fn store_si = Op::i64_store_wrap32_si;
        fn store_is = Op::i64_store_wrap32_is;
        fn store_ii = Op::i64_store_wrap32_ii;
        fn store_mem0_offset16_ss = Op::i64_store_wrap32_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i64_store_wrap32_mem0_offset16_si;
    }
}
