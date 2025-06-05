use super::{FuncTranslator, LocalIdx};
use crate::{core::wasm, ir::Instruction, Error};
use wasmparser::VisitOperator;

macro_rules! impl_visit_operator {
    ( @mvp $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @sign_extension $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @saturating_float_to_int $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @bulk_memory $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @reference_types $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @tail_call $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @wide_arithmetic $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @@skipped $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $_ann:tt $($rest:tt)* ) => {
        // We skip Wasm operators that we already implement manually.
        impl_visit_operator!($($rest)*);
    };
    ( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $_ann:tt $($rest:tt)* ) => {
        // Wildcard match arm for all the other (yet) unsupported Wasm proposals.
        fn $visit(&mut self $($(, $arg: $argty)*)?) -> Self::Output {
            self.translate_unsupported_operator(stringify!($op))
        }
        impl_visit_operator!($($rest)*);
    };
    () => {};
}

impl FuncTranslator {
    /// Called when translating an unsupported Wasm operator.
    ///
    /// # Note
    ///
    /// We panic instead of returning an error because unsupported Wasm
    /// errors should have been filtered out by the validation procedure
    /// already, therefore encountering an unsupported Wasm operator
    /// in the function translation procedure can be considered a bug.
    pub fn translate_unsupported_operator(&self, name: &str) -> Result<(), Error> {
        panic!("tried to translate an unsupported Wasm operator: {name}")
    }
}

impl<'a> VisitOperator<'a> for FuncTranslator {
    type Output = Result<(), Error>;

    #[cfg(feature = "simd")]
    fn simd_visitor(
        &mut self,
    ) -> Option<&mut dyn wasmparser::VisitSimdOperator<'a, Output = Self::Output>> {
        Some(self)
    }

    wasmparser::for_each_visit_operator!(impl_visit_operator);

    fn visit_unreachable(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_nop(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_block(&mut self, blockty: wasmparser::BlockType) -> Self::Output {
        todo!()
    }

    fn visit_loop(&mut self, blockty: wasmparser::BlockType) -> Self::Output {
        todo!()
    }

    fn visit_if(&mut self, blockty: wasmparser::BlockType) -> Self::Output {
        todo!()
    }

    fn visit_else(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_end(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_br(&mut self, relative_depth: u32) -> Self::Output {
        todo!()
    }

    fn visit_br_if(&mut self, relative_depth: u32) -> Self::Output {
        todo!()
    }

    fn visit_br_table(&mut self, targets: wasmparser::BrTable<'a>) -> Self::Output {
        todo!()
    }

    fn visit_return(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_call(&mut self, function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_call_indirect(&mut self, type_index: u32, table_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_drop(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_select(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_local_get(&mut self, local_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_local(LocalIdx::from(local_index))?;
        Ok(())
    }

    fn visit_local_set(&mut self, local_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_local_tee(&mut self, local_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_global_get(&mut self, global_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_global_set(&mut self, global_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load32_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load32_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store32(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_memory_size(&mut self, mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_grow(&mut self, mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_const(&mut self, value: i32) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_immediate(value)?;
        Ok(())
    }

    fn visit_i64_const(&mut self, value: i64) -> Self::Output {
        todo!()
    }

    fn visit_f32_const(&mut self, value: wasmparser::Ieee32) -> Self::Output {
        todo!()
    }

    fn visit_f64_const(&mut self, value: wasmparser::Ieee64) -> Self::Output {
        todo!()
    }

    fn visit_i32_eqz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_lt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_lt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_gt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_gt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_le_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_le_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ge_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ge_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_eqz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_lt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_lt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_gt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_gt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_le_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_le_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ge_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ge_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_lt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_gt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_le(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_ge(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_lt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_gt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_le(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_ge(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_clz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ctz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_popcnt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_add(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, i32>(
            Instruction::i32_add,
            Instruction::i32_add_imm16,
            wasm::i32_add,
        )
    }

    fn visit_i32_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_div_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_div_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rem_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rem_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_and(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_or(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_xor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_shl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_shr_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_shr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rotl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rotr(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_clz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ctz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_popcnt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_div_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_div_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rem_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rem_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_and(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_or(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_xor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_shl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_shr_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_shr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rotl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rotr(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_ceil(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_floor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_trunc(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_nearest(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_sqrt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_div(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_min(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_max(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_copysign(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_ceil(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_floor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_trunc(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_nearest(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_sqrt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_div(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_min(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_max(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_copysign(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_wrap_i64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend_i32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend_i32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_demote_f64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_promote_f32(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_reinterpret_f32(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_reinterpret_f64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_reinterpret_i32(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_reinterpret_i64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_extend8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_extend16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_sat_f32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_sat_f32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_sat_f64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_sat_f64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_sat_f32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_sat_f32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_sat_f64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_sat_f64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_memory_init(&mut self, data_index: u32, mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_data_drop(&mut self, data_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_copy(&mut self, dst_mem: u32, src_mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_fill(&mut self, mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_init(&mut self, elem_index: u32, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_elem_drop(&mut self, elem_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_copy(&mut self, dst_table: u32, src_table: u32) -> Self::Output {
        todo!()
    }

    fn visit_typed_select(&mut self, ty: wasmparser::ValType) -> Self::Output {
        todo!()
    }

    fn visit_ref_null(&mut self, hty: wasmparser::HeapType) -> Self::Output {
        todo!()
    }

    fn visit_ref_is_null(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_ref_func(&mut self, function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_fill(&mut self, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_get(&mut self, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_set(&mut self, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_grow(&mut self, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_size(&mut self, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_return_call(&mut self, function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_return_call_indirect(&mut self, type_index: u32, table_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_add128(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_sub128(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_mul_wide_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_mul_wide_u(&mut self) -> Self::Output {
        todo!()
    }
}
