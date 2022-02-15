#![allow(dead_code, unused_variables)] // TODO: remove annotation once done

mod control_frame;
mod control_stack;
mod inst_builder;
mod locals_registry;
mod value_stack;

pub use self::inst_builder::{InstructionIdx, InstructionsBuilder, LabelIdx, RelativeDepth, Reloc};
use self::{
    control_frame::{
        BlockControlFrame,
        ControlFrame,
        ControlFrameKind,
        IfControlFrame,
        LoopControlFrame,
        UnreachableControlFrame,
    },
    control_stack::ControlFlowStack,
    locals_registry::LocalsRegistry,
    value_stack::ValueStack,
};
use super::{DropKeep, FuncBody, Instruction, Target};
use crate::{
    engine::bytecode::Offset,
    module::{
        BlockType,
        FuncIdx,
        FuncTypeIdx,
        GlobalIdx,
        MemoryIdx,
        ModuleResources,
        TableIdx,
        DEFAULT_MEMORY_INDEX,
    },
    Engine,
    FuncType,
    ModuleError,
    Mutability,
};
use wasmi_core::{Value, ValueType, F32, F64};

/// The interface to translate a `wasmi` bytecode function using Wasm bytecode.
#[derive(Debug)]
pub struct FunctionBuilder<'engine, 'parser> {
    /// The [`Engine`] for which the function is translated.
    engine: &'engine Engine,
    /// The function under construction.
    func: FuncIdx,
    /// The immutable `wasmi` module resources.
    res: ModuleResources<'parser>,
    /// The control flow frame stack that represents the Wasm control flow.
    control_frames: ControlFlowStack,
    /// The emulated value stack.
    value_stack: ValueStack,
    /// The instruction builder.
    ///
    /// # Note
    ///
    /// Allows to incrementally construct the instruction of a function.
    inst_builder: InstructionsBuilder,
    /// Stores and resolves local variable types.
    locals: LocalsRegistry,
    /// This represents the reachability of the currently translated code.
    ///
    /// - `true`: The currently translated code is reachable.
    /// - `false`: The currently translated code is unreachable and can be skipped.
    ///
    /// # Note
    ///
    /// Visiting the Wasm `Else` or `End` control flow operator resets
    /// reachability to `true` again.
    reachable: bool,
}

impl<'engine, 'parser> FunctionBuilder<'engine, 'parser> {
    /// Creates a new [`FunctionBuilder`].
    pub fn new(engine: &'engine Engine, func: FuncIdx, res: ModuleResources<'parser>) -> Self {
        let mut inst_builder = InstructionsBuilder::default();
        let mut control_frames = ControlFlowStack::default();
        Self::register_func_body_block(func, res, &mut inst_builder, &mut control_frames);
        let mut value_stack = ValueStack::default();
        let mut locals = LocalsRegistry::default();
        Self::register_func_params(func, res, &mut value_stack, &mut locals);
        Self {
            engine,
            func,
            res,
            control_frames,
            value_stack,
            inst_builder,
            locals,
            reachable: true,
        }
    }

    /// Returns the [`FuncType`] of the function that is currently translated.
    fn func_type(&self) -> FuncType {
        let dedup_func_type = self.res.get_type_of_func(self.func);
        self.engine.resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Resolves the [`FuncType`] of the given [`FuncTypeIdx`].
    fn func_type_at(&self, func_type_index: FuncTypeIdx) -> FuncType {
        let dedup_func_type = self.res.get_func_type(func_type_index);
        self.res
            .engine()
            .resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Resolves the [`FuncType`] of the given [`FuncIdx`].
    fn func_type_of(&self, func_index: FuncIdx) -> FuncType {
        let dedup_func_type = self.res.get_type_of_func(func_index);
        self.res
            .engine()
            .resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Registers the `block` control frame surrounding the entire function body.
    fn register_func_body_block(
        func: FuncIdx,
        res: ModuleResources<'parser>,
        inst_builder: &mut InstructionsBuilder,
        control_frames: &mut ControlFlowStack,
    ) {
        let func_type = res.get_type_of_func(func);
        let block_type = BlockType::func_type(func_type);
        let end_label = inst_builder.new_label();
        let block_frame = BlockControlFrame::new(block_type, end_label, 0);
        control_frames.push_frame(block_frame);
    }

    /// Registers the function parameters in the emulated value stack.
    fn register_func_params(
        func: FuncIdx,
        res: ModuleResources<'parser>,
        value_stack: &mut ValueStack,
        locals: &mut LocalsRegistry,
    ) -> usize {
        let dedup_func_type = res.get_type_of_func(func);
        let func_type = res
            .engine()
            .resolve_func_type(dedup_func_type, Clone::clone);
        let params = func_type.params();
        for param_type in params {
            locals.register_locals(*param_type, 1);
        }
        params.len()
    }

    /// Try to resolve the given label.
    ///
    /// In case the label cannot yet be resolved register the [`Reloc`] as its user.
    fn try_resolve_label<F>(&mut self, label: LabelIdx, reloc_provider: F) -> InstructionIdx
    where
        F: FnOnce(InstructionIdx) -> Reloc,
    {
        let pc = self.inst_builder.current_pc();
        self.inst_builder
            .try_resolve_label(label, || reloc_provider(pc))
    }

    /// Translates the given local variables for the translated function.
    pub fn translate_locals(
        &mut self,
        amount: u32,
        value_type: ValueType,
    ) -> Result<(), ModuleError> {
        self.locals.register_locals(value_type, amount);
        Ok(())
    }

    /// Returns the number of local variables of the function under construction.
    fn len_locals(&self) -> usize {
        let len_params_locals = self.locals.len_registered() as usize;
        let len_params = self.func_type().params().len();
        debug_assert!(len_params_locals >= len_params);
        len_params_locals - len_params
    }

    /// Finishes constructing the function and returns its [`FuncBody`].
    pub fn finish(mut self) -> FuncBody {
        self.inst_builder.finish(
            self.engine,
            self.len_locals(),
            self.value_stack.max_stack_height() as usize,
        )
    }

    /// Returns `true` if the code at the current translation position is reachable.
    fn is_reachable(&self) -> bool {
        self.reachable
    }

    /// Translates into `wasmi` bytecode if the current code path is reachable.
    ///
    /// # Note
    ///
    /// Ignores the `translator` closure if the current code path is unreachable.
    fn translate_if_reachable<F>(&mut self, translator: F) -> Result<(), ModuleError>
    where
        F: FnOnce(&mut Self) -> Result<(), ModuleError>,
    {
        if self.is_reachable() {
            translator(self)?;
        }
        Ok(())
    }

    /// Computes how many values should be dropped and kept for the specific branch.
    ///
    /// # Panics
    ///
    /// If underflow of the value stack is detected.
    fn compute_drop_keep(&self, depth: u32) -> DropKeep {
        debug_assert!(self.is_reachable());
        let frame = self.control_frames.nth_back(depth);
        // Find out how many values we need to keep (copy to the new stack location after the drop).
        let keep = match frame.kind() {
            ControlFrameKind::Block | ControlFrameKind::If => {
                frame.block_type().len_results(self.engine)
            }
            ControlFrameKind::Loop => frame.block_type().len_params(self.engine),
        };
        // Find out how many values we need to drop.
        let current_height = self.value_stack.len();
        let origin_height = frame.stack_height();
        assert!(
            origin_height <= current_height,
            "encountered value stack underflow: current height {}, original height {}",
            current_height,
            origin_height,
        );
        let height_diff = current_height - origin_height;
        assert!(
            keep <= height_diff,
            "tried to keep {} values while having only {} values available on the frame",
            keep,
            height_diff,
        );
        let drop = height_diff - keep;
        DropKeep::new32(drop, keep)
    }

    /// Compute [`DropKeep`] for the return statement.
    ///
    /// # Panics
    ///
    /// - If the control flow frame stack is empty.
    /// - If the value stack is underflown.
    fn drop_keep_return(&self) -> DropKeep {
        debug_assert!(self.is_reachable());
        assert!(
            !self.control_frames.is_empty(),
            "drop_keep_return cannot be called with the frame stack empty"
        );
        let max_depth = self
            .control_frames
            .len()
            .checked_sub(1)
            .expect("control flow frame stack must not be empty") as u32;
        let drop_keep = self.compute_drop_keep(max_depth);
        let len_params_locals = self.locals.len_registered() as usize;
        DropKeep::new(
            // Drop all local variables and parameters upon exit.
            drop_keep.drop() + len_params_locals,
            drop_keep.keep(),
        )
    }

    /// Returns the relative depth on the stack of the local variable.
    ///
    /// # Note
    ///
    /// See stack layout definition in `isa.rs`.
    fn relative_local_depth(&self, local_idx: u32) -> u32 {
        debug_assert!(self.is_reachable());
        let stack_height = self.value_stack.len();
        let len_params_locals = self.locals.len_registered();
        stack_height
            .checked_add(len_params_locals)
            .and_then(|x| x.checked_sub(local_idx))
            .unwrap_or_else(|| panic!("cannot convert local index into local depth: {}", local_idx))
    }

    /// Returns the target at the given `depth` together with its [`DropKeep`].
    ///
    /// # Panics
    ///
    /// - If the `depth` is greater than the current height of the control frame stack.
    /// - If the value stack underflowed.
    fn acquire_target(&self, relative_depth: u32) -> AquiredTarget {
        debug_assert!(self.is_reachable());
        if self.control_frames.is_root(relative_depth) {
            let drop_keep = self.drop_keep_return();
            AquiredTarget::Return(drop_keep)
        } else {
            let label = self
                .control_frames
                .nth_back(relative_depth)
                .branch_destination();
            let drop_keep = self.compute_drop_keep(relative_depth);
            AquiredTarget::Branch(label, drop_keep)
        }
    }
}

/// An aquired target.
///
/// Returned by [`FunctionBuilder::acquire_target`].
#[derive(Debug)]
pub enum AquiredTarget {
    /// The branch jumps to the label.
    Branch(LabelIdx, DropKeep),
    /// The branch returns to the caller.
    ///
    /// # Note
    ///
    /// This is returned if the `relative_depth` points to the outmost
    /// function body `block`. WebAssembly defines branches to this control
    /// flow frame as equivalent to returning from the function.
    Return(DropKeep),
}

impl<'engine, 'parser> FunctionBuilder<'engine, 'parser> {
    /// Translates a Wasm `unreachable` instruction.
    pub fn translate_unreachable(&mut self) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            builder.inst_builder.push_inst(Instruction::Unreachable);
            builder.reachable = false;
            Ok(())
        })
    }

