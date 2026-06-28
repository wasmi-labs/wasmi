use crate::{
    core::Typed,
    ir::{Address, Offset, Offset16, Op, Slot, index::Memory},
};

/// Trait implemented by all Wasm operators that can be translated as load extend instructions.
pub trait LoadOp {
    /// The type of the loaded value.
    type Result: Typed;

    fn op_rr(offset: Offset, memory: Memory) -> Op;
    fn op_rs(ptr: Slot, offset: Offset, memory: Memory) -> Op;
    fn op_ri(address: Address, memory: Memory) -> Op;
    fn op_rr_mem0_offset16(offset: Offset16) -> Op;
    fn op_rs_mem0_offset16(ptr: Slot, offset: Offset16) -> Op;
}

macro_rules! impl_load_extend {
    ( $(
        impl LoadOp for $name:ident {
            type Result = $result_ty:ty;

            fn op_rr = $store_rr:expr;
            fn op_rs = $store_rs:expr;
            fn op_ri = $store_ri:expr;
            fn op_rr_mem0_offset16 = $store_mem0_offset16_rr:expr;
            fn op_rs_mem0_offset16 = $store_mem0_offset16_rs:expr;
        }
    )* ) => {
        $(
            pub enum $name {}
            impl LoadOp for $name {
                type Result = $result_ty;

                fn op_rr(offset: Offset, memory: Memory) -> Op {
                    $store_rr(offset, memory)
                }

                fn op_rs(ptr: Slot, offset: Offset, memory: Memory) -> Op {
                    $store_rs(ptr, offset, memory)
                }

                fn op_ri(address: Address, memory: Memory) -> Op {
                    $store_ri(address, memory)
                }

                fn op_rr_mem0_offset16(offset: Offset16) -> Op {
                    $store_mem0_offset16_rr(offset)
                }

                fn op_rs_mem0_offset16(ptr: Slot, offset: Offset16) -> Op {
                    $store_mem0_offset16_rs(ptr, offset)
                }
            }
        )*
    };
}
impl_load_extend! {
    // load

    impl LoadOp for I32Load {
        type Result = u32;

        fn op_rr = Op::u32_load_rr;
        fn op_rs = Op::u32_load_rs;
        fn op_ri = Op::u32_load_ri;
        fn op_rr_mem0_offset16 = Op::u32_load_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::u32_load_mem0_offset16_rs;
    }

    impl LoadOp for I64Load {
        type Result = u64;

        fn op_rr = Op::u64_load_rr;
        fn op_rs = Op::u64_load_rs;
        fn op_ri = Op::u64_load_ri;
        fn op_rr_mem0_offset16 = Op::u64_load_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::u64_load_mem0_offset16_rs;
    }

    impl LoadOp for F32Load {
        type Result = f32;

        fn op_rr = Op::f32_load_rr;
        fn op_rs = Op::f32_load_rs;
        fn op_ri = Op::f32_load_ri;
        fn op_rr_mem0_offset16 = Op::f32_load_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::f32_load_mem0_offset16_rs;
    }

    impl LoadOp for F64Load {
        type Result = f64;

        fn op_rr = Op::f64_load_rr;
        fn op_rs = Op::f64_load_rs;
        fn op_ri = Op::f64_load_ri;
        fn op_rr_mem0_offset16 = Op::f64_load_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::f64_load_mem0_offset16_rs;
    }

    // i32: load-extend

    impl LoadOp for I32Load8 {
        type Result = i32;

        fn op_rr = Op::i32_load_extend8_rr;
        fn op_rs = Op::i32_load_extend8_rs;
        fn op_ri = Op::i32_load_extend8_ri;
        fn op_rr_mem0_offset16 = Op::i32_load_extend8_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::i32_load_extend8_mem0_offset16_rs;
    }

    impl LoadOp for I32Load16 {
        type Result = i32;

        fn op_rr = Op::i32_load_extend16_rr;
        fn op_rs = Op::i32_load_extend16_rs;
        fn op_ri = Op::i32_load_extend16_ri;
        fn op_rr_mem0_offset16 = Op::i32_load_extend16_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::i32_load_extend16_mem0_offset16_rs;
    }

    impl LoadOp for U32Load8 {
        type Result = u32;

        fn op_rr = Op::u32_load_extend8_rr;
        fn op_rs = Op::u32_load_extend8_rs;
        fn op_ri = Op::u32_load_extend8_ri;
        fn op_rr_mem0_offset16 = Op::u32_load_extend8_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::u32_load_extend8_mem0_offset16_rs;
    }

    impl LoadOp for U32Load16 {
        type Result = u32;

        fn op_rr = Op::u32_load_extend16_rr;
        fn op_rs = Op::u32_load_extend16_rs;
        fn op_ri = Op::u32_load_extend16_ri;
        fn op_rr_mem0_offset16 = Op::u32_load_extend16_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::u32_load_extend16_mem0_offset16_rs;
    }

    // i64: load-extend

    impl LoadOp for I64Load8 {
        type Result = i64;

        fn op_rr = Op::i64_load_extend8_rr;
        fn op_rs = Op::i64_load_extend8_rs;
        fn op_ri = Op::i64_load_extend8_ri;
        fn op_rr_mem0_offset16 = Op::i64_load_extend8_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::i64_load_extend8_mem0_offset16_rs;
    }

    impl LoadOp for I64Load16 {
        type Result = i64;

        fn op_rr = Op::i64_load_extend16_rr;
        fn op_rs = Op::i64_load_extend16_rs;
        fn op_ri = Op::i64_load_extend16_ri;
        fn op_rr_mem0_offset16 = Op::i64_load_extend16_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::i64_load_extend16_mem0_offset16_rs;
    }

    impl LoadOp for I64Load32 {
        type Result = i64;

        fn op_rr = Op::i64_load_extend32_rr;
        fn op_rs = Op::i64_load_extend32_rs;
        fn op_ri = Op::i64_load_extend32_ri;
        fn op_rr_mem0_offset16 = Op::i64_load_extend32_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::i64_load_extend32_mem0_offset16_rs;
    }

    impl LoadOp for U64Load8 {
        type Result = u64;

        fn op_rr = Op::u64_load_extend8_rr;
        fn op_rs = Op::u64_load_extend8_rs;
        fn op_ri = Op::u64_load_extend8_ri;
        fn op_rr_mem0_offset16 = Op::u64_load_extend8_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::u64_load_extend8_mem0_offset16_rs;
    }

    impl LoadOp for U64Load16 {
        type Result = u64;

        fn op_rr = Op::u64_load_extend16_rr;
        fn op_rs = Op::u64_load_extend16_rs;
        fn op_ri = Op::u64_load_extend16_ri;
        fn op_rr_mem0_offset16 = Op::u64_load_extend16_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::u64_load_extend16_mem0_offset16_rs;
    }

    impl LoadOp for U64Load32 {
        type Result = u64;

        fn op_rr = Op::u64_load_extend32_rr;
        fn op_rs = Op::u64_load_extend32_rs;
        fn op_ri = Op::u64_load_extend32_ri;
        fn op_rr_mem0_offset16 = Op::u64_load_extend32_mem0_offset16_rr;
        fn op_rs_mem0_offset16 = Op::u64_load_extend32_mem0_offset16_rs;
    }
}
