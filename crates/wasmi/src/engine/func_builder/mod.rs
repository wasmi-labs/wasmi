mod control_frame;
mod control_stack;
mod error;
mod inst_builder;
mod labels;
mod locals_registry;
mod value_stack;
mod visit;

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
    labels::LabelRef,
    locals_registry::LocalsRegistry,
    value_stack::ValueStackHeight,
};
pub use self::{
    error::TranslationError,
    inst_builder::{Instr, InstructionsBuilder, RelativeDepth},
};
use super::{DropKeep, FuncBody, Instruction};
use crate::{
    engine::bytecode::{
        BranchParams,
        DataSegmentIdx,
        ElementSegmentIdx,
        Offset,
        SignatureIdx,
        TableIdx,
    },
    module::{
        BlockType,
        FuncIdx,
        FuncTypeIdx,
        GlobalIdx,
        InitExpr,
        MemoryIdx,
        ModuleResources,
        ReusableAllocations,
        DEFAULT_MEMORY_INDEX,
    },
    Engine,
    FuncType,
    Mutability,
    Value,
};
use alloc::vec::Vec;
use wasmi_core::{ValueType, F32, F64};

/// The used function validator type.
type FuncValidator = wasmparser::FuncValidator<wasmparser::ValidatorResources>;

/// The interface to translate a `wasmi` bytecode function using Wasm bytecode.
pub struct FuncBuilder<'parser> {
    /// The [`Engine`] for which the function is translated.
    engine: Engine,
    /// The function under construction.
    func: FuncIdx,
    /// The immutable `wasmi` module resources.
    res: ModuleResources<'parser>,
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
    /// The Wasm function validator.
    validator: FuncValidator,
    /// The height of the emulated value stack.
    stack_height: ValueStackHeight,
    /// Stores and resolves local variable types.
    locals: LocalsRegistry,
    /// The reusable data structures of the [`FuncBuilder`].
    alloc: FunctionBuilderAllocations,
}

/// Reusable allocations of a [`FuncBuilder`].
#[derive(Debug, Default)]
pub struct FunctionBuilderAllocations {
    /// The control flow frame stack that represents the Wasm control flow.
    control_frames: ControlFlowStack,
    /// The instruction builder.
    ///
    /// # Note
    ///
    /// Allows to incrementally construct the instruction of a function.
    inst_builder: InstructionsBuilder,
    /// Buffer for translating `br_table`.
    br_table_branches: Vec<Instruction>,
}

impl FunctionBuilderAllocations {
    /// Resets the data structures of the [`FunctionBuilderAllocations`].
    ///
    /// # Note
    ///
    /// This must be called before reusing this [`FunctionBuilderAllocations`]
    /// by another [`FuncBuilder`].
    fn reset(&mut self) {
        self.control_frames.reset();
        self.inst_builder.reset();
        self.br_table_branches.clear();
    }
}

impl<'parser> FuncBuilder<'parser> {
    /// Creates a new [`FuncBuilder`].
    pub fn new(
        engine: &Engine,
        func: FuncIdx,
        res: ModuleResources<'parser>,
        validator: FuncValidator,
        mut allocations: FunctionBuilderAllocations,
    ) -> Self {
        let mut locals = LocalsRegistry::default();
        Self::register_func_body_block(func, res, &mut allocations);
        Self::register_func_params(func, res, &mut locals);
        Self {
            engine: engine.clone(),
            func,
            res,
            reachable: true,
            validator,
            stack_height: ValueStackHeight::default(),
            locals,
            alloc: allocations,
        }
    }

