use super::{ControlFrame, ControlFrameKind, FuncTranslator, LocalIdx};
use crate::{
    Error,
    ExternRef,
    F32,
    F64,
    Func,
    FuncType,
    Mutability,
    Nullable,
    RefType,
    TrapCode,
    ValType,
    core::{FuelCostsProvider, IndexType, TypedRawRef, TypedRawVal, wasm},
    engine::{
        BlockType,
        translator::func::{
            ControlFrameBase,
            Input,
            Operand,
            op,
            stack::{AcquiredTarget, IfReachability},
        },
    },
    ir::{self, Op, index},
    module::{self, MemoryIdx, TableIdx, WasmiValueType},
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
        let index = self.layout.operand_to_slot(index)?;
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
        self.translate_call_indirect(type_index, table_index, Op::call_indirect)
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
            if let Some(value) = init_expr.eval_const() {
                // Case: access to immutable internally defined global variables
                //       can be replaced with their constant initialization value.
                self.stack
                    .push_immediate(TypedRawVal::new(content, value))?;
                return Ok(());
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
        let make_op = match global_type.content() {
            #[cfg(feature = "simd")]
            ValType::V128 => Op::global_get128,
            _ => Op::global_get64,
        };
        self.push_instr_with_result(
            content,
            |result| make_op(global_idx, result),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_global_set(&mut self, global_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let global = index::Global::from(global_index);
        // Note: at this point we handle the different immediate `global.set` instructions.
        let (global_type, _init_value) = self
            .module
            .get_global(module::GlobalIdx::from(global_index));
        let input = self.stack.pop();
        let value = match input {
            Operand::Immediate(input) => input.val(),
            input => {
                // Case: `global.set64` or `global.set128` with simple register input.
                debug_assert_eq!(global_type.content(), input.ty());
                let make_op = match global_type.content() {
                    #[cfg(feature = "simd")]
                    ValType::V128 => Op::global_set128_s,
                    _ => Op::global_set64_s,
                };
                let input = self.layout.operand_to_slot(input)?;
                self.push_instr(make_op(global, input), FuelCostsProvider::instance)?;
                return Ok(());
            }
        };
        debug_assert_eq!(global_type.content(), value.ty());
        let global_set_instr = match global_type.content() {
            ValType::I32 => Op::global_set32_i(global, u32::from(value)),
            ValType::I64 => Op::global_set64_i(u64::from(value), global),
            ValType::F32 => Op::global_set32_i(global, f32::from(value).to_bits()),
            ValType::F64 => Op::global_set64_i(f64::from(value).to_bits(), global),
            ValType::FuncRef | ValType::ExternRef => {
                Op::global_set64_i(u64::from(value.raw()), global)
            }
            #[cfg(feature = "simd")]
            ValType::V128 => {
                let consume_fuel = self.stack.consume_fuel_instr();
                let temp = self.copy_operand_to_temp(input, consume_fuel)?;
                Op::global_set128_s(global, temp)
            }
            #[cfg(not(feature = "simd"))]
            unexpected => panic!("unexpected value type found: {unexpected:?}"),
        };
        // Note: at this point we have to allocate a function local constant.
        self.push_instr(global_set_instr, FuelCostsProvider::instance)?;
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
        self.push_instr_with_result(
            index_ty,
            |result| Op::memory_size(result, memory),
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
                self.push_instr_with_result(
                    index_ty.ty(),
                    |result| Op::memory_size(result, memory),
                    FuelCostsProvider::instance,
                )?;
                return Ok(());
            }
        }
        // Case: fallback to generic `memory.grow` instruction
        let delta = self.copy_if_immediate(delta)?;
        self.push_instr_with_result(
            index_ty.ty(),
            |result| Op::memory_grow(result, delta, memory),
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
        self.translate_binary_commutative::<i32, bool>(
            Op::i32_eq_sss,
            Op::i32_eq_ssi,
            wasm::i32_eq,
            FuncTranslator::fuse_eqz,
        )
    }

    #[inline(never)]
    fn visit_i32_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, bool>(
            Op::i32_not_eq_sss,
            Op::i32_not_eq_ssi,
            wasm::i32_ne,
            FuncTranslator::fuse_nez,
        )
    }

    #[inline(never)]
    fn visit_i32_lt_s(&mut self) -> Self::Output {
        self.translate_binary::<i32, bool>(
            Op::i32_lt_sss,
            Op::i32_lt_ssi,
            Op::i32_lt_sis,
            wasm::i32_lt_s,
        )
    }

    #[inline(never)]
    fn visit_i32_lt_u(&mut self) -> Self::Output {
        self.translate_binary::<u32, bool>(
            Op::u32_lt_sss,
            Op::u32_lt_ssi,
            Op::u32_lt_sis,
            wasm::i32_lt_u,
        )
    }

    #[inline(never)]
    fn visit_i32_gt_s(&mut self) -> Self::Output {
        self.translate_binary::<i32, bool>(
            swap_ops!(Op::i32_lt_sss),
            swap_ops!(Op::i32_lt_sis),
            swap_ops!(Op::i32_lt_ssi),
            wasm::i32_gt_s,
        )
    }

    #[inline(never)]
    fn visit_i32_gt_u(&mut self) -> Self::Output {
        self.translate_binary::<u32, bool>(
            swap_ops!(Op::u32_lt_sss),
            swap_ops!(Op::u32_lt_sis),
            swap_ops!(Op::u32_lt_ssi),
            wasm::i32_gt_u,
        )
    }

    #[inline(never)]
    fn visit_i32_le_s(&mut self) -> Self::Output {
        self.translate_binary::<i32, bool>(
            Op::i32_le_sss,
            Op::i32_le_ssi,
            Op::i32_le_sis,
            wasm::i32_le_s,
        )
    }

    #[inline(never)]
    fn visit_i32_le_u(&mut self) -> Self::Output {
        self.translate_binary::<u32, bool>(
            Op::u32_le_sss,
            Op::u32_le_ssi,
            Op::u32_le_sis,
            wasm::i32_le_u,
        )
    }

    #[inline(never)]
    fn visit_i32_ge_s(&mut self) -> Self::Output {
        self.translate_binary::<i32, bool>(
            swap_ops!(Op::i32_le_sss),
            swap_ops!(Op::i32_le_sis),
            swap_ops!(Op::i32_le_ssi),
            wasm::i32_ge_s,
        )
    }

    #[inline(never)]
    fn visit_i32_ge_u(&mut self) -> Self::Output {
        self.translate_binary::<u32, bool>(
            swap_ops!(Op::u32_le_sss),
            swap_ops!(Op::u32_le_sis),
            swap_ops!(Op::u32_le_ssi),
            wasm::i32_ge_u,
        )
    }

    #[inline(never)]
    fn visit_i64_eqz(&mut self) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_immediate(0_i64)?;
        self.visit_i64_eq()
    }

    #[inline(never)]
    fn visit_i64_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, bool>(
            Op::i64_eq_sss,
            Op::i64_eq_ssi,
            wasm::i64_eq,
            FuncTranslator::fuse_eqz,
        )
    }

    #[inline(never)]
    fn visit_i64_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, bool>(
            Op::i64_not_eq_sss,
            Op::i64_not_eq_ssi,
            wasm::i64_ne,
            FuncTranslator::fuse_nez,
        )
    }

    #[inline(never)]
    fn visit_i64_lt_s(&mut self) -> Self::Output {
        self.translate_binary::<i64, bool>(
            Op::i64_lt_sss,
            Op::i64_lt_ssi,
            Op::i64_lt_sis,
            wasm::i64_lt_s,
        )
    }

    #[inline(never)]
    fn visit_i64_lt_u(&mut self) -> Self::Output {
        self.translate_binary::<u64, bool>(
            Op::u64_lt_sss,
            Op::u64_lt_ssi,
            Op::u64_lt_sis,
            wasm::i64_lt_u,
        )
    }

    #[inline(never)]
    fn visit_i64_gt_s(&mut self) -> Self::Output {
        self.translate_binary::<i64, bool>(
            swap_ops!(Op::i64_lt_sss),
            swap_ops!(Op::i64_lt_sis),
            swap_ops!(Op::i64_lt_ssi),
            wasm::i64_gt_s,
        )
    }

    #[inline(never)]
    fn visit_i64_gt_u(&mut self) -> Self::Output {
        self.translate_binary::<u64, bool>(
            swap_ops!(Op::u64_lt_sss),
            swap_ops!(Op::u64_lt_sis),
            swap_ops!(Op::u64_lt_ssi),
            wasm::i64_gt_u,
        )
    }

    #[inline(never)]
    fn visit_i64_le_s(&mut self) -> Self::Output {
        self.translate_binary::<i64, bool>(
            Op::i64_le_sss,
            Op::i64_le_ssi,
            Op::i64_le_sis,
            wasm::i64_le_s,
        )
    }

    #[inline(never)]
    fn visit_i64_le_u(&mut self) -> Self::Output {
        self.translate_binary::<u64, bool>(
            Op::u64_le_sss,
            Op::u64_le_ssi,
            Op::u64_le_sis,
            wasm::i64_le_u,
        )
    }

    #[inline(never)]
    fn visit_i64_ge_s(&mut self) -> Self::Output {
        self.translate_binary::<i64, bool>(
            swap_ops!(Op::i64_le_sss),
            swap_ops!(Op::i64_le_sis),
            swap_ops!(Op::i64_le_ssi),
            wasm::i64_ge_s,
        )
    }

    #[inline(never)]
    fn visit_i64_ge_u(&mut self) -> Self::Output {
        self.translate_binary::<u64, bool>(
            swap_ops!(Op::u64_le_sss),
            swap_ops!(Op::u64_le_sis),
            swap_ops!(Op::u64_le_ssi),
            wasm::i64_ge_u,
        )
    }

    #[inline(never)]
    fn visit_f32_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Op::f32_eq_sss,
            Op::f32_eq_ssi,
            wasm::f32_eq,
            Self::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_f32_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Op::f32_not_eq_sss,
            Op::f32_not_eq_ssi,
            wasm::f32_ne,
            Self::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_f32_lt(&mut self) -> Self::Output {
        self.translate_binary(Op::f32_lt_sss, Op::f32_lt_ssi, Op::f32_lt_sis, wasm::f32_lt)
    }

    #[inline(never)]
    fn visit_f32_gt(&mut self) -> Self::Output {
        self.translate_binary(
            swap_ops!(Op::f32_lt_sss),
            swap_ops!(Op::f32_lt_sis),
            swap_ops!(Op::f32_lt_ssi),
            wasm::f32_gt,
        )
    }

    #[inline(never)]
    fn visit_f32_le(&mut self) -> Self::Output {
        self.translate_binary(Op::f32_le_sss, Op::f32_le_ssi, Op::f32_le_sis, wasm::f32_le)
    }

    #[inline(never)]
    fn visit_f32_ge(&mut self) -> Self::Output {
        self.translate_binary(
            swap_ops!(Op::f32_le_sss),
            swap_ops!(Op::f32_le_sis),
            swap_ops!(Op::f32_le_ssi),
            wasm::f32_ge,
        )
    }

    #[inline(never)]
    fn visit_f64_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Op::f64_eq_sss,
            Op::f64_eq_ssi,
            wasm::f64_eq,
            Self::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_f64_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Op::f64_not_eq_sss,
            Op::f64_not_eq_ssi,
            wasm::f64_ne,
            Self::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_f64_lt(&mut self) -> Self::Output {
        self.translate_binary(Op::f64_lt_sss, Op::f64_lt_ssi, Op::f64_lt_sis, wasm::f64_lt)
    }

    #[inline(never)]
    fn visit_f64_gt(&mut self) -> Self::Output {
        self.translate_binary(
            swap_ops!(Op::f64_lt_sss),
            swap_ops!(Op::f64_lt_sis),
            swap_ops!(Op::f64_lt_ssi),
            wasm::f64_gt,
        )
    }

    #[inline(never)]
    fn visit_f64_le(&mut self) -> Self::Output {
        self.translate_binary(Op::f64_le_sss, Op::f64_le_ssi, Op::f64_le_sis, wasm::f64_le)
    }

    #[inline(never)]
    fn visit_f64_ge(&mut self) -> Self::Output {
        self.translate_binary(
            swap_ops!(Op::f64_le_sss),
            swap_ops!(Op::f64_le_sis),
            swap_ops!(Op::f64_le_ssi),
            wasm::f64_ge,
        )
    }

    #[inline(never)]
    fn visit_i32_clz(&mut self) -> Self::Output {
        self.translate_unary::<i32, i32>(Op::i32_clz_ss, wasm::i32_clz)
    }

    #[inline(never)]
    fn visit_i32_ctz(&mut self) -> Self::Output {
        self.translate_unary::<i32, i32>(Op::i32_ctz_ss, wasm::i32_ctz)
    }

    #[inline(never)]
    fn visit_i32_popcnt(&mut self) -> Self::Output {
        self.translate_unary::<i32, i32>(Op::i32_popcnt_ss, wasm::i32_popcnt)
    }

    #[inline(never)]
    fn visit_i32_add(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, i32>(
            Op::i32_add_sss,
            Op::i32_add_ssi,
            wasm::i32_add,
            FuncTranslator::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_i32_sub(&mut self) -> Self::Output {
        self.translate_binary(
            Op::i32_sub_sss,
            Op::i32_sub_ssi,
            Op::i32_sub_sis,
            wasm::i32_sub,
        )
    }

    #[inline(never)]
    fn visit_i32_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, i32>(
            Op::i32_mul_sss,
            Op::i32_mul_ssi,
            wasm::i32_mul,
            FuncTranslator::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_i32_div_s(&mut self) -> Self::Output {
        self.translate_divrem::<i32>(
            Op::i32_div_sss,
            Op::i32_div_ssi,
            Op::i32_div_sis,
            wasm::i32_div_s,
        )
    }

    #[inline(never)]
    fn visit_i32_div_u(&mut self) -> Self::Output {
        self.translate_divrem::<u32>(
            Op::u32_div_sss,
            Op::u32_div_ssi,
            Op::u32_div_sis,
            wasm::i32_div_u,
        )
    }

    #[inline(never)]
    fn visit_i32_rem_s(&mut self) -> Self::Output {
        self.translate_divrem::<i32>(
            Op::i32_rem_sss,
            Op::i32_rem_ssi,
            Op::i32_rem_sis,
            wasm::i32_rem_s,
        )
    }

    #[inline(never)]
    fn visit_i32_rem_u(&mut self) -> Self::Output {
        self.translate_divrem::<u32>(
            Op::u32_rem_sss,
            Op::u32_rem_ssi,
            Op::u32_rem_sis,
            wasm::i32_rem_u,
        )
    }

    #[inline(never)]
    fn visit_i32_and(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, i32>(
            Op::i32_bitand_sss,
            Op::i32_bitand_ssi,
            wasm::i32_bitand,
            FuncTranslator::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_i32_or(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, i32>(
            Op::i32_bitor_sss,
            Op::i32_bitor_ssi,
            wasm::i32_bitor,
            FuncTranslator::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_i32_xor(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, i32>(
            Op::i32_bitxor_sss,
            Op::i32_bitxor_ssi,
            wasm::i32_bitxor,
            FuncTranslator::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_i32_shl(&mut self) -> Self::Output {
        self.translate_shift::<i32>(
            Op::i32_shl_sss,
            Op::i32_shl_ssi,
            Op::i32_shl_sis,
            wasm::i32_shl,
        )
    }

    #[inline(never)]
    fn visit_i32_shr_s(&mut self) -> Self::Output {
        self.translate_shift::<i32>(
            Op::i32_shr_sss,
            Op::i32_shr_ssi,
            Op::i32_shr_sis,
            wasm::i32_shr_s,
        )
    }

    #[inline(never)]
    fn visit_i32_shr_u(&mut self) -> Self::Output {
        self.translate_shift::<u32>(
            Op::u32_shr_sss,
            Op::u32_shr_ssi,
            Op::u32_shr_sis,
            wasm::i32_shr_u,
        )
    }

    #[inline(never)]
    fn visit_i32_rotl(&mut self) -> Self::Output {
        self.translate_shift::<i32>(
            Op::i32_rotl_sss,
            Op::i32_rotl_ssi,
            Op::i32_rotl_sis,
            wasm::i32_rotl,
        )
    }

    #[inline(never)]
    fn visit_i32_rotr(&mut self) -> Self::Output {
        self.translate_shift::<i32>(
            Op::i32_rotr_sss,
            Op::i32_rotr_ssi,
            Op::i32_rotr_sis,
            wasm::i32_rotr,
        )
    }

    #[inline(never)]
    fn visit_i64_clz(&mut self) -> Self::Output {
        self.translate_unary::<i64, i64>(Op::i64_clz_ss, wasm::i64_clz)
    }

    #[inline(never)]
    fn visit_i64_ctz(&mut self) -> Self::Output {
        self.translate_unary::<i64, i64>(Op::i64_ctz_ss, wasm::i64_ctz)
    }

    #[inline(never)]
    fn visit_i64_popcnt(&mut self) -> Self::Output {
        self.translate_unary::<i64, i64>(Op::i64_popcnt_ss, wasm::i64_popcnt)
    }

    #[inline(never)]
    fn visit_i64_add(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, i64>(
            Op::i64_add_sss,
            Op::i64_add_ssi,
            wasm::i64_add,
            FuncTranslator::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_i64_sub(&mut self) -> Self::Output {
        self.translate_binary(
            Op::i64_sub_sss,
            Op::i64_sub_ssi,
            Op::i64_sub_sis,
            wasm::i64_sub,
        )
    }

    #[inline(never)]
    fn visit_i64_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, i64>(
            Op::i64_mul_sss,
            Op::i64_mul_ssi,
            wasm::i64_mul,
            FuncTranslator::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_i64_div_s(&mut self) -> Self::Output {
        self.translate_divrem::<i64>(
            Op::i64_div_sss,
            Op::i64_div_ssi,
            Op::i64_div_sis,
            wasm::i64_div_s,
        )
    }

    #[inline(never)]
    fn visit_i64_div_u(&mut self) -> Self::Output {
        self.translate_divrem::<u64>(
            Op::u64_div_sss,
            Op::u64_div_ssi,
            Op::u64_div_sis,
            wasm::i64_div_u,
        )
    }

    #[inline(never)]
    fn visit_i64_rem_s(&mut self) -> Self::Output {
        self.translate_divrem::<i64>(
            Op::i64_rem_sss,
            Op::i64_rem_ssi,
            Op::i64_rem_sis,
            wasm::i64_rem_s,
        )
    }

    #[inline(never)]
    fn visit_i64_rem_u(&mut self) -> Self::Output {
        self.translate_divrem::<u64>(
            Op::u64_rem_sss,
            Op::u64_rem_ssi,
            Op::u64_rem_sis,
            wasm::i64_rem_u,
        )
    }

    #[inline(never)]
    fn visit_i64_and(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, i64>(
            Op::i64_bitand_sss,
            Op::i64_bitand_ssi,
            wasm::i64_bitand,
            FuncTranslator::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_i64_or(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, i64>(
            Op::i64_bitor_sss,
            Op::i64_bitor_ssi,
            wasm::i64_bitor,
            FuncTranslator::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_i64_xor(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, i64>(
            Op::i64_bitxor_sss,
            Op::i64_bitxor_ssi,
            wasm::i64_bitxor,
            FuncTranslator::no_opt_ri,
        )
    }

    #[inline(never)]
    fn visit_i64_shl(&mut self) -> Self::Output {
        self.translate_shift::<i64>(
            Op::i64_shl_sss,
            Op::i64_shl_ssi,
            Op::i64_shl_sis,
            wasm::i64_shl,
        )
    }

    #[inline(never)]
    fn visit_i64_shr_s(&mut self) -> Self::Output {
        self.translate_shift::<i64>(
            Op::i64_shr_sss,
            Op::i64_shr_ssi,
            Op::i64_shr_sis,
            wasm::i64_shr_s,
        )
    }

    #[inline(never)]
    fn visit_i64_shr_u(&mut self) -> Self::Output {
        self.translate_shift::<u64>(
            Op::u64_shr_sss,
            Op::u64_shr_ssi,
            Op::u64_shr_sis,
            wasm::i64_shr_u,
        )
    }

    #[inline(never)]
    fn visit_i64_rotl(&mut self) -> Self::Output {
        self.translate_shift::<i64>(
            Op::i64_rotl_sss,
            Op::i64_rotl_ssi,
            Op::i64_rotl_sis,
            wasm::i64_rotl,
        )
    }

    #[inline(never)]
    fn visit_i64_rotr(&mut self) -> Self::Output {
        self.translate_shift::<i64>(
            Op::i64_rotr_sss,
            Op::i64_rotr_ssi,
            Op::i64_rotr_sis,
            wasm::i64_rotr,
        )
    }

    #[inline(never)]
    fn visit_f32_abs(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_abs_ss, wasm::f32_abs)
    }

    #[inline(never)]
    fn visit_f32_neg(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_neg_ss, wasm::f32_neg)
    }

    #[inline(never)]
    fn visit_f32_ceil(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_ceil_ss, wasm::f32_ceil)
    }

    #[inline(never)]
    fn visit_f32_floor(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_floor_ss, wasm::f32_floor)
    }

    #[inline(never)]
    fn visit_f32_trunc(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_trunc_ss, wasm::f32_trunc)
    }

    #[inline(never)]
    fn visit_f32_nearest(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_nearest_ss, wasm::f32_nearest)
    }

    #[inline(never)]
    fn visit_f32_sqrt(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_sqrt_ss, wasm::f32_sqrt)
    }

    #[inline(never)]
    fn visit_f32_add(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f32_add_sss,
            Op::f32_add_ssi,
            Op::f32_add_sis,
            wasm::f32_add,
        )
    }

    #[inline(never)]
    fn visit_f32_sub(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f32_sub_sss,
            Op::f32_sub_ssi,
            Op::f32_sub_sis,
            wasm::f32_sub,
        )
    }

    #[inline(never)]
    fn visit_f32_mul(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f32_mul_sss,
            Op::f32_mul_ssi,
            Op::f32_mul_sis,
            wasm::f32_mul,
        )
    }

    #[inline(never)]
    fn visit_f32_div(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f32_div_sss,
            Op::f32_div_ssi,
            Op::f32_div_sis,
            wasm::f32_div,
        )
    }

    #[inline(never)]
    fn visit_f32_min(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f32_min_sss,
            Op::f32_min_ssi,
            Op::f32_min_sis,
            wasm::f32_min,
        )
    }

    #[inline(never)]
    fn visit_f32_max(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f32_max_sss,
            Op::f32_max_ssi,
            Op::f32_max_sis,
            wasm::f32_max,
        )
    }

    #[inline(never)]
    fn visit_f32_copysign(&mut self) -> Self::Output {
        self.translate_fcopysign::<f32>(
            Op::f32_copysign_sss,
            Op::f32_copysign_ssi,
            Op::f32_copysign_sis,
            wasm::f32_copysign,
        )
    }

    #[inline(never)]
    fn visit_f64_abs(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_abs_ss, wasm::f64_abs)
    }

    #[inline(never)]
    fn visit_f64_neg(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_neg_ss, wasm::f64_neg)
    }

    #[inline(never)]
    fn visit_f64_ceil(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_ceil_ss, wasm::f64_ceil)
    }

    #[inline(never)]
    fn visit_f64_floor(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_floor_ss, wasm::f64_floor)
    }

    #[inline(never)]
    fn visit_f64_trunc(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_trunc_ss, wasm::f64_trunc)
    }

    #[inline(never)]
    fn visit_f64_nearest(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_nearest_ss, wasm::f64_nearest)
    }

    #[inline(never)]
    fn visit_f64_sqrt(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_sqrt_ss, wasm::f64_sqrt)
    }

    #[inline(never)]
    fn visit_f64_add(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f64_add_sss,
            Op::f64_add_ssi,
            Op::f64_add_sis,
            wasm::f64_add,
        )
    }

    #[inline(never)]
    fn visit_f64_sub(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f64_sub_sss,
            Op::f64_sub_ssi,
            Op::f64_sub_sis,
            wasm::f64_sub,
        )
    }

    #[inline(never)]
    fn visit_f64_mul(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f64_mul_sss,
            Op::f64_mul_ssi,
            Op::f64_mul_sis,
            wasm::f64_mul,
        )
    }

    #[inline(never)]
    fn visit_f64_div(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f64_div_sss,
            Op::f64_div_ssi,
            Op::f64_div_sis,
            wasm::f64_div,
        )
    }

    #[inline(never)]
    fn visit_f64_min(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f64_min_sss,
            Op::f64_min_ssi,
            Op::f64_min_sis,
            wasm::f64_min,
        )
    }

    #[inline(never)]
    fn visit_f64_max(&mut self) -> Self::Output {
        self.translate_binary(
            Op::f64_max_sss,
            Op::f64_max_ssi,
            Op::f64_max_sis,
            wasm::f64_max,
        )
    }

    #[inline(never)]
    fn visit_f64_copysign(&mut self) -> Self::Output {
        self.translate_fcopysign::<f64>(
            Op::f64_copysign_sss,
            Op::f64_copysign_ssi,
            Op::f64_copysign_sis,
            wasm::f64_copysign,
        )
    }

    #[inline(never)]
    fn visit_i32_wrap_i64(&mut self) -> Self::Output {
        self.translate_unary(Op::i32_wrap_i64_ss, wasm::i32_wrap_i64)
    }

    #[inline(never)]
    fn visit_i32_trunc_f32_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Op::i32_trunc_f32_ss, wasm::i32_trunc_f32_s)
    }

    #[inline(never)]
    fn visit_i32_trunc_f32_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Op::u32_trunc_f32_ss, wasm::i32_trunc_f32_u)
    }

    #[inline(never)]
    fn visit_i32_trunc_f64_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Op::i32_trunc_f64_ss, wasm::i32_trunc_f64_s)
    }

    #[inline(never)]
    fn visit_i32_trunc_f64_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Op::u32_trunc_f64_ss, wasm::i32_trunc_f64_u)
    }

    #[inline(never)]
    fn visit_i64_extend_i32_s(&mut self) -> Self::Output {
        self.translate_unary::<i32, i64>(Op::i64_sext32_ss, wasm::i64_extend_i32_s)
    }

    #[inline(never)]
    fn visit_i64_extend_i32_u(&mut self) -> Self::Output {
        self.translate_reinterpret(wasm::i64_extend_i32_u)
    }

    #[inline(never)]
    fn visit_i64_trunc_f32_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Op::i64_trunc_f32_ss, wasm::i64_trunc_f32_s)
    }

    #[inline(never)]
    fn visit_i64_trunc_f32_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Op::u64_trunc_f32_ss, wasm::i64_trunc_f32_u)
    }

    #[inline(never)]
    fn visit_i64_trunc_f64_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Op::i64_trunc_f64_ss, wasm::i64_trunc_f64_s)
    }

    #[inline(never)]
    fn visit_i64_trunc_f64_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Op::u64_trunc_f64_ss, wasm::i64_trunc_f64_u)
    }

    #[inline(never)]
    fn visit_f32_convert_i32_s(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_convert_i32_ss, wasm::f32_convert_i32_s)
    }

    #[inline(never)]
    fn visit_f32_convert_i32_u(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_convert_u32_ss, wasm::f32_convert_i32_u)
    }

    #[inline(never)]
    fn visit_f32_convert_i64_s(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_convert_i64_ss, wasm::f32_convert_i64_s)
    }

    #[inline(never)]
    fn visit_f32_convert_i64_u(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_convert_u64_ss, wasm::f32_convert_i64_u)
    }

    #[inline(never)]
    fn visit_f32_demote_f64(&mut self) -> Self::Output {
        self.translate_unary(Op::f32_demote_f64_ss, wasm::f32_demote_f64)
    }

    #[inline(never)]
    fn visit_f64_convert_i32_s(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_convert_i32_ss, wasm::f64_convert_i32_s)
    }

    #[inline(never)]
    fn visit_f64_convert_i32_u(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_convert_u32_ss, wasm::f64_convert_i32_u)
    }

    #[inline(never)]
    fn visit_f64_convert_i64_s(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_convert_i64_ss, wasm::f64_convert_i64_s)
    }

    #[inline(never)]
    fn visit_f64_convert_i64_u(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_convert_u64_ss, wasm::f64_convert_i64_u)
    }

    #[inline(never)]
    fn visit_f64_promote_f32(&mut self) -> Self::Output {
        self.translate_unary(Op::f64_promote_f32_ss, wasm::f64_promote_f32)
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
        self.translate_unary(Op::i32_sext8_ss, wasm::i32_extend8_s)
    }

    #[inline(never)]
    fn visit_i32_extend16_s(&mut self) -> Self::Output {
        self.translate_unary(Op::i32_sext16_ss, wasm::i32_extend16_s)
    }

    #[inline(never)]
    fn visit_i64_extend8_s(&mut self) -> Self::Output {
        self.translate_unary(Op::i64_sext8_ss, wasm::i64_extend8_s)
    }

    #[inline(never)]
    fn visit_i64_extend16_s(&mut self) -> Self::Output {
        self.translate_unary(Op::i64_sext16_ss, wasm::i64_extend16_s)
    }

    #[inline(never)]
    fn visit_i64_extend32_s(&mut self) -> Self::Output {
        self.translate_unary(Op::i64_sext32_ss, wasm::i64_extend32_s)
    }

    #[inline(never)]
    fn visit_i32_trunc_sat_f32_s(&mut self) -> Self::Output {
        self.translate_unary(Op::i32_trunc_sat_f32_ss, wasm::i32_trunc_sat_f32_s)
    }

    #[inline(never)]
    fn visit_i32_trunc_sat_f32_u(&mut self) -> Self::Output {
        self.translate_unary(Op::u32_trunc_sat_f32_ss, wasm::i32_trunc_sat_f32_u)
    }

    #[inline(never)]
    fn visit_i32_trunc_sat_f64_s(&mut self) -> Self::Output {
        self.translate_unary(Op::i32_trunc_sat_f64_ss, wasm::i32_trunc_sat_f64_s)
    }

    #[inline(never)]
    fn visit_i32_trunc_sat_f64_u(&mut self) -> Self::Output {
        self.translate_unary(Op::u32_trunc_sat_f64_ss, wasm::i32_trunc_sat_f64_u)
    }

    #[inline(never)]
    fn visit_i64_trunc_sat_f32_s(&mut self) -> Self::Output {
        self.translate_unary(Op::i64_trunc_sat_f32_ss, wasm::i64_trunc_sat_f32_s)
    }

    #[inline(never)]
    fn visit_i64_trunc_sat_f32_u(&mut self) -> Self::Output {
        self.translate_unary(Op::u64_trunc_sat_f32_ss, wasm::i64_trunc_sat_f32_u)
    }

    #[inline(never)]
    fn visit_i64_trunc_sat_f64_s(&mut self) -> Self::Output {
        self.translate_unary(Op::i64_trunc_sat_f64_ss, wasm::i64_trunc_sat_f64_s)
    }

    #[inline(never)]
    fn visit_i64_trunc_sat_f64_u(&mut self) -> Self::Output {
        self.translate_unary(Op::u64_trunc_sat_f64_ss, wasm::i64_trunc_sat_f64_u)
    }

    #[inline(never)]
    fn visit_memory_init(&mut self, data_index: u32, mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.stack.pop3();
        let memory = index::Memory::try_from(mem)?;
        let data = index::Data::from(data_index);
        let dst = self.copy_if_immediate(dst)?;
        let src = self.copy_if_immediate(src)?;
        let len = self.copy_if_immediate(len)?;
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
        let dst = self.copy_if_immediate(dst)?;
        let src = self.copy_if_immediate(src)?;
        let len = self.copy_if_immediate(len)?;
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
        let dst = self.copy_if_immediate(dst)?;
        let len = self.copy_if_immediate(len)?;
        let value = self.copy_if_immediate(value)?;
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
        let dst = self.copy_if_immediate(dst)?;
        let src = self.copy_if_immediate(src)?;
        let len = self.copy_if_immediate(len)?;
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
        let dst = self.copy_if_immediate(dst)?;
        let src = self.copy_if_immediate(src)?;
        let len = self.copy_if_immediate(len)?;
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
            ty => panic!("expected a Wasm `reftype` but found: {ty:?}"),
        };
        self.stack.push_immediate(null)?;
        Ok(())
    }

    #[inline(never)]
    fn visit_ref_is_null(&mut self) -> Self::Output {
        bail_unreachable!(self);
        match self.stack.pop() {
            Operand::Local(input) => {
                // Note: `funcref` and `externref` both serialize to `RawValue`
                //       as `u64` so we can use `i64.eqz` translation for `ref.is_null`
                //       via reinterpretation of the value's type.
                self.stack.push_local(input.local_index(), ValType::I64)?;
                self.visit_i64_eqz()
            }
            Operand::Temp(_) => {
                // Note: `funcref` and `externref` both serialize to `RawValue`
                //       as `u64` so we can use `i64.eqz` translation for `ref.is_null`
                //       via reinterpretation of the value's type.
                self.stack.push_temp(ValType::I64)?;
                self.visit_i64_eqz()
            }
            Operand::Immediate(input) => {
                let raw = input.val().raw();
                let is_null = match input.ty() {
                    ValType::FuncRef => <Nullable<Func>>::from(raw).is_null(),
                    ValType::ExternRef => <Nullable<ExternRef>>::from(raw).is_null(),
                    invalid => panic!("`ref.is_null`: encountered invalid input type: {invalid:?}"),
                };
                self.stack.push_immediate(i32::from(is_null))?;
                Ok(())
            }
        }
    }

    #[inline(never)]
    fn visit_ref_func(&mut self, function_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.push_instr_with_result(
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
        let dst = self.copy_if_immediate(dst)?;
        let value = self.copy_if_immediate(value)?;
        let len = self.copy_if_immediate(len)?;
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
        let index = self.make_index32_or_copy(index, index_ty)?;
        self.push_instr_with_result(
            item_ty.into(),
            |result| match index {
                Input::Slot(index) => Op::table_get_ss(result, index, table),
                Input::Immediate(index) => Op::table_get_si(result, index, table),
            },
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    #[inline(never)]
    fn visit_table_set(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let table_type = *self.module.get_type_of_table(TableIdx::from(table));
        let table = index::Table::from(table);
        let index_ty = table_type.index_ty();
        let (index, value) = self.stack.pop2();
        let index = self.make_index32_or_copy(index, index_ty)?;
        let value = self.make_input(value, |_this, value| {
            Ok(Input::Immediate(u32::from(value.raw())))
        })?;
        let instr = match (index, value) {
            (Input::Slot(index), Input::Slot(value)) => Op::table_set_ss(table, index, value),
            (Input::Slot(index), Input::Immediate(value)) => Op::table_set_si(table, index, value),
            (Input::Immediate(index), Input::Slot(value)) => Op::table_set_is(table, index, value),
            (Input::Immediate(index), Input::Immediate(value)) => {
                Op::table_set_ii(table, index, value)
            }
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
                self.push_instr_with_result(
                    index_ty.ty(),
                    |result| Op::table_size(result, table),
                    FuelCostsProvider::instance,
                )?;
                return Ok(());
            }
        }
        let value = self.copy_if_immediate(value)?;
        let delta = self.copy_if_immediate(delta)?;
        self.push_instr_with_result(
            index_ty.ty(),
            |result| Op::table_grow(result, delta, value, table),
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
        self.push_instr_with_result(
            index_ty.ty(),
            |result| Op::table_size(result, table),
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
        self.translate_call_indirect(type_index, table_index, Op::return_call_indirect)?;
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