    /// Calculates the stack height upon entering a control flow frame.
    ///
    /// # Note
    ///
    /// This does not include the parameters of the control flow frame
    /// so that when shrinking the emulated value stack to the control flow
    /// frame's original stack height the control flow frame parameters are
    /// no longer on the emulated value stack.
    ///
    /// # Panics
    ///
    /// When the emulated value stack underflows. This should not happen
    /// since we have already validated the input Wasm prior.
    fn frame_stack_height(&self, block_type: BlockType) -> u32 {
        let len_params = block_type.len_params(self.engine);
        let stack_height = self.value_stack.len();
        stack_height.checked_sub(len_params).unwrap_or_else(|| {
            panic!(
                "encountered emulated value stack underflow with \
                 stack height {} and {} block parameters",
                stack_height, len_params
            )
        })
    }

    /// Translates a Wasm `block` control flow operator.
    pub fn translate_block(&mut self, block_type: BlockType) -> Result<(), ModuleError> {
        let stack_height = self.frame_stack_height(block_type);
        if self.is_reachable() {
            let end_label = self.inst_builder.new_label();
            self.control_frames.push_frame(BlockControlFrame::new(
                block_type,
                end_label,
                stack_height,
            ));
        } else {
            self.control_frames.push_frame(UnreachableControlFrame::new(
                ControlFrameKind::Block,
                block_type,
                stack_height,
            ));
        }
        Ok(())
    }

    /// Translates a Wasm `loop` control flow operator.
    pub fn translate_loop(&mut self, block_type: BlockType) -> Result<(), ModuleError> {
        let stack_height = self.frame_stack_height(block_type);
        if self.is_reachable() {
            let header = self.inst_builder.new_label();
            self.inst_builder.resolve_label(header);
            self.control_frames
                .push_frame(LoopControlFrame::new(block_type, header, stack_height));
        } else {
            self.control_frames.push_frame(UnreachableControlFrame::new(
                ControlFrameKind::Loop,
                block_type,
                stack_height,
            ));
        }
        Ok(())
    }

    /// Translates a Wasm `if` control flow operator.
    pub fn translate_if(&mut self, block_type: BlockType) -> Result<(), ModuleError> {
        if self.is_reachable() {
            let condition = self.value_stack.pop1();
            debug_assert_eq!(condition, ValueType::I32);
            let stack_height = self.frame_stack_height(block_type);
            let else_label = self.inst_builder.new_label();
            let end_label = self.inst_builder.new_label();
            self.control_frames.push_frame(IfControlFrame::new(
                block_type,
                end_label,
                else_label,
                stack_height,
            ));
            let dst_pc = self.try_resolve_label(else_label, |pc| Reloc::Br { inst_idx: pc });
            let branch_target = Target::new(dst_pc, DropKeep::new(0, 0));
            self.inst_builder
                .push_inst(Instruction::BrIfEqz(branch_target));
        } else {
            let stack_height = self.frame_stack_height(block_type);
            self.control_frames.push_frame(UnreachableControlFrame::new(
                ControlFrameKind::If,
                block_type,
                stack_height,
            ));
        }
        Ok(())
    }