    /// Returns the [`FuncType`] of the function that is currently translated.
    fn func_type(&self) -> FuncType {
        let dedup_func_type = self.res.get_type_of_func(self.func);
        self.engine.resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Resolves the [`FuncType`] of the given [`FuncTypeIdx`].
    fn func_type_at(&self, func_type_index: SignatureIdx) -> FuncType {
        let func_type_index = FuncTypeIdx(func_type_index.into_inner()); // TODO: use the same type
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
        allocations: &mut FunctionBuilderAllocations,
    ) {
        allocations.reset();
        let func_type = res.get_type_of_func(func);
        let block_type = BlockType::func_type(func_type);
        let end_label = allocations.inst_builder.new_label();
        let block_frame = BlockControlFrame::new(block_type, end_label, 0);
        allocations.control_frames.push_frame(block_frame);
    }

    /// Registers the function parameters in the emulated value stack.
    fn register_func_params(
        func: FuncIdx,
        res: ModuleResources<'parser>,
        locals: &mut LocalsRegistry,
    ) -> usize {
        let dedup_func_type = res.get_type_of_func(func);
        let func_type = res
            .engine()
            .resolve_func_type(dedup_func_type, Clone::clone);
        let params = func_type.params();
        for _param_type in params {
            locals.register_locals(1);
        }
        params.len()
    }

    /// Translates the given local variables for the translated function.
    pub fn translate_locals(
        &mut self,
        offset: usize,
        amount: u32,
        value_type: wasmparser::ValType,
    ) -> Result<(), TranslationError> {
        self.validator.define_locals(offset, amount, value_type)?;
        self.locals.register_locals(amount);
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
    pub fn finish(
        mut self,
        offset: usize,
    ) -> Result<(FuncBody, ReusableAllocations), TranslationError> {
        self.validator.finish(offset)?;
        let func_body = self.alloc.inst_builder.finish(
            &self.engine,
            self.len_locals(),
            self.stack_height.max_stack_height() as usize,
        );
        let allocations = ReusableAllocations {
            translation: self.alloc,
            validation: self.validator.into_allocations(),
        };
        Ok((func_body, allocations))
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
    fn translate_if_reachable<F>(&mut self, translator: F) -> Result<(), TranslationError>
    where
        F: FnOnce(&mut Self) -> Result<(), TranslationError>,
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
    fn compute_drop_keep(&self, depth: u32) -> Result<DropKeep, TranslationError> {
        debug_assert!(self.is_reachable());
        let frame = self.alloc.control_frames.nth_back(depth);
        // Find out how many values we need to keep (copy to the new stack location after the drop).
        let keep = match frame.kind() {
            ControlFrameKind::Block | ControlFrameKind::If => {
                frame.block_type().len_results(&self.engine)
            }
            ControlFrameKind::Loop => frame.block_type().len_params(&self.engine),
        };
        // Find out how many values we need to drop.
        let current_height = self.stack_height.height();
        let origin_height = frame.stack_height().expect("frame is reachable");
        assert!(
            origin_height <= current_height,
            "encountered value stack underflow: \
            current height {current_height}, original height {origin_height}",
        );
        let height_diff = current_height - origin_height;
        assert!(
            keep <= height_diff,
            "tried to keep {keep} values while having \
            only {height_diff} values available on the frame",
        );
        let drop = height_diff - keep;
        DropKeep::new(drop as usize, keep as usize).map_err(Into::into)
    }

    /// Compute [`DropKeep`] for the return statement.
    ///
    /// # Panics
    ///
    /// - If the control flow frame stack is empty.
    /// - If the value stack is underflown.
    fn drop_keep_return(&self) -> Result<DropKeep, TranslationError> {
        debug_assert!(self.is_reachable());
        assert!(
            !self.alloc.control_frames.is_empty(),
            "drop_keep_return cannot be called with the frame stack empty"
        );
        let max_depth = self
            .alloc
            .control_frames
            .len()
            .checked_sub(1)
            .expect("control flow frame stack must not be empty") as u32;
        let drop_keep = self.compute_drop_keep(max_depth)?;
        let len_params_locals = self.locals.len_registered() as usize;
        DropKeep::new(
            // Drop all local variables and parameters upon exit.
            drop_keep.drop() + len_params_locals,
            drop_keep.keep(),
        )
        .map_err(Into::into)
    }

    /// Returns the relative depth on the stack of the local variable.
    fn relative_local_depth(&self, local_idx: u32) -> usize {
        debug_assert!(self.is_reachable());
        let stack_height = self.stack_height.height() as usize;
        let len_params_locals = self.locals.len_registered() as usize;
        stack_height
            .checked_add(len_params_locals)
            .and_then(|x| x.checked_sub(local_idx as usize))
            .unwrap_or_else(|| panic!("cannot convert local index into local depth: {local_idx}"))
    }

    /// Returns the target at the given `depth` together with its [`DropKeep`].
    ///
    /// # Panics
    ///
    /// - If the `depth` is greater than the current height of the control frame stack.
    /// - If the value stack underflowed.
    fn acquire_target(&self, relative_depth: u32) -> Result<AcquiredTarget, TranslationError> {
        debug_assert!(self.is_reachable());
        if self.alloc.control_frames.is_root(relative_depth) {
            let drop_keep = self.drop_keep_return()?;
            Ok(AcquiredTarget::Return(drop_keep))
        } else {
            let label = self
                .alloc
                .control_frames
                .nth_back(relative_depth)
                .branch_destination();
            let drop_keep = self.compute_drop_keep(relative_depth)?;
            Ok(AcquiredTarget::Branch(label, drop_keep))
        }
    }

    /// Creates [`BranchParams`] to `target` using `drop_keep` for the current instruction.
    fn branch_params(&mut self, target: LabelRef, drop_keep: DropKeep) -> BranchParams {
        BranchParams::new(self.alloc.inst_builder.try_resolve_label(target), drop_keep)
    }
}

/// An acquired target.
///
/// Returned by [`FuncBuilder::acquire_target`].
#[derive(Debug)]
pub enum AcquiredTarget {
    /// The branch jumps to the label.
    Branch(LabelRef, DropKeep),
    /// The branch returns to the caller.
    ///
    /// # Note
    ///
    /// This is returned if the `relative_depth` points to the outmost
    /// function body `block`. WebAssembly defines branches to this control
    /// flow frame as equivalent to returning from the function.
    Return(DropKeep),
}

impl<'parser> FuncBuilder<'parser> {
    /// Translates a Wasm `unreachable` instruction.
    pub fn translate_nop(&mut self) -> Result<(), TranslationError> {
        Ok(())
    }

    /// Translates a Wasm `unreachable` instruction.
    pub fn translate_unreachable(&mut self) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::Unreachable);
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
        let len_params = block_type.len_params(&self.engine);
        let stack_height = self.stack_height.height();
        stack_height.checked_sub(len_params).unwrap_or_else(|| {
            panic!(
                "encountered emulated value stack underflow with \
                 stack height {stack_height} and {len_params} block parameters",
            )
        })
    }

    /// Translates a Wasm `block` control flow operator.
    pub fn translate_block(
        &mut self,
        block_type: wasmparser::BlockType,
    ) -> Result<(), TranslationError> {
        let block_type = BlockType::try_from_wasmparser(block_type, self.res)?;
        if self.is_reachable() {
            let stack_height = self.frame_stack_height(block_type);
            let end_label = self.alloc.inst_builder.new_label();
            self.alloc.control_frames.push_frame(BlockControlFrame::new(
                block_type,
                end_label,
                stack_height,
            ));
        } else {
            self.alloc
                .control_frames
                .push_frame(UnreachableControlFrame::new(
                    ControlFrameKind::Block,
                    block_type,
                ));
        }
        Ok(())
    }

    /// Translates a Wasm `loop` control flow operator.
    pub fn translate_loop(
        &mut self,
        block_type: wasmparser::BlockType,
    ) -> Result<(), TranslationError> {
        let block_type = BlockType::try_from_wasmparser(block_type, self.res)?;
        if self.is_reachable() {
            let stack_height = self.frame_stack_height(block_type);
            let header = self.alloc.inst_builder.new_label();
            self.alloc.inst_builder.pin_label(header);
            self.alloc.control_frames.push_frame(LoopControlFrame::new(
                block_type,
                header,
                stack_height,
            ));
        } else {
            self.alloc
                .control_frames
                .push_frame(UnreachableControlFrame::new(
                    ControlFrameKind::Loop,
                    block_type,
                ));
        }
        Ok(())
    }

    /// Translates a Wasm `if` control flow operator.
    pub fn translate_if(
        &mut self,
        block_type: wasmparser::BlockType,
    ) -> Result<(), TranslationError> {
        let block_type = BlockType::try_from_wasmparser(block_type, self.res)?;
        if self.is_reachable() {
            self.stack_height.pop1();
            let stack_height = self.frame_stack_height(block_type);
            let else_label = self.alloc.inst_builder.new_label();
            let end_label = self.alloc.inst_builder.new_label();
            self.alloc.control_frames.push_frame(IfControlFrame::new(
                block_type,
                end_label,
                else_label,
                stack_height,
            ));
            let branch_params = self.branch_params(else_label, DropKeep::none());
            self.alloc
                .inst_builder
                .push_inst(Instruction::BrIfEqz(branch_params));
        } else {
            self.alloc
                .control_frames
                .push_frame(UnreachableControlFrame::new(
                    ControlFrameKind::If,
                    block_type,
                ));
        }
        Ok(())
    }

    /// Translates a Wasm `else` control flow operator.
    pub fn translate_else(&mut self) -> Result<(), TranslationError> {
        let mut if_frame = match self.alloc.control_frames.pop_frame() {
            ControlFrame::If(if_frame) => if_frame,
            ControlFrame::Unreachable(frame) if matches!(frame.kind(), ControlFrameKind::If) => {
                // Encountered `Else` block for unreachable `If` block.
                //
                // In this case we can simply ignore the entire `Else` block
                // since it is unreachable anyways.
                self.alloc.control_frames.push_frame(frame);
                return Ok(());
            }
            unexpected => panic!(
                "expected `if` control flow frame on top \
                for `else` but found: {unexpected:?}",
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
            let params = self.branch_params(if_frame.end_label(), DropKeep::none());
            self.alloc.inst_builder.push_inst(Instruction::Br(params));
        }
        // Now resolve labels for the instructions of the `else` block
        self.alloc.inst_builder.pin_label(if_frame.else_label());
        // We need to reset the value stack to exactly how it has been
        // when entering the `if` in the first place so that the `else`
        // block has the same parameters on top of the stack.
        self.stack_height.shrink_to(if_frame.stack_height());
        if_frame.block_type().foreach_param(&self.engine, |_param| {
            self.stack_height.push();
        });
        self.alloc.control_frames.push_frame(if_frame);
        // We can reset reachability now since the parent `if` block was reachable.
        self.reachable = true;
        Ok(())
    }

    /// Translates a Wasm `end` control flow operator.
    pub fn translate_end(&mut self) -> Result<(), TranslationError> {
        let frame = self.alloc.control_frames.last();
        if let ControlFrame::If(if_frame) = &frame {
            // At this point we can resolve the `Else` label.
            //
            // Note: The `Else` label might have already been resolved
            //       in case there was an `Else` block.
            self.alloc
                .inst_builder
                .pin_label_if_unpinned(if_frame.else_label());
        }
        if frame.is_reachable() && !matches!(frame.kind(), ControlFrameKind::Loop) {
            // At this point we can resolve the `End` labels.
            // Note that `loop` control frames do not have an `End` label.
            self.alloc.inst_builder.pin_label(frame.end_label());
        }
        // These bindings are required because of borrowing issues.
        let frame_reachable = frame.is_reachable();
        let frame_stack_height = frame.stack_height();
        if self.alloc.control_frames.len() == 1 {
            // If the control flow frames stack is empty after this point
            // we know that we are ending the function body `block`
            // frame and therefore we have to return from the function.
            self.translate_return()?;
        } else {
            // The following code is only reachable if the ended control flow
            // frame was reachable upon entering to begin with.
            self.reachable = frame_reachable;
        }
        if let Some(frame_stack_height) = frame_stack_height {
            self.stack_height.shrink_to(frame_stack_height);
        }
        let frame = self.alloc.control_frames.pop_frame();
        frame
            .block_type()
            .foreach_result(&self.engine, |_result| self.stack_height.push());
        Ok(())
    }

    /// Translates a Wasm `br` control flow operator.
    pub fn translate_br(&mut self, relative_depth: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            match builder.acquire_target(relative_depth)? {
                AcquiredTarget::Branch(end_label, drop_keep) => {
                    let params = builder.branch_params(end_label, drop_keep);
                    builder
                        .alloc
                        .inst_builder
                        .push_inst(Instruction::Br(params));
                }
                AcquiredTarget::Return(_) => {
                    // In this case the `br` can be directly translated as `return`.
                    builder.translate_return()?;
                }
            }
            builder.reachable = false;
            Ok(())
        })
    }

