use crate::{
    core::Typed,
    engine::translator::utils::{ToBits, Wrap},
    ir::{Address, Offset, Offset16, Op, Slot, index::Memory},
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

    fn store_rr(offset: Offset, memory: Memory) -> Option<Op> {
        _ = (offset, memory);
        None
    }
    fn store_rs(offset: Offset, value: Slot, memory: Memory) -> Op;
    fn store_ri(offset: Offset, value: Self::Immediate, memory: Memory) -> Op;
    fn store_sr(ptr: Slot, offset: Offset, memory: Memory) -> Op;
    fn store_ss(ptr: Slot, offset: Offset, value: Slot, memory: Memory) -> Op;
    fn store_si(ptr: Slot, offset: Offset, value: Self::Immediate, memory: Memory) -> Op;
    fn store_ir(address: Address, memory: Memory) -> Op;
    fn store_is(address: Address, value: Slot, memory: Memory) -> Op;
    fn store_ii(address: Address, value: Self::Immediate, memory: Memory) -> Op;

    fn store_mem0_offset16_rr(offset: Offset16) -> Option<Op> {
        _ = offset;
        None
    }
    fn store_mem0_offset16_rs(offset: Offset16, value: Slot) -> Op;
    fn store_mem0_offset16_ri(offset: Offset16, value: Self::Immediate) -> Op;
    fn store_mem0_offset16_sr(ptr: Slot, offset: Offset16) -> Op;
    fn store_mem0_offset16_ss(ptr: Slot, offset: Offset16, value: Slot) -> Op;
    fn store_mem0_offset16_si(ptr: Slot, offset: Offset16, value: Self::Immediate) -> Op;
}

