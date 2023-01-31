use super::{FuncBuilder, FuncValidator, TranslationError};
use wasmparser::{BinaryReaderError, VisitOperator};

/// A helper macro to conveniently iterate over all opcodes supported by this
/// crate. This can be used to work with either the [`Operator`] enumeration or
/// the [`VisitOperator`] trait if your use case uniformly handles all operators
/// the same way.
///
/// This is an "iterator macro" where this macro is invoked with the name of
/// another macro, and then that macro is invoked with the list of all
/// operators.
///
/// This macro is heavily inspired by [`wasmparser::for_each_operator`] macro
/// and represents the subset of Wasm operators from unsupported Wasm proposals
/// that introduce many new operators.
///
/// [`wasmparser::for_each_operator`]:
/// https://docs.rs/wasmparser/0.90.0/wasmparser/macro.for_each_operator.html
///
/// [`Operator`]: [`wasmparser::Operator`]
/// [`VisitOperator`]: [`wasmparser::VisitOperator`]
macro_rules! for_each_supported_operator {
    ($mac:ident) => {
        $mac! {
            fn visit_unreachable() => fn translate_unreachable
            fn visit_nop() => fn translate_nop
            fn visit_block(ty: wasmparser::BlockType) => fn translate_block
            fn visit_loop(ty: wasmparser::BlockType) => fn translate_loop
            fn visit_if(ty: wasmparser::BlockType) => fn translate_if
            fn visit_else() => fn translate_else
            fn visit_end() => fn translate_end
            fn visit_br(relative_depth: u32) => fn translate_br
            fn visit_br_if(relative_depth: u32) => fn translate_br_if
            // fn visit_br_table(table: wasmparser::BrTable<'parser>) => fn translate_br_table
            fn visit_return() => fn translate_return
            fn visit_call(function_index: u32) => fn translate_call
            fn visit_call_indirect(index: u32, table_index: u32, table_byte: u8) => fn translate_call_indirect
            fn visit_drop() => fn translate_drop
            fn visit_select() => fn translate_select
            fn visit_typed_select(ty: wasmparser::ValType) => fn translate_typed_select
            fn visit_ref_null(ty: wasmparser::ValType) => fn translate_ref_null
            fn visit_ref_is_null() => fn translate_ref_is_null
            fn visit_ref_func(func_index: u32) => fn translate_ref_func
            fn visit_local_get(local_index: u32) => fn translate_local_get
            fn visit_local_set(local_index: u32) => fn translate_local_set
            fn visit_local_tee(local_index: u32) => fn translate_local_tee
            fn visit_global_get(global_index: u32) => fn translate_global_get
            fn visit_global_set(global_index: u32) => fn translate_global_set
            fn visit_i32_load(memarg: wasmparser::MemArg) => fn translate_i32_load
            fn visit_i64_load(memarg: wasmparser::MemArg) => fn translate_i64_load
            fn visit_f32_load(memarg: wasmparser::MemArg) => fn translate_f32_load
            fn visit_f64_load(memarg: wasmparser::MemArg) => fn translate_f64_load
            fn visit_i32_load8_s(memarg: wasmparser::MemArg) => fn translate_i32_load8_s
            fn visit_i32_load8_u(memarg: wasmparser::MemArg) => fn translate_i32_load8_u
            fn visit_i32_load16_s(memarg: wasmparser::MemArg) => fn translate_i32_load16_s
            fn visit_i32_load16_u(memarg: wasmparser::MemArg) => fn translate_i32_load16_u
            fn visit_i64_load8_s(memarg: wasmparser::MemArg) => fn translate_i64_load8_s
            fn visit_i64_load8_u(memarg: wasmparser::MemArg) => fn translate_i64_load8_u
            fn visit_i64_load16_s(memarg: wasmparser::MemArg) => fn translate_i64_load16_s
            fn visit_i64_load16_u(memarg: wasmparser::MemArg) => fn translate_i64_load16_u
            fn visit_i64_load32_s(memarg: wasmparser::MemArg) => fn translate_i64_load32_s
            fn visit_i64_load32_u(memarg: wasmparser::MemArg) => fn translate_i64_load32_u
            fn visit_i32_store(memarg: wasmparser::MemArg) => fn translate_i32_store
            fn visit_i64_store(memarg: wasmparser::MemArg) => fn translate_i64_store
            fn visit_f32_store(memarg: wasmparser::MemArg) => fn translate_f32_store
            fn visit_f64_store(memarg: wasmparser::MemArg) => fn translate_f64_store
            fn visit_i32_store8(memarg: wasmparser::MemArg) => fn translate_i32_store8
            fn visit_i32_store16(memarg: wasmparser::MemArg) => fn translate_i32_store16
            fn visit_i64_store8(memarg: wasmparser::MemArg) => fn translate_i64_store8
            fn visit_i64_store16(memarg: wasmparser::MemArg) => fn translate_i64_store16
            fn visit_i64_store32(memarg: wasmparser::MemArg) => fn translate_i64_store32
            fn visit_memory_size(mem: u32, mem_byte: u8) => fn translate_memory_size
            fn visit_memory_grow(mem: u32, mem_byte: u8) => fn translate_memory_grow
            fn visit_memory_copy(dst_mem: u32, src_mem: u32) => fn translate_memory_copy
            fn visit_memory_fill(mem: u32) => fn translate_memory_fill
            fn visit_memory_init(seg: u32, mem: u32) => fn translate_memory_init
            fn visit_data_drop(seg: u32) => fn translate_data_drop
            fn visit_table_size(table: u32) => fn translate_table_size
            fn visit_table_grow(table: u32) => fn translate_table_grow
            fn visit_table_copy(dst_table: u32, src_table: u32) => fn translate_table_copy
            fn visit_table_fill(table: u32) => fn translate_table_fill
            fn visit_table_get(table: u32) => fn translate_table_get
            fn visit_table_set(table: u32) => fn translate_table_set
            fn visit_table_init(seg: u32, table: u32) => fn translate_table_init
            fn visit_elem_drop(seg: u32) => fn translate_elem_drop
            fn visit_i32_const(value: i32) => fn translate_i32_const
            fn visit_i64_const(value: i64) => fn translate_i64_const
            fn visit_f32_const(value: wasmparser::Ieee32) => fn translate_f32_const
            fn visit_f64_const(value: wasmparser::Ieee64) => fn translate_f64_const
            fn visit_i32_eqz() => fn translate_i32_eqz
            fn visit_i32_eq() => fn translate_i32_eq
            fn visit_i32_ne() => fn translate_i32_ne
            fn visit_i32_lt_s() => fn translate_i32_lt_s
            fn visit_i32_lt_u() => fn translate_i32_lt_u
            fn visit_i32_gt_s() => fn translate_i32_gt_s
            fn visit_i32_gt_u() => fn translate_i32_gt_u
            fn visit_i32_le_s() => fn translate_i32_le_s
            fn visit_i32_le_u() => fn translate_i32_le_u
            fn visit_i32_ge_s() => fn translate_i32_ge_s
            fn visit_i32_ge_u() => fn translate_i32_ge_u
            fn visit_i64_eqz() => fn translate_i64_eqz
            fn visit_i64_eq() => fn translate_i64_eq
            fn visit_i64_ne() => fn translate_i64_ne
            fn visit_i64_lt_s() => fn translate_i64_lt_s
            fn visit_i64_lt_u() => fn translate_i64_lt_u
            fn visit_i64_gt_s() => fn translate_i64_gt_s
            fn visit_i64_gt_u() => fn translate_i64_gt_u
            fn visit_i64_le_s() => fn translate_i64_le_s
            fn visit_i64_le_u() => fn translate_i64_le_u
            fn visit_i64_ge_s() => fn translate_i64_ge_s
            fn visit_i64_ge_u() => fn translate_i64_ge_u
            fn visit_f32_eq() => fn translate_f32_eq
            fn visit_f32_ne() => fn translate_f32_ne
            fn visit_f32_lt() => fn translate_f32_lt
            fn visit_f32_gt() => fn translate_f32_gt
            fn visit_f32_le() => fn translate_f32_le
            fn visit_f32_ge() => fn translate_f32_ge
            fn visit_f64_eq() => fn translate_f64_eq
            fn visit_f64_ne() => fn translate_f64_ne
            fn visit_f64_lt() => fn translate_f64_lt
            fn visit_f64_gt() => fn translate_f64_gt
            fn visit_f64_le() => fn translate_f64_le
            fn visit_f64_ge() => fn translate_f64_ge
            fn visit_i32_clz() => fn translate_i32_clz
            fn visit_i32_ctz() => fn translate_i32_ctz
            fn visit_i32_popcnt() => fn translate_i32_popcnt
            fn visit_i32_add() => fn translate_i32_add
            fn visit_i32_sub() => fn translate_i32_sub
            fn visit_i32_mul() => fn translate_i32_mul
            fn visit_i32_div_s() => fn translate_i32_div_s
            fn visit_i32_div_u() => fn translate_i32_div_u
            fn visit_i32_rem_s() => fn translate_i32_rem_s
            fn visit_i32_rem_u() => fn translate_i32_rem_u
            fn visit_i32_and() => fn translate_i32_and
            fn visit_i32_or() => fn translate_i32_or
            fn visit_i32_xor() => fn translate_i32_xor
            fn visit_i32_shl() => fn translate_i32_shl
            fn visit_i32_shr_s() => fn translate_i32_shr_s
            fn visit_i32_shr_u() => fn translate_i32_shr_u
            fn visit_i32_rotl() => fn translate_i32_rotl
            fn visit_i32_rotr() => fn translate_i32_rotr
            fn visit_i64_clz() => fn translate_i64_clz
            fn visit_i64_ctz() => fn translate_i64_ctz
            fn visit_i64_popcnt() => fn translate_i64_popcnt
            fn visit_i64_add() => fn translate_i64_add
            fn visit_i64_sub() => fn translate_i64_sub
            fn visit_i64_mul() => fn translate_i64_mul
            fn visit_i64_div_s() => fn translate_i64_div_s
            fn visit_i64_div_u() => fn translate_i64_div_u
            fn visit_i64_rem_s() => fn translate_i64_rem_s
            fn visit_i64_rem_u() => fn translate_i64_rem_u
            fn visit_i64_and() => fn translate_i64_and
            fn visit_i64_or() => fn translate_i64_or
            fn visit_i64_xor() => fn translate_i64_xor
            fn visit_i64_shl() => fn translate_i64_shl
            fn visit_i64_shr_s() => fn translate_i64_shr_s
            fn visit_i64_shr_u() => fn translate_i64_shr_u
            fn visit_i64_rotl() => fn translate_i64_rotl
            fn visit_i64_rotr() => fn translate_i64_rotr
            fn visit_f32_abs() => fn translate_f32_abs
            fn visit_f32_neg() => fn translate_f32_neg
            fn visit_f32_ceil() => fn translate_f32_ceil
            fn visit_f32_floor() => fn translate_f32_floor
            fn visit_f32_trunc() => fn translate_f32_trunc
            fn visit_f32_nearest() => fn translate_f32_nearest
            fn visit_f32_sqrt() => fn translate_f32_sqrt
            fn visit_f32_add() => fn translate_f32_add
            fn visit_f32_sub() => fn translate_f32_sub
            fn visit_f32_mul() => fn translate_f32_mul
            fn visit_f32_div() => fn translate_f32_div
            fn visit_f32_min() => fn translate_f32_min
            fn visit_f32_max() => fn translate_f32_max
            fn visit_f32_copysign() => fn translate_f32_copysign
            fn visit_f64_abs() => fn translate_f64_abs
            fn visit_f64_neg() => fn translate_f64_neg
            fn visit_f64_ceil() => fn translate_f64_ceil
            fn visit_f64_floor() => fn translate_f64_floor
            fn visit_f64_trunc() => fn translate_f64_trunc
            fn visit_f64_nearest() => fn translate_f64_nearest
            fn visit_f64_sqrt() => fn translate_f64_sqrt
            fn visit_f64_add() => fn translate_f64_add
            fn visit_f64_sub() => fn translate_f64_sub
            fn visit_f64_mul() => fn translate_f64_mul
            fn visit_f64_div() => fn translate_f64_div
            fn visit_f64_min() => fn translate_f64_min
            fn visit_f64_max() => fn translate_f64_max
            fn visit_f64_copysign() => fn translate_f64_copysign
            fn visit_i32_wrap_i64() => fn translate_i32_wrap_i64
            fn visit_i32_trunc_f32_s() => fn translate_i32_trunc_f32_s
            fn visit_i32_trunc_f32_u() => fn translate_i32_trunc_f32_u
            fn visit_i32_trunc_f64_s() => fn translate_i32_trunc_f64_s
            fn visit_i32_trunc_f64_u() => fn translate_i32_trunc_f64_u
            fn visit_i64_extend_i32_s() => fn translate_i64_extend_i32_s
            fn visit_i64_extend_i32_u() => fn translate_i64_extend_i32_u
            fn visit_i64_trunc_f32_s() => fn translate_i64_trunc_f32_s
            fn visit_i64_trunc_f32_u() => fn translate_i64_trunc_f32_u
            fn visit_i64_trunc_f64_s() => fn translate_i64_trunc_f64_s
            fn visit_i64_trunc_f64_u() => fn translate_i64_trunc_f64_u
            fn visit_f32_convert_i32_s() => fn translate_f32_convert_i32_s
            fn visit_f32_convert_i32_u() => fn translate_f32_convert_i32_u
            fn visit_f32_convert_i64_s() => fn translate_f32_convert_i64_s
            fn visit_f32_convert_i64_u() => fn translate_f32_convert_i64_u
            fn visit_f32_demote_f64() => fn translate_f32_demote_f64
            fn visit_f64_convert_i32_s() => fn translate_f64_convert_i32_s
            fn visit_f64_convert_i32_u() => fn translate_f64_convert_i32_u
            fn visit_f64_convert_i64_s() => fn translate_f64_convert_i64_s
            fn visit_f64_convert_i64_u() => fn translate_f64_convert_i64_u
            fn visit_f64_promote_f32() => fn translate_f64_promote_f32
            fn visit_i32_reinterpret_f32() => fn translate_i32_reinterpret_f32
            fn visit_i64_reinterpret_f64() => fn translate_i64_reinterpret_f64
            fn visit_f32_reinterpret_i32() => fn translate_f32_reinterpret_i32
            fn visit_f64_reinterpret_i64() => fn translate_f64_reinterpret_i64
            fn visit_i32_extend8_s() => fn translate_i32_extend8_s
            fn visit_i32_extend16_s() => fn translate_i32_extend16_s
            fn visit_i64_extend8_s() => fn translate_i64_extend8_s
            fn visit_i64_extend16_s() => fn translate_i64_extend16_s
            fn visit_i64_extend32_s() => fn translate_i64_extend32_s
            fn visit_i32_trunc_sat_f32_s() => fn translate_i32_trunc_sat_f32_s
            fn visit_i32_trunc_sat_f32_u() => fn translate_i32_trunc_sat_f32_u
            fn visit_i32_trunc_sat_f64_s() => fn translate_i32_trunc_sat_f64_s
            fn visit_i32_trunc_sat_f64_u() => fn translate_i32_trunc_sat_f64_u
            fn visit_i64_trunc_sat_f32_s() => fn translate_i64_trunc_sat_f32_s
            fn visit_i64_trunc_sat_f32_u() => fn translate_i64_trunc_sat_f32_u
            fn visit_i64_trunc_sat_f64_s() => fn translate_i64_trunc_sat_f64_s
            fn visit_i64_trunc_sat_f64_u() => fn translate_i64_trunc_sat_f64_u
        }
    }
}