    /// Translates a Wasm `br_if` control flow operator.
    pub fn translate_br_if(&mut self, relative_depth: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder.stack_height.pop1();
            match builder.acquire_target(relative_depth)? {
                AcquiredTarget::Branch(end_label, drop_keep) => {
                    let params = builder.branch_params(end_label, drop_keep);
                    builder
                        .alloc
                        .inst_builder
                        .push_inst(Instruction::BrIfNez(params));
                }
                AcquiredTarget::Return(drop_keep) => {
                    builder
                        .alloc
                        .inst_builder
                        .push_inst(Instruction::ReturnIfNez(drop_keep));
                }
            }
            Ok(())
        })
    }

    /// Translates a Wasm `br_table` control flow operator.
    pub fn translate_br_table(
        &mut self,
        table: wasmparser::BrTable<'parser>,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            fn offset_instr(base: Instr, offset: usize) -> Instr {
                Instr::from_u32(base.into_u32() + offset as u32)
            }

            fn compute_instr(
                builder: &mut FuncBuilder,
                n: usize,
                depth: RelativeDepth,
            ) -> Result<Instruction, TranslationError> {
                match builder.acquire_target(depth.into_u32())? {
                    AcquiredTarget::Branch(label, drop_keep) => {
                        let base = builder.alloc.inst_builder.current_pc();
                        let instr = offset_instr(base, n + 1);
                        let offset = builder
                            .alloc
                            .inst_builder
                            .try_resolve_label_for(label, instr);
                        let params = BranchParams::new(offset, drop_keep);
                        Ok(Instruction::Br(params))
                    }
                    AcquiredTarget::Return(drop_keep) => Ok(Instruction::Return(drop_keep)),
                }
            }

            let default = RelativeDepth::from_u32(table.default());
            let targets = table
                .targets()
                .map(|relative_depth| {
                    relative_depth.unwrap_or_else(|error| {
                        panic!(
                            "encountered unexpected invalid relative depth \
                            for `br_table` target: {error}",
                        )
                    })
                })
                .map(RelativeDepth::from_u32);

            builder.stack_height.pop1();
            builder.alloc.br_table_branches.clear();
            for (n, depth) in targets.into_iter().enumerate() {
                let relative_depth = compute_instr(builder, n, depth)?;
                builder.alloc.br_table_branches.push(relative_depth);
            }

            // We include the default target in `len_branches`.
            let len_branches = builder.alloc.br_table_branches.len();
            let default_branch = compute_instr(builder, len_branches, default)?;
            builder.alloc.inst_builder.push_inst(Instruction::BrTable {
                len_targets: len_branches + 1,
            });
            for branch in builder.alloc.br_table_branches.drain(..) {
                builder.alloc.inst_builder.push_inst(branch);
            }
            builder.alloc.inst_builder.push_inst(default_branch);
            builder.reachable = false;
            Ok(())
        })
    }

    /// Translates a Wasm `return` control flow operator.
    pub fn translate_return(&mut self) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let drop_keep = builder.drop_keep_return()?;
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::Return(drop_keep));
            builder.reachable = false;
            Ok(())
        })
    }

    /// Adjusts the emulated value stack given the [`FuncType`] of the call.
    fn adjust_value_stack_for_call(&mut self, func_type: &FuncType) {
        let (params, results) = func_type.params_results();
        self.stack_height.pop_n(params.len() as u32);
        self.stack_height.push_n(results.len() as u32);
    }

    /// Translates a Wasm `call` instruction.
    pub fn translate_call(&mut self, func_idx: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let func_idx = FuncIdx(func_idx);
            let func_type = builder.func_type_of(func_idx);
            builder.adjust_value_stack_for_call(&func_type);
            let func_idx = func_idx.into_u32().into();
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::Call(func_idx));
            Ok(())
        })
    }

    /// Translates a Wasm `call_indirect` instruction.
    pub fn translate_call_indirect(
        &mut self,
        func_type_index: u32,
        table_index: u32,
        _table_byte: u8,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let func_type_index = SignatureIdx::from(func_type_index);
            let table = TableIdx::from(table_index);
            builder.stack_height.pop1();
            let func_type = builder.func_type_at(func_type_index);
            builder.adjust_value_stack_for_call(&func_type);
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::CallIndirect {
                    table,
                    func_type: func_type_index,
                });
            Ok(())
        })
    }

    /// Translates a Wasm `drop` instruction.
    pub fn translate_drop(&mut self) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder.stack_height.pop1();
            builder.alloc.inst_builder.push_inst(Instruction::Drop);
            Ok(())
        })
    }

    /// Translates a Wasm `select` instruction.
    pub fn translate_select(&mut self) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder.stack_height.pop3();
            builder.stack_height.push();
            builder.alloc.inst_builder.push_inst(Instruction::Select);
            Ok(())
        })
    }

    /// Translate a Wasm `local.get` instruction.
    pub fn translate_local_get(&mut self, local_idx: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let local_depth = builder.relative_local_depth(local_idx);
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::local_get(local_depth));
            builder.stack_height.push();
            Ok(())
        })
    }

    /// Translate a Wasm `local.set` instruction.
    pub fn translate_local_set(&mut self, local_idx: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder.stack_height.pop1();
            let local_depth = builder.relative_local_depth(local_idx);
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::local_set(local_depth));
            Ok(())
        })
    }

    /// Translate a Wasm `local.tee` instruction.
    pub fn translate_local_tee(&mut self, local_idx: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let local_depth = builder.relative_local_depth(local_idx);
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::local_tee(local_depth));
            Ok(())
        })
    }

    /// Translate a Wasm `global.get` instruction.
    pub fn translate_global_get(&mut self, global_idx: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let global_idx = GlobalIdx(global_idx);
            builder.stack_height.push();
            let (global_type, init_value) = builder.res.get_global(global_idx);
            let instr = match init_value.and_then(InitExpr::to_const) {
                Some(value) if global_type.mutability().is_const() => {
                    Instruction::constant(value.clone())
                }
                _ => Instruction::GlobalGet(global_idx.into_u32().into()),
            };
            builder.alloc.inst_builder.push_inst(instr);
            Ok(())
        })
    }

    /// Translate a Wasm `global.set` instruction.
    pub fn translate_global_set(&mut self, global_idx: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let global_idx = GlobalIdx(global_idx);
            let global_type = builder.res.get_type_of_global(global_idx);
            debug_assert_eq!(global_type.mutability(), Mutability::Var);
            builder.stack_height.pop1();
            let global_idx = global_idx.into_u32().into();
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::GlobalSet(global_idx));
            Ok(())
        })
    }

    /// Decompose a [`wasmparser::MemArg`] into its raw parts.
    fn decompose_memarg(memarg: wasmparser::MemArg) -> (MemoryIdx, u32) {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        (memory_idx, offset)
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
        memarg: wasmparser::MemArg,
        _loaded_type: ValueType,
        make_inst: fn(Offset) -> Instruction,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let (memory_idx, offset) = Self::decompose_memarg(memarg);
            debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
            builder.stack_height.pop1();
            builder.stack_height.push();
            let offset = Offset::from(offset);
            builder.alloc.inst_builder.push_inst(make_inst(offset));
            Ok(())
        })
    }

    /// Translate a Wasm `i32.load` instruction.
    pub fn translate_i32_load(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I32, Instruction::I32Load)
    }

    /// Translate a Wasm `i64.load` instruction.
    pub fn translate_i64_load(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I64, Instruction::I64Load)
    }

    /// Translate a Wasm `f32.load` instruction.
    pub fn translate_f32_load(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::F32, Instruction::F32Load)
    }

    /// Translate a Wasm `f64.load` instruction.
    pub fn translate_f64_load(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::F64, Instruction::F64Load)
    }

    /// Translate a Wasm `i32.load_i8` instruction.
    pub fn translate_i32_load8_s(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I32, Instruction::I32Load8S)
    }

    /// Translate a Wasm `i32.load_u8` instruction.
    pub fn translate_i32_load8_u(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I32, Instruction::I32Load8U)
    }

    /// Translate a Wasm `i32.load_i16` instruction.
    pub fn translate_i32_load16_s(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I32, Instruction::I32Load16S)
    }

    /// Translate a Wasm `i32.load_u16` instruction.
    pub fn translate_i32_load16_u(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I32, Instruction::I32Load16U)
    }

    /// Translate a Wasm `i64.load_i8` instruction.
    pub fn translate_i64_load8_s(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I64, Instruction::I64Load8S)
    }

    /// Translate a Wasm `i64.load_u8` instruction.
    pub fn translate_i64_load8_u(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I64, Instruction::I64Load8U)
    }

    /// Translate a Wasm `i64.load_i16` instruction.
    pub fn translate_i64_load16_s(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I64, Instruction::I64Load16S)
    }

    /// Translate a Wasm `i64.load_u16` instruction.
    pub fn translate_i64_load16_u(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I64, Instruction::I64Load16U)
    }

    /// Translate a Wasm `i64.load_i32` instruction.
    pub fn translate_i64_load32_s(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I64, Instruction::I64Load32S)
    }

    /// Translate a Wasm `i64.load_u32` instruction.
    pub fn translate_i64_load32_u(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_load(memarg, ValueType::I64, Instruction::I64Load32U)
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
        memarg: wasmparser::MemArg,
        _stored_value: ValueType,
        make_inst: fn(Offset) -> Instruction,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let (memory_idx, offset) = Self::decompose_memarg(memarg);
            debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
            builder.stack_height.pop2();
            let offset = Offset::from(offset);
            builder.alloc.inst_builder.push_inst(make_inst(offset));
            Ok(())
        })
    }

    /// Translate a Wasm `i32.store` instruction.
    pub fn translate_i32_store(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_store(memarg, ValueType::I32, Instruction::I32Store)
    }

    /// Translate a Wasm `i64.store` instruction.
    pub fn translate_i64_store(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_store(memarg, ValueType::I64, Instruction::I64Store)
    }

    /// Translate a Wasm `f32.store` instruction.
    pub fn translate_f32_store(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_store(memarg, ValueType::F32, Instruction::F32Store)
    }

    /// Translate a Wasm `f64.store` instruction.
    pub fn translate_f64_store(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_store(memarg, ValueType::F64, Instruction::F64Store)
    }

    /// Translate a Wasm `i32.store_i8` instruction.
    pub fn translate_i32_store8(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_store(memarg, ValueType::I32, Instruction::I32Store8)
    }

    /// Translate a Wasm `i32.store_i16` instruction.
    pub fn translate_i32_store16(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_store(memarg, ValueType::I32, Instruction::I32Store16)
    }

    /// Translate a Wasm `i64.store_i8` instruction.
    pub fn translate_i64_store8(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_store(memarg, ValueType::I64, Instruction::I64Store8)
    }

    /// Translate a Wasm `i64.store_i16` instruction.
    pub fn translate_i64_store16(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_store(memarg, ValueType::I64, Instruction::I64Store16)
    }

    /// Translate a Wasm `i64.store_i32` instruction.
    pub fn translate_i64_store32(
        &mut self,
        memarg: wasmparser::MemArg,
    ) -> Result<(), TranslationError> {
        self.translate_store(memarg, ValueType::I64, Instruction::I64Store32)
    }

    /// Translate a Wasm `memory.size` instruction.
    pub fn translate_memory_size(
        &mut self,
        memory_idx: u32,
        _mem_byte: u8,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let memory_idx = MemoryIdx(memory_idx);
            debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
            builder.stack_height.push();
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::MemorySize);
            Ok(())
        })
    }

    /// Translate a Wasm `memory.grow` instruction.
    pub fn translate_memory_grow(
        &mut self,
        memory_index: u32,
        _mem_byte: u8,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(memory_index, DEFAULT_MEMORY_INDEX);
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::MemoryGrow);
            Ok(())
        })
    }

    /// Translate a Wasm `memory.init` instruction.
    pub fn translate_memory_init(
        &mut self,
        segment_index: u32,
        memory_index: u32,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(memory_index, DEFAULT_MEMORY_INDEX);
            builder.stack_height.pop3();
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::MemoryInit(DataSegmentIdx::from(segment_index)));
            Ok(())
        })
    }

    /// Translate a Wasm `memory.fill` instruction.
    pub fn translate_memory_fill(&mut self, memory_index: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(memory_index, DEFAULT_MEMORY_INDEX);
            builder.stack_height.pop3();
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::MemoryFill);
            Ok(())
        })
    }

    /// Translate a Wasm `memory.copy` instruction.
    pub fn translate_memory_copy(
        &mut self,
        dst_mem: u32,
        src_mem: u32,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(dst_mem, DEFAULT_MEMORY_INDEX);
            debug_assert_eq!(src_mem, DEFAULT_MEMORY_INDEX);
            builder.stack_height.pop3();
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::MemoryCopy);
            Ok(())
        })
    }

    /// Translate a Wasm `data.drop` instruction.
    pub fn translate_data_drop(&mut self, segment_index: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let segment_index = DataSegmentIdx::from(segment_index);
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::DataDrop(segment_index));
            Ok(())
        })
    }

    /// Translate a Wasm `table.size` instruction.
    pub fn translate_table_size(&mut self, table_index: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let table = TableIdx::from(table_index);
            builder.stack_height.push();
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::TableSize { table });
            Ok(())
        })
    }

    /// Translate a Wasm `table.grow` instruction.
    pub fn translate_table_grow(&mut self, table_index: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let table = TableIdx::from(table_index);
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::TableGrow { table });
            Ok(())
        })
    }

    pub fn translate_table_copy(
        &mut self,
        dst_table: u32,
        src_table: u32,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let dst = TableIdx::from(dst_table);
            let src = TableIdx::from(src_table);
            builder.stack_height.pop3();
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::TableCopy { dst, src });
            Ok(())
        })
    }

    pub fn translate_table_fill(&mut self, _table_index: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|_builder| {
            // debug_assert_eq!(table_index, DEFAULT_TABLE_INDEX);
            // let memory_index = TableIdx(table_index);
            // builder.stack_height.pop3();
            // builder
            //     .alloc
            //     .inst_builder
            //     .push_inst(Instruction::MemoryFill { table_index });
            unimplemented!("wasmi does not yet support the `reference-types` Wasm proposal")
        })
    }

    /// Translate a Wasm `table.get` instruction.
    pub fn translate_table_get(&mut self, table_index: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let table = TableIdx::from(table_index);
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::TableGet { table });
            Ok(())
        })
    }

    /// Translate a Wasm `table.set` instruction.
    pub fn translate_table_set(&mut self, table_index: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            let table = TableIdx::from(table_index);
            builder.stack_height.pop1();
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::TableSet { table });
            Ok(())
        })
    }

    /// Translate a Wasm `table.init` instruction.
    pub fn translate_table_init(
        &mut self,
        segment_index: u32,
        table_index: u32,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder.stack_height.pop3();
            let table = TableIdx::from(table_index);
            let elem = ElementSegmentIdx::from(segment_index);
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::TableInit { table, elem });
            Ok(())
        })
    }

    /// Translate a Wasm `elem.drop` instruction.
    pub fn translate_elem_drop(&mut self, segment_index: u32) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::ElemDrop(ElementSegmentIdx::from(
                    segment_index,
                )));
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
    fn translate_const<T>(&mut self, value: T) -> Result<(), TranslationError>
    where
        T: Into<Value>,
    {
        self.translate_if_reachable(|builder| {
            let value = value.into();
            builder.stack_height.push();
            builder
                .alloc
                .inst_builder
                .push_inst(Instruction::constant(value));
            Ok(())
        })
    }

    /// Translate a Wasm `i32.const` instruction.
    pub fn translate_i32_const(&mut self, value: i32) -> Result<(), TranslationError> {
        self.translate_const(value)
    }

    /// Translate a Wasm `i64.const` instruction.
    pub fn translate_i64_const(&mut self, value: i64) -> Result<(), TranslationError> {
        self.translate_const(value)
    }

    /// Translate a Wasm `f32.const` instruction.
    pub fn translate_f32_const(
        &mut self,
        value: wasmparser::Ieee32,
    ) -> Result<(), TranslationError> {
        self.translate_const(F32::from_bits(value.bits()))
    }

    /// Translate a Wasm `f64.const` instruction.
    pub fn translate_f64_const(
        &mut self,
        value: wasmparser::Ieee64,
    ) -> Result<(), TranslationError> {
        self.translate_const(F64::from_bits(value.bits()))
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
        _input_type: ValueType,
        inst: Instruction,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder.stack_height.pop1();
            builder.stack_height.push();
            builder.alloc.inst_builder.push_inst(inst);
            Ok(())
        })
    }

    /// Translate a Wasm `i32.eqz` instruction.
    pub fn translate_i32_eqz(&mut self) -> Result<(), TranslationError> {
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
        _input_type: ValueType,
        inst: Instruction,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder.stack_height.pop2();
            builder.stack_height.push();
            builder.alloc.inst_builder.push_inst(inst);
            Ok(())
        })
    }

    /// Translate a Wasm `i32.eq` instruction.
    pub fn translate_i32_eq(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32Eq)
    }

    /// Translate a Wasm `i32.ne` instruction.
    pub fn translate_i32_ne(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32Ne)
    }

    /// Translate a Wasm `i32.lt` instruction.
    pub fn translate_i32_lt_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32LtS)
    }

    /// Translate a Wasm `u32.lt` instruction.
    pub fn translate_i32_lt_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32LtU)
    }

    /// Translate a Wasm `i32.gt` instruction.
    pub fn translate_i32_gt_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32GtS)
    }

    /// Translate a Wasm `u32.gt` instruction.
    pub fn translate_i32_gt_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32GtU)
    }

    /// Translate a Wasm `i32.le` instruction.
    pub fn translate_i32_le_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32LeS)
    }

    /// Translate a Wasm `u32.le` instruction.
    pub fn translate_i32_le_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32LeU)
    }

    /// Translate a Wasm `i32.ge` instruction.
    pub fn translate_i32_ge_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32GeS)
    }

    /// Translate a Wasm `u32.ge` instruction.
    pub fn translate_i32_ge_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I32, Instruction::I32GeU)
    }

    /// Translate a Wasm `i64.eqz` instruction.
    pub fn translate_i64_eqz(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_cmp(ValueType::I64, Instruction::I64Eqz)
    }

    /// Translate a Wasm `i64.eq` instruction.
    pub fn translate_i64_eq(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64Eq)
    }

    /// Translate a Wasm `i64.ne` instruction.
    pub fn translate_i64_ne(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64Ne)
    }

    /// Translate a Wasm `i64.lt` instruction.
    pub fn translate_i64_lt_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64LtS)
    }

    /// Translate a Wasm `u64.lt` instruction.
    pub fn translate_i64_lt_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64LtU)
    }

    /// Translate a Wasm `i64.gt` instruction.
    pub fn translate_i64_gt_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64GtS)
    }

    /// Translate a Wasm `u64.gt` instruction.
    pub fn translate_i64_gt_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64GtU)
    }

    /// Translate a Wasm `i64.le` instruction.
    pub fn translate_i64_le_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64LeS)
    }

    /// Translate a Wasm `u64.le` instruction.
    pub fn translate_i64_le_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64LeU)
    }

    /// Translate a Wasm `i64.ge` instruction.
    pub fn translate_i64_ge_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64GeS)
    }

    /// Translate a Wasm `u64.ge` instruction.
    pub fn translate_i64_ge_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::I64, Instruction::I64GeU)
    }

    /// Translate a Wasm `f32.eq` instruction.
    pub fn translate_f32_eq(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Eq)
    }

    /// Translate a Wasm `f32.ne` instruction.
    pub fn translate_f32_ne(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Ne)
    }

    /// Translate a Wasm `f32.lt` instruction.
    pub fn translate_f32_lt(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Lt)
    }

    /// Translate a Wasm `f32.gt` instruction.
    pub fn translate_f32_gt(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Gt)
    }

    /// Translate a Wasm `f32.le` instruction.
    pub fn translate_f32_le(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Le)
    }

    /// Translate a Wasm `f32.ge` instruction.
    pub fn translate_f32_ge(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::F32, Instruction::F32Ge)
    }

    /// Translate a Wasm `f64.eq` instruction.
    pub fn translate_f64_eq(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::F64, Instruction::F64Eq)
    }

    /// Translate a Wasm `f64.ne` instruction.
    pub fn translate_f64_ne(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::F64, Instruction::F64Ne)
    }

    /// Translate a Wasm `f64.lt` instruction.
    pub fn translate_f64_lt(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::F64, Instruction::F64Lt)
    }

    /// Translate a Wasm `f64.gt` instruction.
    pub fn translate_f64_gt(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::F64, Instruction::F64Gt)
    }

    /// Translate a Wasm `f64.le` instruction.
    pub fn translate_f64_le(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_cmp(ValueType::F64, Instruction::F64Le)
    }

    /// Translate a Wasm `f64.ge` instruction.
    pub fn translate_f64_ge(&mut self) -> Result<(), TranslationError> {
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
        _value_type: ValueType,
        inst: Instruction,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder.alloc.inst_builder.push_inst(inst);
            Ok(())
        })
    }

    /// Translate a Wasm `i32.clz` instruction.
    pub fn translate_i32_clz(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::I32, Instruction::I32Clz)
    }

    /// Translate a Wasm `i32.ctz` instruction.
    pub fn translate_i32_ctz(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::I32, Instruction::I32Ctz)
    }

    /// Translate a Wasm `i32.popcnt` instruction.
    pub fn translate_i32_popcnt(&mut self) -> Result<(), TranslationError> {
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
        _value_type: ValueType,
        inst: Instruction,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder.stack_height.pop2();
            builder.stack_height.push();
            builder.alloc.inst_builder.push_inst(inst);
            Ok(())
        })
    }

    /// Translate a Wasm `i32.add` instruction.
    pub fn translate_i32_add(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Add)
    }

    /// Translate a Wasm `i32.sub` instruction.
    pub fn translate_i32_sub(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Sub)
    }

    /// Translate a Wasm `i32.mul` instruction.
    pub fn translate_i32_mul(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Mul)
    }

    /// Translate a Wasm `i32.div` instruction.
    pub fn translate_i32_div_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32DivS)
    }

    /// Translate a Wasm `u32.div` instruction.
    pub fn translate_i32_div_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32DivU)
    }

    /// Translate a Wasm `i32.rem` instruction.
    pub fn translate_i32_rem_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32RemS)
    }

    /// Translate a Wasm `u32.rem` instruction.
    pub fn translate_i32_rem_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32RemU)
    }

    /// Translate a Wasm `i32.and` instruction.
    pub fn translate_i32_and(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32And)
    }

    /// Translate a Wasm `i32.or` instruction.
    pub fn translate_i32_or(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Or)
    }

    /// Translate a Wasm `i32.xor` instruction.
    pub fn translate_i32_xor(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Xor)
    }

    /// Translate a Wasm `i32.shl` instruction.
    pub fn translate_i32_shl(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Shl)
    }

    /// Translate a Wasm `i32.shr` instruction.
    pub fn translate_i32_shr_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32ShrS)
    }

    /// Translate a Wasm `u32.shr` instruction.
    pub fn translate_i32_shr_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32ShrU)
    }

    /// Translate a Wasm `i32.rotl` instruction.
    pub fn translate_i32_rotl(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Rotl)
    }

    /// Translate a Wasm `i32.rotr` instruction.
    pub fn translate_i32_rotr(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I32, Instruction::I32Rotr)
    }

    /// Translate a Wasm `i64.clz` instruction.
    pub fn translate_i64_clz(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Clz)
    }

    /// Translate a Wasm `i64.ctz` instruction.
    pub fn translate_i64_ctz(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Ctz)
    }

    /// Translate a Wasm `i64.popcnt` instruction.
    pub fn translate_i64_popcnt(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Popcnt)
    }

    /// Translate a Wasm `i64.add` instruction.
    pub fn translate_i64_add(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Add)
    }

    /// Translate a Wasm `i64.sub` instruction.
    pub fn translate_i64_sub(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Sub)
    }

    /// Translate a Wasm `i64.mul` instruction.
    pub fn translate_i64_mul(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Mul)
    }

    /// Translate a Wasm `i64.div` instruction.
    pub fn translate_i64_div_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64DivS)
    }

    /// Translate a Wasm `u64.div` instruction.
    pub fn translate_i64_div_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64DivU)
    }

    /// Translate a Wasm `i64.rem` instruction.
    pub fn translate_i64_rem_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64RemS)
    }

    /// Translate a Wasm `u64.rem` instruction.
    pub fn translate_i64_rem_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64RemU)
    }

    /// Translate a Wasm `i64.and` instruction.
    pub fn translate_i64_and(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64And)
    }

    /// Translate a Wasm `i64.or` instruction.
    pub fn translate_i64_or(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Or)
    }

    /// Translate a Wasm `i64.xor` instruction.
    pub fn translate_i64_xor(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Xor)
    }

    /// Translate a Wasm `i64.shl` instruction.
    pub fn translate_i64_shl(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Shl)
    }

    /// Translate a Wasm `i64.shr` instruction.
    pub fn translate_i64_shr_s(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64ShrS)
    }

    /// Translate a Wasm `u64.shr` instruction.
    pub fn translate_i64_shr_u(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64ShrU)
    }

    /// Translate a Wasm `i64.rotl` instruction.
    pub fn translate_i64_rotl(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Rotl)
    }

    /// Translate a Wasm `i64.rotr` instruction.
    pub fn translate_i64_rotr(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::I64, Instruction::I64Rotr)
    }

    /// Translate a Wasm `f32.abs` instruction.
    pub fn translate_f32_abs(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Abs)
    }

    /// Translate a Wasm `f32.neg` instruction.
    pub fn translate_f32_neg(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Neg)
    }

    /// Translate a Wasm `f32.ceil` instruction.
    pub fn translate_f32_ceil(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Ceil)
    }

    /// Translate a Wasm `f32.floor` instruction.
    pub fn translate_f32_floor(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Floor)
    }

    /// Translate a Wasm `f32.trunc` instruction.
    pub fn translate_f32_trunc(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Trunc)
    }

    /// Translate a Wasm `f32.nearest` instruction.
    pub fn translate_f32_nearest(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Nearest)
    }

    /// Translate a Wasm `f32.sqrt` instruction.
    pub fn translate_f32_sqrt(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F32, Instruction::F32Sqrt)
    }

    /// Translate a Wasm `f32.add` instruction.
    pub fn translate_f32_add(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Add)
    }

    /// Translate a Wasm `f32.sub` instruction.
    pub fn translate_f32_sub(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Sub)
    }

    /// Translate a Wasm `f32.mul` instruction.
    pub fn translate_f32_mul(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Mul)
    }

    /// Translate a Wasm `f32.div` instruction.
    pub fn translate_f32_div(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Div)
    }

    /// Translate a Wasm `f32.min` instruction.
    pub fn translate_f32_min(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Min)
    }

    /// Translate a Wasm `f32.max` instruction.
    pub fn translate_f32_max(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Max)
    }

    /// Translate a Wasm `f32.copysign` instruction.
    pub fn translate_f32_copysign(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F32, Instruction::F32Copysign)
    }

    /// Translate a Wasm `f64.abs` instruction.
    pub fn translate_f64_abs(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Abs)
    }

    /// Translate a Wasm `f64.neg` instruction.
    pub fn translate_f64_neg(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Neg)
    }

    /// Translate a Wasm `f64.ceil` instruction.
    pub fn translate_f64_ceil(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Ceil)
    }

    /// Translate a Wasm `f64.floor` instruction.
    pub fn translate_f64_floor(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Floor)
    }

    /// Translate a Wasm `f64.trunc` instruction.
    pub fn translate_f64_trunc(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Trunc)
    }

    /// Translate a Wasm `f64.nearest` instruction.
    pub fn translate_f64_nearest(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Nearest)
    }

    /// Translate a Wasm `f64.sqrt` instruction.
    pub fn translate_f64_sqrt(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::F64, Instruction::F64Sqrt)
    }

    /// Translate a Wasm `f64.add` instruction.
    pub fn translate_f64_add(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Add)
    }

    /// Translate a Wasm `f64.sub` instruction.
    pub fn translate_f64_sub(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Sub)
    }

    /// Translate a Wasm `f64.mul` instruction.
    pub fn translate_f64_mul(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Mul)
    }

    /// Translate a Wasm `f64.div` instruction.
    pub fn translate_f64_div(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Div)
    }

    /// Translate a Wasm `f64.min` instruction.
    pub fn translate_f64_min(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Min)
    }

    /// Translate a Wasm `f64.max` instruction.
    pub fn translate_f64_max(&mut self) -> Result<(), TranslationError> {
        self.translate_binary_operation(ValueType::F64, Instruction::F64Max)
    }

    /// Translate a Wasm `f64.copysign` instruction.
    pub fn translate_f64_copysign(&mut self) -> Result<(), TranslationError> {
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
        _input_type: ValueType,
        _output_type: ValueType,
        inst: Instruction,
    ) -> Result<(), TranslationError> {
        self.translate_if_reachable(|builder| {
            builder.stack_height.pop1();
            builder.stack_height.push();
            builder.alloc.inst_builder.push_inst(inst);
            Ok(())
        })
    }

    /// Translate a Wasm `i32.wrap_i64` instruction.
    pub fn translate_i32_wrap_i64(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::I64, ValueType::I32, Instruction::I32WrapI64)
    }

    /// Translate a Wasm `i32.trunc_f32` instruction.
    pub fn translate_i32_trunc_f32_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F32, ValueType::I32, Instruction::I32TruncF32S)
    }

    /// Translate a Wasm `u32.trunc_f32` instruction.
    pub fn translate_i32_trunc_f32_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F32, ValueType::I32, Instruction::I32TruncF32U)
    }

    /// Translate a Wasm `i32.trunc_f64` instruction.
    pub fn translate_i32_trunc_f64_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F64, ValueType::I32, Instruction::I32TruncF64S)
    }

    /// Translate a Wasm `u32.trunc_f64` instruction.
    pub fn translate_i32_trunc_f64_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F64, ValueType::I32, Instruction::I32TruncF64U)
    }

    /// Translate a Wasm `i64.extend_i32` instruction.
    pub fn translate_i64_extend_i32_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::I32, ValueType::I64, Instruction::I64ExtendI32S)
    }

    /// Translate a Wasm `u64.extend_i32` instruction.
    pub fn translate_i64_extend_i32_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::I32, ValueType::I64, Instruction::I64ExtendI32U)
    }

    /// Translate a Wasm `i64.trunc_f32` instruction.
    pub fn translate_i64_trunc_f32_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F32, ValueType::I64, Instruction::I64TruncF32S)
    }

    /// Translate a Wasm `u64.trunc_f32` instruction.
    pub fn translate_i64_trunc_f32_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F32, ValueType::I64, Instruction::I64TruncF32U)
    }

    /// Translate a Wasm `i64.trunc_f64` instruction.
    pub fn translate_i64_trunc_f64_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F64, ValueType::I64, Instruction::I64TruncF64S)
    }

    /// Translate a Wasm `u64.trunc_f64` instruction.
    pub fn translate_i64_trunc_f64_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F64, ValueType::I64, Instruction::I64TruncF64U)
    }

    /// Translate a Wasm `f32.convert_i32` instruction.
    pub fn translate_f32_convert_i32_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::I32, ValueType::F32, Instruction::F32ConvertI32S)
    }

    /// Translate a Wasm `f32.convert_u32` instruction.
    pub fn translate_f32_convert_i32_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::I32, ValueType::F32, Instruction::F32ConvertI32U)
    }

    /// Translate a Wasm `f32.convert_i64` instruction.
    pub fn translate_f32_convert_i64_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::I64, ValueType::F32, Instruction::F32ConvertI64S)
    }

    /// Translate a Wasm `f32.convert_u64` instruction.
    pub fn translate_f32_convert_i64_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::I64, ValueType::F32, Instruction::F32ConvertI64U)
    }

    /// Translate a Wasm `f32.demote_f64` instruction.
    pub fn translate_f32_demote_f64(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F64, ValueType::F32, Instruction::F32DemoteF64)
    }

    /// Translate a Wasm `f64.convert_i32` instruction.
    pub fn translate_f64_convert_i32_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::I32, ValueType::F64, Instruction::F64ConvertI32S)
    }

    /// Translate a Wasm `f64.convert_u32` instruction.
    pub fn translate_f64_convert_i32_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::I32, ValueType::F64, Instruction::F64ConvertI32U)
    }

    /// Translate a Wasm `f64.convert_i64` instruction.
    pub fn translate_f64_convert_i64_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::I64, ValueType::F64, Instruction::F64ConvertI64S)
    }

    /// Translate a Wasm `f64.convert_u64` instruction.
    pub fn translate_f64_convert_i64_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::I64, ValueType::F64, Instruction::F64ConvertI64U)
    }

    /// Translate a Wasm `f64.promote_f32` instruction.
    pub fn translate_f64_promote_f32(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F32, ValueType::F64, Instruction::F64PromoteF32)
    }

    /// Translate a Wasm reinterpret instruction.
    ///
    /// Since `wasmi` bytecode is untyped those reinterpret
    /// instructions from Wasm are simply dropped from codegen.
    ///
    /// - `i32.reinterpret_f32`
    /// - `i64.reinterpret_f64`
    /// - `f32.reinterpret_i32`
    /// - `f64.reinterpret_i64`
    pub fn translate_reinterpret(
        &mut self,
        _input_type: ValueType,
        _output_type: ValueType,
    ) -> Result<(), TranslationError> {
        Ok(())
    }

    /// Translate a Wasm `i32.reinterpret_f32` instruction.
    pub fn translate_i32_reinterpret_f32(&mut self) -> Result<(), TranslationError> {
        self.translate_reinterpret(ValueType::F32, ValueType::I32)
    }

    /// Translate a Wasm `i64.reinterpret_f64` instruction.
    pub fn translate_i64_reinterpret_f64(&mut self) -> Result<(), TranslationError> {
        self.translate_reinterpret(ValueType::F64, ValueType::I64)
    }

    /// Translate a Wasm `f32.reinterpret_i32` instruction.
    pub fn translate_f32_reinterpret_i32(&mut self) -> Result<(), TranslationError> {
        self.translate_reinterpret(ValueType::I32, ValueType::F32)
    }

    /// Translate a Wasm `f64.reinterpret_i64` instruction.
    pub fn translate_f64_reinterpret_i64(&mut self) -> Result<(), TranslationError> {
        self.translate_reinterpret(ValueType::I64, ValueType::F64)
    }

    /// Translate a Wasm `i32.extend_8s` instruction.
    pub fn translate_i32_extend8_s(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::I32, Instruction::I32Extend8S)
    }

    /// Translate a Wasm `i32.extend_16s` instruction.
    pub fn translate_i32_extend16_s(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::I32, Instruction::I32Extend16S)
    }

    /// Translate a Wasm `i64.extend_8s` instruction.
    pub fn translate_i64_extend8_s(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Extend8S)
    }

    /// Translate a Wasm `i64.extend_16s` instruction.
    pub fn translate_i64_extend16_s(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Extend16S)
    }

    /// Translate a Wasm `i64.extend_32s` instruction.
    pub fn translate_i64_extend32_s(&mut self) -> Result<(), TranslationError> {
        self.translate_unary_operation(ValueType::I64, Instruction::I64Extend32S)
    }

    /// Translate a Wasm `i32.trunc_sat_f32` instruction.
    pub fn translate_i32_trunc_sat_f32_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F32, ValueType::I32, Instruction::I32TruncSatF32S)
    }

    /// Translate a Wasm `u32.trunc_sat_f32` instruction.
    pub fn translate_i32_trunc_sat_f32_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F32, ValueType::I32, Instruction::I32TruncSatF32U)
    }

    /// Translate a Wasm `i32.trunc_sat_f64` instruction.
    pub fn translate_i32_trunc_sat_f64_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F64, ValueType::I32, Instruction::I32TruncSatF64S)
    }

    /// Translate a Wasm `u32.trunc_sat_f64` instruction.
    pub fn translate_i32_trunc_sat_f64_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F64, ValueType::I32, Instruction::I32TruncSatF64U)
    }

    /// Translate a Wasm `i64.trunc_sat_f32` instruction.
    pub fn translate_i64_trunc_sat_f32_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F32, ValueType::I64, Instruction::I64TruncSatF32S)
    }

    /// Translate a Wasm `u64.trunc_sat_f32` instruction.
    pub fn translate_i64_trunc_sat_f32_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F32, ValueType::I64, Instruction::I64TruncSatF32U)
    }

    /// Translate a Wasm `i64.trunc_sat_f64` instruction.
    pub fn translate_i64_trunc_sat_f64_s(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F64, ValueType::I64, Instruction::I64TruncSatF64S)
    }

    /// Translate a Wasm `u64.trunc_sat_f64` instruction.
    pub fn translate_i64_trunc_sat_f64_u(&mut self) -> Result<(), TranslationError> {
        self.translate_conversion(ValueType::F64, ValueType::I64, Instruction::I64TruncSatF64U)
    }
}