macro_rules! impl_store_wrap {
    ( $(
        impl StoreOp for $name:ident {
            type Value = $value_ty:ty;
            type Immediate = $immediate_ty:ty;

            fn into_immediate = $apply:expr;

            $( fn store_rr = $store_rr:expr; )?
            fn store_rs = $store_rs:expr;
            fn store_ri = $store_ri:expr;
            fn store_sr = $store_sr:expr;
            fn store_ss = $store_ss:expr;
            fn store_si = $store_si:expr;
            fn store_ir = $store_ir:expr;
            fn store_is = $store_is:expr;
            fn store_ii = $store_ii:expr;

            $( fn store_mem0_offset16_rr = $store_mem0_offset16_rr:expr; )?
            fn store_mem0_offset16_rs = $store_mem0_offset16_rs:expr;
            fn store_mem0_offset16_ri = $store_mem0_offset16_ri:expr;
            fn store_mem0_offset16_sr = $store_mem0_offset16_sr:expr;
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

                $(
                    fn store_rr(offset: Offset, memory: Memory) -> Option<Op> {
                        Some($store_rr(offset, memory))
                    }
                )?

                fn store_rs(offset: Offset, value: Slot, memory: Memory) -> Op {
                    $store_rs(offset, value, memory)
                }

                fn store_ri(offset: Offset, value: Self::Immediate, memory: Memory) -> Op {
                    $store_ri(offset, value, memory)
                }

                fn store_sr(ptr: Slot, offset: Offset, memory: Memory) -> Op {
                    $store_sr(ptr, offset, memory)
                }

                fn store_ss(ptr: Slot, offset: Offset, value: Slot, memory: Memory) -> Op {
                    $store_ss(ptr, offset, value, memory)
                }

                fn store_si(ptr: Slot, offset: Offset, value: Self::Immediate, memory: Memory) -> Op {
                    $store_si(ptr, offset, value, memory)
                }

                fn store_ir(address: Address, memory: Memory) -> Op {
                    $store_ir(address, memory)
                }

                fn store_is(address: Address, value: Slot, memory: Memory) -> Op {
                    $store_is(address, value, memory)
                }

                fn store_ii(address: Address, value: Self::Immediate, memory: Memory) -> Op {
                    $store_ii(address, value, memory)
                }

                $(
                    fn store_mem0_offset16_rr(offset: Offset16) -> Option<Op> {
                        Some($store_mem0_offset16_rr(offset))
                    }
                )?

                fn store_mem0_offset16_rs(offset: Offset16, value: Slot) -> Op {
                    $store_mem0_offset16_rs(offset, value)
                }

                fn store_mem0_offset16_ri(offset: Offset16, value: Self::Immediate) -> Op {
                    $store_mem0_offset16_ri(offset, value)
                }

                fn store_mem0_offset16_sr(ptr: Slot, offset: Offset16) -> Op {
                    $store_mem0_offset16_sr(ptr, offset)
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

        fn store_rs = Op::u32_store_rs;
        fn store_ri = Op::u32_store_ri;
        fn store_sr = Op::u32_store_sr;
        fn store_ss = Op::u32_store_ss;
        fn store_si = Op::u32_store_si;
        fn store_ir = Op::u32_store_ir;
        fn store_is = Op::u32_store_is;
        fn store_ii = Op::u32_store_ii;

        fn store_mem0_offset16_rs = Op::u32_store_mem0_offset16_rs;
        fn store_mem0_offset16_ri = Op::u32_store_mem0_offset16_ri;
        fn store_mem0_offset16_sr = Op::u32_store_mem0_offset16_sr;
        fn store_mem0_offset16_ss = Op::u32_store_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::u32_store_mem0_offset16_si;
    }

    impl StoreOp for I64Store {
        type Value = i64;
        type Immediate = u64;

        fn into_immediate = <i64 as ToBits>::to_bits;

        fn store_rs = Op::u64_store_rs;
        fn store_ri = Op::u64_store_ri;
        fn store_sr = Op::u64_store_sr;
        fn store_ss = Op::u64_store_ss;
        fn store_si = Op::u64_store_si;
        fn store_ir = Op::u64_store_ir;
        fn store_is = Op::u64_store_is;
        fn store_ii = Op::u64_store_ii;

        fn store_mem0_offset16_rs = Op::u64_store_mem0_offset16_rs;
        fn store_mem0_offset16_ri = Op::u64_store_mem0_offset16_ri;
        fn store_mem0_offset16_sr = Op::u64_store_mem0_offset16_sr;
        fn store_mem0_offset16_ss = Op::u64_store_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::u64_store_mem0_offset16_si;
    }

    impl StoreOp for F32Store {
        type Value = f32;
        type Immediate = u32;

        fn into_immediate = <f32 as ToBits>::to_bits;

        fn store_rr = Op::f32_store_rr;
        fn store_rs = Op::u32_store_rs;
        fn store_ri = Op::u32_store_ri;
        fn store_sr = Op::f32_store_sr;
        fn store_ss = Op::u32_store_ss;
        fn store_si = Op::u32_store_si;
        fn store_ir = Op::f32_store_ir;
        fn store_is = Op::u32_store_is;
        fn store_ii = Op::u32_store_ii;

        fn store_mem0_offset16_rr = Op::f32_store_mem0_offset16_rr;
        fn store_mem0_offset16_rs = Op::u32_store_mem0_offset16_rs;
        fn store_mem0_offset16_ri = Op::u32_store_mem0_offset16_ri;
        fn store_mem0_offset16_sr = Op::f32_store_mem0_offset16_sr;
        fn store_mem0_offset16_ss = Op::u32_store_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::u32_store_mem0_offset16_si;
    }

    impl StoreOp for F64Store {
        type Value = f64;
        type Immediate = u64;

        fn into_immediate = <f64 as ToBits>::to_bits;

        fn store_rr = Op::f64_store_rr;
        fn store_rs = Op::u64_store_rs;
        fn store_ri = Op::u64_store_ri;
        fn store_sr = Op::f64_store_sr;
        fn store_ss = Op::u64_store_ss;
        fn store_si = Op::u64_store_si;
        fn store_ir = Op::f64_store_ir;
        fn store_is = Op::u64_store_is;
        fn store_ii = Op::u64_store_ii;

        fn store_mem0_offset16_rr = Op::f64_store_mem0_offset16_rr;
        fn store_mem0_offset16_rs = Op::u64_store_mem0_offset16_rs;
        fn store_mem0_offset16_ri = Op::u64_store_mem0_offset16_ri;
        fn store_mem0_offset16_sr = Op::f64_store_mem0_offset16_sr;
        fn store_mem0_offset16_ss = Op::u64_store_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::u64_store_mem0_offset16_si;
    }

    impl StoreOp for I32Store8 {
        type Value = i32;
        type Immediate = i8;

        fn into_immediate = <i32 as Wrap<Self::Immediate>>::wrap;

        fn store_rs = Op::i32_store_wrap8_rs;
        fn store_ri = Op::i32_store_wrap8_ri;
        fn store_sr = Op::i32_store_wrap8_sr;
        fn store_ss = Op::i32_store_wrap8_ss;
        fn store_si = Op::i32_store_wrap8_si;
        fn store_ir = Op::i32_store_wrap8_ir;
        fn store_is = Op::i32_store_wrap8_is;
        fn store_ii = Op::i32_store_wrap8_ii;

        fn store_mem0_offset16_rs = Op::i32_store_wrap8_mem0_offset16_rs;
        fn store_mem0_offset16_ri = Op::i32_store_wrap8_mem0_offset16_ri;
        fn store_mem0_offset16_sr = Op::i32_store_wrap8_mem0_offset16_sr;
        fn store_mem0_offset16_ss = Op::i32_store_wrap8_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i32_store_wrap8_mem0_offset16_si;
    }

    impl StoreOp for I32Store16 {
        type Value = i32;
        type Immediate = i16;

        fn into_immediate = <i32 as Wrap<Self::Immediate>>::wrap;

        fn store_rs = Op::i32_store_wrap16_rs;
        fn store_ri = Op::i32_store_wrap16_ri;
        fn store_sr = Op::i32_store_wrap16_sr;
        fn store_ss = Op::i32_store_wrap16_ss;
        fn store_si = Op::i32_store_wrap16_si;
        fn store_ir = Op::i32_store_wrap16_ir;
        fn store_is = Op::i32_store_wrap16_is;
        fn store_ii = Op::i32_store_wrap16_ii;

        fn store_mem0_offset16_rs = Op::i32_store_wrap16_mem0_offset16_rs;
        fn store_mem0_offset16_ri = Op::i32_store_wrap16_mem0_offset16_ri;
        fn store_mem0_offset16_sr = Op::i32_store_wrap16_mem0_offset16_sr;
        fn store_mem0_offset16_ss = Op::i32_store_wrap16_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i32_store_wrap16_mem0_offset16_si;
    }

    impl StoreOp for I64Store8 {
        type Value = i64;
        type Immediate = i8;

        fn into_immediate = <i64 as Wrap<Self::Immediate>>::wrap;

        fn store_rs = Op::i64_store_wrap8_rs;
        fn store_ri = Op::i64_store_wrap8_ri;
        fn store_sr = Op::i64_store_wrap8_sr;
        fn store_ss = Op::i64_store_wrap8_ss;
        fn store_si = Op::i64_store_wrap8_si;
        fn store_ir = Op::i64_store_wrap8_ir;
        fn store_is = Op::i64_store_wrap8_is;
        fn store_ii = Op::i64_store_wrap8_ii;

        fn store_mem0_offset16_rs = Op::i64_store_wrap8_mem0_offset16_rs;
        fn store_mem0_offset16_ri = Op::i64_store_wrap8_mem0_offset16_ri;
        fn store_mem0_offset16_sr = Op::i64_store_wrap8_mem0_offset16_sr;
        fn store_mem0_offset16_ss = Op::i64_store_wrap8_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i64_store_wrap8_mem0_offset16_si;
    }

    impl StoreOp for I64Store16 {
        type Value = i64;
        type Immediate = i16;

        fn into_immediate = <i64 as Wrap<Self::Immediate>>::wrap;

        fn store_rs = Op::i64_store_wrap16_rs;
        fn store_ri = Op::i64_store_wrap16_ri;
        fn store_sr = Op::i64_store_wrap16_sr;
        fn store_ss = Op::i64_store_wrap16_ss;
        fn store_si = Op::i64_store_wrap16_si;
        fn store_ir = Op::i64_store_wrap16_ir;
        fn store_is = Op::i64_store_wrap16_is;
        fn store_ii = Op::i64_store_wrap16_ii;

        fn store_mem0_offset16_rs = Op::i64_store_wrap16_mem0_offset16_rs;
        fn store_mem0_offset16_ri = Op::i64_store_wrap16_mem0_offset16_ri;
        fn store_mem0_offset16_sr = Op::i64_store_wrap16_mem0_offset16_sr;
        fn store_mem0_offset16_ss = Op::i64_store_wrap16_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i64_store_wrap16_mem0_offset16_si;
    }

    impl StoreOp for I64Store32 {
        type Value = i64;
        type Immediate = i32;

        fn into_immediate = <i64 as Wrap<Self::Immediate>>::wrap;

        fn store_rs = Op::i64_store_wrap32_rs;
        fn store_ri = Op::i64_store_wrap32_ri;
        fn store_sr = Op::i64_store_wrap32_sr;
        fn store_ss = Op::i64_store_wrap32_ss;
        fn store_si = Op::i64_store_wrap32_si;
        fn store_ir = Op::i64_store_wrap32_ir;
        fn store_is = Op::i64_store_wrap32_is;
        fn store_ii = Op::i64_store_wrap32_ii;

        fn store_mem0_offset16_rs = Op::i64_store_wrap32_mem0_offset16_rs;
        fn store_mem0_offset16_ri = Op::i64_store_wrap32_mem0_offset16_ri;
        fn store_mem0_offset16_sr = Op::i64_store_wrap32_mem0_offset16_sr;
        fn store_mem0_offset16_ss = Op::i64_store_wrap32_mem0_offset16_ss;
        fn store_mem0_offset16_si = Op::i64_store_wrap32_mem0_offset16_si;
    }
}