    /// Translates a Wasm `else` control flow operator.
    pub fn translate_else(&mut self) -> Result<(), ModuleError> {
        let mut if_frame = match self.control_frames.pop_frame() {
            ControlFrame::If(if_frame) => if_frame,
            ControlFrame::Unreachable(frame) if matches!(frame.kind(), ControlFrameKind::If) => {
                // Encountered `Else` block for unreachable `If` block.
                //
                // In this case we can simply ignore the entire `Else` block
                // since it is unreachable anyways.
                self.control_frames.push_frame(frame);
                return Ok(());
            }
            unexpected => panic!(
                "expected `if` control flow frame on top for `else` but found: {:?}",
                unexpected,
            ),
        };
        let reachable = self.is_reachable();
        // At this point we know if the end of the `then` block of the paren
        // `if` block is reachable so we update the parent `if` frame.
        //
        // Note: This information is important to decide whether code is
        //       reachable after the `if` block (including `else`) ends.
        if_frame.update_end_of_then_reachability(reachable);
        // Create the jump from the end of the `then` block to the `if`
        // block's end label in case the end of `then` is reachable.
        if reachable {
            let dst_pc =
                self.try_resolve_label(if_frame.end_label(), |pc| Reloc::Br { inst_idx: pc });
            let target = Target::new(dst_pc, DropKeep::new(0, 0));
            self.inst_builder.push_inst(Instruction::Br(target));
        }
        // Now resolve labels for the instructions of the `else` block
        self.inst_builder.resolve_label(if_frame.else_label());
        // We need to reset the value stack to exactly how it has been
        // when entering the `if` in the first place so that the `else`
        // block has the same parameters on top of the stack.
        self.value_stack.shrink_to(if_frame.stack_height());
        if_frame.block_type().foreach_param(self.engine, |param| {
            self.value_stack.push(param);
        });
        self.control_frames.push_frame(if_frame);
        // We can reset reachability now since the parent `if` block was reachable.
        self.reachable = true;
        Ok(())
    }

    /// Translates a Wasm `end` control flow operator.
    pub fn translate_end(&mut self) -> Result<(), ModuleError> {
        let frame = self.control_frames.last();
        if let ControlFrame::If(if_frame) = &frame {
            // At this point we can resolve the `Else` label.
            //
            // Note: The `Else` label might have already been resolved
            //       in case there was an `Else` block.
            self.inst_builder
                .resolve_label_if_unresolved(if_frame.else_label());
        }
        if frame.is_reachable() && !matches!(frame.kind(), ControlFrameKind::Loop) {
            // At this point we can resolve the `End` labels.
            // Note that `loop` control frames do not have an `End` label.
            self.inst_builder.resolve_label(frame.end_label());
        }
        // These bindings are required because of borrowing issues.
        let frame_reachable = frame.is_reachable();
        let frame_stack_height = frame.stack_height();
        if self.control_frames.len() == 1 {
            // If the control flow frames stack is empty after this point
            // we know that we are endeding the function body `block`
            // frame and therefore we have to return from the function.
            self.translate_return()?;
        } else {
            // The following code is only reachable if the ended control flow
            // frame was reachable upon entering to begin with.
            self.reachable = frame_reachable;
        }
        self.value_stack.shrink_to(frame_stack_height);
        let frame = self.control_frames.pop_frame();
        frame
            .block_type()
            .foreach_result(self.engine, |result| self.value_stack.push(result));
        Ok(())
    }