impl<'parser> FuncBuilder<'parser> {
    /// Translates into `wasmi` bytecode if the current code path is reachable.
    fn validate_then_translate<V, F>(
        &mut self,
        validate: V,
        translator: F,
    ) -> Result<(), TranslationError>
    where
        V: FnOnce(&mut FuncValidator) -> Result<(), BinaryReaderError>,
        F: FnOnce(&mut Self) -> Result<(), TranslationError>,
    {
        validate(&mut self.validator)?;
        translator(self)?;
        Ok(())
    }
}

macro_rules! define_supported_visit_operator {
    ($( fn $visit:ident $(( $($arg:ident: $argty:ty),* ))? => fn $translate:ident)*) => {
        $(
            fn $visit(&mut self, offset: usize $($(,$arg: $argty)*)?) -> Self::Output {
                self.validate_then_translate(
                    |v| v.$visit(offset $($(,$arg)*)?),
                    |this| {
                        this.$translate($($($arg),*)?)
                    },
                )
            }
        )*
    };
}

macro_rules! define_unsupported_visit_operator {
    ( @mvp $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // Supported operators are handled by `define_supported_visit_operator`.
        define_unsupported_visit_operator!($($rest)*);
    };
    ( @sign_ext_ops $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // Supported operators are handled by `define_supported_visit_operator`.
        define_unsupported_visit_operator!($($rest)*);
    };
    ( @non_trapping_f2i_conversions $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // Supported operators are handled by `define_supported_visit_operator`.
        define_unsupported_visit_operator!($($rest)*);
    };
    ( @bulk_memory $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // Supported operators are handled by `define_supported_visit_operator`.
        define_unsupported_visit_operator!($($rest)*);
    };
    ( @reference_types $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // Supported operators are handled by `define_supported_visit_operator`.
        define_unsupported_visit_operator!($($rest)*);
    };
    ( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        fn $visit(&mut self, offset: usize $($(,$arg: $argty)*)?) -> Self::Output {
            self.validator.$visit(offset $($(,$arg)*)?).map_err(::core::convert::Into::into)
        }
        define_unsupported_visit_operator!($($rest)*);
    };
    () => {};
}

impl<'a> VisitOperator<'a> for FuncBuilder<'a> {
    type Output = Result<(), TranslationError>;

    for_each_supported_operator!(define_supported_visit_operator);
    wasmparser::for_each_operator!(define_unsupported_visit_operator);

    fn visit_br_table(&mut self, offset: usize, table: wasmparser::BrTable<'a>) -> Self::Output {
        let table_cloned = table.clone();
        self.validate_then_translate(
            |v| v.visit_br_table(offset, table_cloned),
            |this| this.translate_br_table(table),
        )
    }
}
