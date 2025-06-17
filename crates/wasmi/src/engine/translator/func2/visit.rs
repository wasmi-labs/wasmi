use super::{ControlFrame, FuncTranslator, LocalIdx, UnreachableControlFrame};
use crate::{
    core::wasm,
    engine::{
        translator::func2::{stack::IfReachability, Operand},
        BlockType,
    },
    ir::Instruction,
    Error,
};
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

    fn visit_block(&mut self, block_ty: wasmparser::BlockType) -> Self::Output {
        if !self.reachable {
            self.stack
                .push_unreachable(UnreachableControlFrame::Block)?;
            return Ok(());
        }
        let block_ty = BlockType::new(block_ty, &self.module);
        let end_label = self.labels.new_label();
        self.stack.push_block(block_ty, end_label)?;
        Ok(())
    }

    fn visit_loop(&mut self, block_ty: wasmparser::BlockType) -> Self::Output {
        if !self.reachable {
            self.stack.push_unreachable(UnreachableControlFrame::Loop)?;
            return Ok(());
        }
        let block_ty = BlockType::new(block_ty, &self.module);
        let len_params = block_ty.len_params(&self.engine);
        let continue_label = self.labels.new_label();
        let consume_fuel = self.stack.consume_fuel_instr();
        self.copy_branch_params(usize::from(len_params), consume_fuel)?;
        self.pin_label(continue_label);
        let consume_fuel = self.instrs.push_consume_fuel_instr()?;
        self.stack
            .push_loop(block_ty, continue_label, consume_fuel)?;
        Ok(())
    }

    fn visit_if(&mut self, block_ty: wasmparser::BlockType) -> Self::Output {
        if !self.reachable {
            self.stack.push_unreachable(UnreachableControlFrame::If)?;
            return Ok(());
        }
        let end_label = self.labels.new_label();
        let condition = self.stack.pop();
        let (reachability, consume_fuel_instr) = match condition {
            Operand::Immediate(operand) => {
                let condition = i32::from(operand.val());
                let reachability = match condition {
                    0 => {
                        self.reachable = false;
                        IfReachability::OnlyElse
                    }
                    _ => IfReachability::OnlyThen,
                };
                let consume_fuel_instr = self.stack.consume_fuel_instr();
                (reachability, consume_fuel_instr)
            }
            _ => {
                let else_label = self.labels.new_label();
                self.translate_br_eqz(condition, else_label)?;
                let reachability = IfReachability::Both { else_label };
                let consume_fuel_instr = self.instrs.push_consume_fuel_instr()?;
                (reachability, consume_fuel_instr)
            }
        };
        let block_ty = BlockType::new(block_ty, &self.module);
        self.stack
            .push_if(block_ty, end_label, reachability, consume_fuel_instr)?;
        Ok(())
    }

    fn visit_else(&mut self) -> Self::Output {
        let mut frame = match self.stack.pop_control() {
            ControlFrame::If(frame) => frame,
            ControlFrame::Unreachable(UnreachableControlFrame::If) => {
                debug_assert!(!self.reachable);
                self.stack.push_unreachable(UnreachableControlFrame::Else)?;
                return Ok(());
            }
            unexpected => panic!("expected `if` control frame but found: {unexpected:?}"),
        };
        // After `then` block, before `else` block:
        // - Copy `if` branch parameters.
        // - Branch from end of `then` to end of `if`.
        let is_end_of_then_reachable = self.reachable;
        if let Some(else_label) = frame.else_label() {
            debug_assert!(frame.is_then_reachable() && frame.is_else_reachable());
            if is_end_of_then_reachable {
                let len_values = usize::from(frame.ty().len_results(&self.engine));
                let consume_fuel_instr = frame.consume_fuel_instr();
                self.copy_branch_params(len_values, consume_fuel_instr)?;
                frame.branch_to();
                self.translate_br(else_label)?;
            }
        }
        // Start of `else` block:
        if let Some(else_label) = frame.else_label() {
            self.labels
                .pin_label(else_label, self.instrs.next_instr())
                .unwrap();
        }
        let consume_fuel_instr = self.instrs.push_consume_fuel_instr()?;
        self.reachable = frame.is_else_reachable();
        self.stack
            .push_else(frame, is_end_of_then_reachable, consume_fuel_instr)?;
        Ok(())
    }

    fn visit_end(&mut self) -> Self::Output {
        match self.stack.pop_control() {
            ControlFrame::Block(frame) => self.translate_end_block(frame),
            ControlFrame::Loop(frame) => self.translate_end_loop(frame),
            ControlFrame::If(frame) => self.translate_end_if(frame),
            ControlFrame::Else(frame) => self.translate_end_else(frame),
            ControlFrame::Unreachable(frame) => self.translate_end_unreachable(frame),
        }
    }

    fn visit_br(&mut self, _relative_depth: u32) -> Self::Output {
        todo!()
    }

    fn visit_br_if(&mut self, _relative_depth: u32) -> Self::Output {
        todo!()
    }

    fn visit_br_table(&mut self, _targets: wasmparser::BrTable<'a>) -> Self::Output {
        todo!()
    }

    fn visit_return(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_call(&mut self, _function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_call_indirect(&mut self, _type_index: u32, _table_index: u32) -> Self::Output {
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

    fn visit_local_set(&mut self, _local_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_local_tee(&mut self, _local_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_global_get(&mut self, _global_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_global_set(&mut self, _global_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_load(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f32_load(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f64_load(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load8_s(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load8_u(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load16_s(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load16_u(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load8_s(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load8_u(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load16_s(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load16_u(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load32_s(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load32_u(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_store(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f32_store(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f64_store(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_store8(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_store16(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store8(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store16(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store32(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_memory_size(&mut self, _mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_grow(&mut self, _mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_const(&mut self, value: i32) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_immediate(value)?;
        Ok(())
    }

    fn visit_i64_const(&mut self, _value: i64) -> Self::Output {
        todo!()
    }

    fn visit_f32_const(&mut self, _value: wasmparser::Ieee32) -> Self::Output {
        todo!()
    }

    fn visit_f64_const(&mut self, _value: wasmparser::Ieee64) -> Self::Output {
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

    fn visit_memory_init(&mut self, _data_index: u32, _mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_data_drop(&mut self, _data_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_copy(&mut self, _dst_mem: u32, _src_mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_fill(&mut self, _mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_init(&mut self, _elem_index: u32, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_elem_drop(&mut self, _elem_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_copy(&mut self, _dst_table: u32, _src_table: u32) -> Self::Output {
        todo!()
    }

    fn visit_typed_select(&mut self, _ty: wasmparser::ValType) -> Self::Output {
        todo!()
    }

    fn visit_ref_null(&mut self, _hty: wasmparser::HeapType) -> Self::Output {
        todo!()
    }

    fn visit_ref_is_null(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_ref_func(&mut self, _function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_fill(&mut self, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_get(&mut self, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_set(&mut self, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_grow(&mut self, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_size(&mut self, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_return_call(&mut self, _function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_return_call_indirect(&mut self, _type_index: u32, _table_index: u32) -> Self::Output {
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