    /// Translates a Wasm `br` control flow operator.
    pub fn translate_br(&mut self, relative_depth: u32) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            match builder.acquire_target(relative_depth) {
                AquiredTarget::Branch(end_label, drop_keep) => {
                    let dst_pc =
                        builder.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
                    builder
                        .inst_builder
                        .push_inst(Instruction::Br(Target::new(dst_pc, drop_keep)));
                }
                AquiredTarget::Return(drop_keep) => {
                    // In this case the `br` can be directly translated as `return`.
                    builder.translate_return()?;
                }
            }
            builder.reachable = false;
            Ok(())
        })
    }

    /// Translates a Wasm `br_if` control flow operator.
    pub fn translate_br_if(&mut self, relative_depth: u32) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let condition = builder.value_stack.pop1();
            debug_assert_eq!(condition, ValueType::I32);
            match builder.acquire_target(relative_depth) {
                AquiredTarget::Branch(end_label, drop_keep) => {
                    let dst_pc =
                        builder.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
                    builder
                        .inst_builder
                        .push_inst(Instruction::BrIfNez(Target::new(dst_pc, drop_keep)));
                }
                AquiredTarget::Return(drop_keep) => {
                    builder
                        .inst_builder
                        .push_inst(Instruction::ReturnIfNez(drop_keep));
                }
            }
            Ok(())
        })
    }

    /// Translates a Wasm `br_table` control flow operator.
    pub fn translate_br_table<T>(
        &mut self,
        default: RelativeDepth,
        targets: T,
    ) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = RelativeDepth>,
    {
        self.translate_if_reachable(|builder| {
            let case = builder.value_stack.pop1();
            debug_assert_eq!(case, ValueType::I32);

            fn compute_inst(
                builder: &mut FunctionBuilder,
                n: usize,
                depth: RelativeDepth,
            ) -> Instruction {
                match builder.acquire_target(depth.into_u32()) {
                    AquiredTarget::Branch(label_idx, drop_keep) => {
                        let dst_pc = builder.try_resolve_label(label_idx, |pc| Reloc::BrTable {
                            inst_idx: pc,
                            target_idx: n,
                        });
                        Instruction::Br(Target::new(dst_pc, drop_keep))
                    }
                    AquiredTarget::Return(drop_keep) => Instruction::Return(drop_keep),
                }
            }

            let branches = targets
                .into_iter()
                .enumerate()
                .map(|(n, depth)| compute_inst(builder, n, depth))
                .collect::<Vec<_>>();
            // We include the default target in `len_branches`.
            let len_branches = branches.len();
            let default_branch = compute_inst(builder, len_branches, default);
            builder.inst_builder.push_inst(Instruction::BrTable {
                len_targets: len_branches + 1,
            });
            for branch in branches {
                builder.inst_builder.push_inst(branch);
            }
            builder.inst_builder.push_inst(default_branch);
            builder.reachable = false;
            Ok(())
        })
    }

    /// Translates a Wasm `return` control flow operator.
    pub fn translate_return(&mut self) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let drop_keep = builder.drop_keep_return();
            builder
                .inst_builder
                .push_inst(Instruction::Return(drop_keep));
            builder.reachable = false;
            Ok(())
        })
    }

    /// Adjusts the emulated [`ValueStack`] given the [`FuncType`] of the call.
    fn adjust_value_stack_for_call(&mut self, func_type: &FuncType) {
        let (params, results) = func_type.params_results();
        for param in params.iter().rev() {
            let popped = self.value_stack.pop1();
            debug_assert_eq!(popped, *param);
        }
        for result in results {
            self.value_stack.push(*result);
        }
    }

    /// Translates a Wasm `call` instruction.
    pub fn translate_call(&mut self, func_idx: FuncIdx) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let func_type = builder.func_type_of(func_idx);
            builder.adjust_value_stack_for_call(&func_type);
            let func_idx = func_idx.into_u32().into();
            builder.inst_builder.push_inst(Instruction::Call(func_idx));
            Ok(())
        })
    }

    /// Translates a Wasm `call_indirect` instruction.
    pub fn translate_call_indirect(
        &mut self,
        func_type_idx: FuncTypeIdx,
        table_idx: TableIdx,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            /// The default Wasm MVP table index.
            const DEFAULT_TABLE_INDEX: u32 = 0;
            assert_eq!(table_idx.into_u32(), DEFAULT_TABLE_INDEX);
            let func_type_offset = builder.value_stack.pop1();
            debug_assert_eq!(func_type_offset, ValueType::I32);
            let func_type = builder.func_type_at(func_type_idx);
            builder.adjust_value_stack_for_call(&func_type);
            let func_type_idx = func_type_idx.into_u32().into();
            builder
                .inst_builder
                .push_inst(Instruction::CallIndirect(func_type_idx));
            Ok(())
        })
    }

    /// Translates a Wasm `drop` instruction.
    pub fn translate_drop(&mut self) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            builder.value_stack.pop1();
            builder.inst_builder.push_inst(Instruction::Drop);
            Ok(())
        })
    }

    /// Translates a Wasm `select` instruction.
    pub fn translate_select(&mut self) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let (v0, v1, selector) = builder.value_stack.pop3();
            debug_assert_eq!(selector, ValueType::I32);
            debug_assert_eq!(v0, v1);
            builder.value_stack.push(v0);
            builder.inst_builder.push_inst(Instruction::Select);
            Ok(())
        })
    }

    /// Translate a Wasm `local.get` instruction.
    pub fn translate_local_get(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let local_depth = builder.relative_local_depth(local_idx);
            builder
                .inst_builder
                .push_inst(Instruction::local_get(local_depth));
            let value_type = builder
                .locals
                .resolve_local(local_idx)
                .unwrap_or_else(|| panic!("failed to resolve local {}", local_idx));
            builder.value_stack.push(value_type);
            Ok(())
        })
    }

    /// Translate a Wasm `local.set` instruction.
    pub fn translate_local_set(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let actual = builder.value_stack.pop1();
            let local_depth = builder.relative_local_depth(local_idx);
            builder
                .inst_builder
                .push_inst(Instruction::local_set(local_depth));
            let expected = builder
                .locals
                .resolve_local(local_idx)
                .unwrap_or_else(|| panic!("failed to resolve local {}", local_idx));
            debug_assert_eq!(actual, expected);
            Ok(())
        })
    }

    /// Translate a Wasm `local.tee` instruction.
    pub fn translate_local_tee(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let local_depth = builder.relative_local_depth(local_idx);
            builder
                .inst_builder
                .push_inst(Instruction::local_tee(local_depth));
            let expected = builder
                .locals
                .resolve_local(local_idx)
                .unwrap_or_else(|| panic!("failed to resolve local {}", local_idx));
            let actual = builder.value_stack.top();
            debug_assert_eq!(actual, expected);
            Ok(())
        })
    }

    /// Translate a Wasm `global.get` instruction.
    pub fn translate_global_get(&mut self, global_idx: GlobalIdx) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let global_type = builder.res.get_type_of_global(global_idx);
            builder.value_stack.push(global_type.value_type());
            let global_idx = global_idx.into_u32().into();
            builder
                .inst_builder
                .push_inst(Instruction::GetGlobal(global_idx));
            Ok(())
        })
    }

    /// Translate a Wasm `global.set` instruction.
    pub fn translate_global_set(&mut self, global_idx: GlobalIdx) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let global_type = builder.res.get_type_of_global(global_idx);
            debug_assert_eq!(global_type.mutability(), Mutability::Mutable);
            let expected = global_type.value_type();
            let actual = builder.value_stack.pop1();
            debug_assert_eq!(actual, expected);
            let global_idx = global_idx.into_u32().into();
            builder
                .inst_builder
                .push_inst(Instruction::SetGlobal(global_idx));
            Ok(())
        })
    }

    /// Translate a Wasm `<ty>.load` instruction.
    ///
    /// # Note
    ///
    /// This is used as the translation backend of the following Wasm instructions:
    ///
    /// - `i32.load`
    /// - `i64.load`
    /// - `f32.load`
    /// - `f64.load`
    /// - `i32.load_i8`
    /// - `i32.load_u8`
    /// - `i32.load_i16`
    /// - `i32.load_u16`
    /// - `i64.load_i8`
    /// - `i64.load_u8`
    /// - `i64.load_i16`
    /// - `i64.load_u16`
    /// - `i64.load_i32`
    /// - `i64.load_u32`
    fn translate_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
        loaded_type: ValueType,
        make_inst: fn(Offset) -> Instruction,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
            let pointer = builder.value_stack.pop1();
            debug_assert_eq!(pointer, ValueType::I32);
            builder.value_stack.push(loaded_type);
            let offset = Offset::from(offset);
            builder.inst_builder.push_inst(make_inst(offset));
            Ok(())
        })
    }

    /// Translate a Wasm `i32.load` instruction.
    pub fn translate_i32_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I32, Instruction::I32Load)
    }

    /// Translate a Wasm `i64.load` instruction.
    pub fn translate_i64_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I64, Instruction::I64Load)
    }

    /// Translate a Wasm `f32.load` instruction.
    pub fn translate_f32_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::F32, Instruction::F32Load)
    }

    /// Translate a Wasm `f64.load` instruction.
    pub fn translate_f64_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::F64, Instruction::F64Load)
    }

    /// Translate a Wasm `i32.load_i8` instruction.
    pub fn translate_i32_load_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I32, Instruction::I32Load8S)
    }

    /// Translate a Wasm `i32.load_u8` instruction.
    pub fn translate_i32_load_u8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I32, Instruction::I32Load8U)
    }

    /// Translate a Wasm `i32.load_i16` instruction.
    pub fn translate_i32_load_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I32, Instruction::I32Load16S)
    }

    /// Translate a Wasm `i32.load_u16` instruction.
    pub fn translate_i32_load_u16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I32, Instruction::I32Load16U)
    }

    /// Translate a Wasm `i64.load_i8` instruction.
    pub fn translate_i64_load_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I64, Instruction::I64Load8S)
    }

    /// Translate a Wasm `i64.load_u8` instruction.
    pub fn translate_i64_load_u8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I64, Instruction::I64Load8U)
    }

    /// Translate a Wasm `i64.load_i16` instruction.
    pub fn translate_i64_load_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I64, Instruction::I64Load16S)
    }

    /// Translate a Wasm `i64.load_u16` instruction.
    pub fn translate_i64_load_u16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I64, Instruction::I64Load16U)
    }

    /// Translate a Wasm `i64.load_i32` instruction.
    pub fn translate_i64_load_i32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I64, Instruction::I64Load32S)
    }

    /// Translate a Wasm `i64.load_u32` instruction.
    pub fn translate_i64_load_u32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, ValueType::I64, Instruction::I64Load32U)
    }

    /// Translate a Wasm `<ty>.store` instruction.
    ///
    /// # Note
    ///
    /// This is used as the translation backend of the following Wasm instructions:
    ///
    /// - `i32.store`
    /// - `i64.store`
    /// - `f32.store`
    /// - `f64.store`
    /// - `i32.store_i8`
    /// - `i32.store_i16`
    /// - `i64.store_i8`
    /// - `i64.store_i16`
    /// - `i64.store_i32`
    fn translate_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
        stored_value: ValueType,
        make_inst: fn(Offset) -> Instruction,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
            let (pointer, stored) = builder.value_stack.pop2();
            debug_assert_eq!(pointer, ValueType::I32);
            assert_eq!(stored_value, stored);
            let offset = Offset::from(offset);
            builder.inst_builder.push_inst(make_inst(offset));
            Ok(())
        })
    }

    /// Translate a Wasm `i32.store` instruction.
    pub fn translate_i32_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, ValueType::I32, Instruction::I32Store)
    }

    /// Translate a Wasm `i64.store` instruction.
    pub fn translate_i64_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, ValueType::I64, Instruction::I64Store)
    }

    /// Translate a Wasm `f32.store` instruction.
    pub fn translate_f32_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, ValueType::F32, Instruction::F32Store)
    }

    /// Translate a Wasm `f64.store` instruction.
    pub fn translate_f64_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, ValueType::F64, Instruction::F64Store)
    }

    /// Translate a Wasm `i32.store_i8` instruction.
    pub fn translate_i32_store_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, ValueType::I32, Instruction::I32Store8)
    }

    /// Translate a Wasm `i32.store_i16` instruction.
    pub fn translate_i32_store_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, ValueType::I32, Instruction::I32Store16)
    }

    /// Translate a Wasm `i64.store_i8` instruction.
    pub fn translate_i64_store_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, ValueType::I64, Instruction::I64Store8)
    }

    /// Translate a Wasm `i64.store_i16` instruction.
    pub fn translate_i64_store_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, ValueType::I64, Instruction::I64Store16)
    }

    /// Translate a Wasm `i64.store_i32` instruction.
    pub fn translate_i64_store_i32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, ValueType::I64, Instruction::I64Store32)
    }

    /// Translate a Wasm `memory.size` instruction.
    pub fn translate_memory_size(&mut self, memory_idx: MemoryIdx) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
            builder.value_stack.push(ValueType::I32);
            builder.inst_builder.push_inst(Instruction::CurrentMemory);
            Ok(())
        })
    }

    /// Translate a Wasm `memory.grow` instruction.
    pub fn translate_memory_grow(&mut self, memory_idx: MemoryIdx) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
            debug_assert_eq!(builder.value_stack.top(), ValueType::I32);
            builder.inst_builder.push_inst(Instruction::GrowMemory);
            Ok(())
        })
    }

    /// Translate a Wasm `<ty>.const` instruction.
    ///
    /// # Note
    ///
    /// This is used as the translation backend of the following Wasm instructions:
    ///
    /// - `i32.const`
    /// - `i64.const`
    /// - `f32.const`
    /// - `f64.const`
    fn translate_const<T>(&mut self, value: T) -> Result<(), ModuleError>
    where
        T: Into<Value>,
    {
        self.translate_if_reachable(|builder| {
            let value = value.into();
            builder.value_stack.push(value.value_type());
            builder.inst_builder.push_inst(Instruction::constant(value));
            Ok(())
        })
    }

    /// Translate a Wasm `i32.const` instruction.
    pub fn translate_i32_const(&mut self, value: i32) -> Result<(), ModuleError> {
        self.translate_const(value)
    }

    /// Translate a Wasm `i64.const` instruction.
    pub fn translate_i64_const(&mut self, value: i64) -> Result<(), ModuleError> {
        self.translate_const(value)
    }

    /// Translate a Wasm `f32.const` instruction.
    pub fn translate_f32_const(&mut self, value: F32) -> Result<(), ModuleError> {
        self.translate_const(value)
    }

    /// Translate a Wasm `f64.const` instruction.
    pub fn translate_f64_const(&mut self, value: F64) -> Result<(), ModuleError> {
        self.translate_const(value)
    }

    /// Translate a Wasm unary comparison instruction.
    ///
    /// # Note
    ///
    /// This is used to translate the following Wasm instructions:
    ///
    /// - `i32.eqz`
    /// - `i64.eqz`
    fn translate_unary_cmp(
        &mut self,
        input_type: ValueType,
        inst: Instruction,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let condition = builder.value_stack.pop1();
            debug_assert_eq!(condition, input_type);
            builder.value_stack.push(ValueType::I32);
            builder.inst_builder.push_inst(inst);
            Ok(())
        })
    }

    /// Translate a Wasm `i32.eqz` instruction.
    pub fn translate_i32_eqz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_cmp(ValueType::I32, Instruction::I32Eqz)
    }

    /// Translate a Wasm binary comparison instruction.
    ///
    /// # Note
    ///
    /// This is used to translate the following Wasm instructions:
    ///
    /// - `{i32, i64, f32, f64}.eq`
    /// - `{i32, i64, f32, f64}.ne`
    /// - `{i32, u32, i64, u64, f32, f64}.lt`
    /// - `{i32, u32, i64, u64, f32, f64}.le`
    /// - `{i32, u32, i64, u64, f32, f64}.gt`
    /// - `{i32, u32, i64, u64, f32, f64}.ge`
    fn translate_binary_cmp(
        &mut self,
        input_type: ValueType,
        inst: Instruction,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let (v0, v1) = builder.value_stack.pop2();
            debug_assert_eq!(v0, v1);
            debug_assert_eq!(v0, input_type);
            builder.value_stack.push(ValueType::I32);
            builder.inst_builder.push_inst(inst);
            Ok(())
        })
    }

    /// Translate a Wasm `i32.eq` instruction.
    pub fn translate_i32_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32Eq)
    }

    /// Translate a Wasm `i32.ne` instruction.
    pub fn translate_i32_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32Ne)
    }

    /// Translate a Wasm `i32.lt` instruction.
    pub fn translate_i32_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32LtS)
    }

    /// Translate a Wasm `u32.lt` instruction.
    pub fn translate_u32_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32LtU)
    }

    /// Translate a Wasm `i32.gt` instruction.
    pub fn translate_i32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32GtS)
    }

    /// Translate a Wasm `u32.gt` instruction.
    pub fn translate_u32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32GtU)
    }

    /// Translate a Wasm `i32.le` instruction.
    pub fn translate_i32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32LeS)
    }

    /// Translate a Wasm `u32.le` instruction.
    pub fn translate_u32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32LeU)
    }

    /// Translate a Wasm `i32.ge` instruction.
    pub fn translate_i32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32GeS)
    }

    /// Translate a Wasm `u32.ge` instruction.
    pub fn translate_u32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32GeU)
    }

    /// Translate a Wasm `i64.eqz` instruction.
    pub fn translate_i64_eqz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_cmp(ValueType::I64, Instruction::I64Eqz)
    }

    /// Translate a Wasm `i64.eq` instruction.
    pub fn translate_i64_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64Eq)
    }

    /// Translate a Wasm `i64.ne` instruction.
    pub fn translate_i64_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64Ne)
    }

    /// Translate a Wasm `i64.lt` instruction.
    pub fn translate_i64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64LtS)
    }

    /// Translate a Wasm `u64.lt` instruction.
    pub fn translate_u64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64LtU)
    }

    /// Translate a Wasm `i64.gt` instruction.
    pub fn translate_i64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64GtS)
    }

    /// Translate a Wasm `u64.gt` instruction.
    pub fn translate_u64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64GtU)
    }

    /// Translate a Wasm `i64.le` instruction.
    pub fn translate_i64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64LeS)
    }

    /// Translate a Wasm `u64.le` instruction.
    pub fn translate_u64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64LeU)
    }

    /// Translate a Wasm `i64.ge` instruction.
    pub fn translate_i64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64GeS)
    }

    /// Translate a Wasm `u64.ge` instruction.
    pub fn translate_u64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64GeU)
    }

    /// Translate a Wasm `f32.eq` instruction.
    pub fn translate_f32_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Eq)
    }

    /// Translate a Wasm `f32.ne` instruction.
    pub fn translate_f32_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Ne)
    }

    /// Translate a Wasm `f32.lt` instruction.
    pub fn translate_f32_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Lt)
    }

    /// Translate a Wasm `f32.gt` instruction.
    pub fn translate_f32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Gt)
    }

    /// Translate a Wasm `f32.le` instruction.
    pub fn translate_f32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Le)
    }

    /// Translate a Wasm `f32.ge` instruction.
    pub fn translate_f32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Ge)
    }

    /// Translate a Wasm `f64.eq` instruction.
    pub fn translate_f64_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F64, Instruction::F64Eq)
    }

    /// Translate a Wasm `f64.ne` instruction.
    pub fn translate_f64_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F64, Instruction::F64Ne)
    }

    /// Translate a Wasm `f64.lt` instruction.
    pub fn translate_f64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F64, Instruction::F64Lt)
    }

    /// Translate a Wasm `f64.gt` instruction.
    pub fn translate_f64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F64, Instruction::F64Gt)
    }

    /// Translate a Wasm `f64.le` instruction.
    pub fn translate_f64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F64, Instruction::F64Le)
    }

    /// Translate a Wasm `f64.ge` instruction.
    pub fn translate_f64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(ValueType::F64, Instruction::F64Ge)
    }

    /// Translate a unary Wasm instruction.
    ///
    /// # Note
    ///
    /// This is used to translate the following Wasm instructions:
    ///
    /// - `i32.clz`
    /// - `i32.ctz`
    /// - `i32.popcnt`
    /// - `{f32, f64}.abs`
    /// - `{f32, f64}.neg`
    /// - `{f32, f64}.ceil`
    /// - `{f32, f64}.floor`
    /// - `{f32, f64}.trunc`
    /// - `{f32, f64}.nearest`
    /// - `{f32, f64}.sqrt`
    pub fn translate_unary_operation(
        &mut self,
        value_type: ValueType,
        inst: Instruction,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let actual_type = builder.value_stack.top();
            debug_assert_eq!(actual_type, value_type);
            builder.inst_builder.push_inst(inst);
            Ok(())
        })
    }

    /// Translate a Wasm `i32.clz` instruction.
    pub fn translate_i32_clz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::I32, Instruction::I32Clz)
    }

    /// Translate a Wasm `i32.ctz` instruction.
    pub fn translate_i32_ctz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::I32, Instruction::I32Ctz)
    }

    /// Translate a Wasm `i32.popcnt` instruction.
    pub fn translate_i32_popcnt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::I32, Instruction::I32Popcnt)
    }

    /// Translate a binary Wasm instruction.
    ///
    /// - `{i32, i64}.add`
    /// - `{i32, i64}.sub`
    /// - `{i32, i64}.mul`
    /// - `{i32, u32, i64, u64}.div`
    /// - `{i32, u32, i64, u64}.rem`
    /// - `{i32, i64}.and`
    /// - `{i32, i64}.or`
    /// - `{i32, i64}.xor`
    /// - `{i32, i64}.shl`
    /// - `{i32, u32, i64, u64}.shr`
    /// - `{i32, i64}.rotl`
    /// - `{i32, i64}.rotr`
    /// - `{f32, f64}.add`
    /// - `{f32, f64}.sub`
    /// - `{f32, f64}.mul`
    /// - `{f32, f64}.div`
    /// - `{f32, f64}.min`
    /// - `{f32, f64}.max`
    /// - `{f32, f64}.copysign`
    pub fn translate_binary_operation(
        &mut self,
        value_type: ValueType,
        inst: Instruction,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let (v0, v1) = builder.value_stack.pop2();
            debug_assert_eq!(v0, v1);
            debug_assert_eq!(v0, value_type);
            builder.value_stack.push(value_type);
            builder.inst_builder.push_inst(inst);
            Ok(())
        })
    }

    /// Translate a Wasm `i32.add` instruction.
    pub fn translate_i32_add(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Add)
    }

    /// Translate a Wasm `i32.sub` instruction.
    pub fn translate_i32_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Sub)
    }

    /// Translate a Wasm `i32.mul` instruction.
    pub fn translate_i32_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Mul)
    }

    /// Translate a Wasm `i32.div` instruction.
    pub fn translate_i32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32DivS)
    }

    /// Translate a Wasm `u32.div` instruction.
    pub fn translate_u32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32DivU)
    }

    /// Translate a Wasm `i32.rem` instruction.
    pub fn translate_i32_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32RemS)
    }

    /// Translate a Wasm `u32.rem` instruction.
    pub fn translate_u32_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32RemU)
    }

    /// Translate a Wasm `i32.and` instruction.
    pub fn translate_i32_and(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32And)
    }

    /// Translate a Wasm `i32.or` instruction.
    pub fn translate_i32_or(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Or)
    }

    /// Translate a Wasm `i32.xor` instruction.
    pub fn translate_i32_xor(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Xor)
    }

    /// Translate a Wasm `i32.shl` instruction.
    pub fn translate_i32_shl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Shl)
    }

    /// Translate a Wasm `i32.shr` instruction.
    pub fn translate_i32_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32ShrS)
    }

    /// Translate a Wasm `u32.shr` instruction.
    pub fn translate_u32_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32ShrU)
    }

    /// Translate a Wasm `i32.rotl` instruction.
    pub fn translate_i32_rotl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Rotl)
    }

    /// Translate a Wasm `i32.rotr` instruction.
    pub fn translate_i32_rotr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Rotr)
    }

    /// Translate a Wasm `i64.clz` instruction.
    pub fn translate_i64_clz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Clz)
    }

    /// Translate a Wasm `i64.ctz` instruction.
    pub fn translate_i64_ctz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Ctz)
    }

    /// Translate a Wasm `i64.popcnt` instruction.
    pub fn translate_i64_popcnt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Popcnt)
    }

    /// Translate a Wasm `i64.add` instruction.
    pub fn translate_i64_add(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Add)
    }

    /// Translate a Wasm `i64.sub` instruction.
    pub fn translate_i64_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Sub)
    }

    /// Translate a Wasm `i64.mul` instruction.
    pub fn translate_i64_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Mul)
    }

    /// Translate a Wasm `i64.div` instruction.
    pub fn translate_i64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64DivS)
    }

    /// Translate a Wasm `u64.div` instruction.
    pub fn translate_u64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64DivU)
    }

    /// Translate a Wasm `i64.rem` instruction.
    pub fn translate_i64_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64RemS)
    }

    /// Translate a Wasm `u64.rem` instruction.
    pub fn translate_u64_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64RemU)
    }

    /// Translate a Wasm `i64.and` instruction.
    pub fn translate_i64_and(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64And)
    }

    /// Translate a Wasm `i64.or` instruction.
    pub fn translate_i64_or(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Or)
    }

    /// Translate a Wasm `i64.xor` instruction.
    pub fn translate_i64_xor(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Xor)
    }

    /// Translate a Wasm `i64.shl` instruction.
    pub fn translate_i64_shl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Shl)
    }

    /// Translate a Wasm `i64.shr` instruction.
    pub fn translate_i64_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64ShrS)
    }

    /// Translate a Wasm `u64.shr` instruction.
    pub fn translate_u64_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64ShrU)
    }

    /// Translate a Wasm `i64.rotl` instruction.
    pub fn translate_i64_rotl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Rotl)
    }

    /// Translate a Wasm `i64.rotr` instruction.
    pub fn translate_i64_rotr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Rotr)
    }

    /// Translate a Wasm `f32.abs` instruction.
    pub fn translate_f32_abs(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Abs)
    }

    /// Translate a Wasm `f32.neg` instruction.
    pub fn translate_f32_neg(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Neg)
    }

    /// Translate a Wasm `f32.ceil` instruction.
    pub fn translate_f32_ceil(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Ceil)
    }

    /// Translate a Wasm `f32.floor` instruction.
    pub fn translate_f32_floor(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Floor)
    }

    /// Translate a Wasm `f32.trunc` instruction.
    pub fn translate_f32_trunc(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Trunc)
    }

    /// Translate a Wasm `f32.nearest` instruction.
    pub fn translate_f32_nearest(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Nearest)
    }

    /// Translate a Wasm `f32.sqrt` instruction.
    pub fn translate_f32_sqrt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Sqrt)
    }

    /// Translate a Wasm `f32.add` instruction.
    pub fn translate_f32_add(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Add)
    }

    /// Translate a Wasm `f32.sub` instruction.
    pub fn translate_f32_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Sub)
    }

    /// Translate a Wasm `f32.mul` instruction.
    pub fn translate_f32_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Mul)
    }

    /// Translate a Wasm `f32.div` instruction.
    pub fn translate_f32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Div)
    }

    /// Translate a Wasm `f32.min` instruction.
    pub fn translate_f32_min(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Min)
    }

    /// Translate a Wasm `f32.max` instruction.
    pub fn translate_f32_max(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Max)
    }

    /// Translate a Wasm `f32.copysign` instruction.
    pub fn translate_f32_copysign(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Copysign)
    }

    /// Translate a Wasm `f64.abs` instruction.
    pub fn translate_f64_abs(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Abs)
    }

    /// Translate a Wasm `f64.neg` instruction.
    pub fn translate_f64_neg(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Neg)
    }

    /// Translate a Wasm `f64.ceil` instruction.
    pub fn translate_f64_ceil(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Ceil)
    }

    /// Translate a Wasm `f64.floor` instruction.
    pub fn translate_f64_floor(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Floor)
    }

    /// Translate a Wasm `f64.trunc` instruction.
    pub fn translate_f64_trunc(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Trunc)
    }

    /// Translate a Wasm `f64.nearest` instruction.
    pub fn translate_f64_nearest(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Nearest)
    }

    /// Translate a Wasm `f64.sqrt` instruction.
    pub fn translate_f64_sqrt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Sqrt)
    }

    /// Translate a Wasm `f64.add` instruction.
    pub fn translate_f64_add(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Add)
    }

    /// Translate a Wasm `f64.sub` instruction.
    pub fn translate_f64_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Sub)
    }

    /// Translate a Wasm `f64.mul` instruction.
    pub fn translate_f64_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Mul)
    }

    /// Translate a Wasm `f64.div` instruction.
    pub fn translate_f64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Div)
    }

    /// Translate a Wasm `f64.min` instruction.
    pub fn translate_f64_min(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Min)
    }

    /// Translate a Wasm `f64.max` instruction.
    pub fn translate_f64_max(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Max)
    }

    /// Translate a Wasm `f64.copysign` instruction.
    pub fn translate_f64_copysign(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Copysign)
    }

    /// Translate a Wasm conversion instruction.
    ///
    /// - `i32.wrap_i64`
    /// - `{i32, u32}.trunc_f32
    /// - `{i32, u32}.trunc_f64`
    /// - `{i64, u64}.extend_i32`
    /// - `{i64, u64}.trunc_f32`
    /// - `{i64, u64}.trunc_f64`
    /// - `f32.convert_{i32, u32, i64, u64}`
    /// - `f32.demote_f64`
    /// - `f64.convert_{i32, u32, i64, u64}`
    /// - `f64.promote_f32`
    /// - `i32.reinterpret_f32`
    /// - `i64.reinterpret_f64`
    /// - `f32.reinterpret_i32`
    /// - `f64.reinterpret_i64`
    pub fn translate_conversion(
        &mut self,
        input_type: ValueType,
        output_type: ValueType,
        inst: Instruction,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let input = builder.value_stack.pop1();
            debug_assert_eq!(input, input_type);
            builder.value_stack.push(output_type);
            builder.inst_builder.push_inst(inst);
            Ok(())
        })
    }

    /// Translate a Wasm `i32.wrap_i64` instruction.
    pub fn translate_i32_wrap_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::I64, ValueType::I32, Instruction::I32WrapI64)
    }

    /// Translate a Wasm `i32.trunc_f32` instruction.
    pub fn translate_i32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F32, ValueType::I32, Instruction::I32TruncSF32)
    }

    /// Translate a Wasm `u32.trunc_f32` instruction.
    pub fn translate_u32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F32, ValueType::I32, Instruction::I32TruncUF32)
    }

    /// Translate a Wasm `i32.trunc_f64` instruction.
    pub fn translate_i32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F64, ValueType::I32, Instruction::I32TruncSF64)
    }

    /// Translate a Wasm `u32.trunc_f64` instruction.
    pub fn translate_u32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F64, ValueType::I32, Instruction::I32TruncUF64)
    }

    /// Translate a Wasm `i64.extend_i32` instruction.
    pub fn translate_i64_extend_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::I32, ValueType::I64, Instruction::I64ExtendSI32)
    }

    /// Translate a Wasm `u64.extend_i32` instruction.
    pub fn translate_u64_extend_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::I32, ValueType::I64, Instruction::I64ExtendUI32)
    }

    /// Translate a Wasm `i64.trunc_f32` instruction.
    pub fn translate_i64_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F32, ValueType::I64, Instruction::I64TruncSF32)
    }

    /// Translate a Wasm `u64.trunc_f32` instruction.
    pub fn translate_u64_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F32, ValueType::I64, Instruction::I64TruncUF32)
    }

    /// Translate a Wasm `i64.trunc_f64` instruction.
    pub fn translate_i64_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F64, ValueType::I64, Instruction::I64TruncSF64)
    }

    /// Translate a Wasm `u64.trunc_f64` instruction.
    pub fn translate_u64_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F64, ValueType::I64, Instruction::I64TruncUF64)
    }

    /// Translate a Wasm `f32.convert_i32` instruction.
    pub fn translate_f32_convert_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::I32, ValueType::F32, Instruction::F32ConvertSI32)
    }

    /// Translate a Wasm `f32.convert_u32` instruction.
    pub fn translate_f32_convert_u32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::I32, ValueType::F32, Instruction::F32ConvertUI32)
    }

    /// Translate a Wasm `f32.convert_i64` instruction.
    pub fn translate_f32_convert_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::I64, ValueType::F32, Instruction::F32ConvertSI64)
    }

    /// Translate a Wasm `f32.convert_u64` instruction.
    pub fn translate_f32_convert_u64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::I64, ValueType::F32, Instruction::F32ConvertUI64)
    }

    /// Translate a Wasm `f32.demote_f64` instruction.
    pub fn translate_f32_demote_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F64, ValueType::F32, Instruction::F32DemoteF64)
    }

    /// Translate a Wasm `f64.convert_i32` instruction.
    pub fn translate_f64_convert_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::I32, ValueType::F64, Instruction::F64ConvertSI32)
    }

    /// Translate a Wasm `f64.convert_u32` instruction.
    pub fn translate_f64_convert_u32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::I32, ValueType::F64, Instruction::F64ConvertUI32)
    }

    /// Translate a Wasm `f64.convert_i64` instruction.
    pub fn translate_f64_convert_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::I64, ValueType::F64, Instruction::F64ConvertSI64)
    }

    /// Translate a Wasm `f64.convert_u64` instruction.
    pub fn translate_f64_convert_u64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::I64, ValueType::F64, Instruction::F64ConvertUI64)
    }

    /// Translate a Wasm `f64.promote_f32` instruction.
    pub fn translate_f64_promote_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F32, ValueType::F64, Instruction::F64PromoteF32)
    }

    /// Translate a Wasm `i32.reinterpret_f32` instruction.
    pub fn translate_i32_reinterpret_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F32,
            ValueType::I32,
            Instruction::I32ReinterpretF32,
        )
    }

    /// Translate a Wasm `i64.reinterpret_f64` instruction.
    pub fn translate_i64_reinterpret_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F64,
            ValueType::I64,
            Instruction::I64ReinterpretF64,
        )
    }

    /// Translate a Wasm `f32.reinterpret_i32` instruction.
    pub fn translate_f32_reinterpret_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I32,
            ValueType::F32,
            Instruction::F32ReinterpretI32,
        )
    }

    /// Translate a Wasm `f64.reinterpret_i64` instruction.
    pub fn translate_f64_reinterpret_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I64,
            ValueType::F64,
            Instruction::F64ReinterpretI64,
        )
    }

    /// Translate a Wasm `i32.extend_8s` instruction.
    pub fn translate_i32_sign_extend8(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::I32, Instruction::I32Extend8S)
    }

    /// Translate a Wasm `i32.extend_16s` instruction.
    pub fn translate_i32_sign_extend16(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::I32, Instruction::I32Extend16S)
    }

    /// Translate a Wasm `i64.extend_8s` instruction.
    pub fn translate_i64_sign_extend8(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Extend8S)
    }

    /// Translate a Wasm `i64.extend_16s` instruction.
    pub fn translate_i64_sign_extend16(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Extend16S)
    }

    /// Translate a Wasm `i64.extend_32s` instruction.
    pub fn translate_i64_sign_extend32(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Extend32S)
    }

    /// Translate a Wasm `i32.truncate_sat_f32` instruction.
    pub fn translate_i32_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F32, ValueType::I32, Instruction::I32TruncSatF32S)
    }

    /// Translate a Wasm `u32.truncate_sat_f32` instruction.
    pub fn translate_u32_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F32, ValueType::I32, Instruction::I32TruncSatF32U)
    }

    /// Translate a Wasm `i32.truncate_sat_f64` instruction.
    pub fn translate_i32_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F64, ValueType::I32, Instruction::I32TruncSatF64S)
    }

    /// Translate a Wasm `u32.truncate_sat_f64` instruction.
    pub fn translate_u32_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F64, ValueType::I32, Instruction::I32TruncSatF64U)
    }

    /// Translate a Wasm `i64.truncate_sat_f32` instruction.
    pub fn translate_i64_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F32, ValueType::I64, Instruction::I64TruncSatF32S)
    }

    /// Translate a Wasm `u64.truncate_sat_f32` instruction.
    pub fn translate_u64_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F32, ValueType::I32, Instruction::I64TruncSatF32U)
    }

    /// Translate a Wasm `i64.truncate_sat_f64` instruction.
    pub fn translate_i64_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F64, ValueType::I64, Instruction::I64TruncSatF64S)
    }

    /// Translate a Wasm `u64.truncate_sat_f64` instruction.
    pub fn translate_u64_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(ValueType::F64, ValueType::I32, Instruction::I64TruncSatF64U)
    }
}
