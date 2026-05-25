use super::{ControlFrame, ControlFrameKind, FuncTranslator, LocalIdx};
use crate::{
    Error,
    F32,
    F64,
    FuncType,
    Mutability,
    RefType,
    TrapCode,
    ValType,
    core::{FuelCostsProvider, IndexType, RawRef, RawVal, TypedRawRef, TypedRawVal, wasm},
    engine::{
        BlockType,
        translator::func::{
            ControlFrameBase,
            Operand,
            op,
            stack::{AcquiredTarget, IfReachability, ResolvedOperand},
        },
    },
    ir::{self, Op, index},
    module::{
        self,
        MemoryIdx,
        TableIdx,
        WasmiValueType,
        init_expr::{EmptyEvalContext, Eval},
    },
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
            $( $( let _ = $arg; )* )?
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

    #[inline(never)]
    fn visit_unreachable(&mut self) -> Self::Output {
        bail_unreachable!(self);
        self.translate_trap(TrapCode::UnreachableCodeReached)
    }

    #[inline(never)]
    fn visit_nop(&mut self) -> Self::Output {
        Ok(())
    }

    #[inline(never)]
    fn visit_block(&mut self, block_ty: wasmparser::BlockType) -> Self::Output {
        if !self.reachable {
            self.stack.push_unreachable(ControlFrameKind::Block)?;
            return Ok(());
        }
        self.preserve_all_locals()?;
        let block_ty = BlockType::new(block_ty, &self.module);
        let end_label = self.instrs.new_label();
        self.stack.push_block(block_ty, end_label)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_loop(&mut self, block_ty: wasmparser::BlockType) -> Self::Output {
        if !self.reachable {
            self.stack.push_unreachable(ControlFrameKind::Loop)?;
            return Ok(());
        }
        self.preserve_all_locals()?;
        let block_ty = BlockType::new(block_ty, &self.module);
        let len_params = block_ty.len_params(&self.engine);
        let continue_label = self.instrs.new_label();
        let consume_fuel = self.stack.consume_fuel_instr();
        if len_params > 0 {
            self.move_operands_to_temp(usize::from(len_params), consume_fuel)?;
        }
        self.instrs.pin_label(continue_label)?;
        let consume_fuel = self.instrs.encode_consume_fuel()?;
        self.stack
            .push_loop(block_ty, continue_label, consume_fuel)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_if(&mut self, block_ty: wasmparser::BlockType) -> Self::Output {
        if !self.reachable {
            self.stack.push_unreachable(ControlFrameKind::If)?;
            return Ok(());
        }
        let end_label = self.instrs.new_label();
        let condition = self.stack.pop();
        self.preserve_all_locals()?;
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
                let else_label = self.instrs.new_label();
                self.encode_br_eqz(condition, else_label)?;
                let reachability = IfReachability::Both { else_label };
                let consume_fuel_instr = self.instrs.encode_consume_fuel()?;
                (reachability, consume_fuel_instr)
            }
        };
        let block_ty = BlockType::new(block_ty, &self.module);
        self.stack
            .push_if(block_ty, end_label, reachability, consume_fuel_instr)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_else(&mut self) -> Self::Output {
        let mut frame = match self.stack.pop_control() {
            ControlFrame::If(frame) => frame,
            ControlFrame::Unreachable(ControlFrameKind::If) => {
                debug_assert!(!self.reachable);
                self.stack.push_unreachable(ControlFrameKind::Else)?;
                return Ok(());
            }
            unexpected => panic!("expected `if` control frame but found: {unexpected:?}"),
        };
        // After `then` block, before `else` block:
        // - Copy `if` branch parameters.
        // - Branch from end of `then` to end of `if`.
        let is_end_of_then_reachable = self.reachable;
        if let IfReachability::Both { else_label } = frame.reachability() {
            if is_end_of_then_reachable {
                let consume_fuel_instr = frame.consume_fuel_instr();
                self.copy_branch_params(&frame, consume_fuel_instr)?;
                frame.branch_to();
                self.encode_br(frame.label())?;
            }
            // Start of `else` block:
            self.instrs.pin_label(else_label)?;
        }
        let consume_fuel_instr = self.instrs.encode_consume_fuel()?;
        self.reachable = frame.is_else_reachable();
        self.stack
            .push_else(frame, is_end_of_then_reachable, consume_fuel_instr)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_end(&mut self) -> Self::Output {
        match self.stack.pop_control() {
            ControlFrame::Block(frame) => self.translate_end_block(frame),
            ControlFrame::Loop(frame) => self.translate_end_loop(frame),
            ControlFrame::If(frame) => self.translate_end_if(frame),
            ControlFrame::Else(frame) => self.translate_end_else(frame),
            ControlFrame::Unreachable(frame) => self.translate_end_unreachable(frame),
        }
    }

    #[inline(never)]
    fn visit_br(&mut self, depth: u32) -> Self::Output {
        bail_unreachable!(self);
        let Ok(depth) = usize::try_from(depth) else {
            panic!("out of bounds depth: {depth}")
        };
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        match self.stack.peek_control_mut(depth) {
            AcquiredTarget::Return(_) => self.visit_return(),
            AcquiredTarget::Branch(mut frame) => {
                frame.branch_to();
                let label = frame.label();
                let len_params = frame.len_branch_params(&self.engine);
                if let Some(branch_results) = frame.branch_slots() {
                    self.encode_copies(branch_results.span(), len_params, consume_fuel_instr)?;
                }
                self.encode_br(label)?;
                self.reachable = false;
                Ok(())
            }
        }
    }

    #[inline(never)]
    fn visit_br_if(&mut self, depth: u32) -> Self::Output {
        bail_unreachable!(self);
        let condition = self.stack.pop();
        if let Operand::Immediate(condition) = condition {
            if i32::from(condition.val()) != 0 {
                // Case (true): always takes the branch
                self.visit_br(depth)?;
            }
            return Ok(());
        }
        let Ok(depth) = usize::try_from(depth) else {
            panic!("out of bounds depth: {depth}")
        };
        let mut frame = self.stack.peek_control_mut(depth).control_frame();
        frame.branch_to();
        let label = frame.label();
        let Some(branch_slots) = frame.branch_slots() else {
            // Case: no branch values are required to be copied
            self.encode_br_nez(condition, label)?;
            return Ok(());
        };
        let len_branch_params = frame.len_branch_params(&self.engine);
        if !self.requires_branch_param_copies(depth) {
            // Case: no branch values are required to be copied
            self.encode_br_nez(condition, label)?;
            return Ok(());
        }
        // Case: fallback to copy branch parameters conditionally
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        let skip_label = self.instrs.new_label();
        self.encode_br_eqz(condition, skip_label)?;
        self.encode_copies(branch_slots.span(), len_branch_params, consume_fuel_instr)?;
        self.encode_br(label)?;
        self.instrs.pin_label(skip_label)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_br_table(&mut self, table: wasmparser::BrTable<'a>) -> Self::Output {
        bail_unreachable!(self);
        let index = self.stack.pop();
        let default_target = table.default();
        if table.is_empty() {
            // Case: the `br_table` only has a single target `t` which is equal to a `br t`.
            return self.visit_br(default_target);
        }
        if let Operand::Immediate(index) = index {
            // Case: the `br_table` index is a constant value, therefore always taking the same branch.
            // Note: `usize::MAX` is used to fallback to the default target.
            let chosen_index = usize::try_from(u32::from(index.val())).unwrap_or(usize::MAX);
            let chosen_target = table
                .targets()
                .nth(chosen_index)
                .transpose()?
                .unwrap_or(default_target);
            return self.visit_br(chosen_target);
        }
        Self::copy_targets_from_br_table(&table, &mut self.immediates)?;
        let targets = &self.immediates[..];
        if targets
            .iter()
            .all(|&target| u32::from(target) == default_target)
        {
            // Case: all targets are the same and thus the `br_table` is equal to a `br`.
            return self.visit_br(default_target);
        }
        // Note: The Wasm spec mandates that all `br_table` targets manipulate the
        //       Wasm value stack the same. This implies for Wasmi that all `br_table`
        //       targets have the same branch parameter arity.
        let Ok(default_target) = usize::try_from(default_target) else {
            panic!("out of bounds `default_target` does not fit into `usize`: {default_target}");
        };
        let index = self.copy_immediate_to_slot(index)?;
        let len_branch_params = self
            .stack
            .peek_control(default_target)
            .len_branch_params(&self.engine);
        match len_branch_params {
            0 => self.encode_br_table_0(table, index)?,
            n => self.encode_br_table_n(table, index, n)?,
        };
        self.reachable = false;
        Ok(())
    }

    #[inline(never)]
    fn visit_return(&mut self) -> Self::Output {
        bail_unreachable!(self);
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        self.encode_return(consume_fuel_instr)?;
        let len_results = self.func_type_with(FuncType::len_results);
        for _ in 0..len_results {
            self.stack.pop();
        }
        self.reachable = false;
        Ok(())
    }

    #[inline(never)]
    fn visit_call(&mut self, function_index: u32) -> Self::Output {
        self.translate_call(function_index, Op::call_internal, Op::call_imported)
    }

    #[inline(never)]
    fn visit_call_indirect(&mut self, type_index: u32, table_index: u32) -> Self::Output {
        self.translate_call_indirect(
            type_index,
            table_index,
            Op::call_indirect_s,
            Op::call_indirect_r,
        )
    }

    #[inline(never)]
    fn visit_drop(&mut self) -> Self::Output {
        bail_unreachable!(self);
        _ = self.stack.pop();
        Ok(())
    }

    #[inline(never)]
    fn visit_select(&mut self) -> Self::Output {
        self.translate_select(None)
    }

    #[inline(never)]
    fn visit_local_get(&mut self, local_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let local_idx = LocalIdx::from(local_index);
        let ty = self.locals.ty(local_idx);
        self.stack.push_local(local_idx, ty)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_local_set(&mut self, local_index: u32) -> Self::Output {
        self.translate_local_set(local_index, false)
    }

    #[inline(never)]
    fn visit_local_tee(&mut self, local_index: u32) -> Self::Output {
        self.translate_local_set(local_index, true)
    }

    #[inline(never)]
    fn visit_global_get(&mut self, global_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let global_idx = module::GlobalIdx::from(global_index);
        let (global_type, init_value) = self.module.get_global(global_idx);
        let content = global_type.content();
        if let (Mutability::Const, Some(init_expr)) = (global_type.mutability(), init_value) {
            if let Some(value) = init_expr.eval(&EmptyEvalContext) {
                if let Some(value) = value.as_raw_or_none() {
                    // Case: access to immutable internally defined global variables
                    //       can be replaced with their constant initialization value.
                    self.stack
                        .push_immediate(TypedRawVal::new(content, value))?;
                    return Ok(());
                }
            }
            if let Some(func_index) = init_expr.funcref() {
                // Case: forward to `ref.func x` translation.
                self.visit_ref_func(func_index.into_u32())?;
                return Ok(());
            }
        }
        // Case: The `global.get` instruction accesses a mutable or imported
        //       global variable and thus cannot be optimized away.
        let global_idx = ir::index::Global::from(global_index);
        #[cfg(feature = "simd")]
        if matches!(content, ValType::V128) {
            self.push_instr_with_result_slot(
                content,
                |result| Op::global_get_v128_s(global_idx, result),
                FuelCostsProvider::instance,
            )?;
            return Ok(());
        }
        let operator = match content {
            ValType::I32 | ValType::I64 | ValType::FuncRef | ValType::ExternRef => {
                Op::global_get_u64_r(global_idx)
            }
            ValType::F32 => Op::global_get_f32_r(global_idx),
            ValType::F64 => Op::global_get_f64_r(global_idx),
            _ => unreachable!(),
        };
        self.stage_op_with_result_reg(content, operator, FuelCostsProvider::instance)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_global_set(&mut self, global_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let global = index::Global::from(global_index);
        let (global_type, _init_value) = self
            .module
            .get_global(module::GlobalIdx::from(global_index));
        let ty = global_type.content();
        let input = self.stack.pop();
        let op = match self.resolve_operand_as::<RawVal>(input)? {
            ResolvedOperand::Reg => match ty {
                | ValType::I32 | ValType::I64 | ValType::FuncRef | ValType::ExternRef => {
                    Op::global_set_u64_r(global)
                }
                | ValType::F32 => Op::global_set_f32_r(global),
                | ValType::F64 => Op::global_set_f64_r(global),
                | ValType::V128 => unreachable!(),
            },
            ResolvedOperand::Slot(value) => Op::global_set_u64_s(global, value),
            ResolvedOperand::Immediate(value) => match ty {
                | ValType::I32 | ValType::F32 | ValType::FuncRef | ValType::ExternRef => {
                    Op::global_set_u32_i(global, u32::from(value))
                }
                | ValType::I64 | ValType::F64 => Op::global_set_u64_i(global, u64::from(value)),
                | ValType::V128 => {
                    let fuel_op = self.stack.consume_fuel_instr();
                    let v128 = self.copy_operand_to_temp(input, fuel_op)?;
                    Op::global_set_u64_s(global, v128)
                }
            },
        };
        self.push_instr(op, FuelCostsProvider::instance)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_i32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::I32Load>(memarg)
    }

    #[inline(never)]
    fn visit_i64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::I64Load>(memarg)
    }

    #[inline(never)]
    fn visit_f32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::F32Load>(memarg)
    }

    #[inline(never)]
    fn visit_f64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::F64Load>(memarg)
    }

    #[inline(never)]
    fn visit_i32_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::I32Load8>(memarg)
    }

    #[inline(never)]
    fn visit_i32_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::U32Load8>(memarg)
    }

    #[inline(never)]
    fn visit_i32_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::I32Load16>(memarg)
    }

    #[inline(never)]
    fn visit_i32_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::U32Load16>(memarg)
    }

    #[inline(never)]
    fn visit_i64_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::I64Load8>(memarg)
    }

    #[inline(never)]
    fn visit_i64_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::U64Load8>(memarg)
    }

    #[inline(never)]
    fn visit_i64_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::I64Load16>(memarg)
    }

    #[inline(never)]
    fn visit_i64_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::U64Load16>(memarg)
    }

    #[inline(never)]
    fn visit_i64_load32_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::I64Load32>(memarg)
    }

    #[inline(never)]
    fn visit_i64_load32_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load::<op::U64Load32>(memarg)
    }

    #[inline(never)]
    fn visit_i32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<op::I32Store>(memarg)
    }

    #[inline(never)]
    fn visit_i64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<op::I64Store>(memarg)
    }

    #[inline(never)]
    fn visit_f32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<op::F32Store>(memarg)
    }

    #[inline(never)]
    fn visit_f64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<op::F64Store>(memarg)
    }

    #[inline(never)]
    fn visit_i32_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<op::I32Store8>(memarg)
    }

    #[inline(never)]
    fn visit_i32_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<op::I32Store16>(memarg)
    }

    #[inline(never)]
    fn visit_i64_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<op::I64Store8>(memarg)
    }

    #[inline(never)]
    fn visit_i64_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<op::I64Store16>(memarg)
    }

    #[inline(never)]
    fn visit_i64_store32(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<op::I64Store32>(memarg)
    }

    #[inline(never)]
    fn visit_memory_size(&mut self, mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let index_ty = self
            .module
            .get_type_of_memory(MemoryIdx::from(mem))
            .index_ty()
            .ty();
        let memory = index::Memory::try_from(mem)?;
        self.stage_op_with_result_reg(
            index_ty,
            Op::memory_size(memory),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_memory_grow(&mut self, mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let index_ty = self
            .module
            .get_type_of_memory(MemoryIdx::from(mem))
            .index_ty();
        let memory = index::Memory::try_from(mem)?;
        let delta = self.stack.pop();
        if let Operand::Immediate(delta) = delta {
            let delta = delta.val();
            let delta = match index_ty {
                IndexType::I32 => u64::from(u32::from(delta)),
                IndexType::I64 => u64::from(delta),
            };
            if delta == 0 {
                // Case: growing by 0 pages.
                //
                // Since `memory.grow` returns the `memory.size` before the
                // operation a `memory.grow` with `delta` of 0 can be translated
                // as `memory.size` instruction instead.
                self.stage_op_with_result_reg(
                    index_ty.ty(),
                    Op::memory_size(memory),
                    FuelCostsProvider::instance,
                )?;
                return Ok(());
            }
        }
        // Case: fallback to generic `memory.grow` instruction
        let delta = self.copy_operand_to_slot(delta)?;
        self.stage_op_with_result_reg(
            index_ty.ty(),
            Op::memory_grow(delta, memory),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_i32_const(&mut self, value: i32) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_immediate(value)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_i64_const(&mut self, value: i64) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_immediate(value)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_f32_const(&mut self, value: wasmparser::Ieee32) -> Self::Output {
        bail_unreachable!(self);
        let value = F32::from_bits(value.bits());
        self.stack.push_immediate(value)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_f64_const(&mut self, value: wasmparser::Ieee64) -> Self::Output {
        bail_unreachable!(self);
        let value = F64::from_bits(value.bits());
        self.stack.push_immediate(value)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_i32_eqz(&mut self) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_immediate(0_i32)?;
        self.visit_i32_eq()
    }

    #[inline(never)]
    fn visit_i32_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I32Eq>(Self::fuse_eqz)
    }

    #[inline(never)]
    fn visit_i32_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I32NotEq>(Self::fuse_nez)
    }

    #[inline(never)]
    fn visit_i32_lt_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I32Lt>()
    }

    #[inline(never)]
    fn visit_i32_lt_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U32Lt>()
    }

    #[inline(never)]
    fn visit_i32_gt_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I32Gt>()
    }

    #[inline(never)]
    fn visit_i32_gt_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U32Gt>()
    }

    #[inline(never)]
    fn visit_i32_le_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I32Lt>()
    }

    #[inline(never)]
    fn visit_i32_le_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U32Lt>()
    }

    #[inline(never)]
    fn visit_i32_ge_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I32Ge>()
    }

    #[inline(never)]
    fn visit_i32_ge_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U32Ge>()
    }

    #[inline(never)]
    fn visit_i64_eqz(&mut self) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_immediate(0_i64)?;
        self.visit_i64_eq()
    }

    #[inline(never)]
    fn visit_i64_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I64Eq>(Self::fuse_eqz)
    }

    #[inline(never)]
    fn visit_i64_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I64NotEq>(Self::fuse_eqz)
    }

    #[inline(never)]
    fn visit_i64_lt_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I64Lt>()
    }

    #[inline(never)]
    fn visit_i64_lt_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U64Lt>()
    }

    #[inline(never)]
    fn visit_i64_gt_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I64Gt>()
    }

    #[inline(never)]
    fn visit_i64_gt_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U64Gt>()
    }

    #[inline(never)]
    fn visit_i64_le_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I64Le>()
    }

    #[inline(never)]
    fn visit_i64_le_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U64Le>()
    }

    #[inline(never)]
    fn visit_i64_ge_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I64Ge>()
    }

    #[inline(never)]
    fn visit_i64_ge_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U64Ge>()
    }

    #[inline(never)]
    fn visit_f32_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::F32Eq>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_f32_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::F32NotEq>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_f32_lt(&mut self) -> Self::Output {
        self.translate_binary::<op::F32Lt>()
    }

    #[inline(never)]
    fn visit_f32_gt(&mut self) -> Self::Output {
        self.translate_binary::<op::F32Gt>()
    }

    #[inline(never)]
    fn visit_f32_le(&mut self) -> Self::Output {
        self.translate_binary::<op::F32Le>()
    }

    #[inline(never)]
    fn visit_f32_ge(&mut self) -> Self::Output {
        self.translate_binary::<op::F32Ge>()
    }

    #[inline(never)]
    fn visit_f64_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::F64Eq>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_f64_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::F64NotEq>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_f64_lt(&mut self) -> Self::Output {
        self.translate_binary::<op::F64Lt>()
    }

    #[inline(never)]
    fn visit_f64_gt(&mut self) -> Self::Output {
        self.translate_binary::<op::F64Gt>()
    }

    #[inline(never)]
    fn visit_f64_le(&mut self) -> Self::Output {
        self.translate_binary::<op::F64Le>()
    }

    #[inline(never)]
    fn visit_f64_ge(&mut self) -> Self::Output {
        self.translate_binary::<op::F64Ge>()
    }

    #[inline(never)]
    fn visit_i32_clz(&mut self) -> Self::Output {
        self.translate_unary::<op::I32Clz>()
    }

    #[inline(never)]
    fn visit_i32_ctz(&mut self) -> Self::Output {
        self.translate_unary::<op::I32Ctz>()
    }

    #[inline(never)]
    fn visit_i32_popcnt(&mut self) -> Self::Output {
        self.translate_unary::<op::I32Popcnt>()
    }

    #[inline(never)]
    fn visit_i32_add(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I32Add>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_i32_sub(&mut self) -> Self::Output {
        self.translate_binary::<op::I32Sub>()
    }

    #[inline(never)]
    fn visit_i32_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I32Mul>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_i32_div_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I32Div>()
    }

    #[inline(never)]
    fn visit_i32_div_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U32Div>()
    }

    #[inline(never)]
    fn visit_i32_rem_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I32Rem>()
    }

    #[inline(never)]
    fn visit_i32_rem_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U32Rem>()
    }

    #[inline(never)]
    fn visit_i32_and(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I32BitAnd>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_i32_or(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I32BitOr>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_i32_xor(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I32BitXor>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_i32_shl(&mut self) -> Self::Output {
        self.translate_binary::<op::I32Shl>()
    }

    #[inline(never)]
    fn visit_i32_shr_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I32Shr>()
    }

    #[inline(never)]
    fn visit_i32_shr_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U32Shr>()
    }

    #[inline(never)]
    fn visit_i32_rotl(&mut self) -> Self::Output {
        self.translate_binary::<op::I32Rotl>()
    }

    #[inline(never)]
    fn visit_i32_rotr(&mut self) -> Self::Output {
        self.translate_binary::<op::I32Rotr>()
    }

    #[inline(never)]
    fn visit_i64_clz(&mut self) -> Self::Output {
        self.translate_unary::<op::I64Clz>()
    }

    #[inline(never)]
    fn visit_i64_ctz(&mut self) -> Self::Output {
        self.translate_unary::<op::I64Ctz>()
    }

    #[inline(never)]
    fn visit_i64_popcnt(&mut self) -> Self::Output {
        self.translate_unary::<op::I64Popcnt>()
    }

    #[inline(never)]
    fn visit_i64_add(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I64Add>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_i64_sub(&mut self) -> Self::Output {
        self.translate_binary::<op::I64Sub>()
    }

    #[inline(never)]
    fn visit_i64_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I64Mul>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_i64_div_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I64Div>()
    }

    #[inline(never)]
    fn visit_i64_div_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U64Div>()
    }

    #[inline(never)]
    fn visit_i64_rem_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I64Rem>()
    }

    #[inline(never)]
    fn visit_i64_rem_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U64Rem>()
    }

    #[inline(never)]
    fn visit_i64_and(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I64BitAnd>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_i64_or(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I64BitOr>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_i64_xor(&mut self) -> Self::Output {
        self.translate_binary_commutative::<op::I64BitXor>(Self::no_opt_ri)
    }

    #[inline(never)]
    fn visit_i64_shl(&mut self) -> Self::Output {
        self.translate_binary::<op::I64Shl>()
    }

    #[inline(never)]
    fn visit_i64_shr_s(&mut self) -> Self::Output {
        self.translate_binary::<op::I64Shr>()
    }

    #[inline(never)]
    fn visit_i64_shr_u(&mut self) -> Self::Output {
        self.translate_binary::<op::U64Shr>()
    }

    #[inline(never)]
    fn visit_i64_rotl(&mut self) -> Self::Output {
        self.translate_binary::<op::I64Rotl>()
    }

    #[inline(never)]
    fn visit_i64_rotr(&mut self) -> Self::Output {
        self.translate_binary::<op::I64Rotr>()
    }

    #[inline(never)]
    fn visit_f32_abs(&mut self) -> Self::Output {
        self.translate_unary::<op::F32Abs>()
    }

    #[inline(never)]
    fn visit_f32_neg(&mut self) -> Self::Output {
        self.translate_unary::<op::F32Neg>()
    }

    #[inline(never)]
    fn visit_f32_ceil(&mut self) -> Self::Output {
        self.translate_unary::<op::F32Ceil>()
    }

    #[inline(never)]
    fn visit_f32_floor(&mut self) -> Self::Output {
        self.translate_unary::<op::F32Floor>()
    }

    #[inline(never)]
    fn visit_f32_trunc(&mut self) -> Self::Output {
        self.translate_unary::<op::F32Trunc>()
    }

    #[inline(never)]
    fn visit_f32_nearest(&mut self) -> Self::Output {
        self.translate_unary::<op::F32Nearest>()
    }

    #[inline(never)]
    fn visit_f32_sqrt(&mut self) -> Self::Output {
        self.translate_unary::<op::F32Sqrt>()
    }

    #[inline(never)]
    fn visit_f32_add(&mut self) -> Self::Output {
        self.translate_binary::<op::F32Add>()
    }

    #[inline(never)]
    fn visit_f32_sub(&mut self) -> Self::Output {
        self.translate_binary::<op::F32Sub>()
    }

    #[inline(never)]
    fn visit_f32_mul(&mut self) -> Self::Output {
        self.translate_binary::<op::F32Mul>()
    }

    #[inline(never)]
    fn visit_f32_div(&mut self) -> Self::Output {
        self.translate_binary::<op::F32Div>()
    }

    #[inline(never)]
    fn visit_f32_min(&mut self) -> Self::Output {
        self.translate_binary::<op::F32Min>()
    }

    #[inline(never)]
    fn visit_f32_max(&mut self) -> Self::Output {
        self.translate_binary::<op::F32Max>()
    }

    #[inline(never)]
    fn visit_f32_copysign(&mut self) -> Self::Output {
        self.translate_binary::<op::F32Copysign>()
    }

    #[inline(never)]
    fn visit_f64_abs(&mut self) -> Self::Output {
        self.translate_unary::<op::F64Abs>()
    }

    #[inline(never)]
    fn visit_f64_neg(&mut self) -> Self::Output {
        self.translate_unary::<op::F64Neg>()
    }

    #[inline(never)]
    fn visit_f64_ceil(&mut self) -> Self::Output {
        self.translate_unary::<op::F64Ceil>()
    }

    #[inline(never)]
    fn visit_f64_floor(&mut self) -> Self::Output {
        self.translate_unary::<op::F64Floor>()
    }

    #[inline(never)]
    fn visit_f64_trunc(&mut self) -> Self::Output {
        self.translate_unary::<op::F64Trunc>()
    }

    #[inline(never)]
    fn visit_f64_nearest(&mut self) -> Self::Output {
        self.translate_unary::<op::F64Nearest>()
    }

    #[inline(never)]
    fn visit_f64_sqrt(&mut self) -> Self::Output {
        self.translate_unary::<op::F64Sqrt>()
    }

    #[inline(never)]
    fn visit_f64_add(&mut self) -> Self::Output {
        self.translate_binary::<op::F64Add>()
    }

    #[inline(never)]
    fn visit_f64_sub(&mut self) -> Self::Output {
        self.translate_binary::<op::F64Sub>()
    }

    #[inline(never)]
    fn visit_f64_mul(&mut self) -> Self::Output {
        self.translate_binary::<op::F64Mul>()
    }

    #[inline(never)]
    fn visit_f64_div(&mut self) -> Self::Output {
        self.translate_binary::<op::F64Div>()
    }

    #[inline(never)]
    fn visit_f64_min(&mut self) -> Self::Output {
        self.translate_binary::<op::F64Min>()
    }

    #[inline(never)]
    fn visit_f64_max(&mut self) -> Self::Output {
        self.translate_binary::<op::F64Max>()
    }

    #[inline(never)]
    fn visit_f64_copysign(&mut self) -> Self::Output {
        self.translate_binary::<op::F64Copysign>()
    }

    #[inline(never)]
    fn visit_i32_wrap_i64(&mut self) -> Self::Output {
        self.translate_unary::<op::I32WrapI64>()
    }

    #[inline(never)]
    fn visit_i32_trunc_f32_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I32TruncF32>()
    }

    #[inline(never)]
    fn visit_i32_trunc_f32_u(&mut self) -> Self::Output {
        self.translate_unary::<op::U32TruncF32>()
    }

    #[inline(never)]
    fn visit_i32_trunc_f64_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I32TruncF64>()
    }

    #[inline(never)]
    fn visit_i32_trunc_f64_u(&mut self) -> Self::Output {
        self.translate_unary::<op::U32TruncF64>()
    }

    #[inline(never)]
    fn visit_i64_extend_i32_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I64ExtendI32>()
    }

    #[inline(never)]
    fn visit_i64_extend_i32_u(&mut self) -> Self::Output {
        self.translate_reinterpret(wasm::i64_extend_i32_u)
    }

    #[inline(never)]
    fn visit_i64_trunc_f32_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I64TruncF32>()
    }

    #[inline(never)]
    fn visit_i64_trunc_f32_u(&mut self) -> Self::Output {
        self.translate_unary::<op::U64TruncF32>()
    }

    #[inline(never)]
    fn visit_i64_trunc_f64_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I64TruncF64>()
    }

    #[inline(never)]
    fn visit_i64_trunc_f64_u(&mut self) -> Self::Output {
        self.translate_unary::<op::U64TruncF64>()
    }

    #[inline(never)]
    fn visit_f32_convert_i32_s(&mut self) -> Self::Output {
        self.translate_unary::<op::F32ConvertI32>()
    }

    #[inline(never)]
    fn visit_f32_convert_i32_u(&mut self) -> Self::Output {
        self.translate_unary::<op::F32ConvertU32>()
    }

    #[inline(never)]
    fn visit_f32_convert_i64_s(&mut self) -> Self::Output {
        self.translate_unary::<op::F32ConvertI64>()
    }

    #[inline(never)]
    fn visit_f32_convert_i64_u(&mut self) -> Self::Output {
        self.translate_unary::<op::F32ConvertU64>()
    }

    #[inline(never)]
    fn visit_f32_demote_f64(&mut self) -> Self::Output {
        self.translate_unary::<op::F32DemoteF64>()
    }

    #[inline(never)]
    fn visit_f64_convert_i32_s(&mut self) -> Self::Output {
        self.translate_unary::<op::F64ConvertI32>()
    }

    #[inline(never)]
    fn visit_f64_convert_i32_u(&mut self) -> Self::Output {
        self.translate_unary::<op::F64ConvertU32>()
    }

    #[inline(never)]
    fn visit_f64_convert_i64_s(&mut self) -> Self::Output {
        self.translate_unary::<op::F64ConvertI64>()
    }

    #[inline(never)]
    fn visit_f64_convert_i64_u(&mut self) -> Self::Output {
        self.translate_unary::<op::F64ConvertU64>()
    }

    #[inline(never)]
    fn visit_f64_promote_f32(&mut self) -> Self::Output {
        self.translate_unary::<op::F64PromoteF32>()
    }

    #[inline(never)]
    fn visit_i32_reinterpret_f32(&mut self) -> Self::Output {
        self.translate_reinterpret(wasm::i32_reinterpret_f32)
    }

    #[inline(never)]
    fn visit_i64_reinterpret_f64(&mut self) -> Self::Output {
        self.translate_reinterpret(wasm::i64_reinterpret_f64)
    }

    #[inline(never)]
    fn visit_f32_reinterpret_i32(&mut self) -> Self::Output {
        self.translate_reinterpret(wasm::f32_reinterpret_i32)
    }

    #[inline(never)]
    fn visit_f64_reinterpret_i64(&mut self) -> Self::Output {
        self.translate_reinterpret(wasm::f64_reinterpret_i64)
    }

    #[inline(never)]
    fn visit_i32_extend8_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I32Sext8>()
    }

    #[inline(never)]
    fn visit_i32_extend16_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I32Sext16>()
    }

    #[inline(never)]
    fn visit_i64_extend8_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I64Sext8>()
    }

    #[inline(never)]
    fn visit_i64_extend16_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I64Sext16>()
    }

    #[inline(never)]
    fn visit_i64_extend32_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I64Sext32>()
    }

    #[inline(never)]
    fn visit_i32_trunc_sat_f32_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I32TruncSatF32>()
    }

    #[inline(never)]
    fn visit_i32_trunc_sat_f32_u(&mut self) -> Self::Output {
        self.translate_unary::<op::U32TruncSatF32>()
    }

    #[inline(never)]
    fn visit_i32_trunc_sat_f64_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I32TruncSatF64>()
    }

    #[inline(never)]
    fn visit_i32_trunc_sat_f64_u(&mut self) -> Self::Output {
        self.translate_unary::<op::U32TruncSatF64>()
    }

    #[inline(never)]
    fn visit_i64_trunc_sat_f32_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I64TruncSatF32>()
    }

    #[inline(never)]
    fn visit_i64_trunc_sat_f32_u(&mut self) -> Self::Output {
        self.translate_unary::<op::U64TruncSatF32>()
    }

    #[inline(never)]
    fn visit_i64_trunc_sat_f64_s(&mut self) -> Self::Output {
        self.translate_unary::<op::I64TruncSatF64>()
    }

    #[inline(never)]
    fn visit_i64_trunc_sat_f64_u(&mut self) -> Self::Output {
        self.translate_unary::<op::U64TruncSatF64>()
    }

    #[inline(never)]
    fn visit_memory_init(&mut self, data_index: u32, mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.stack.pop3();
        let memory = index::Memory::try_from(mem)?;
        let data = index::Data::from(data_index);
        let dst = self.copy_operand_to_slot(dst)?;
        let src = self.copy_operand_to_slot(src)?;
        let len = self.copy_operand_to_slot(len)?;
        self.push_instr(
            Op::memory_init(memory, data, dst, src, len),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_data_drop(&mut self, data_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.push_instr(
            Op::data_drop(index::Data::from(data_index)),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_memory_copy(&mut self, dst_mem: u32, src_mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.stack.pop3();
        let dst_memory = index::Memory::try_from(dst_mem)?;
        let src_memory = index::Memory::try_from(src_mem)?;
        let dst = self.copy_operand_to_slot(dst)?;
        let src = self.copy_operand_to_slot(src)?;
        let len = self.copy_operand_to_slot(len)?;
        self.push_instr(
            Op::memory_copy(dst_memory, src_memory, dst, src, len),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_memory_fill(&mut self, mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, value, len) = self.stack.pop3();
        let memory = index::Memory::try_from(mem)?;
        let dst = self.copy_operand_to_slot(dst)?;
        let len = self.copy_operand_to_slot(len)?;
        let value = self.copy_operand_to_slot(value)?;
        self.push_instr(
            Op::memory_fill(memory, dst, len, value),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_table_init(&mut self, elem_index: u32, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.stack.pop3();
        let table = index::Table::from(table);
        let elem = index::Elem::from(elem_index);
        let dst = self.copy_operand_to_slot(dst)?;
        let src = self.copy_operand_to_slot(src)?;
        let len = self.copy_operand_to_slot(len)?;
        self.push_instr(
            Op::table_init(table, elem, dst, src, len),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_elem_drop(&mut self, elem_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.push_instr(
            Op::elem_drop(index::Elem::from(elem_index)),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_table_copy(&mut self, dst_table: u32, src_table: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.stack.pop3();
        let dst_table = index::Table::from(dst_table);
        let src_table = index::Table::from(src_table);
        let dst = self.copy_operand_to_slot(dst)?;
        let src = self.copy_operand_to_slot(src)?;
        let len = self.copy_operand_to_slot(len)?;
        self.push_instr(
            Op::table_copy(dst_table, src_table, dst, src, len),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_typed_select(&mut self, ty: wasmparser::ValType) -> Self::Output {
        let type_hint = WasmiValueType::from(ty).into_inner();
        self.translate_select(Some(type_hint))
    }

    #[inline(never)]
    fn visit_ref_null(&mut self, ty: wasmparser::HeapType) -> Self::Output {
        bail_unreachable!(self);
        let type_hint = WasmiValueType::from(ty).into_inner();
        let null = match type_hint {
            ValType::FuncRef => TypedRawRef::null(RefType::Func),
            ValType::ExternRef => TypedRawRef::null(RefType::Extern),
            ty => unreachable!("expected a Wasm `reftype` but found: {ty:?}"),
        };
        self.stack.push_immediate(null)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_ref_is_null(&mut self) -> Self::Output {
        bail_unreachable!(self);
        // Note: `funcref` and `externref` both serialize to `RawValue`
        //       as `u32` so we can use `i32.eqz` translation for `ref.is_null`
        //       via reinterpretation of the value's type.
        match self.stack.pop() {
            Operand::Immediate(input) => {
                debug_assert!(matches!(input.ty(), ValType::FuncRef | ValType::ExternRef));
                let raw = input.val().raw();
                let is_null = RawRef::from(raw).is_null();
                self.stack.push_immediate(i32::from(is_null))?;
                return Ok(());
            }
            Operand::Reg(_input) => {
                self.stack.push_reg(ValType::I32)?;
            }
            Operand::Local(input) => {
                self.stack.push_local(input.local_index(), ValType::I32)?;
            }
            Operand::Temp(_) => {
                self.stack.push_temp(ValType::I32)?;
            }
        };
        self.visit_i32_eqz()
    }

    #[inline(never)]
    fn visit_ref_func(&mut self, function_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.push_instr_with_result_slot(
            ValType::FuncRef,
            |result| Op::ref_func(result, index::Func::from(function_index)),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_table_fill(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, value, len) = self.stack.pop3();
        let table = index::Table::from(table);
        let dst = self.copy_operand_to_slot(dst)?;
        let value = self.copy_operand_to_slot(value)?;
        let len = self.copy_operand_to_slot(len)?;
        self.push_instr(
            Op::table_fill(table, dst, len, value),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_table_get(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let table_type = *self.module.get_type_of_table(TableIdx::from(table));
        let table = index::Table::from(table);
        let index = self.stack.pop();
        let item_ty = table_type.element();
        let index_ty = table_type.index_ty();
        let index = self.resolve_operand_as_index32_or_copy(index, index_ty)?;
        self.stage_op_with_result_reg(
            item_ty.into(),
            match index {
                ResolvedOperand::Reg => Op::table_get_rr(table),
                ResolvedOperand::Slot(index) => Op::table_get_rs(index, table),
                ResolvedOperand::Immediate(index) => Op::table_get_ri(index, table),
            },
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_table_set(&mut self, table: u32) -> Self::Output {
        use ResolvedOperand as Opd;
        bail_unreachable!(self);
        let table_type = *self.module.get_type_of_table(TableIdx::from(table));
        let table = index::Table::from(table);
        let index_ty = table_type.index_ty();
        let (index, value) = self.stack.pop2();
        let index = self.resolve_operand_as_index32_or_copy(index, index_ty)?;
        let value = self.resolve_operand_as::<u32>(value)?;
        let instr = match (index, value) {
            (Opd::Reg, Opd::Slot(value)) => Op::table_set_rs(table, value),
            (Opd::Reg, Opd::Immediate(value)) => Op::table_set_ri(table, value),
            (Opd::Slot(index), Opd::Reg) => Op::table_set_sr(table, index),
            (Opd::Slot(index), Opd::Slot(value)) => Op::table_set_ss(table, index, value),
            (Opd::Slot(index), Opd::Immediate(value)) => Op::table_set_si(table, index, value),
            (Opd::Immediate(index), Opd::Reg) => Op::table_set_ir(table, index),
            (Opd::Immediate(index), Opd::Slot(value)) => Op::table_set_is(table, index, value),
            (Opd::Immediate(index), Opd::Immediate(value)) => Op::table_set_ii(table, index, value),
            _ => todo!(), // unsupported operand pair
        };
        self.push_instr(instr, FuelCostsProvider::instance)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_table_grow(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let table_type = *self.module.get_type_of_table(TableIdx::from(table));
        let table = index::Table::from(table);
        let index_ty = table_type.index_ty();
        let (value, delta) = self.stack.pop2();
        if let Operand::Immediate(delta) = delta {
            let delta = delta.val();
            let delta = match index_ty {
                IndexType::I32 => u64::from(u32::from(delta)),
                IndexType::I64 => u64::from(delta),
            };
            if delta == 0 {
                // Case: growing by 0 elements.
                //
                // Since `table.grow` returns the `table.size` before the
                // operation a `table.grow` with `delta` of 0 can be translated
                // as `table.size` instruction instead.
                self.stage_op_with_result_reg(
                    index_ty.ty(),
                    Op::table_size(table),
                    FuelCostsProvider::instance,
                )?;
                return Ok(());
            }
        }
        let value = self.copy_operand_to_slot(value)?;
        let delta = self.copy_operand_to_slot(delta)?;
        self.stage_op_with_result_reg(
            index_ty.ty(),
            Op::table_grow(delta, value, table),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_table_size(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let table_type = *self.module.get_type_of_table(TableIdx::from(table));
        let table = index::Table::from(table);
        let index_ty = table_type.index_ty();
        self.stage_op_with_result_reg(
            index_ty.ty(),
            Op::table_size(table),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_return_call(&mut self, function_index: u32) -> Self::Output {
        self.translate_call(
            function_index,
            Op::return_call_internal,
            Op::return_call_imported,
        )?;
        self.reachable = false;
        Ok(())
    }

    #[inline(never)]
    fn visit_return_call_indirect(&mut self, type_index: u32, table_index: u32) -> Self::Output {
        self.translate_call_indirect(
            type_index,
            table_index,
            Op::return_call_indirect_s,
            Op::return_call_indirect_r,
        )?;
        self.reachable = false;
        Ok(())
    }

    #[inline(never)]
    fn visit_i64_add128(&mut self) -> Self::Output {
        self.translate_i64_binop128(Op::i64_add128, wasm::i64_add128)
    }

    #[inline(never)]
    fn visit_i64_sub128(&mut self) -> Self::Output {
        self.translate_i64_binop128(Op::i64_sub128, wasm::i64_sub128)
    }

    #[inline(never)]
    fn visit_i64_mul_wide_s(&mut self) -> Self::Output {
        self.translate_i64_mul_wide_sx(Op::i64_mul_wide, wasm::i64_mul_wide_s, true)
    }

    #[inline(never)]
    fn visit_i64_mul_wide_u(&mut self) -> Self::Output {
        self.translate_i64_mul_wide_sx(Op::u64_mul_wide, wasm::i64_mul_wide_u, false)
    }
}
