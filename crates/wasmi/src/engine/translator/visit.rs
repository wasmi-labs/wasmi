use super::{
    bail_unreachable,
    control_frame::{
        BlockControlFrame,
        BlockHeight,
        ControlFrame,
        IfControlFrame,
        IfReachability,
        LoopControlFrame,
        UnreachableControlFrame,
    },
    stack::TypedProvider,
    ControlFrameKind,
    FuelInfo,
    FuncTranslator,
    LabelRef,
    TypedVal,
};
use crate::{
    core::{TrapCode, ValType, F32, F64},
    engine::{
        bytecode::{self, Const16, Instruction, Provider, Register, SignatureIdx},
        translator::AcquiredTarget,
        BlockType,
        FuelCosts,
    },
    module::{self, FuncIdx, WasmiValueType},
    Error,
    ExternRef,
    FuncRef,
    Mutability,
};
use core::num::{NonZeroU32, NonZeroU64};
use std::collections::BTreeMap;
use wasmparser::VisitOperator;

/// Used to swap operands of a `rev` variant [`Instruction`] constructor.
macro_rules! swap_ops {
    ($fn_name:path) => {
        |result: Register, lhs: Const16<_>, rhs: Register| -> Instruction {
            $fn_name(result, rhs, lhs)
        }
    };
}

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
    ( @@skipped $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // We skip Wasm operators that we already implement manually.
        impl_visit_operator!($($rest)*);
    };
    ( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // Wildcard match arm for all the other (yet) unsupported Wasm proposals.
        fn $visit(&mut self $($(, $arg: $argty)*)?) -> Self::Output {
            self.unsupported_operator(stringify!($op))
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
    fn unsupported_operator(&self, name: &str) -> Result<(), Error> {
        panic!("tried to translate an unsupported Wasm operator: {name}")
    }
}

impl<'a> VisitOperator<'a> for FuncTranslator {
    type Output = Result<(), Error>;

    wasmparser::for_each_operator!(impl_visit_operator);

    fn visit_unreachable(&mut self) -> Self::Output {
        bail_unreachable!(self);
        self.push_base_instr(Instruction::Trap(TrapCode::UnreachableCodeReached))?;
        self.reachable = false;
        Ok(())
    }

    fn visit_nop(&mut self) -> Self::Output {
        // Nothing to do for Wasm `nop` instructions.
        Ok(())
    }

    fn visit_block(&mut self, block_type: wasmparser::BlockType) -> Self::Output {
        let block_type = BlockType::new(block_type, &self.module);
        if !self.is_reachable() {
            // We keep track of unreachable control flow frames so that we
            // can associated `end` operators to their respective control flow
            // frames and precisely know when the code is reachable again.
            self.alloc
                .control_stack
                .push_frame(UnreachableControlFrame::new(
                    ControlFrameKind::Block,
                    block_type,
                ));
            return Ok(());
        }
        self.preserve_locals()?;
        // Inherit [`Instruction::ConsumeFuel`] from parent control frame.
        //
        // # Note
        //
        // This is an optimization to reduce the number of [`Instruction::ConsumeFuel`]
        // and is applicable since Wasm `block` are entered unconditionally.
        let fuel_instr = self.fuel_instr();
        let stack_height = BlockHeight::new(self.engine(), self.alloc.stack.height(), block_type)?;
        let end_label = self.alloc.instr_encoder.new_label();
        let len_block_params = block_type.len_params(self.engine());
        let len_branch_params = block_type.len_results(self.engine());
        let branch_params = self.alloc_branch_params(len_block_params, len_branch_params)?;
        self.alloc.control_stack.push_frame(BlockControlFrame::new(
            block_type,
            end_label,
            branch_params,
            stack_height,
            fuel_instr,
        ));
        Ok(())
    }

    fn visit_loop(&mut self, block_type: wasmparser::BlockType) -> Self::Output {
        let block_type = BlockType::new(block_type, &self.module);
        if !self.is_reachable() {
            // See `visit_block` for rational of tracking unreachable control flow.
            self.alloc
                .control_stack
                .push_frame(UnreachableControlFrame::new(
                    ControlFrameKind::Loop,
                    block_type,
                ));
            return Ok(());
        }
        // Copy `loop` parameters over to where it expects its branch parameters.
        let len_block_params = block_type.len_params(self.engine());
        self.alloc
            .stack
            .pop_n(len_block_params, &mut self.alloc.buffer.providers);
        let branch_params = self.alloc.stack.push_dynamic_n(len_block_params)?;
        // self.preserve_locals()?; // TODO: find a case where local preservation before loops is necessary
        let fuel_info = self.fuel_info();
        self.alloc.instr_encoder.encode_copies(
            &mut self.alloc.stack,
            branch_params.iter(len_block_params),
            &self.alloc.buffer.providers[..],
            fuel_info,
        )?;
        self.alloc.instr_encoder.reset_last_instr();
        // Create loop header label and immediately pin it.
        let stack_height = BlockHeight::new(self.engine(), self.alloc.stack.height(), block_type)?;
        let header = self.alloc.instr_encoder.new_label();
        self.alloc.instr_encoder.pin_label(header);
        // Optionally create the loop's [`Instruction::ConsumeFuel`].
        //
        // This is handling the fuel required for a single iteration of the loop.
        //
        // Note: The fuel instruction for the loop must be encoded after the loop header is
        //       pinned so that loop iterations will properly consume fuel per iteration.
        let consume_fuel = self.make_fuel_instr()?;
        // Finally create the loop control frame.
        self.alloc.control_stack.push_frame(LoopControlFrame::new(
            block_type,
            header,
            stack_height,
            branch_params,
            consume_fuel,
        ));
        Ok(())
    }

    fn visit_if(&mut self, block_type: wasmparser::BlockType) -> Self::Output {
        let block_type = BlockType::new(block_type, &self.module);
        if !self.is_reachable() {
            // We keep track of unreachable control flow frames so that we
            // can associated `end` operators to their respective control flow
            // frames and precisely know when the code is reachable again.
            self.alloc
                .control_stack
                .push_frame(UnreachableControlFrame::new(
                    ControlFrameKind::If,
                    block_type,
                ));
            return Ok(());
        }
        let condition = self.alloc.stack.pop();
        let stack_height = BlockHeight::new(self.engine(), self.alloc.stack.height(), block_type)?;
        self.preserve_locals()?;
        let end_label = self.alloc.instr_encoder.new_label();
        let len_block_params = block_type.len_params(self.engine());
        let len_branch_params = block_type.len_results(self.engine());
        let branch_params = self.alloc_branch_params(len_block_params, len_branch_params)?;
        let (reachability, fuel_instr) = match condition {
            TypedProvider::Const(condition) => {
                // Case: the `if` condition is a constant value and
                //       therefore it is known upfront which branch
                //       it will take.
                //       Furthermore the non-taken branch is known
                //       to be unreachable code.
                let reachability = match i32::from(condition) != 0 {
                    true => IfReachability::OnlyThen,
                    false => {
                        // We know that the `then` block is unreachable therefore
                        // we update the reachability until we see the `else` branch.
                        self.reachable = false;
                        IfReachability::OnlyElse
                    }
                };
                // An `if` control frame with a constant condition behaves very
                // similarly to a Wasm `block`. Therefore we can apply the same
                // optimization and inherit the [`Instruction::ConsumeFuel`] of
                // the parent control frame.
                let fuel_instr = self.fuel_instr();
                (reachability, fuel_instr)
            }
            TypedProvider::Register(condition) => {
                // Push the `if` parameters on the `else` provider stack for
                // later use in case we eventually visit the `else` branch.
                self.alloc
                    .stack
                    .peek_n(len_block_params, &mut self.alloc.buffer.providers);
                self.alloc
                    .control_stack
                    .push_else_providers(self.alloc.buffer.providers.iter().copied())?;
                // Note: We increase preservation register usage of else providers
                //       so that they cannot be invalidated in the `then` block before
                //       arriving at the `else` block of an `if`.
                //       We manually decrease the usage of the else providers at the
                //       end of the `else`, or `if` in case `else` is missing.
                self.alloc
                    .buffer
                    .providers
                    .iter()
                    .copied()
                    .filter_map(TypedProvider::into_register)
                    .for_each(|register| self.alloc.stack.inc_register_usage(register));
                // Create the `else` label and the conditional branch to `else`.
                let else_label = self.alloc.instr_encoder.new_label();
                self.alloc.instr_encoder.encode_branch_eqz(
                    &mut self.alloc.stack,
                    condition,
                    else_label,
                )?;
                let reachability = IfReachability::both(else_label);
                // Optionally create the [`Instruction::ConsumeFuel`] for the `then` branch.
                //
                // # Note
                //
                // The [`Instruction::ConsumeFuel`] for the `else` branch is
                // created on the fly when visiting the `else` block.
                let fuel_instr = self.make_fuel_instr()?;
                (reachability, fuel_instr)
            }
        };
        self.alloc.control_stack.push_frame(IfControlFrame::new(
            block_type,
            end_label,
            branch_params,
            stack_height,
            fuel_instr,
            reachability,
        ));
        Ok(())
    }

    fn visit_else(&mut self) -> Self::Output {
        let mut frame = match self.alloc.control_stack.pop_frame() {
            ControlFrame::If(frame) => frame,
            ControlFrame::Unreachable(frame) if matches!(frame.kind(), ControlFrameKind::If) => {
                // Case: `else` branch for unreachable `if` block.
                //
                // In this case we can simply ignore the entire `else`
                // branch since it is unreachable anyways.
                self.alloc.control_stack.push_frame(frame);
                return Ok(());
            }
            unexpected => panic!(
                "expected `if` control flow frame on top for `else` but found: {:?}",
                unexpected,
            ),
        };
        frame.visited_else();
        if frame.is_then_reachable() {
            frame.update_end_of_then_reachability(self.reachable);
        }
        if let Some(else_label) = frame.else_label() {
            // Case: the `if` control frame has reachable `then` and `else` branches.
            debug_assert!(frame.is_then_reachable());
            debug_assert!(frame.is_else_reachable());
            let branch_params = frame.branch_params(self.engine());
            if self.reachable {
                self.translate_copy_branch_params(branch_params)?;
                let end_offset = self
                    .alloc
                    .instr_encoder
                    .try_resolve_label(frame.end_label())?;
                // We are jumping to the end of the `if` so technically we need to bump branches.
                frame.bump_branches();
                self.push_base_instr(Instruction::branch(end_offset))?;
            }
            self.reachable = true;
            self.alloc.instr_encoder.pin_label(else_label);
            if let Some(fuel_instr) = self.make_fuel_instr()? {
                frame.update_consume_fuel_instr(fuel_instr);
            }
            // At this point we can restore the `else` branch parameters
            // so that the `else` branch translation has the same set of
            // parameters as the `then` branch.
            self.alloc
                .stack
                .trunc(frame.block_height().into_u16() as usize);
            for provider in self.alloc.control_stack.pop_else_providers() {
                self.alloc.stack.push_provider(provider)?;
                if let TypedProvider::Register(register) = provider {
                    self.alloc.stack.dec_register_usage(register);
                }
            }
        }
        match (frame.is_then_reachable(), frame.is_else_reachable()) {
            (true, false) => {
                // Case: only `then` branch is reachable.
                //
                // Not much needs to be done since an `if` control frame
                // where only one branch is statically reachable is similar
                // to a `block` control frame.
                self.reachable = false;
            }
            (false, true) => {
                // Case: only `else` branch is reachable.
                //
                // Not much needs to be done since an `if` control frame
                // where only one branch is statically reachable is similar
                // to a `block` control frame.
                debug_assert!(!self.reachable);
                self.reachable = true;
            }
            (false, false) => unreachable!(
                "the if control frame is reachable so either then or else must be reachable"
            ),
            (true, true) => {
                // Note: this case has already been handled above.
            }
        }
        // At last we need to push the popped and adjusted [`IfControlFrame`] back.
        self.alloc.control_stack.push_frame(frame);
        Ok(())
    }

    fn visit_end(&mut self) -> Self::Output {
        match self.alloc.control_stack.pop_frame() {
            ControlFrame::Block(frame) => self.translate_end_block(frame),
            ControlFrame::Loop(frame) => self.translate_end_loop(frame),
            ControlFrame::If(frame) => self.translate_end_if(frame),
            ControlFrame::Unreachable(frame) => self.translate_end_unreachable(frame),
        }?;
        self.alloc.instr_encoder.reset_last_instr();
        Ok(())
    }

    fn visit_br(&mut self, relative_depth: u32) -> Self::Output {
        bail_unreachable!(self);
        let engine = self.engine().clone();
        match self.alloc.control_stack.acquire_target(relative_depth) {
            AcquiredTarget::Return(_frame) => self.translate_return(),
            AcquiredTarget::Branch(frame) => {
                frame.bump_branches();
                let branch_dst = frame.branch_destination();
                let branch_params = frame.branch_params(&engine);
                self.translate_copy_branch_params(branch_params)?;
                let branch_offset = self.alloc.instr_encoder.try_resolve_label(branch_dst)?;
                self.push_base_instr(Instruction::branch(branch_offset))?;
                self.reachable = false;
                Ok(())
            }
        }
    }

    fn visit_br_if(&mut self, relative_depth: u32) -> Self::Output {
        bail_unreachable!(self);
        let engine = self.engine().clone();
        match self.alloc.stack.pop() {
            TypedProvider::Const(condition) => {
                if i32::from(condition) != 0 {
                    // Case: `condition == 1` so the branch is always taken.
                    // Therefore we can simplify the `br_if` to a `br` instruction.
                    self.visit_br(relative_depth)
                } else {
                    // Case: `condition != 1` so the branch is never taken.
                    // Therefore the `br_if` is a `nop` and can be ignored.
                    Ok(())
                }
            }
            TypedProvider::Register(condition) => {
                let fuel_info = self.fuel_info();
                match self.alloc.control_stack.acquire_target(relative_depth) {
                    AcquiredTarget::Return(_frame) => self.translate_return_if(condition),
                    AcquiredTarget::Branch(frame) => {
                        frame.bump_branches();
                        let branch_dst = frame.branch_destination();
                        let branch_params = frame.branch_params(&engine);
                        if branch_params.is_empty() {
                            // Case: no values need to be copied so we can directly
                            //       encode the `br_if` as efficient `branch_nez`.
                            self.alloc.instr_encoder.encode_branch_nez(
                                &mut self.alloc.stack,
                                condition,
                                branch_dst,
                            )?;
                            return Ok(());
                        }
                        self.alloc
                            .stack
                            .peek_n(branch_params.len(), &mut self.alloc.buffer.providers);
                        if self
                            .alloc
                            .buffer
                            .providers
                            .iter()
                            .copied()
                            .eq(branch_params.map(TypedProvider::Register))
                        {
                            // Case: the providers on the stack are already as
                            //       expected by the branch params and therefore
                            //       no copies are required.
                            //
                            // This means we can encode the `br_if` as efficient `branch_nez`.
                            self.alloc.instr_encoder.encode_branch_nez(
                                &mut self.alloc.stack,
                                condition,
                                branch_dst,
                            )?;
                            return Ok(());
                        }
                        // Case: We need to copy the branch inputs to where the
                        //       control frame expects them before actually branching
                        //       to it.
                        //       We do this by performing a negated `br_eqz` and skip
                        //       the copy process with it in cases where no branch is
                        //       needed.
                        //       Otherwise we copy the values to their expected locations
                        //       and finally perform the actual branch to the target
                        //       control frame.
                        let skip_label = self.alloc.instr_encoder.new_label();
                        self.alloc.instr_encoder.encode_branch_eqz(
                            &mut self.alloc.stack,
                            condition,
                            skip_label,
                        )?;
                        self.alloc.instr_encoder.encode_copies(
                            &mut self.alloc.stack,
                            branch_params,
                            &self.alloc.buffer.providers[..],
                            fuel_info,
                        )?;
                        let branch_offset =
                            self.alloc.instr_encoder.try_resolve_label(branch_dst)?;
                        self.push_base_instr(Instruction::branch(branch_offset))?;
                        self.alloc.instr_encoder.pin_label(skip_label);
                        Ok(())
                    }
                }
            }
        }
    }

    fn visit_br_table(&mut self, targets: wasmparser::BrTable<'a>) -> Self::Output {
        bail_unreachable!(self);
        let engine = self.engine().clone();
        let fuel_info = self.fuel_info();
        let index = self.alloc.stack.pop();
        if targets.is_empty() {
            // Case: the `br_table` only has a default target.
            //
            // This means the `br_table` always branches to the default target
            // independent of the `index` value. Therefore we can encode it as
            // a simpler `br` instruction instead.
            return self.visit_br(targets.default());
        }
        let default_target = targets.default();
        let index: Register = match index {
            TypedProvider::Register(index) => index,
            TypedProvider::Const(index) => {
                // Case: the index is a constant value.
                //
                // This means the `br_table` always branches to the same target
                // and therefore acts exactly the same as a normal `br` instruction.
                let chosen_index = u32::from(index) as usize;
                let chosen_target = targets
                    .targets()
                    .nth(chosen_index)
                    .transpose()?
                    .unwrap_or(default_target);
                return self.visit_br(chosen_target);
            }
        };
        // Add `br_table` targets to `br_table_targets` buffer including default target.
        // This allows us to uniformely treat all `br_table` targets the same and only parse once.
        self.alloc.buffer.br_table_targets.clear();
        for target in targets.targets() {
            self.alloc.buffer.br_table_targets.push(target?);
        }
        self.alloc.buffer.br_table_targets.push(default_target);
        // We check if all `br_table` targets expect their results at the same
        // registers which allows us to encode the `br_table` more efficiently
        // by using a single copy instruction before branching instead of having
        // to copy in each branch.
        let default_branch_params = self
            .alloc
            .control_stack
            .acquire_target(default_target)
            .control_frame()
            .branch_params(&engine);
        let same_branch_params = self
            .alloc
            .buffer
            .br_table_targets
            .iter()
            .copied()
            .all(|target| {
                match self.alloc.control_stack.acquire_target(target) {
                    AcquiredTarget::Return(_) => {
                        // Return instructions never need to copy anything and
                        // thus we can simply ignore them for this conditional.
                        true
                    }
                    AcquiredTarget::Branch(frame) => {
                        default_branch_params == frame.branch_params(&engine)
                    }
                }
            });
        if default_branch_params.is_empty() || same_branch_params {
            // Case 1: No `br_target` target requires values to be copied around.
            // Case 2: All `br_target` targets use the same branch parameter registers.
            //
            // In both cases it is sufficient to copy values to the destination of
            // the default branch target and encode the `br_table` with a series of
            // simple direct branches without any further copy instructions.
            self.push_base_instr(Instruction::branch_table(index, targets.len() + 1))?;
            self.translate_copy_branch_params(default_branch_params)?;
            let return_instr = match default_branch_params.len() {
                0 => Instruction::Return,
                1 => Instruction::return_reg(default_branch_params.span().head()),
                _ => Instruction::return_span(default_branch_params),
            };
            for target in self.alloc.buffer.br_table_targets.iter().copied() {
                match self.alloc.control_stack.acquire_target(target) {
                    AcquiredTarget::Return(_) => {
                        self.alloc.instr_encoder.append_instr(return_instr)?;
                    }
                    AcquiredTarget::Branch(frame) => {
                        frame.bump_branches();
                        let branch_dst = frame.branch_destination();
                        let branch_offset =
                            self.alloc.instr_encoder.try_resolve_label(branch_dst)?;
                        self.alloc
                            .instr_encoder
                            .append_instr(Instruction::branch(branch_offset))?;
                    }
                }
            }
            self.reachable = false;
            return Ok(());
        }
        // Case: Some branch targets differ in their branch parameter registers.
        //
        // Therefore we cannot copy registers before the `br_table` instruction and
        // instead need to perform the copy instructions in the `br_table` branch arms.
        //
        // Since `br_table` target depths are often shared we use a btree-set to
        // share codegen for `br_table` arms that have the same branch target.
        self.push_base_instr(Instruction::branch_table(index, targets.len() + 1))?;
        let mut shared_targets = <BTreeMap<u32, LabelRef>>::new();
        for target in self.alloc.buffer.br_table_targets.iter().copied() {
            let shared_label = *shared_targets
                .entry(target)
                .or_insert_with(|| self.alloc.instr_encoder.new_label());
            let branch_offset = self.alloc.instr_encoder.try_resolve_label(shared_label)?;
            self.alloc
                .instr_encoder
                .append_instr(Instruction::branch(branch_offset))?;
        }
        let values = &mut self.alloc.buffer.providers;
        self.alloc.stack.pop_n(default_branch_params.len(), values);
        for (depth, label) in shared_targets {
            self.alloc.instr_encoder.pin_label(label);
            match self.alloc.control_stack.acquire_target(depth) {
                AcquiredTarget::Return(_frame) => {
                    // Note: We do not use fuel metering for the below
                    //       `encode_return` instruction since it is part
                    //       of the `br_table` instruction above.
                    self.alloc.instr_encoder.encode_return(
                        &mut self.alloc.stack,
                        values,
                        FuelInfo::None,
                    )?;
                }
                AcquiredTarget::Branch(frame) => {
                    frame.bump_branches();
                    self.alloc.instr_encoder.encode_copies(
                        &mut self.alloc.stack,
                        frame.branch_params(&engine),
                        values,
                        fuel_info,
                    )?;
                    let branch_dst = frame.branch_destination();
                    let branch_offset = self.alloc.instr_encoder.try_resolve_label(branch_dst)?;
                    // Note: No need to bump fuel consumption for each branch table target
                    //       since only one of the branch targets is going to be executed.
                    self.alloc
                        .instr_encoder
                        .push_instr(Instruction::branch(branch_offset))?;
                }
            }
        }
        self.reachable = false;
        Ok(())
    }

    fn visit_return(&mut self) -> Self::Output {
        bail_unreachable!(self);
        self.translate_return()
    }

    fn visit_call(&mut self, function_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.bump_fuel_consumption(FuelCosts::call)?;
        let func_idx = FuncIdx::from(function_index);
        let func_type = self.func_type_of(func_idx);
        let (params, results) = func_type.params_results();
        let provider_params = &mut self.alloc.buffer.providers;
        self.alloc.stack.pop_n(params.len(), provider_params);
        let results = self.alloc.stack.push_dynamic_n(results.len())?;
        let instr = match self.module.get_engine_func(func_idx) {
            Some(engine_func) => {
                // Case: We are calling an internal function and can optimize
                //       this case by using the special instruction for it.
                match params.len() {
                    0 => Instruction::call_internal_0(results, engine_func),
                    _ => Instruction::call_internal(results, engine_func),
                }
            }
            None => {
                // Case: We are calling an imported function and must use the
                //       general calling operator for it.
                match params.len() {
                    0 => Instruction::call_imported_0(results, function_index),
                    _ => Instruction::call_imported(results, function_index),
                }
            }
        };
        self.alloc.instr_encoder.push_instr(instr)?;
        self.alloc
            .instr_encoder
            .encode_register_list(&mut self.alloc.stack, provider_params)?;
        Ok(())
    }

    fn visit_call_indirect(
        &mut self,
        type_index: u32,
        table_index: u32,
        _table_byte: u8,
    ) -> Self::Output {
        bail_unreachable!(self);
        self.bump_fuel_consumption(FuelCosts::call)?;
        let type_index = SignatureIdx::from(type_index);
        let func_type = self.func_type_at(type_index);
        let (params, results) = func_type.params_results();
        let index = self.alloc.stack.pop();
        let provider_params = &mut self.alloc.buffer.providers;
        self.alloc.stack.pop_n(params.len(), provider_params);
        let table_params = match index {
            TypedProvider::Const(index) => match <Const16<u32>>::try_from(u32::from(index)).ok() {
                Some(index) => {
                    // Case: the index is encodable as 16-bit constant value
                    //       which allows us to use an optimized instruction.
                    Instruction::call_indirect_params_imm16(index, table_index)
                }
                None => {
                    // Case: the index is not encodable as 16-bit constant value
                    //       and we need to allocate it as function local constant.
                    let index = self.alloc.stack.alloc_const(index)?;
                    Instruction::call_indirect_params(index, table_index)
                }
            },
            TypedProvider::Register(index) => Instruction::call_indirect_params(index, table_index),
        };
        let results = self.alloc.stack.push_dynamic_n(results.len())?;
        let instr = match params.len() {
            0 => Instruction::call_indirect_0(results, type_index),
            _ => Instruction::call_indirect(results, type_index),
        };
        self.alloc.instr_encoder.push_instr(instr)?;
        self.alloc.instr_encoder.append_instr(table_params)?;
        self.alloc
            .instr_encoder
            .encode_register_list(&mut self.alloc.stack, provider_params)?;
        Ok(())
    }

    fn visit_return_call(&mut self, function_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.bump_fuel_consumption(FuelCosts::call)?;
        let func_idx = FuncIdx::from(function_index);
        let func_type = self.func_type_of(func_idx);
        let params = func_type.params();
        let provider_params = &mut self.alloc.buffer.providers;
        self.alloc.stack.pop_n(params.len(), provider_params);
        let instr = match self.module.get_engine_func(func_idx) {
            Some(engine_func) => {
                // Case: We are calling an internal function and can optimize
                //       this case by using the special instruction for it.
                match params.len() {
                    0 => Instruction::return_call_internal_0(engine_func),
                    _ => Instruction::return_call_internal(engine_func),
                }
            }
            None => {
                // Case: We are calling an imported function and must use the
                //       general calling operator for it.
                match params.len() {
                    0 => Instruction::return_call_imported_0(function_index),
                    _ => Instruction::return_call_imported(function_index),
                }
            }
        };
        self.alloc.instr_encoder.push_instr(instr)?;
        self.alloc
            .instr_encoder
            .encode_register_list(&mut self.alloc.stack, provider_params)?;
        self.reachable = false;
        Ok(())
    }

    fn visit_return_call_indirect(&mut self, type_index: u32, table_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.bump_fuel_consumption(FuelCosts::call)?;
        let type_index = SignatureIdx::from(type_index);
        let func_type = self.func_type_at(type_index);
        let params = func_type.params();
        let index = self.alloc.stack.pop();
        let provider_params = &mut self.alloc.buffer.providers;
        self.alloc.stack.pop_n(params.len(), provider_params);
        let table_params = match index {
            TypedProvider::Const(index) => match <Const16<u32>>::try_from(u32::from(index)).ok() {
                Some(index) => {
                    // Case: the index is encodable as 16-bit constant value
                    //       which allows us to use an optimized instruction.
                    Instruction::call_indirect_params_imm16(index, table_index)
                }
                None => {
                    // Case: the index is not encodable as 16-bit constant value
                    //       and we need to allocate it as function local constant.
                    let index = self.alloc.stack.alloc_const(index)?;
                    Instruction::call_indirect_params(index, table_index)
                }
            },
            TypedProvider::Register(index) => Instruction::call_indirect_params(index, table_index),
        };
        let instr = match params.len() {
            0 => Instruction::return_call_indirect_0(type_index),
            _ => Instruction::return_call_indirect(type_index),
        };
        self.alloc.instr_encoder.push_instr(instr)?;
        self.alloc.instr_encoder.append_instr(table_params)?;
        self.alloc
            .instr_encoder
            .encode_register_list(&mut self.alloc.stack, provider_params)?;
        self.reachable = false;
        Ok(())
    }

    fn visit_drop(&mut self) -> Self::Output {
        bail_unreachable!(self);
        self.alloc.stack.pop();
        Ok(())
    }

    fn visit_select(&mut self) -> Self::Output {
        self.translate_select(None)
    }

    fn visit_typed_select(&mut self, ty: wasmparser::ValType) -> Self::Output {
        let type_hint = WasmiValueType::from(ty).into_inner();
        self.translate_select(Some(type_hint))
    }

    fn visit_local_get(&mut self, local_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.alloc.stack.push_local(local_index)?;
        Ok(())
    }

    fn visit_local_set(&mut self, local_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.alloc.stack.gc_preservations();
        let value = self.alloc.stack.pop();
        let local = Register::try_from(local_index)?;
        if let TypedProvider::Register(value) = value {
            if value == local {
                // Case: `(local.set $n (local.get $n))` is a no-op so we can ignore it.
                //
                // Note: This does not require any preservation since it won't change
                //       the value of `local $n`.
                return Ok(());
            }
        }
        let preserved = self.alloc.stack.preserve_locals(local_index)?;
        let fuel_info = self.fuel_info();
        self.alloc.instr_encoder.encode_local_set(
            &mut self.alloc.stack,
            &self.module,
            local,
            value,
            preserved,
            fuel_info,
        )?;
        Ok(())
    }

    fn visit_local_tee(&mut self, local_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let input = self.alloc.stack.peek();
        self.visit_local_set(local_index)?;
        match input {
            Provider::Register(_register) => {
                self.alloc.stack.push_local(local_index)?;
            }
            Provider::Const(value) => {
                self.alloc.stack.push_const(value);
            }
        }
        Ok(())
    }

    fn visit_global_get(&mut self, global_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let global_idx = module::GlobalIdx::from(global_index);
        let (global_type, init_value) = self.module.get_global(global_idx);
        let content = global_type.content();
        if let (Mutability::Const, Some(init_expr)) = (global_type.mutability(), init_value) {
            if let Some(value) = init_expr.eval_const() {
                // Optimization: Access to immutable internally defined global variables
                //               can be replaced with their constant initialization value.
                self.alloc.stack.push_const(TypedVal::new(content, value));
                return Ok(());
            }
            if let Some(func_index) = init_expr.funcref() {
                // Optimization: Forward to `ref.func x` translation.
                self.visit_ref_func(func_index.into_u32())?;
                return Ok(());
            }
        }
        // Case: The `global.get` instruction accesses a mutable or imported
        //       global variable and thus cannot be optimized away.
        let global_idx = bytecode::GlobalIdx::from(global_index);
        let result = self.alloc.stack.push_dynamic()?;
        self.push_fueled_instr(
            Instruction::global_get(result, global_idx),
            FuelCosts::entity,
        )?;
        Ok(())
    }

    fn visit_global_set(&mut self, global_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let global = bytecode::GlobalIdx::from(global_index);
        match self.alloc.stack.pop() {
            TypedProvider::Register(input) => {
                self.push_fueled_instr(Instruction::global_set(global, input), FuelCosts::entity)?;
                Ok(())
            }
            TypedProvider::Const(input) => {
                let (global_type, _init_value) = self
                    .module
                    .get_global(module::GlobalIdx::from(global_index));
                debug_assert_eq!(global_type.content(), input.ty());
                match global_type.content() {
                    ValType::I32 => {
                        if let Ok(value) = Const16::try_from(i32::from(input)) {
                            self.push_fueled_instr(
                                Instruction::global_set_i32imm16(global, value),
                                FuelCosts::entity,
                            )?;
                            return Ok(());
                        }
                    }
                    ValType::I64 => {
                        if let Ok(value) = Const16::try_from(i64::from(input)) {
                            self.push_fueled_instr(
                                Instruction::global_set_i64imm16(global, value),
                                FuelCosts::entity,
                            )?;
                            return Ok(());
                        }
                    }
                    _ => {}
                };
                let cref = self.alloc.stack.alloc_const(input)?;
                self.push_fueled_instr(Instruction::global_set(global, cref), FuelCosts::entity)?;
                Ok(())
            }
        }
    }

    fn visit_i32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i32_load,
            Instruction::i32_load_offset16,
            Instruction::i32_load_at,
        )
    }

    fn visit_i64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load,
            Instruction::i64_load_offset16,
            Instruction::i64_load_at,
        )
    }

    fn visit_f32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::f32_load,
            Instruction::f32_load_offset16,
            Instruction::f32_load_at,
        )
    }

    fn visit_f64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::f64_load,
            Instruction::f64_load_offset16,
            Instruction::f64_load_at,
        )
    }

    fn visit_i32_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i32_load8_s,
            Instruction::i32_load8_s_offset16,
            Instruction::i32_load8_s_at,
        )
    }

    fn visit_i32_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i32_load8_u,
            Instruction::i32_load8_u_offset16,
            Instruction::i32_load8_u_at,
        )
    }

    fn visit_i32_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i32_load16_s,
            Instruction::i32_load16_s_offset16,
            Instruction::i32_load16_s_at,
        )
    }

    fn visit_i32_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i32_load16_u,
            Instruction::i32_load16_u_offset16,
            Instruction::i32_load16_u_at,
        )
    }

    fn visit_i64_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load8_s,
            Instruction::i64_load8_s_offset16,
            Instruction::i64_load8_s_at,
        )
    }

    fn visit_i64_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load8_u,
            Instruction::i64_load8_u_offset16,
            Instruction::i64_load8_u_at,
        )
    }

    fn visit_i64_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load16_s,
            Instruction::i64_load16_s_offset16,
            Instruction::i64_load16_s_at,
        )
    }

    fn visit_i64_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load16_u,
            Instruction::i64_load16_u_offset16,
            Instruction::i64_load16_u_at,
        )
    }

    fn visit_i64_load32_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load32_s,
            Instruction::i64_load32_s_offset16,
            Instruction::i64_load32_s_at,
        )
    }

    fn visit_i64_load32_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load32_u,
            Instruction::i64_load32_u_offset16,
            Instruction::i64_load32_u_at,
        )
    }

    fn visit_i32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore::<i32, i16>(
            memarg,
            Instruction::i32_store,
            Instruction::i32_store_offset16,
            Instruction::i32_store_offset16_imm16,
            Instruction::i32_store_at,
            Instruction::i32_store_at_imm16,
        )
    }

    fn visit_i64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore::<i64, i16>(
            memarg,
            Instruction::i64_store,
            Instruction::i64_store_offset16,
            Instruction::i64_store_offset16_imm16,
            Instruction::i64_store_at,
            Instruction::i64_store_at_imm16,
        )
    }

    fn visit_f32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_fstore(
            memarg,
            Instruction::f32_store,
            Instruction::f32_store_offset16,
            Instruction::f32_store_at,
        )
    }

    fn visit_f64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_fstore(
            memarg,
            Instruction::f64_store,
            Instruction::f64_store_offset16,
            Instruction::f64_store_at,
        )
    }

    fn visit_i32_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore::<i32, i8>(
            memarg,
            Instruction::i32_store8,
            Instruction::i32_store8_offset16,
            Instruction::i32_store8_offset16_imm,
            Instruction::i32_store8_at,
            Instruction::i32_store8_at_imm,
        )
    }

    fn visit_i32_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore::<i32, i16>(
            memarg,
            Instruction::i32_store16,
            Instruction::i32_store16_offset16,
            Instruction::i32_store16_offset16_imm,
            Instruction::i32_store16_at,
            Instruction::i32_store16_at_imm,
        )
    }

    fn visit_i64_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore::<i64, i8>(
            memarg,
            Instruction::i64_store8,
            Instruction::i64_store8_offset16,
            Instruction::i64_store8_offset16_imm,
            Instruction::i64_store8_at,
            Instruction::i64_store8_at_imm,
        )
    }

    fn visit_i64_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore::<i64, i16>(
            memarg,
            Instruction::i64_store16,
            Instruction::i64_store16_offset16,
            Instruction::i64_store16_offset16_imm,
            Instruction::i64_store16_at,
            Instruction::i64_store16_at_imm,
        )
    }

    fn visit_i64_store32(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_istore::<i64, i16>(
            memarg,
            Instruction::i64_store32,
            Instruction::i64_store32_offset16,
            Instruction::i64_store32_offset16_imm16,
            Instruction::i64_store32_at,
            Instruction::i64_store32_at_imm16,
        )
    }

    fn visit_memory_size(&mut self, mem: u32, _mem_byte: u8) -> Self::Output {
        debug_assert_eq!(
            mem, 0,
            "wasmi does not yet support the multi-memory Wasm proposal"
        );
        bail_unreachable!(self);
        let result = self.alloc.stack.push_dynamic()?;
        self.push_fueled_instr(Instruction::memory_size(result), FuelCosts::entity)?;
        Ok(())
    }

    fn visit_memory_grow(&mut self, _mem: u32, _mem_byte: u8) -> Self::Output {
        bail_unreachable!(self);
        let delta = self.alloc.stack.pop();
        let delta = <Provider<Const16<u32>>>::new(delta, &mut self.alloc.stack)?;
        let result = self.alloc.stack.push_dynamic()?;
        let instr = match delta {
            Provider::Register(delta) => Instruction::memory_grow(result, delta),
            Provider::Const(delta) if u32::from(delta) == 0 => {
                // Case: growing by 0 pages.
                //
                // Since `memory.grow` returns the `memory.size` before the
                // operation a `memory.grow` with `delta` of 0 can be translated
                // as `memory.size` instruction instead.
                Instruction::memory_size(result)
            }
            Provider::Const(delta) => Instruction::memory_grow_by(result, delta),
        };
        self.push_fueled_instr(instr, FuelCosts::entity)?;
        Ok(())
    }

    fn visit_i32_const(&mut self, value: i32) -> Self::Output {
        bail_unreachable!(self);
        self.alloc.stack.push_const(value);
        Ok(())
    }

    fn visit_i64_const(&mut self, value: i64) -> Self::Output {
        bail_unreachable!(self);
        self.alloc.stack.push_const(value);
        Ok(())
    }

    fn visit_f32_const(&mut self, value: wasmparser::Ieee32) -> Self::Output {
        bail_unreachable!(self);
        self.alloc.stack.push_const(F32::from_bits(value.bits()));
        Ok(())
    }

    fn visit_f64_const(&mut self, value: wasmparser::Ieee64) -> Self::Output {
        bail_unreachable!(self);
        self.alloc.stack.push_const(F64::from_bits(value.bits()));
        Ok(())
    }

    fn visit_ref_null(&mut self, ty: wasmparser::ValType) -> Self::Output {
        bail_unreachable!(self);
        let type_hint = WasmiValueType::from(ty).into_inner();
        let null = match type_hint {
            ValType::FuncRef => TypedVal::from(FuncRef::null()),
            ValType::ExternRef => TypedVal::from(ExternRef::null()),
            _ => panic!("must be a Wasm reftype"),
        };
        self.alloc.stack.push_const(null);
        Ok(())
    }

    fn visit_ref_is_null(&mut self) -> Self::Output {
        // Note: Since `funcref` and `externref` both serialize to `UntypedValue`
        //       as raw `u64` values we can use `i64.eqz` translation for `ref.is_null`.
        self.visit_i64_eqz()
    }

    fn visit_ref_func(&mut self, function_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let result = self.alloc.stack.push_dynamic()?;
        self.push_fueled_instr(
            Instruction::ref_func(result, function_index),
            FuelCosts::entity,
        )?;
        Ok(())
    }

    fn visit_i32_eqz(&mut self) -> Self::Output {
        bail_unreachable!(self);
        if self.alloc.instr_encoder.fuse_i32_eqz(&mut self.alloc.stack) {
            // Optimization of `i32.eqz` was applied so we can bail out.
            return Ok(());
        }
        // Push a zero on the value stack so we can translate `i32.eqz` as `i32.eq(x, 0)`.
        self.alloc.stack.push_const(0_i32);
        self.visit_i32_eq()
    }

    fn visit_i32_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32>(
            Instruction::i32_eq,
            Instruction::i32_eq_imm16,
            TypedVal::i32_eq,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x == x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i32_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32>(
            Instruction::i32_ne,
            Instruction::i32_ne_imm16,
            TypedVal::i32_ne,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x != x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i32_lt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_lt_s,
            Instruction::i32_lt_s_imm16,
            swap_ops!(Instruction::i32_gt_s_imm16),
            TypedVal::i32_lt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i32| {
                if rhs == i32::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, _rhs: Register| {
                if lhs == i32::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_lt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_lt_u,
            Instruction::i32_lt_u_imm16,
            swap_ops!(Instruction::i32_gt_u_imm16),
            TypedVal::i32_lt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u32| {
                if rhs == u32::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, _rhs: Register| {
                if lhs == u32::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_gt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_gt_s,
            Instruction::i32_gt_s_imm16,
            swap_ops!(Instruction::i32_lt_s_imm16),
            TypedVal::i32_gt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i32| {
                if rhs == i32::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, _rhs: Register| {
                if lhs == i32::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_gt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_gt_u,
            Instruction::i32_gt_u_imm16,
            swap_ops!(Instruction::i32_lt_u_imm16),
            TypedVal::i32_gt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u32| {
                if rhs == u32::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, _rhs: Register| {
                if lhs == u32::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_le_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_le_s,
            Instruction::i32_le_s_imm16,
            swap_ops!(Instruction::i32_ge_s_imm16),
            TypedVal::i32_le_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i32| {
                if rhs == i32::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, _rhs: Register| {
                if lhs == i32::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_le_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_le_u,
            Instruction::i32_le_u_imm16,
            swap_ops!(Instruction::i32_ge_u_imm16),
            TypedVal::i32_le_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u32| {
                if rhs == u32::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, _rhs: Register| {
                if lhs == u32::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_ge_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_ge_s,
            Instruction::i32_ge_s_imm16,
            swap_ops!(Instruction::i32_le_s_imm16),
            TypedVal::i32_ge_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i32| {
                if rhs == i32::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, _rhs: Register| {
                if lhs == i32::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_ge_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_ge_u,
            Instruction::i32_ge_u_imm16,
            swap_ops!(Instruction::i32_le_u_imm16),
            TypedVal::i32_ge_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u32| {
                if rhs == u32::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, _rhs: Register| {
                if lhs == u32::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_eqz(&mut self) -> Self::Output {
        bail_unreachable!(self);
        // Push a zero on the value stack so we can translate `i64.eqz` as `i64.eq(x, 0)`.
        self.alloc.stack.push_const(0_i64);
        self.visit_i64_eq()
    }

    fn visit_i64_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64>(
            Instruction::i64_eq,
            Instruction::i64_eq_imm16,
            TypedVal::i64_eq,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x == x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i64_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64>(
            Instruction::i64_ne,
            Instruction::i64_ne_imm16,
            TypedVal::i64_ne,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x != x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i64_lt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_lt_s,
            Instruction::i64_lt_s_imm16,
            swap_ops!(Instruction::i64_gt_s_imm16),
            TypedVal::i64_lt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i64| {
                if rhs == i64::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, _rhs: Register| {
                if lhs == i64::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_lt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_lt_u,
            Instruction::i64_lt_u_imm16,
            swap_ops!(Instruction::i64_gt_u_imm16),
            TypedVal::i64_lt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u64| {
                if rhs == u64::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, _rhs: Register| {
                if lhs == u64::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_gt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_gt_s,
            Instruction::i64_gt_s_imm16,
            swap_ops!(Instruction::i64_lt_s_imm16),
            TypedVal::i64_gt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i64| {
                if rhs == i64::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, _rhs: Register| {
                if lhs == i64::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_gt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_gt_u,
            Instruction::i64_gt_u_imm16,
            swap_ops!(Instruction::i64_lt_u_imm16),
            TypedVal::i64_gt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u64| {
                if rhs == u64::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, _rhs: Register| {
                if lhs == u64::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_le_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_le_s,
            Instruction::i64_le_s_imm16,
            swap_ops!(Instruction::i64_ge_s_imm16),
            TypedVal::i64_le_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i64| {
                if rhs == i64::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, _rhs: Register| {
                if lhs == i64::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_le_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_le_u,
            Instruction::i64_le_u_imm16,
            swap_ops!(Instruction::i64_ge_u_imm16),
            TypedVal::i64_le_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u64| {
                if rhs == u64::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, _rhs: Register| {
                if lhs == u64::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_ge_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_ge_s,
            Instruction::i64_ge_s_imm16,
            swap_ops!(Instruction::i64_le_s_imm16),
            TypedVal::i64_ge_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i64| {
                if rhs == i64::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, _rhs: Register| {
                if lhs == i64::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_ge_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_ge_u,
            Instruction::i64_ge_u_imm16,
            swap_ops!(Instruction::i64_le_u_imm16),
            TypedVal::i64_ge_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u64| {
                if rhs == u64::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, _rhs: Register| {
                if lhs == u64::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_eq(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f32>(
            Instruction::f32_eq,
            TypedVal::f32_eq,
            Self::no_custom_opt,
            |this, _reg_in: Register, imm_in: f32| {
                if imm_in.is_nan() {
                    // Optimization: `NaN == x` or `x == NaN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_ne(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f32>(
            Instruction::f32_ne,
            TypedVal::f32_ne,
            Self::no_custom_opt,
            |this, _reg_in: Register, imm_in: f32| {
                if imm_in.is_nan() {
                    // Optimization: `NaN != x` or `x != NaN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_lt(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f32_lt,
            TypedVal::f32_lt,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f32| {
                if rhs.is_nan() {
                    // Optimization: `x < NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if rhs.is_infinite() && rhs.is_sign_negative() {
                    // Optimization: `x < -INF` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f32, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if lhs.is_infinite() && lhs.is_sign_positive() {
                    // Optimization: `+INF < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_gt(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f32_gt,
            TypedVal::f32_gt,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f32| {
                if rhs.is_nan() {
                    // Optimization: `x > NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if rhs.is_infinite() && rhs.is_sign_positive() {
                    // Optimization: `x > INF` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f32, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if lhs.is_infinite() && lhs.is_sign_negative() {
                    // Optimization: `-INF > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_le(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f32_le,
            TypedVal::f32_le,
            Self::no_custom_opt,
            |this, _lhs: Register, rhs: f32| {
                if rhs.is_nan() {
                    // Optimization: `x <= NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f32, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN <= x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_ge(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f32_ge,
            TypedVal::f32_ge,
            Self::no_custom_opt,
            |this, _lhs: Register, rhs: f32| {
                if rhs.is_nan() {
                    // Optimization: `x >= NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f32, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN >= x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_eq(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f64>(
            Instruction::f64_eq,
            TypedVal::f64_eq,
            Self::no_custom_opt,
            |this, _reg_in: Register, imm_in: f64| {
                if imm_in.is_nan() {
                    // Optimization: `NaN == x` or `x == NaN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_ne(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f64>(
            Instruction::f64_ne,
            TypedVal::f64_ne,
            Self::no_custom_opt,
            |this, _reg_in: Register, imm_in: f64| {
                if imm_in.is_nan() {
                    // Optimization: `NaN != x` or `x != NaN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_lt(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f64_lt,
            TypedVal::f64_lt,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f64| {
                if rhs.is_nan() {
                    // Optimization: `x < NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if rhs.is_infinite() && rhs.is_sign_negative() {
                    // Optimization: `x < -INF` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f64, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if lhs.is_infinite() && lhs.is_sign_positive() {
                    // Optimization: `+INF < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_gt(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f64_gt,
            TypedVal::f64_gt,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f64| {
                if rhs.is_nan() {
                    // Optimization: `x > NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if rhs.is_infinite() && rhs.is_sign_positive() {
                    // Optimization: `x > INF` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f64, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if lhs.is_infinite() && lhs.is_sign_negative() {
                    // Optimization: `-INF > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_le(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f64_le,
            TypedVal::f64_le,
            Self::no_custom_opt,
            |this, _lhs: Register, rhs: f64| {
                if rhs.is_nan() {
                    // Optimization: `x <= NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f64, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN <= x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_ge(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f64_ge,
            TypedVal::f64_ge,
            Self::no_custom_opt,
            |this, _lhs: Register, rhs: f64| {
                if rhs.is_nan() {
                    // Optimization: `x >= NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f64, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN >= x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_clz(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_clz, TypedVal::i32_clz)
    }

    fn visit_i32_ctz(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_ctz, TypedVal::i32_ctz)
    }

    fn visit_i32_popcnt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_popcnt, TypedVal::i32_popcnt)
    }

    fn visit_i32_add(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_add,
            Instruction::i32_add_imm16,
            TypedVal::i32_add,
            Self::no_custom_opt,
            |this, reg: Register, value: i32| {
                if value == 0 {
                    // Optimization: `add x + 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_sub(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_sub,
            |_, _, _| unreachable!("`i32.sub r c` is translated as `i32.add r -c`"),
            Instruction::i32_sub_imm16_rev,
            TypedVal::i32_sub,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `sub x - x` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == 0 {
                    // Optimization: `sub x - 0` is same as `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                if this.try_push_binary_instr_imm16(
                    lhs,
                    rhs.wrapping_neg(),
                    Instruction::i32_add_imm16,
                )? {
                    // Simplification: Translate `i32.sub r c` as `i32.add r -c`
                    return Ok(true);
                }
                this.push_binary_instr_imm(lhs, rhs.wrapping_neg(), Instruction::i32_add)?;
                Ok(true)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i32_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_mul,
            Instruction::i32_mul_imm16,
            TypedVal::i32_mul,
            Self::no_custom_opt,
            |this, reg: Register, value: i32| {
                if value == 0 {
                    // Optimization: `add x * 0` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                if value == 1 {
                    // Optimization: `add x * 1` is always `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_div_s(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i32_div_s,
            Instruction::i32_div_s_imm16,
            Instruction::i32_div_s_imm16_rev,
            TypedVal::i32_div_s,
            Self::no_custom_opt,
            |this, lhs: Register, rhs: i32| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_div_u(&mut self) -> Self::Output {
        self.translate_divrem::<u32, NonZeroU32>(
            Instruction::i32_div_u,
            Instruction::i32_div_u_imm16,
            Instruction::i32_div_u_imm16_rev,
            TypedVal::i32_div_u,
            Self::no_custom_opt,
            |this, lhs: Register, rhs: u32| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_rem_s(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i32_rem_s,
            Instruction::i32_rem_s_imm16,
            Instruction::i32_rem_s_imm16_rev,
            TypedVal::i32_rem_s,
            Self::no_custom_opt,
            |this, _lhs: Register, rhs: i32| {
                if rhs == 1 || rhs == -1 {
                    // Optimization: `x % 1` or `x % -1` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_rem_u(&mut self) -> Self::Output {
        self.translate_divrem::<u32, NonZeroU32>(
            Instruction::i32_rem_u,
            Instruction::i32_rem_u_imm16,
            Instruction::i32_rem_u_imm16_rev,
            TypedVal::i32_rem_u,
            Self::no_custom_opt,
            |this, _lhs: Register, rhs: u32| {
                if rhs == 1 {
                    // Optimization: `x % 1` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_and(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_and,
            Instruction::i32_and_imm16,
            TypedVal::i32_and,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x & x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i32| {
                if value == -1 {
                    // Optimization: `x & -1` is same as `x`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x & 0` is same as `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_or(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_or,
            Instruction::i32_or_imm16,
            TypedVal::i32_or,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x | x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i32| {
                if value == -1 {
                    // Optimization: `x | -1` is same as `-1`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_const(-1_i32);
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x | 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_xor(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_xor,
            Instruction::i32_xor_imm16,
            TypedVal::i32_xor,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x ^ x` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i32| {
                if value == 0 {
                    // Optimization: `x ^ 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_shl(&mut self) -> Self::Output {
        self.translate_shift::<i32>(
            Instruction::i32_shl,
            Instruction::i32_shl_imm,
            Instruction::i32_shl_imm16_rev,
            TypedVal::i32_shl,
            Self::no_custom_opt,
        )
    }

    fn visit_i32_shr_s(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_shr_s,
            Instruction::i32_shr_s_imm,
            Instruction::i32_shr_s_imm16_rev,
            TypedVal::i32_shr_s,
            |this, lhs: i32, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_shr_u(&mut self) -> Self::Output {
        self.translate_shift::<i32>(
            Instruction::i32_shr_u,
            Instruction::i32_shr_u_imm,
            Instruction::i32_shr_u_imm16_rev,
            TypedVal::i32_shr_u,
            Self::no_custom_opt,
        )
    }

    fn visit_i32_rotl(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_rotl,
            Instruction::i32_rotl_imm,
            Instruction::i32_rotl_imm16_rev,
            TypedVal::i32_rotl,
            |this, lhs: i32, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1.rotate_left(x)` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_rotr(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_rotr,
            Instruction::i32_rotr_imm,
            Instruction::i32_rotr_imm16_rev,
            TypedVal::i32_rotr,
            |this, lhs: i32, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1.rotate_right(x)` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_clz(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_clz, TypedVal::i64_clz)
    }

    fn visit_i64_ctz(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_ctz, TypedVal::i64_ctz)
    }

    fn visit_i64_popcnt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_popcnt, TypedVal::i64_popcnt)
    }

    fn visit_i64_add(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_add,
            Instruction::i64_add_imm16,
            TypedVal::i64_add,
            Self::no_custom_opt,
            |this, reg: Register, value: i64| {
                if value == 0 {
                    // Optimization: `add x + 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_sub(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_sub,
            |_, _, _| unreachable!("`i64.sub r c` is translated as `i64.add r -c`"),
            Instruction::i64_sub_imm16_rev,
            TypedVal::i64_sub,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `sub x - x` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == 0 {
                    // Optimization: `sub x - 0` is same as `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                if this.try_push_binary_instr_imm16(
                    lhs,
                    rhs.wrapping_neg(),
                    Instruction::i64_add_imm16,
                )? {
                    // Simplification: Translate `i64.sub r c` as `i64.add r -c`
                    return Ok(true);
                }
                this.push_binary_instr_imm(lhs, rhs.wrapping_neg(), Instruction::i64_add)?;
                Ok(true)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i64_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_mul,
            Instruction::i64_mul_imm16,
            TypedVal::i64_mul,
            Self::no_custom_opt,
            |this, reg: Register, value: i64| {
                if value == 0 {
                    // Optimization: `add x * 0` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                if value == 1 {
                    // Optimization: `add x * 1` is always `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_div_s(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i64_div_s,
            Instruction::i64_div_s_imm16,
            Instruction::i64_div_s_imm16_rev,
            TypedVal::i64_div_s,
            Self::no_custom_opt,
            |this, lhs: Register, rhs: i64| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_div_u(&mut self) -> Self::Output {
        self.translate_divrem::<u64, NonZeroU64>(
            Instruction::i64_div_u,
            Instruction::i64_div_u_imm16,
            Instruction::i64_div_u_imm16_rev,
            TypedVal::i64_div_u,
            Self::no_custom_opt,
            |this, lhs: Register, rhs: u64| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_rem_s(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i64_rem_s,
            Instruction::i64_rem_s_imm16,
            Instruction::i64_rem_s_imm16_rev,
            TypedVal::i64_rem_s,
            Self::no_custom_opt,
            |this, _lhs: Register, rhs: i64| {
                if rhs == 1 || rhs == -1 {
                    // Optimization: `x % 1` or `x % -1` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_rem_u(&mut self) -> Self::Output {
        self.translate_divrem::<u64, NonZeroU64>(
            Instruction::i64_rem_u,
            Instruction::i64_rem_u_imm16,
            Instruction::i64_rem_u_imm16_rev,
            TypedVal::i64_rem_u,
            Self::no_custom_opt,
            |this, _lhs: Register, rhs: u64| {
                if rhs == 1 {
                    // Optimization: `x % 1` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_and(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_and,
            Instruction::i64_and_imm16,
            TypedVal::i64_and,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x & x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i64| {
                if value == -1 {
                    // Optimization: `x & -1` is same as `x`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x & 0` is same as `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_or(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_or,
            Instruction::i64_or_imm16,
            TypedVal::i64_or,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x | x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i64| {
                if value == -1 {
                    // Optimization: `x | -1` is same as `-1`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_const(-1_i64);
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x | 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_xor(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_xor,
            Instruction::i64_xor_imm16,
            TypedVal::i64_xor,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x ^ x` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i64| {
                if value == 0 {
                    // Optimization: `x ^ 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_shl(&mut self) -> Self::Output {
        self.translate_shift::<i64>(
            Instruction::i64_shl,
            Instruction::i64_shl_imm,
            Instruction::i64_shl_imm16_rev,
            TypedVal::i64_shl,
            Self::no_custom_opt,
        )
    }

    fn visit_i64_shr_s(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_shr_s,
            Instruction::i64_shr_s_imm,
            Instruction::i64_shr_s_imm16_rev,
            TypedVal::i64_shr_s,
            |this, lhs: i64, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_shr_u(&mut self) -> Self::Output {
        self.translate_shift::<i64>(
            Instruction::i64_shr_u,
            Instruction::i64_shr_u_imm,
            Instruction::i64_shr_u_imm16_rev,
            TypedVal::i64_shr_u,
            Self::no_custom_opt,
        )
    }

    fn visit_i64_rotl(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_rotl,
            Instruction::i64_rotl_imm,
            Instruction::i64_rotl_imm16_rev,
            TypedVal::i64_rotl,
            |this, lhs: i64, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_rotr(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_rotr,
            Instruction::i64_rotr_imm,
            Instruction::i64_rotr_imm16_rev,
            TypedVal::i64_rotr,
            |this, lhs: i64, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_abs(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_abs, TypedVal::f32_abs)
    }

    fn visit_f32_neg(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_neg, TypedVal::f32_neg)
    }

    fn visit_f32_ceil(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_ceil, TypedVal::f32_ceil)
    }

    fn visit_f32_floor(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_floor, TypedVal::f32_floor)
    }

    fn visit_f32_trunc(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_trunc, TypedVal::f32_trunc)
    }

    fn visit_f32_nearest(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_nearest, TypedVal::f32_nearest)
    }

    fn visit_f32_sqrt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_sqrt, TypedVal::f32_sqrt)
    }

    fn visit_f32_add(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f32_add,
            TypedVal::f32_add,
            Self::no_custom_opt,
            Self::no_custom_opt::<Register, f32>,
        )
    }

    fn visit_f32_sub(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f32_sub,
            TypedVal::f32_sub,
            Self::no_custom_opt,
            Self::no_custom_opt::<Register, f32>,
            // Unfortunately we cannot optimize for the case that `lhs == 0.0`
            // since the Wasm specification mandates different behavior in
            // dependence of `rhs` which we do not know at this point.
            Self::no_custom_opt,
        )
    }

    fn visit_f32_mul(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f32>(
            Instruction::f32_mul,
            TypedVal::f32_mul,
            Self::no_custom_opt,
            // Unfortunately we cannot apply `x * 0` or `0 * x` optimizations
            // since Wasm mandates different behaviors if `x` is infinite or
            // NaN in these cases.
            Self::no_custom_opt,
        )
    }

    fn visit_f32_div(&mut self) -> Self::Output {
        self.translate_fbinary::<f32>(
            Instruction::f32_div,
            TypedVal::f32_div,
            Self::no_custom_opt,
            Self::no_custom_opt,
            Self::no_custom_opt,
        )
    }

    fn visit_f32_min(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f32_min,
            TypedVal::f32_min,
            Self::no_custom_opt,
            |this, reg: Register, value: f32| {
                if value.is_infinite() && value.is_sign_positive() {
                    // Optimization: `min(x, +inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_max(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f32_max,
            TypedVal::f32_max,
            Self::no_custom_opt,
            |this, reg: Register, value: f32| {
                if value.is_infinite() && value.is_sign_negative() {
                    // Optimization: `max(x, -inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_copysign(&mut self) -> Self::Output {
        self.translate_fcopysign::<f32>(
            Instruction::f32_copysign,
            Instruction::f32_copysign_imm,
            TypedVal::f32_copysign,
        )
    }

    fn visit_f64_abs(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_abs, TypedVal::f64_abs)
    }

    fn visit_f64_neg(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_neg, TypedVal::f64_neg)
    }

    fn visit_f64_ceil(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_ceil, TypedVal::f64_ceil)
    }

    fn visit_f64_floor(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_floor, TypedVal::f64_floor)
    }

    fn visit_f64_trunc(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_trunc, TypedVal::f64_trunc)
    }

    fn visit_f64_nearest(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_nearest, TypedVal::f64_nearest)
    }

    fn visit_f64_sqrt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_sqrt, TypedVal::f64_sqrt)
    }

    fn visit_f64_add(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f64_add,
            TypedVal::f64_add,
            Self::no_custom_opt,
            Self::no_custom_opt::<Register, f64>,
        )
    }

    fn visit_f64_sub(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f64_sub,
            TypedVal::f64_sub,
            Self::no_custom_opt,
            Self::no_custom_opt::<Register, f64>,
            // Unfortunately we cannot optimize for the case that `lhs == 0.0`
            // since the Wasm specification mandates different behavior in
            // dependence of `rhs` which we do not know at this point.
            Self::no_custom_opt,
        )
    }

    fn visit_f64_mul(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f64>(
            Instruction::f64_mul,
            TypedVal::f64_mul,
            Self::no_custom_opt,
            // Unfortunately we cannot apply `x * 0` or `0 * x` optimizations
            // since Wasm mandates different behaviors if `x` is infinite or
            // NaN in these cases.
            Self::no_custom_opt,
        )
    }

    fn visit_f64_div(&mut self) -> Self::Output {
        self.translate_fbinary::<f64>(
            Instruction::f64_div,
            TypedVal::f64_div,
            Self::no_custom_opt,
            Self::no_custom_opt,
            Self::no_custom_opt,
        )
    }

    fn visit_f64_min(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f64_min,
            TypedVal::f64_min,
            Self::no_custom_opt,
            |this, reg: Register, value: f64| {
                if value.is_infinite() && value.is_sign_positive() {
                    // Optimization: `min(x, +inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_max(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f64_max,
            TypedVal::f64_max,
            Self::no_custom_opt,
            |this, reg: Register, value: f64| {
                if value.is_infinite() && value.is_sign_negative() {
                    // Optimization: `min(x, +inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_copysign(&mut self) -> Self::Output {
        self.translate_fcopysign::<f64>(
            Instruction::f64_copysign,
            Instruction::f64_copysign_imm,
            TypedVal::f64_copysign,
        )
    }

    fn visit_i32_wrap_i64(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_wrap_i64, TypedVal::i32_wrap_i64)
    }

    fn visit_i32_trunc_f32_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f32_s, TypedVal::i32_trunc_f32_s)
    }

    fn visit_i32_trunc_f32_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f32_u, TypedVal::i32_trunc_f32_u)
    }

    fn visit_i32_trunc_f64_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f64_s, TypedVal::i32_trunc_f64_s)
    }

    fn visit_i32_trunc_f64_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f64_u, TypedVal::i32_trunc_f64_u)
    }

    fn visit_i64_extend_i32_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend_i32_s, TypedVal::i64_extend_i32_s)
    }

    fn visit_i64_extend_i32_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend_i32_u, TypedVal::i64_extend_i32_u)
    }

    fn visit_i64_trunc_f32_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f32_s, TypedVal::i64_trunc_f32_s)
    }

    fn visit_i64_trunc_f32_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f32_u, TypedVal::i64_trunc_f32_u)
    }

    fn visit_i64_trunc_f64_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f64_s, TypedVal::i64_trunc_f64_s)
    }

    fn visit_i64_trunc_f64_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f64_u, TypedVal::i64_trunc_f64_u)
    }

    fn visit_f32_convert_i32_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_convert_i32_s, TypedVal::f32_convert_i32_s)
    }

    fn visit_f32_convert_i32_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_convert_i32_u, TypedVal::f32_convert_i32_u)
    }

    fn visit_f32_convert_i64_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_convert_i64_s, TypedVal::f32_convert_i64_s)
    }

    fn visit_f32_convert_i64_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_convert_i64_u, TypedVal::f32_convert_i64_u)
    }

    fn visit_f32_demote_f64(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_demote_f64, TypedVal::f32_demote_f64)
    }

    fn visit_f64_convert_i32_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_convert_i32_s, TypedVal::f64_convert_i32_s)
    }

    fn visit_f64_convert_i32_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_convert_i32_u, TypedVal::f64_convert_i32_u)
    }

    fn visit_f64_convert_i64_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_convert_i64_s, TypedVal::f64_convert_i64_s)
    }

    fn visit_f64_convert_i64_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_convert_i64_u, TypedVal::f64_convert_i64_u)
    }

    fn visit_f64_promote_f32(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_promote_f32, TypedVal::f64_promote_f32)
    }

    fn visit_i32_reinterpret_f32(&mut self) -> Self::Output {
        self.translate_reinterpret(ValType::I32)
    }

    fn visit_i64_reinterpret_f64(&mut self) -> Self::Output {
        self.translate_reinterpret(ValType::I64)
    }

    fn visit_f32_reinterpret_i32(&mut self) -> Self::Output {
        self.translate_reinterpret(ValType::F32)
    }

    fn visit_f64_reinterpret_i64(&mut self) -> Self::Output {
        self.translate_reinterpret(ValType::F64)
    }

    fn visit_i32_extend8_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_extend8_s, TypedVal::i32_extend8_s)
    }

    fn visit_i32_extend16_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_extend16_s, TypedVal::i32_extend16_s)
    }

    fn visit_i64_extend8_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend8_s, TypedVal::i64_extend8_s)
    }

    fn visit_i64_extend16_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend16_s, TypedVal::i64_extend16_s)
    }

    fn visit_i64_extend32_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend32_s, TypedVal::i64_extend32_s)
    }

    fn visit_i32_trunc_sat_f32_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i32_trunc_sat_f32_s,
            TypedVal::i32_trunc_sat_f32_s,
        )
    }

    fn visit_i32_trunc_sat_f32_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i32_trunc_sat_f32_u,
            TypedVal::i32_trunc_sat_f32_u,
        )
    }

    fn visit_i32_trunc_sat_f64_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i32_trunc_sat_f64_s,
            TypedVal::i32_trunc_sat_f64_s,
        )
    }

    fn visit_i32_trunc_sat_f64_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i32_trunc_sat_f64_u,
            TypedVal::i32_trunc_sat_f64_u,
        )
    }

    fn visit_i64_trunc_sat_f32_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i64_trunc_sat_f32_s,
            TypedVal::i64_trunc_sat_f32_s,
        )
    }

    fn visit_i64_trunc_sat_f32_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i64_trunc_sat_f32_u,
            TypedVal::i64_trunc_sat_f32_u,
        )
    }

    fn visit_i64_trunc_sat_f64_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i64_trunc_sat_f64_s,
            TypedVal::i64_trunc_sat_f64_s,
        )
    }

    fn visit_i64_trunc_sat_f64_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i64_trunc_sat_f64_u,
            TypedVal::i64_trunc_sat_f64_u,
        )
    }

    fn visit_memory_init(&mut self, data_index: u32, _mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.alloc.stack.pop3();
        let dst = <Provider<Const16<u32>>>::new(dst, &mut self.alloc.stack)?;
        let src = <Provider<Const16<u32>>>::new(src, &mut self.alloc.stack)?;
        let len = <Provider<Const16<u32>>>::new(len, &mut self.alloc.stack)?;
        let instr = match (dst, src, len) {
            (Provider::Register(dst), Provider::Register(src), Provider::Register(len)) => {
                Instruction::memory_init(dst, src, len)
            }
            (Provider::Register(dst), Provider::Register(src), Provider::Const(len)) => {
                Instruction::memory_init_exact(dst, src, len)
            }
            (Provider::Register(dst), Provider::Const(src), Provider::Register(len)) => {
                Instruction::memory_init_from(dst, src, len)
            }
            (Provider::Register(dst), Provider::Const(src), Provider::Const(len)) => {
                Instruction::memory_init_from_exact(dst, src, len)
            }
            (Provider::Const(dst), Provider::Register(src), Provider::Register(len)) => {
                Instruction::memory_init_to(dst, src, len)
            }
            (Provider::Const(dst), Provider::Register(src), Provider::Const(len)) => {
                Instruction::memory_init_to_exact(dst, src, len)
            }
            (Provider::Const(dst), Provider::Const(src), Provider::Register(len)) => {
                Instruction::memory_init_from_to(dst, src, len)
            }
            (Provider::Const(dst), Provider::Const(src), Provider::Const(len)) => {
                Instruction::memory_init_from_to_exact(dst, src, len)
            }
        };
        self.push_fueled_instr(instr, FuelCosts::entity)?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::data_idx(data_index))?;
        Ok(())
    }

    fn visit_data_drop(&mut self, data_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.push_fueled_instr(Instruction::DataDrop(data_index.into()), FuelCosts::entity)?;
        Ok(())
    }

    fn visit_memory_copy(&mut self, _dst_mem: u32, _src_mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.alloc.stack.pop3();
        let dst = <Provider<Const16<u32>>>::new(dst, &mut self.alloc.stack)?;
        let src = <Provider<Const16<u32>>>::new(src, &mut self.alloc.stack)?;
        let len = <Provider<Const16<u32>>>::new(len, &mut self.alloc.stack)?;
        let instr = match (dst, src, len) {
            (Provider::Register(dst), Provider::Register(src), Provider::Register(len)) => {
                Instruction::memory_copy(dst, src, len)
            }
            (Provider::Register(dst), Provider::Register(src), Provider::Const(len)) => {
                Instruction::memory_copy_exact(dst, src, len)
            }
            (Provider::Register(dst), Provider::Const(src), Provider::Register(len)) => {
                Instruction::memory_copy_from(dst, src, len)
            }
            (Provider::Register(dst), Provider::Const(src), Provider::Const(len)) => {
                Instruction::memory_copy_from_exact(dst, src, len)
            }
            (Provider::Const(dst), Provider::Register(src), Provider::Register(len)) => {
                Instruction::memory_copy_to(dst, src, len)
            }
            (Provider::Const(dst), Provider::Register(src), Provider::Const(len)) => {
                Instruction::memory_copy_to_exact(dst, src, len)
            }
            (Provider::Const(dst), Provider::Const(src), Provider::Register(len)) => {
                Instruction::memory_copy_from_to(dst, src, len)
            }
            (Provider::Const(dst), Provider::Const(src), Provider::Const(len)) => {
                Instruction::memory_copy_from_to_exact(dst, src, len)
            }
        };
        self.push_fueled_instr(instr, FuelCosts::entity)?;
        Ok(())
    }

    fn visit_memory_fill(&mut self, _mem: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, value, len) = self.alloc.stack.pop3();
        let dst = <Provider<Const16<u32>>>::new(dst, &mut self.alloc.stack)?;
        let value = <Provider<u8>>::new(value);
        let len = <Provider<Const16<u32>>>::new(len, &mut self.alloc.stack)?;
        let instr = match (dst, value, len) {
            (Provider::Register(dst), Provider::Register(value), Provider::Register(len)) => {
                Instruction::memory_fill(dst, value, len)
            }
            (Provider::Register(dst), Provider::Register(value), Provider::Const(len)) => {
                Instruction::memory_fill_exact(dst, value, len)
            }
            (Provider::Register(dst), Provider::Const(value), Provider::Register(len)) => {
                Instruction::memory_fill_imm(dst, value, len)
            }
            (Provider::Register(dst), Provider::Const(value), Provider::Const(len)) => {
                Instruction::memory_fill_imm_exact(dst, value, len)
            }
            (Provider::Const(dst), Provider::Register(value), Provider::Register(len)) => {
                Instruction::memory_fill_at(dst, value, len)
            }
            (Provider::Const(dst), Provider::Register(value), Provider::Const(len)) => {
                Instruction::memory_fill_at_exact(dst, value, len)
            }
            (Provider::Const(dst), Provider::Const(value), Provider::Register(len)) => {
                Instruction::memory_fill_at_imm(dst, value, len)
            }
            (Provider::Const(dst), Provider::Const(value), Provider::Const(len)) => {
                Instruction::memory_fill_at_imm_exact(dst, value, len)
            }
        };
        self.push_fueled_instr(instr, FuelCosts::entity)?;
        Ok(())
    }

    fn visit_table_init(&mut self, elem_index: u32, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.alloc.stack.pop3();
        let dst = <Provider<Const16<u32>>>::new(dst, &mut self.alloc.stack)?;
        let src = <Provider<Const16<u32>>>::new(src, &mut self.alloc.stack)?;
        let len = <Provider<Const16<u32>>>::new(len, &mut self.alloc.stack)?;
        let instr = match (dst, src, len) {
            (Provider::Register(dst), Provider::Register(src), Provider::Register(len)) => {
                Instruction::table_init(dst, src, len)
            }
            (Provider::Register(dst), Provider::Register(src), Provider::Const(len)) => {
                Instruction::table_init_exact(dst, src, len)
            }
            (Provider::Register(dst), Provider::Const(src), Provider::Register(len)) => {
                Instruction::table_init_from(dst, src, len)
            }
            (Provider::Register(dst), Provider::Const(src), Provider::Const(len)) => {
                Instruction::table_init_from_exact(dst, src, len)
            }
            (Provider::Const(dst), Provider::Register(src), Provider::Register(len)) => {
                Instruction::table_init_to(dst, src, len)
            }
            (Provider::Const(dst), Provider::Register(src), Provider::Const(len)) => {
                Instruction::table_init_to_exact(dst, src, len)
            }
            (Provider::Const(dst), Provider::Const(src), Provider::Register(len)) => {
                Instruction::table_init_from_to(dst, src, len)
            }
            (Provider::Const(dst), Provider::Const(src), Provider::Const(len)) => {
                Instruction::table_init_from_to_exact(dst, src, len)
            }
        };
        self.push_fueled_instr(instr, FuelCosts::entity)?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::table_idx(table))?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::elem_idx(elem_index))?;
        Ok(())
    }

    fn visit_elem_drop(&mut self, elem_index: u32) -> Self::Output {
        bail_unreachable!(self);
        self.push_fueled_instr(Instruction::ElemDrop(elem_index.into()), FuelCosts::entity)?;
        Ok(())
    }

    fn visit_table_copy(&mut self, dst_table: u32, src_table: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, src, len) = self.alloc.stack.pop3();
        let dst = <Provider<Const16<u32>>>::new(dst, &mut self.alloc.stack)?;
        let src = <Provider<Const16<u32>>>::new(src, &mut self.alloc.stack)?;
        let len = <Provider<Const16<u32>>>::new(len, &mut self.alloc.stack)?;
        let instr = match (dst, src, len) {
            (Provider::Register(dst), Provider::Register(src), Provider::Register(len)) => {
                Instruction::table_copy(dst, src, len)
            }
            (Provider::Register(dst), Provider::Register(src), Provider::Const(len)) => {
                Instruction::table_copy_exact(dst, src, len)
            }
            (Provider::Register(dst), Provider::Const(src), Provider::Register(len)) => {
                Instruction::table_copy_from(dst, src, len)
            }
            (Provider::Register(dst), Provider::Const(src), Provider::Const(len)) => {
                Instruction::table_copy_from_exact(dst, src, len)
            }
            (Provider::Const(dst), Provider::Register(src), Provider::Register(len)) => {
                Instruction::table_copy_to(dst, src, len)
            }
            (Provider::Const(dst), Provider::Register(src), Provider::Const(len)) => {
                Instruction::table_copy_to_exact(dst, src, len)
            }
            (Provider::Const(dst), Provider::Const(src), Provider::Register(len)) => {
                Instruction::table_copy_from_to(dst, src, len)
            }
            (Provider::Const(dst), Provider::Const(src), Provider::Const(len)) => {
                Instruction::table_copy_from_to_exact(dst, src, len)
            }
        };
        self.push_fueled_instr(instr, FuelCosts::entity)?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::table_idx(dst_table))?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::table_idx(src_table))?;
        Ok(())
    }

    fn visit_table_fill(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let (dst, value, len) = self.alloc.stack.pop3();
        let dst = <Provider<Const16<u32>>>::new(dst, &mut self.alloc.stack)?;
        let len = <Provider<Const16<u32>>>::new(len, &mut self.alloc.stack)?;
        let value = match value {
            TypedProvider::Register(value) => value,
            TypedProvider::Const(value) => self.alloc.stack.alloc_const(value)?,
        };
        let instr = match (dst, len) {
            (Provider::Register(dst), Provider::Register(len)) => {
                Instruction::table_fill(dst, len, value)
            }
            (Provider::Register(dst), Provider::Const(len)) => {
                Instruction::table_fill_exact(dst, len, value)
            }
            (Provider::Const(dst), Provider::Register(len)) => {
                Instruction::table_fill_at(dst, len, value)
            }
            (Provider::Const(dst), Provider::Const(len)) => {
                Instruction::table_fill_at_exact(dst, len, value)
            }
        };
        self.push_fueled_instr(instr, FuelCosts::entity)?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::table_idx(table))?;
        Ok(())
    }

    fn visit_table_get(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let index = self.alloc.stack.pop();
        let result = self.alloc.stack.push_dynamic()?;
        match index {
            TypedProvider::Register(index) => {
                self.push_fueled_instr(Instruction::table_get(result, index), FuelCosts::entity)?;
            }
            TypedProvider::Const(index) => {
                self.push_fueled_instr(
                    Instruction::table_get_imm(result, u32::from(index)),
                    FuelCosts::entity,
                )?;
            }
        }
        self.alloc
            .instr_encoder
            .append_instr(Instruction::table_idx(table))?;
        Ok(())
    }

    fn visit_table_set(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let (index, value) = self.alloc.stack.pop2();
        let value = match value {
            TypedProvider::Register(value) => value,
            TypedProvider::Const(value) => self.alloc.stack.alloc_const(value)?,
        };
        let instr = match index {
            TypedProvider::Register(index) => Instruction::table_set(index, value),
            TypedProvider::Const(index) => Instruction::table_set_at(u32::from(index), value),
        };
        self.push_fueled_instr(instr, FuelCosts::entity)?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::table_idx(table))?;
        Ok(())
    }

    fn visit_table_grow(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let (value, delta) = self.alloc.stack.pop2();
        if let Provider::Const(delta) = delta {
            if u32::from(delta) == 0 {
                // Case: growing by 0 elements.
                //
                // Since `table.grow` returns the `table.size` before the
                // operation a `table.grow` with `delta` of 0 can be translated
                // as `table.size` instruction instead.
                let result = self.alloc.stack.push_dynamic()?;
                self.push_fueled_instr(Instruction::table_size(result, table), FuelCosts::entity)?;
                return Ok(());
            }
        }
        let delta = <Provider<Const16<u32>>>::new(delta, &mut self.alloc.stack)?;
        let value = match value {
            TypedProvider::Register(value) => value,
            TypedProvider::Const(value) => self.alloc.stack.alloc_const(value)?,
        };
        let result = self.alloc.stack.push_dynamic()?;
        let instr = match delta {
            Provider::Register(delta) => Instruction::table_grow(result, delta, value),
            Provider::Const(delta) => Instruction::table_grow_imm(result, delta, value),
        };
        self.push_fueled_instr(instr, FuelCosts::entity)?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::table_idx(table))?;
        Ok(())
    }

    fn visit_table_size(&mut self, table: u32) -> Self::Output {
        bail_unreachable!(self);
        let result = self.alloc.stack.push_dynamic()?;
        self.push_fueled_instr(Instruction::table_size(result, table), FuelCosts::entity)?;
        Ok(())
    }
}
