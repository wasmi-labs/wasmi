use super::{ControlFrame, ControlFrameKind, FuncTranslator, LocalIdx};
use crate::{
    core::{wasm, FuelCostsProvider, IndexType, Mutability, TrapCode, TypedVal, ValType, F32, F64},
    engine::{
        translator::func2::{
            op,
            stack::{AcquiredTarget, IfReachability},
            ControlFrameBase,
            Input,
            Operand,
        },
        BlockType,
    },
    ir::{self, Const16, Instruction},
    module::{self, FuncIdx, MemoryIdx, TableIdx, WasmiValueType},
    Error,
    ExternRef,
    FuncRef,
    FuncType,
};
use ir::Const32;
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
        bail_unreachable!(self);
        self.translate_trap(TrapCode::UnreachableCodeReached)
    }

    fn visit_nop(&mut self) -> Self::Output {
        Ok(())
    }

    fn visit_block(&mut self, block_ty: wasmparser::BlockType) -> Self::Output {
        if !self.reachable {
            self.stack.push_unreachable(ControlFrameKind::Block)?;
            return Ok(());
        }
        let block_ty = BlockType::new(block_ty, &self.module);
        let end_label = self.labels.new_label();
        self.stack.push_block(block_ty, end_label)?;
        Ok(())
    }

    fn visit_loop(&mut self, block_ty: wasmparser::BlockType) -> Self::Output {
        if !self.reachable {
            self.stack.push_unreachable(ControlFrameKind::Loop)?;
            return Ok(());
        }
        let block_ty = BlockType::new(block_ty, &self.module);
        let len_params = block_ty.len_params(&self.engine);
        let continue_label = self.labels.new_label();
        let consume_fuel = self.stack.consume_fuel_instr();
        self.move_operands_to_temp(usize::from(len_params), consume_fuel)?;
        self.pin_label(continue_label);
        let consume_fuel = self.instrs.push_consume_fuel_instr()?;
        self.stack
            .push_loop(block_ty, continue_label, consume_fuel)?;
        Ok(())
    }

    fn visit_if(&mut self, block_ty: wasmparser::BlockType) -> Self::Output {
        if !self.reachable {
            self.stack.push_unreachable(ControlFrameKind::If)?;
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
                self.encode_br_eqz(condition, else_label)?;
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
                let branch_results = Self::frame_results_impl(&frame, &self.engine, &self.layout)?;
                if let Some(branch_results) = branch_results {
                    self.encode_copies(branch_results, len_params, consume_fuel_instr)?;
                }
                self.encode_br(label)?;
                self.reachable = false;
                Ok(())
            }
        }
    }

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
        let len_branch_params = frame.len_branch_params(&self.engine);
        let branch_results = Self::frame_results_impl(&frame, &self.engine, &self.layout)?;
        let label = frame.label();
        if len_branch_params == 0 {
            // Case: no branch values are required to be copied
            self.encode_br_nez(condition, label)?;
            return Ok(());
        }
        if !self.requires_branch_param_copies(depth) {
            // Case: no branch values are required to be copied
            self.encode_br_nez(condition, label)?;
            return Ok(());
        }
        // Case: fallback to copy branch parameters conditionally
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        let skip_label = self.labels.new_label();
        self.encode_br_eqz(condition, skip_label)?;
        if let Some(branch_results) = branch_results {
            self.encode_copies(branch_results, len_branch_params, consume_fuel_instr)?;
        }
        self.encode_br(label)?;
        self.labels
            .pin_label(skip_label, self.instrs.next_instr())
            .unwrap();
        Ok(())
    }

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
        let index = self.layout.operand_to_reg(index)?;
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

    fn visit_call(&mut self, function_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let func_idx = FuncIdx::from(function_index);
        let func_type = self.resolve_func_type(func_idx);
        let len_params = usize::from(func_type.len_params());
        let results = self.call_regspan(len_params)?;
        let instr = match self.module.get_engine_func(func_idx) {
            Some(engine_func) => {
                // Case: We are calling an internal function and can optimize
                //       this case by using the special instruction for it.
                match len_params {
                    0 => Instruction::call_internal_0(results, engine_func),
                    _ => Instruction::call_internal(results, engine_func),
                }
            }
            None => {
                // Case: We are calling an imported function and must use the
                //       general calling operator for it.
                match len_params {
                    0 => Instruction::call_imported_0(results, function_index),
                    _ => Instruction::call_imported(results, function_index),
                }
            }
        };
        let call_instr = self.push_instr(instr, FuelCostsProvider::call)?;
        self.stack.pop_n(len_params, &mut self.operands);
        self.instrs
            .encode_register_list(&self.operands, &mut self.layout)?;
        if let Some(span) = self.push_results(call_instr, func_type.results())? {
            debug_assert_eq!(span, results);
        }
        Ok(())
    }

    fn visit_call_indirect(&mut self, type_index: u32, table_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let func_type = self.resolve_type(type_index);
        let index = self.stack.pop();
        let indirect_params = self.call_indirect_params(index, table_index)?;
        let len_params = usize::from(func_type.len_params());
        let results = self.call_regspan(len_params)?;
        let instr = match (len_params, indirect_params) {
            (0, Instruction::CallIndirectParams { .. }) => {
                Instruction::call_indirect_0(results, type_index)
            }
            (0, Instruction::CallIndirectParamsImm16 { .. }) => {
                Instruction::call_indirect_0_imm16(results, type_index)
            }
            (_, Instruction::CallIndirectParams { .. }) => {
                Instruction::call_indirect(results, type_index)
            }
            (_, Instruction::CallIndirectParamsImm16 { .. }) => {
                Instruction::call_indirect_imm16(results, type_index)
            }
            _ => unreachable!(),
        };
        let call_instr = self.push_instr(instr, FuelCostsProvider::call)?;
        self.push_param(indirect_params)?;
        self.stack.pop_n(len_params, &mut self.operands);
        self.instrs
            .encode_register_list(&self.operands, &mut self.layout)?;
        if let Some(span) = self.push_results(call_instr, func_type.results())? {
            debug_assert_eq!(span, results);
        }
        Ok(())
    }

    fn visit_drop(&mut self) -> Self::Output {
        bail_unreachable!(self);
        _ = self.stack.pop();
        Ok(())
    }

    fn visit_select(&mut self) -> Self::Output {
        self.translate_select(None)
    }

    fn visit_local_get(&mut self, local_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_local(LocalIdx::from(local_index))?;
        Ok(())
    }

    fn visit_local_set(&mut self, local_index: u32) -> Self::Output {
        self.translate_local_set(local_index, false)
    }

    fn visit_local_tee(&mut self, local_index: u32) -> Self::Output {
        self.translate_local_set(local_index, true)
    }

    fn visit_global_get(&mut self, global_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let global_idx = module::GlobalIdx::from(global_index);
        let (global_type, init_value) = self.module.get_global(global_idx);
        let content = global_type.content();
        if let (Mutability::Const, Some(init_expr)) = (global_type.mutability(), init_value) {
            if let Some(value) = init_expr.eval_const() {
                // Case: access to immutable internally defined global variables
                //       can be replaced with their constant initialization value.
                self.stack.push_immediate(TypedVal::new(content, value))?;
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
        self.push_instr_with_result(
            content,
            |result| Instruction::global_get(result, global_idx),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    fn visit_global_set(&mut self, global_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let global = ir::index::Global::from(global_index);
        let input = match self.stack.pop() {
            Operand::Immediate(input) => input.val(),
            input => {
                // Case: `global.set` with simple register input.
                let input = self.layout.operand_to_reg(input)?;
                self.push_instr(
                    Instruction::global_set(input, global),
                    FuelCostsProvider::instance,
                )?;
                return Ok(());
            }
        };
        // Note: at this point we handle the different immediate `global.set` instructions.
        let (global_type, _init_value) = self
            .module
            .get_global(module::GlobalIdx::from(global_index));
        debug_assert_eq!(global_type.content(), input.ty());
        match global_type.content() {
            ValType::I32 => {
                if let Ok(value) = Const16::try_from(i32::from(input)) {
                    // Case: `global.set` with 16-bit encoded `i32` value.
                    self.push_instr(
                        Instruction::global_set_i32imm16(value, global),
                        FuelCostsProvider::instance,
                    )?;
                    return Ok(());
                }
            }
            ValType::I64 => {
                if let Ok(value) = Const16::try_from(i64::from(input)) {
                    // Case: `global.set` with 16-bit encoded `i64` value.
                    self.push_instr(
                        Instruction::global_set_i64imm16(value, global),
                        FuelCostsProvider::instance,
                    )?;
                    return Ok(());
                }
            }
            _ => {}
        };
        // Note: at this point we have to allocate a function local constant.
        let cref = self.layout.const_to_reg(input)?;
        self.push_instr(
            Instruction::global_set(cref, global),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    fn visit_i32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I32,
            Instruction::load32,
            Instruction::load32_offset16,
            Instruction::load32_at,
        )
    }

    fn visit_i64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I64,
            Instruction::load64,
            Instruction::load64_offset16,
            Instruction::load64_at,
        )
    }

    fn visit_f32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::F32,
            Instruction::load32,
            Instruction::load32_offset16,
            Instruction::load32_at,
        )
    }

    fn visit_f64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::F64,
            Instruction::load64,
            Instruction::load64_offset16,
            Instruction::load64_at,
        )
    }

    fn visit_i32_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I32,
            Instruction::i32_load8_s,
            Instruction::i32_load8_s_offset16,
            Instruction::i32_load8_s_at,
        )
    }

    fn visit_i32_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I32,
            Instruction::i32_load8_u,
            Instruction::i32_load8_u_offset16,
            Instruction::i32_load8_u_at,
        )
    }

    fn visit_i32_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I32,
            Instruction::i32_load16_s,
            Instruction::i32_load16_s_offset16,
            Instruction::i32_load16_s_at,
        )
    }

    fn visit_i32_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I32,
            Instruction::i32_load16_u,
            Instruction::i32_load16_u_offset16,
            Instruction::i32_load16_u_at,
        )
    }

    fn visit_i64_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I64,
            Instruction::i64_load8_s,
            Instruction::i64_load8_s_offset16,
            Instruction::i64_load8_s_at,
        )
    }

    fn visit_i64_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I64,
            Instruction::i64_load8_u,
            Instruction::i64_load8_u_offset16,
            Instruction::i64_load8_u_at,
        )
    }

    fn visit_i64_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I64,
            Instruction::i64_load16_s,
            Instruction::i64_load16_s_offset16,
            Instruction::i64_load16_s_at,
        )
    }

    fn visit_i64_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I64,
            Instruction::i64_load16_u,
            Instruction::i64_load16_u_offset16,
            Instruction::i64_load16_u_at,
        )
    }

    fn visit_i64_load32_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I64,
            Instruction::i64_load32_s,
            Instruction::i64_load32_s_offset16,
            Instruction::i64_load32_s_at,
        )
    }

    fn visit_i64_load32_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::I64,
            Instruction::i64_load32_u,
            Instruction::i64_load32_u_offset16,
            Instruction::i64_load32_u_at,
        )
    }

    fn visit_i32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore_wrap::<op::I32Store>(memarg)
    }

    fn visit_i64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore_wrap::<op::I64Store>(memarg)
    }

    fn visit_f32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store(
            memarg,
            Instruction::store32,
            Instruction::store32_offset16,
            Instruction::store32_at,
        )
    }

    fn visit_f64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store(
            memarg,
            Instruction::store64,
            Instruction::store64_offset16,
            Instruction::store64_at,
        )
    }

    fn visit_i32_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore_wrap::<op::I32Store8>(memarg)
    }

    fn visit_i32_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore_wrap::<op::I32Store16>(memarg)
    }

    fn visit_i64_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore_wrap::<op::I64Store8>(memarg)
    }

    fn visit_i64_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore_wrap::<op::I64Store16>(memarg)
    }

    fn visit_i64_store32(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore_wrap::<op::I64Store32>(memarg)
    }

    fn visit_memory_size(&mut self, mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let index_ty = self
            .module
            .get_type_of_memory(MemoryIdx::from(mem))
            .index_ty()
            .ty();
        self.push_instr_with_result(
            index_ty,
            |result| Instruction::memory_size(result, mem),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    fn visit_memory_grow(&mut self, mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let index_ty = self
            .module
            .get_type_of_memory(MemoryIdx::from(mem))
            .index_ty();
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
                    |result| Instruction::memory_size(result, mem),
                    FuelCostsProvider::instance,
                )?;
                return Ok(());
            }
            if let Ok(delta) = <Const32<u64>>::try_from(delta) {
                // Case: delta can be 32-bit encoded
                self.push_instr_with_result(
                    index_ty.ty(),
                    |result| Instruction::memory_grow_imm(result, delta),
                    FuelCostsProvider::instance,
                )?;
                self.push_param(Instruction::memory_index(mem))?;
                return Ok(());
            }
        }
        // Case: fallback to generic `memory.grow` instruction
        let delta = self.layout.operand_to_reg(delta)?;
        self.push_instr_with_result(
            index_ty.ty(),
            |result| Instruction::memory_grow(result, delta),
            FuelCostsProvider::instance,
        )?;
        self.push_param(Instruction::memory_index(mem))?;
        Ok(())
    }

    fn visit_i32_const(&mut self, value: i32) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_immediate(value)?;
        Ok(())
    }

    fn visit_i64_const(&mut self, value: i64) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_immediate(value)?;
        Ok(())
    }

    fn visit_f32_const(&mut self, value: wasmparser::Ieee32) -> Self::Output {
        bail_unreachable!(self);
        let value = F32::from_bits(value.bits());
        self.stack.push_immediate(value)?;
        Ok(())
    }

    fn visit_f64_const(&mut self, value: wasmparser::Ieee64) -> Self::Output {
        bail_unreachable!(self);
        let value = F64::from_bits(value.bits());
        self.stack.push_immediate(value)?;
        Ok(())
    }

    fn visit_i32_eqz(&mut self) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_immediate(0_i32)?;
        self.visit_i32_eq()
    }

    fn visit_i32_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, bool>(
            Instruction::i32_eq,
            Instruction::i32_eq_imm16,
            wasm::i32_eq,
            FuncTranslator::fuse_eqz,
        )
    }

    fn visit_i32_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, bool>(
            Instruction::i32_ne,
            Instruction::i32_ne_imm16,
            wasm::i32_ne,
            FuncTranslator::fuse_nez,
        )
    }

    fn visit_i32_lt_s(&mut self) -> Self::Output {
        self.translate_binary::<i32, bool>(
            Instruction::i32_lt_s,
            Instruction::i32_lt_s_imm16_rhs,
            Instruction::i32_lt_s_imm16_lhs,
            wasm::i32_lt_s,
        )
    }

    fn visit_i32_lt_u(&mut self) -> Self::Output {
        self.translate_binary::<u32, bool>(
            Instruction::i32_lt_u,
            Instruction::i32_lt_u_imm16_rhs,
            Instruction::i32_lt_u_imm16_lhs,
            wasm::i32_lt_u,
        )
    }

    fn visit_i32_gt_s(&mut self) -> Self::Output {
        self.translate_binary::<i32, bool>(
            swap_ops!(Instruction::i32_lt_s),
            swap_ops!(Instruction::i32_lt_s_imm16_lhs),
            swap_ops!(Instruction::i32_lt_s_imm16_rhs),
            wasm::i32_gt_s,
        )
    }

    fn visit_i32_gt_u(&mut self) -> Self::Output {
        self.translate_binary::<u32, bool>(
            swap_ops!(Instruction::i32_lt_u),
            swap_ops!(Instruction::i32_lt_u_imm16_lhs),
            swap_ops!(Instruction::i32_lt_u_imm16_rhs),
            wasm::i32_gt_u,
        )
    }

    fn visit_i32_le_s(&mut self) -> Self::Output {
        self.translate_binary::<i32, bool>(
            Instruction::i32_le_s,
            Instruction::i32_le_s_imm16_rhs,
            Instruction::i32_le_s_imm16_lhs,
            wasm::i32_le_s,
        )
    }

    fn visit_i32_le_u(&mut self) -> Self::Output {
        self.translate_binary::<u32, bool>(
            Instruction::i32_le_u,
            Instruction::i32_le_u_imm16_rhs,
            Instruction::i32_le_u_imm16_lhs,
            wasm::i32_le_u,
        )
    }

    fn visit_i32_ge_s(&mut self) -> Self::Output {
        self.translate_binary::<i32, bool>(
            swap_ops!(Instruction::i32_le_s),
            swap_ops!(Instruction::i32_le_s_imm16_lhs),
            swap_ops!(Instruction::i32_le_s_imm16_rhs),
            wasm::i32_ge_s,
        )
    }

    fn visit_i32_ge_u(&mut self) -> Self::Output {
        self.translate_binary::<u32, bool>(
            swap_ops!(Instruction::i32_le_u),
            swap_ops!(Instruction::i32_le_u_imm16_lhs),
            swap_ops!(Instruction::i32_le_u_imm16_rhs),
            wasm::i32_ge_u,
        )
    }

    fn visit_i64_eqz(&mut self) -> Self::Output {
        bail_unreachable!(self);
        self.stack.push_immediate(0_i64)?;
        self.visit_i64_eq()
    }

    fn visit_i64_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, bool>(
            Instruction::i64_eq,
            Instruction::i64_eq_imm16,
            wasm::i64_eq,
            FuncTranslator::fuse_eqz,
        )
    }

    fn visit_i64_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, bool>(
            Instruction::i64_ne,
            Instruction::i64_ne_imm16,
            wasm::i64_ne,
            FuncTranslator::fuse_nez,
        )
    }

    fn visit_i64_lt_s(&mut self) -> Self::Output {
        self.translate_binary::<i64, bool>(
            Instruction::i64_lt_s,
            Instruction::i64_lt_s_imm16_rhs,
            Instruction::i64_lt_s_imm16_lhs,
            wasm::i64_lt_s,
        )
    }

    fn visit_i64_lt_u(&mut self) -> Self::Output {
        self.translate_binary::<u64, bool>(
            Instruction::i64_lt_u,
            Instruction::i64_lt_u_imm16_rhs,
            Instruction::i64_lt_u_imm16_lhs,
            wasm::i64_lt_u,
        )
    }

    fn visit_i64_gt_s(&mut self) -> Self::Output {
        self.translate_binary::<i64, bool>(
            swap_ops!(Instruction::i64_lt_s),
            swap_ops!(Instruction::i64_lt_s_imm16_lhs),
            swap_ops!(Instruction::i64_lt_s_imm16_rhs),
            wasm::i64_gt_s,
        )
    }

    fn visit_i64_gt_u(&mut self) -> Self::Output {
        self.translate_binary::<u64, bool>(
            swap_ops!(Instruction::i64_lt_u),
            swap_ops!(Instruction::i64_lt_u_imm16_lhs),
            swap_ops!(Instruction::i64_lt_u_imm16_rhs),
            wasm::i64_gt_u,
        )
    }

    fn visit_i64_le_s(&mut self) -> Self::Output {
        self.translate_binary::<i64, bool>(
            Instruction::i64_le_s,
            Instruction::i64_le_s_imm16_rhs,
            Instruction::i64_le_s_imm16_lhs,
            wasm::i64_le_s,
        )
    }

    fn visit_i64_le_u(&mut self) -> Self::Output {
        self.translate_binary::<u64, bool>(
            Instruction::i64_le_u,
            Instruction::i64_le_u_imm16_rhs,
            Instruction::i64_le_u_imm16_lhs,
            wasm::i64_le_u,
        )
    }

    fn visit_i64_ge_s(&mut self) -> Self::Output {
        self.translate_binary::<i64, bool>(
            swap_ops!(Instruction::i64_le_s),
            swap_ops!(Instruction::i64_le_s_imm16_lhs),
            swap_ops!(Instruction::i64_le_s_imm16_rhs),
            wasm::i64_ge_s,
        )
    }

    fn visit_i64_ge_u(&mut self) -> Self::Output {
        self.translate_binary::<u64, bool>(
            swap_ops!(Instruction::i64_le_u),
            swap_ops!(Instruction::i64_le_u_imm16_lhs),
            swap_ops!(Instruction::i64_le_u_imm16_rhs),
            wasm::i64_ge_u,
        )
    }

    fn visit_f32_eq(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f32_eq, wasm::f32_eq)
    }

    fn visit_f32_ne(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f32_ne, wasm::f32_ne)
    }

    fn visit_f32_lt(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f32_lt, wasm::f32_lt)
    }

    fn visit_f32_gt(&mut self) -> Self::Output {
        self.translate_fbinary(swap_ops!(Instruction::f32_lt), wasm::f32_gt)
    }

    fn visit_f32_le(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f32_le, wasm::f32_le)
    }

    fn visit_f32_ge(&mut self) -> Self::Output {
        self.translate_fbinary(swap_ops!(Instruction::f32_le), wasm::f32_ge)
    }

    fn visit_f64_eq(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f64_eq, wasm::f64_eq)
    }

    fn visit_f64_ne(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f64_ne, wasm::f64_ne)
    }

    fn visit_f64_lt(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f64_lt, wasm::f64_lt)
    }

    fn visit_f64_gt(&mut self) -> Self::Output {
        self.translate_fbinary(swap_ops!(Instruction::f64_lt), wasm::f64_gt)
    }

    fn visit_f64_le(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f64_le, wasm::f64_le)
    }

    fn visit_f64_ge(&mut self) -> Self::Output {
        self.translate_fbinary(swap_ops!(Instruction::f64_le), wasm::f64_ge)
    }

    fn visit_i32_clz(&mut self) -> Self::Output {
        self.translate_unary::<i32, i32>(Instruction::i32_clz, wasm::i32_clz)
    }

    fn visit_i32_ctz(&mut self) -> Self::Output {
        self.translate_unary::<i32, i32>(Instruction::i32_ctz, wasm::i32_ctz)
    }

    fn visit_i32_popcnt(&mut self) -> Self::Output {
        self.translate_unary::<i32, i32>(Instruction::i32_popcnt, wasm::i32_popcnt)
    }

    fn visit_i32_add(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, i32>(
            Instruction::i32_add,
            Instruction::i32_add_imm16,
            wasm::i32_add,
            FuncTranslator::no_opt_ri,
        )
    }

    fn visit_i32_sub(&mut self) -> Self::Output {
        self.translate_isub(
            Instruction::i32_sub,
            Instruction::i32_add_imm16,
            Instruction::i32_sub_imm16_lhs,
            wasm::i32_sub,
        )
    }

    fn visit_i32_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, i32>(
            Instruction::i32_mul,
            Instruction::i32_mul_imm16,
            wasm::i32_mul,
            FuncTranslator::no_opt_ri,
        )
    }

    fn visit_i32_div_s(&mut self) -> Self::Output {
        self.translate_divrem::<i32>(
            Instruction::i32_div_s,
            Instruction::i32_div_s_imm16_rhs,
            Instruction::i32_div_s_imm16_lhs,
            wasm::i32_div_s,
        )
    }

    fn visit_i32_div_u(&mut self) -> Self::Output {
        self.translate_divrem::<u32>(
            Instruction::i32_div_u,
            Instruction::i32_div_u_imm16_rhs,
            Instruction::i32_div_u_imm16_lhs,
            wasm::i32_div_u,
        )
    }

    fn visit_i32_rem_s(&mut self) -> Self::Output {
        self.translate_divrem::<i32>(
            Instruction::i32_rem_s,
            Instruction::i32_rem_s_imm16_rhs,
            Instruction::i32_rem_s_imm16_lhs,
            wasm::i32_rem_s,
        )
    }

    fn visit_i32_rem_u(&mut self) -> Self::Output {
        self.translate_divrem::<u32>(
            Instruction::i32_rem_u,
            Instruction::i32_rem_u_imm16_rhs,
            Instruction::i32_rem_u_imm16_lhs,
            wasm::i32_rem_u,
        )
    }

    fn visit_i32_and(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, i32>(
            Instruction::i32_bitand,
            Instruction::i32_bitand_imm16,
            wasm::i32_bitand,
            FuncTranslator::no_opt_ri,
        )
    }

    fn visit_i32_or(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, i32>(
            Instruction::i32_bitor,
            Instruction::i32_bitor_imm16,
            wasm::i32_bitor,
            FuncTranslator::no_opt_ri,
        )
    }

    fn visit_i32_xor(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32, i32>(
            Instruction::i32_bitxor,
            Instruction::i32_bitxor_imm16,
            wasm::i32_bitxor,
            FuncTranslator::no_opt_ri,
        )
    }

    fn visit_i32_shl(&mut self) -> Self::Output {
        self.translate_shift::<i32>(
            Instruction::i32_shl,
            Instruction::i32_shl_by,
            Instruction::i32_shl_imm16,
            wasm::i32_shl,
        )
    }

    fn visit_i32_shr_s(&mut self) -> Self::Output {
        self.translate_shift::<i32>(
            Instruction::i32_shr_s,
            Instruction::i32_shr_s_by,
            Instruction::i32_shr_s_imm16,
            wasm::i32_shr_s,
        )
    }

    fn visit_i32_shr_u(&mut self) -> Self::Output {
        self.translate_shift::<i32>(
            Instruction::i32_shr_u,
            Instruction::i32_shr_u_by,
            Instruction::i32_shr_u_imm16,
            wasm::i32_shr_u,
        )
    }

    fn visit_i32_rotl(&mut self) -> Self::Output {
        self.translate_shift::<i32>(
            Instruction::i32_rotl,
            Instruction::i32_rotl_by,
            Instruction::i32_rotl_imm16,
            wasm::i32_rotl,
        )
    }

    fn visit_i32_rotr(&mut self) -> Self::Output {
        self.translate_shift::<i32>(
            Instruction::i32_rotr,
            Instruction::i32_rotr_by,
            Instruction::i32_rotr_imm16,
            wasm::i32_rotr,
        )
    }

    fn visit_i64_clz(&mut self) -> Self::Output {
        self.translate_unary::<i64, i64>(Instruction::i64_clz, wasm::i64_clz)
    }

    fn visit_i64_ctz(&mut self) -> Self::Output {
        self.translate_unary::<i64, i64>(Instruction::i64_ctz, wasm::i64_ctz)
    }

    fn visit_i64_popcnt(&mut self) -> Self::Output {
        self.translate_unary::<i64, i64>(Instruction::i64_popcnt, wasm::i64_popcnt)
    }

    fn visit_i64_add(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, i64>(
            Instruction::i64_add,
            Instruction::i64_add_imm16,
            wasm::i64_add,
            FuncTranslator::no_opt_ri,
        )
    }

    fn visit_i64_sub(&mut self) -> Self::Output {
        self.translate_isub(
            Instruction::i64_sub,
            Instruction::i64_add_imm16,
            Instruction::i64_sub_imm16_lhs,
            wasm::i64_sub,
        )
    }

    fn visit_i64_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, i64>(
            Instruction::i64_mul,
            Instruction::i64_mul_imm16,
            wasm::i64_mul,
            FuncTranslator::no_opt_ri,
        )
    }

    fn visit_i64_div_s(&mut self) -> Self::Output {
        self.translate_divrem::<i64>(
            Instruction::i64_div_s,
            Instruction::i64_div_s_imm16_rhs,
            Instruction::i64_div_s_imm16_lhs,
            wasm::i64_div_s,
        )
    }

    fn visit_i64_div_u(&mut self) -> Self::Output {
        self.translate_divrem::<u64>(
            Instruction::i64_div_u,
            Instruction::i64_div_u_imm16_rhs,
            Instruction::i64_div_u_imm16_lhs,
            wasm::i64_div_u,
        )
    }

    fn visit_i64_rem_s(&mut self) -> Self::Output {
        self.translate_divrem::<i64>(
            Instruction::i64_rem_s,
            Instruction::i64_rem_s_imm16_rhs,
            Instruction::i64_rem_s_imm16_lhs,
            wasm::i64_rem_s,
        )
    }

    fn visit_i64_rem_u(&mut self) -> Self::Output {
        self.translate_divrem::<u64>(
            Instruction::i64_rem_u,
            Instruction::i64_rem_u_imm16_rhs,
            Instruction::i64_rem_u_imm16_lhs,
            wasm::i64_rem_u,
        )
    }

    fn visit_i64_and(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, i64>(
            Instruction::i64_bitand,
            Instruction::i64_bitand_imm16,
            wasm::i64_bitand,
            FuncTranslator::no_opt_ri,
        )
    }

    fn visit_i64_or(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, i64>(
            Instruction::i64_bitor,
            Instruction::i64_bitor_imm16,
            wasm::i64_bitor,
            FuncTranslator::no_opt_ri,
        )
    }

    fn visit_i64_xor(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64, i64>(
            Instruction::i64_bitxor,
            Instruction::i64_bitxor_imm16,
            wasm::i64_bitxor,
            FuncTranslator::no_opt_ri,
        )
    }

    fn visit_i64_shl(&mut self) -> Self::Output {
        self.translate_shift::<i64>(
            Instruction::i64_shl,
            Instruction::i64_shl_by,
            Instruction::i64_shl_imm16,
            wasm::i64_shl,
        )
    }

    fn visit_i64_shr_s(&mut self) -> Self::Output {
        self.translate_shift::<i64>(
            Instruction::i64_shr_s,
            Instruction::i64_shr_s_by,
            Instruction::i64_shr_s_imm16,
            wasm::i64_shr_s,
        )
    }

    fn visit_i64_shr_u(&mut self) -> Self::Output {
        self.translate_shift::<i64>(
            Instruction::i64_shr_u,
            Instruction::i64_shr_u_by,
            Instruction::i64_shr_u_imm16,
            wasm::i64_shr_u,
        )
    }

    fn visit_i64_rotl(&mut self) -> Self::Output {
        self.translate_shift::<i64>(
            Instruction::i64_rotl,
            Instruction::i64_rotl_by,
            Instruction::i64_rotl_imm16,
            wasm::i64_rotl,
        )
    }

    fn visit_i64_rotr(&mut self) -> Self::Output {
        self.translate_shift::<i64>(
            Instruction::i64_rotr,
            Instruction::i64_rotr_by,
            Instruction::i64_rotr_imm16,
            wasm::i64_rotr,
        )
    }

    fn visit_f32_abs(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_abs, wasm::f32_abs)
    }

    fn visit_f32_neg(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_neg, wasm::f32_neg)
    }

    fn visit_f32_ceil(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_ceil, wasm::f32_ceil)
    }

    fn visit_f32_floor(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_floor, wasm::f32_floor)
    }

    fn visit_f32_trunc(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_trunc, wasm::f32_trunc)
    }

    fn visit_f32_nearest(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_nearest, wasm::f32_nearest)
    }

    fn visit_f32_sqrt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_sqrt, wasm::f32_sqrt)
    }

    fn visit_f32_add(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f32_add, wasm::f32_add)
    }

    fn visit_f32_sub(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f32_sub, wasm::f32_sub)
    }

    fn visit_f32_mul(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f32_mul, wasm::f32_mul)
    }

    fn visit_f32_div(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f32_div, wasm::f32_div)
    }

    fn visit_f32_min(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f32_min, wasm::f32_min)
    }

    fn visit_f32_max(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f32_max, wasm::f32_max)
    }

    fn visit_f32_copysign(&mut self) -> Self::Output {
        self.translate_fcopysign::<f32>(
            Instruction::f32_copysign,
            Instruction::f32_copysign_imm,
            wasm::f32_copysign,
        )
    }

    fn visit_f64_abs(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_abs, wasm::f64_abs)
    }

    fn visit_f64_neg(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_neg, wasm::f64_neg)
    }

    fn visit_f64_ceil(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_ceil, wasm::f64_ceil)
    }

    fn visit_f64_floor(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_floor, wasm::f64_floor)
    }

    fn visit_f64_trunc(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_trunc, wasm::f64_trunc)
    }

    fn visit_f64_nearest(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_nearest, wasm::f64_nearest)
    }

    fn visit_f64_sqrt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_sqrt, wasm::f64_sqrt)
    }

    fn visit_f64_add(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f64_add, wasm::f64_add)
    }

    fn visit_f64_sub(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f64_sub, wasm::f64_sub)
    }

    fn visit_f64_mul(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f64_mul, wasm::f64_mul)
    }

    fn visit_f64_div(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f64_div, wasm::f64_div)
    }

    fn visit_f64_min(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f64_min, wasm::f64_min)
    }

    fn visit_f64_max(&mut self) -> Self::Output {
        self.translate_fbinary(Instruction::f64_max, wasm::f64_max)
    }

    fn visit_f64_copysign(&mut self) -> Self::Output {
        self.translate_fcopysign::<f64>(
            Instruction::f64_copysign,
            Instruction::f64_copysign_imm,
            wasm::f64_copysign,
        )
    }

    fn visit_i32_wrap_i64(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_wrap_i64, wasm::i32_wrap_i64)
    }

    fn visit_i32_trunc_f32_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f32_s, wasm::i32_trunc_f32_s)
    }

    fn visit_i32_trunc_f32_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f32_u, wasm::i32_trunc_f32_u)
    }

    fn visit_i32_trunc_f64_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f64_s, wasm::i32_trunc_f64_s)
    }

    fn visit_i32_trunc_f64_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f64_u, wasm::i32_trunc_f64_u)
    }

    fn visit_i64_extend_i32_s(&mut self) -> Self::Output {
        self.translate_unary::<i32, i64>(Instruction::i64_extend32_s, wasm::i64_extend_i32_s)
    }

    fn visit_i64_extend_i32_u(&mut self) -> Self::Output {
        self.translate_reinterpret(wasm::i64_extend_i32_u)
    }

    fn visit_i64_trunc_f32_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f32_s, wasm::i64_trunc_f32_s)
    }

    fn visit_i64_trunc_f32_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f32_u, wasm::i64_trunc_f32_u)
    }

    fn visit_i64_trunc_f64_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f64_s, wasm::i64_trunc_f64_s)
    }

    fn visit_i64_trunc_f64_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f64_u, wasm::i64_trunc_f64_u)
    }

    fn visit_f32_convert_i32_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_convert_i32_s, wasm::f32_convert_i32_s)
    }

    fn visit_f32_convert_i32_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_convert_i32_u, wasm::f32_convert_i32_u)
    }

    fn visit_f32_convert_i64_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_convert_i64_s, wasm::f32_convert_i64_s)
    }

    fn visit_f32_convert_i64_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_convert_i64_u, wasm::f32_convert_i64_u)
    }

    fn visit_f32_demote_f64(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_demote_f64, wasm::f32_demote_f64)
    }

    fn visit_f64_convert_i32_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_convert_i32_s, wasm::f64_convert_i32_s)
    }

    fn visit_f64_convert_i32_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_convert_i32_u, wasm::f64_convert_i32_u)
    }

    fn visit_f64_convert_i64_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_convert_i64_s, wasm::f64_convert_i64_s)
    }

    fn visit_f64_convert_i64_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_convert_i64_u, wasm::f64_convert_i64_u)
    }

    fn visit_f64_promote_f32(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_promote_f32, wasm::f64_promote_f32)
    }

    fn visit_i32_reinterpret_f32(&mut self) -> Self::Output {
        self.translate_reinterpret(wasm::i32_reinterpret_f32)
    }

    fn visit_i64_reinterpret_f64(&mut self) -> Self::Output {
        self.translate_reinterpret(wasm::i64_reinterpret_f64)
    }

    fn visit_f32_reinterpret_i32(&mut self) -> Self::Output {
        self.translate_reinterpret(wasm::f32_reinterpret_i32)
    }

    fn visit_f64_reinterpret_i64(&mut self) -> Self::Output {
        self.translate_reinterpret(wasm::f64_reinterpret_i64)
    }

    fn visit_i32_extend8_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_extend8_s, wasm::i32_extend8_s)
    }

    fn visit_i32_extend16_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_extend16_s, wasm::i32_extend16_s)
    }

    fn visit_i64_extend8_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend8_s, wasm::i64_extend8_s)
    }

    fn visit_i64_extend16_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend16_s, wasm::i64_extend16_s)
    }

    fn visit_i64_extend32_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend32_s, wasm::i64_extend32_s)
    }

    fn visit_i32_trunc_sat_f32_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_trunc_sat_f32_s, wasm::i32_trunc_sat_f32_s)
    }

    fn visit_i32_trunc_sat_f32_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_trunc_sat_f32_u, wasm::i32_trunc_sat_f32_u)
    }

    fn visit_i32_trunc_sat_f64_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_trunc_sat_f64_s, wasm::i32_trunc_sat_f64_s)
    }

    fn visit_i32_trunc_sat_f64_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_trunc_sat_f64_u, wasm::i32_trunc_sat_f64_u)
    }

    fn visit_i64_trunc_sat_f32_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_trunc_sat_f32_s, wasm::i64_trunc_sat_f32_s)
    }

    fn visit_i64_trunc_sat_f32_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_trunc_sat_f32_u, wasm::i64_trunc_sat_f32_u)
    }

    fn visit_i64_trunc_sat_f64_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_trunc_sat_f64_s, wasm::i64_trunc_sat_f64_s)
    }

    fn visit_i64_trunc_sat_f64_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_trunc_sat_f64_u, wasm::i64_trunc_sat_f64_u)
    }

    fn visit_memory_init(&mut self, data_index: u32, mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.stack.pop3();
        let dst = self.layout.operand_to_reg(dst)?;
        let src = self.layout.operand_to_reg(src)?;
        let len = self.make_input16::<u32>(len)?;
        let instr = match len {
            Input::Immediate(len) => Instruction::memory_init_imm(dst, src, len),
            Input::Reg(len) => Instruction::memory_init(dst, src, len),
        };
        self.push_instr(instr, FuelCostsProvider::instance)?;
        self.push_param(Instruction::memory_index(mem))?;
        self.push_param(Instruction::data_index(data_index))?;
        Ok(())
    }

    fn visit_data_drop(&mut self, data_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.push_instr(
            Instruction::data_drop(data_index),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    fn visit_memory_copy(&mut self, dst_mem: u32, src_mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.stack.pop3();
        let dst_memory_type = *self.module.get_type_of_memory(MemoryIdx::from(dst_mem));
        let src_memory_type = *self.module.get_type_of_memory(MemoryIdx::from(src_mem));
        let min_index_ty = dst_memory_type.index_ty().min(&src_memory_type.index_ty());
        let dst = self.layout.operand_to_reg(dst)?;
        let src = self.layout.operand_to_reg(src)?;
        let len = self.make_index16(len, min_index_ty)?;
        let instr = match len {
            Input::Reg(len) => Instruction::memory_copy(dst, src, len),
            Input::Immediate(len) => Instruction::memory_copy_imm(dst, src, len),
        };
        self.push_instr(instr, FuelCostsProvider::instance)?;
        self.push_param(Instruction::memory_index(dst_mem))?;
        self.push_param(Instruction::memory_index(src_mem))?;
        Ok(())
    }

    fn visit_memory_fill(&mut self, mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let memory_type = *self.module.get_type_of_memory(MemoryIdx::from(mem));
        let (dst, value, len) = self.stack.pop3();
        let dst = self.layout.operand_to_reg(dst)?;
        let value = self.make_input(value, |_, value| {
            let byte = u32::from(value) as u8;
            Ok(Input::Immediate(byte))
        })?;
        let len = self.make_index16(len, memory_type.index_ty())?;
        let instr: Instruction = match (value, len) {
            (Input::Reg(value), Input::Reg(len)) => Instruction::memory_fill(dst, value, len),
            (Input::Reg(value), Input::Immediate(len)) => {
                Instruction::memory_fill_exact(dst, value, len)
            }
            (Input::Immediate(value), Input::Reg(len)) => {
                Instruction::memory_fill_imm(dst, value, len)
            }
            (Input::Immediate(value), Input::Immediate(len)) => {
                Instruction::memory_fill_imm_exact(dst, value, len)
            }
        };
        self.push_instr(instr, FuelCostsProvider::instance)?;
        self.push_param(Instruction::memory_index(mem))?;
        Ok(())
    }

    fn visit_table_init(&mut self, elem_index: u32, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.stack.pop3();
        let dst = self.layout.operand_to_reg(dst)?;
        let src = self.layout.operand_to_reg(src)?;
        let len = self.make_input16::<u32>(len)?;
        let instr = match len {
            Input::Reg(len) => Instruction::table_init(dst, src, len),
            Input::Immediate(len) => Instruction::table_init_imm(dst, src, len),
        };
        self.push_instr(instr, FuelCostsProvider::instance)?;
        self.push_param(Instruction::table_index(table))?;
        self.push_param(Instruction::elem_index(elem_index))?;
        Ok(())
    }

    fn visit_elem_drop(&mut self, elem_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.push_instr(
            Instruction::elem_drop(elem_index),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    fn visit_table_copy(&mut self, dst_table: u32, src_table: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.stack.pop3();
        let dst_table_type = *self.module.get_type_of_table(TableIdx::from(dst_table));
        let src_table_type = *self.module.get_type_of_table(TableIdx::from(src_table));
        let min_index_ty = dst_table_type.index_ty().min(&src_table_type.index_ty());
        let dst = self.layout.operand_to_reg(dst)?;
        let src = self.layout.operand_to_reg(src)?;
        let len = self.make_index16(len, min_index_ty)?;
        let instr = match len {
            Input::Reg(len) => Instruction::table_copy(dst, src, len),
            Input::Immediate(len) => Instruction::table_copy_imm(dst, src, len),
        };
        self.push_instr(instr, FuelCostsProvider::instance)?;
        self.push_param(Instruction::table_index(dst_table))?;
        self.push_param(Instruction::table_index(src_table))?;
        Ok(())
    }

    fn visit_typed_select(&mut self, ty: wasmparser::ValType) -> Self::Output {
        let type_hint = WasmiValueType::from(ty).into_inner();
        self.translate_select(Some(type_hint))
    }

    fn visit_ref_null(&mut self, ty: wasmparser::HeapType) -> Self::Output {
        bail_unreachable!(self);
        let type_hint = WasmiValueType::from(ty).into_inner();
        let null = match type_hint {
            ValType::FuncRef => TypedVal::from(FuncRef::null()),
            ValType::ExternRef => TypedVal::from(ExternRef::null()),
            ty => panic!("expected a Wasm `reftype` but found: {ty:?}"),
        };
        self.stack.push_immediate(null)?;
        Ok(())
    }

    fn visit_ref_is_null(&mut self) -> Self::Output {
        bail_unreachable!(self);
        match self.stack.pop() {
            Operand::Local(input) => {
                // Note: `funcref` and `externref` both serialize to `UntypedValue`
                //       as `u64` so we can use `i64.eqz` translation for `ref.is_null`
                //       via reinterpretation of the value's type.
                let input = self.layout.local_to_reg(input.local_index())?;
                // TODO: improve performance by allowing type overwrites for local operands
                self.push_instr_with_result(
                    ValType::I64,
                    |result| Instruction::copy(result, input),
                    FuelCostsProvider::base,
                )?;
                self.visit_i64_eqz()
            }
            Operand::Temp(input) => {
                // Note: `funcref` and `externref` both serialize to `UntypedValue`
                //       as `u64` so we can use `i64.eqz` translation for `ref.is_null`
                //       via reinterpretation of the value's type.
                self.stack.push_temp(ValType::I64, input.instr())?;
                self.visit_i64_eqz()
            }
            Operand::Immediate(input) => {
                let untyped = input.val().untyped();
                let is_null = match input.ty() {
                    ValType::FuncRef => FuncRef::from(untyped).is_null(),
                    ValType::ExternRef => ExternRef::from(untyped).is_null(),
                    invalid => panic!("`ref.is_null`: encountered invalid input type: {invalid:?}"),
                };
                self.stack.push_immediate(i32::from(is_null))?;
                Ok(())
            }
        }
    }

    fn visit_ref_func(&mut self, function_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.push_instr_with_result(
            ValType::FuncRef,
            |result| Instruction::ref_func(result, function_index),
            FuelCostsProvider::instance,
        )?;
        Ok(())
    }

    fn visit_table_fill(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, value, len) = self.stack.pop3();
        let table_type = *self.module.get_type_of_table(TableIdx::from(table));
        let dst = self.layout.operand_to_reg(dst)?;
        let value = self.layout.operand_to_reg(value)?;
        let len = self.make_index16(len, table_type.index_ty())?;
        let instr = match len {
            Input::Reg(len) => Instruction::table_fill(dst, len, value),
            Input::Immediate(len) => Instruction::table_fill_imm(dst, len, value),
        };
        self.push_instr(instr, FuelCostsProvider::instance)?;
        self.push_param(Instruction::table_index(table))?;
        Ok(())
    }

    fn visit_table_get(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let table_type = *self.module.get_type_of_table(TableIdx::from(table));
        let index = self.stack.pop();
        let item_ty = table_type.element();
        let index_ty = table_type.index_ty();
        let index = self.make_index32(index, index_ty)?;
        self.push_instr_with_result(
            item_ty,
            |result| match index {
                Input::Reg(index) => Instruction::table_get(result, index),
                Input::Immediate(index) => Instruction::table_get_imm(result, index),
            },
            FuelCostsProvider::instance,
        )?;
        self.push_param(Instruction::table_index(table))?;
        Ok(())
    }

    fn visit_table_set(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let table_type = *self.module.get_type_of_table(TableIdx::from(table));
        let index_ty = table_type.index_ty();
        let (index, value) = self.stack.pop2();
        let index = self.make_index32(index, index_ty)?;
        let value = self.layout.operand_to_reg(value)?;
        let instr = match index {
            Input::Reg(index) => Instruction::table_set(index, value),
            Input::Immediate(index) => Instruction::table_set_at(value, index),
        };
        self.push_instr(instr, FuelCostsProvider::instance)?;
        self.push_param(Instruction::table_index(table))?;
        Ok(())
    }

    fn visit_table_grow(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let table_type = *self.module.get_type_of_table(TableIdx::from(table));
        let index_ty = table_type.index_ty();
        let (value, delta) = self.stack.pop2();
        let delta = self.make_index16(delta, index_ty)?;
        if let Input::Immediate(delta) = delta {
            if u64::from(delta) == 0 {
                // Case: growing by 0 elements.
                //
                // Since `table.grow` returns the `table.size` before the
                // operation a `table.grow` with `delta` of 0 can be translated
                // as `table.size` instruction instead.
                self.push_instr_with_result(
                    index_ty.ty(),
                    |result| Instruction::table_size(result, table),
                    FuelCostsProvider::instance,
                )?;
                return Ok(());
            }
        }
        let value = self.layout.operand_to_reg(value)?;
        self.push_instr_with_result(
            index_ty.ty(),
            |result| match delta {
                Input::Reg(delta) => Instruction::table_grow(result, delta, value),
                Input::Immediate(delta) => Instruction::table_grow_imm(result, delta, value),
            },
            FuelCostsProvider::instance,
        )?;
        self.push_param(Instruction::table_index(table))?;
        Ok(())
    }

    fn visit_table_size(&mut self, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_return_call(&mut self, function_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let func_idx = FuncIdx::from(function_index);
        let func_type = self.resolve_func_type(func_idx);
        let len_params = usize::from(func_type.len_params());
        let instr = match self.module.get_engine_func(func_idx) {
            Some(engine_func) => {
                // Case: We are calling an internal function and can optimize
                //       this case by using the special instruction for it.
                match len_params {
                    0 => Instruction::return_call_internal_0(engine_func),
                    _ => Instruction::return_call_internal(engine_func),
                }
            }
            None => {
                // Case: We are calling an imported function and must use the
                //       general calling operator for it.
                match len_params {
                    0 => Instruction::return_call_imported_0(function_index),
                    _ => Instruction::return_call_imported(function_index),
                }
            }
        };
        self.push_instr(instr, FuelCostsProvider::call)?;
        self.stack.pop_n(len_params, &mut self.operands);
        self.instrs
            .encode_register_list(&self.operands, &mut self.layout)?;
        self.reachable = false;
        Ok(())
    }

    fn visit_return_call_indirect(&mut self, type_index: u32, table_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let func_type = self.resolve_type(type_index);
        let index = self.stack.pop();
        let indirect_params = self.call_indirect_params(index, table_index)?;
        let len_params = usize::from(func_type.len_params());
        let instr = match (len_params, indirect_params) {
            (0, Instruction::CallIndirectParams { .. }) => {
                Instruction::return_call_indirect_0(type_index)
            }
            (0, Instruction::CallIndirectParamsImm16 { .. }) => {
                Instruction::return_call_indirect_0_imm16(type_index)
            }
            (_, Instruction::CallIndirectParams { .. }) => {
                Instruction::return_call_indirect(type_index)
            }
            (_, Instruction::CallIndirectParamsImm16 { .. }) => {
                Instruction::return_call_indirect_imm16(type_index)
            }
            _ => unreachable!(),
        };
        self.push_instr(instr, FuelCostsProvider::call)?;
        self.push_param(indirect_params)?;
        self.stack.pop_n(len_params, &mut self.operands);
        self.instrs
            .encode_register_list(&self.operands, &mut self.layout)?;
        self.reachable = false;
        Ok(())
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
