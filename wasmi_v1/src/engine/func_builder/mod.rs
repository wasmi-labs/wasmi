#![allow(dead_code, unused_variables)] // TODO: remove annotation once done

mod control_frame;
mod control_stack;
mod inst_builder;
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
    value_stack::ValueStack,
};
use super::{DropKeep, Instruction, Target};
use crate::{
    module2::{BlockType, FuncIdx, FuncTypeIdx, GlobalIdx, MemoryIdx, ModuleResources, TableIdx},
    Engine,
    ModuleError,
};
use wasmi_core::{ValueType, F32, F64};

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
    /// The amount of local variables of the currently compiled function.
    len_locals: usize,
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
        Self::register_func_params(func, res, &mut value_stack);
        Self {
            engine,
            func,
            res,
            control_frames,
            value_stack,
            inst_builder,
            len_locals: 0,
            reachable: true,
        }
    }

    /// Registers the `block` control frame surrounding the entire function body.
    fn register_func_body_block(
        func: FuncIdx,
        res: ModuleResources<'parser>,
        inst_builder: &mut InstructionsBuilder,
        control_frames: &mut ControlFlowStack,
    ) {
        let func_type = res.get_type_idx_of_func(func);
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
    ) -> usize {
        let params = res.get_type_of_func(func).params();
        for param in params {
            value_stack.push(*param);
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
        _value_type: ValueType,
    ) -> Result<(), ModuleError> {
        self.len_locals += amount as usize;
        Ok(())
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
    pub fn compute_drop_keep(&self, depth: u32) -> DropKeep {
        let frame = self.control_frames.nth_back(depth);
        // Find out how many values we need to keep (copy to the new stack location after the drop).
        let keep = match frame.kind() {
            ControlFrameKind::Block | ControlFrameKind::If => {
                frame.block_type().results(self.res).len() as u32
            }
            ControlFrameKind::Loop => frame.block_type().params(self.res).len() as u32,
        };
        // Find out how many values we need to drop.
        let drop = if !self.is_reachable() {
            0
        } else {
            let current_height = self.value_stack.len();
            let origin_height = frame.stack_height();
            assert!(
                origin_height < current_height,
                "encountered value stack underflow: current height {}, original height {}",
                current_height,
                origin_height,
            );
            let height_diff = current_height - origin_height;
            assert!(
                keep < height_diff,
                "tried to keep {} values while having only {} values available on the frame",
                keep,
                current_height - origin_height,
            );
            height_diff - keep
        };
        DropKeep::new32(drop, keep)
    }

    /// Compute [`DropKeep`] for the return statement.
    ///
    /// # Panics
    ///
    /// - If the control flow frame stack is empty.
    /// - If the value stack is underflown.
    pub fn drop_keep_return(&self) -> DropKeep {
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
        let len_locals = self.len_locals;
        let len_params = self.res.get_type_of_func(self.func).params().len();
        DropKeep::new(
            // Drop all local variables and parameters upon exit.
            drop_keep.drop() + len_locals + len_params,
            drop_keep.keep(),
        )
    }

    /// Returns the relative depth on the stack of the local variable.
    ///
    /// # Note
    ///
    /// See stack layout definition in `isa.rs`.
    pub fn relative_local_depth(&self, local_idx: u32) -> u32 {
        let stack_height = self.value_stack.len();
        let len_locals = self.len_locals as u32;
        let len_params = self.res.get_type_of_func(self.func).params().len() as u32;
        stack_height
            .checked_add(len_params)
            .and_then(|x| x.checked_add(len_locals))
            .and_then(|x| x.checked_sub(local_idx))
            .unwrap_or_else(|| panic!("cannot convert local index into local depth: {}", local_idx))
    }

    /// Returns the target at the given `depth` together with its [`DropKeep`].
    ///
    /// # Panics
    ///
    /// - If the `depth` is greater than the current height of the control frame stack.
    /// - If the value stack underflowed.
    pub fn acquire_target(&self, depth: u32) -> (LabelIdx, DropKeep) {
        let label = self.control_frames.nth_back(depth).branch_destination();
        let drop_keep = self.compute_drop_keep(depth);
        (label, drop_keep)
    }
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

    /// Translates a Wasm `block` control flow operator.
    pub fn translate_block(&mut self, block_type: BlockType) -> Result<(), ModuleError> {
        let stack_height = self.value_stack.len();
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
        let stack_height = self.value_stack.len();
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
        self.value_stack.pop1();
        let stack_height = self.value_stack.len();
        if self.is_reachable() {
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
        if self.is_reachable() {
            let if_frame = match self.control_frames.pop_frame() {
                ControlFrame::If(if_frame) => if_frame,
                unexpected => panic!(
                    "expected `if` control flow frame on top for `else` but found: {:?}",
                    unexpected,
                ),
            };
            let dst_pc =
                self.try_resolve_label(if_frame.end_label(), |pc| Reloc::Br { inst_idx: pc });
            let target = Target::new(dst_pc, DropKeep::new(0, 0));
            self.inst_builder.push_inst(Instruction::Br(target));
            self.inst_builder.resolve_label(if_frame.else_label());
            self.control_frames.push_frame(if_frame);
        } else {
            match self.control_frames.last() {
                ControlFrame::Unreachable(frame)
                    if matches!(frame.kind(), ControlFrameKind::If) =>
                {
                    return Ok(())
                }
                unexpected => panic!(
                    "expected unreachable `if` control flow frame on top of the \
                    control flow frame stack but found: {:?}",
                    unexpected
                ),
            }
        }
        Ok(())
    }

    /// Translates a Wasm `end` control flow operator.
    pub fn translate_end(&mut self) -> Result<(), ModuleError> {
        let frame = self.control_frames.pop_frame();
        if let ControlFrame::If(if_frame) = &frame {
            // At this point we can resolve the `Else` label.
            self.inst_builder.resolve_label(if_frame.else_label());
        }
        if !matches!(frame.kind(), ControlFrameKind::Loop) {
            // At this point we can resolve the `End` labels.
            // Note that `loop` control frames do not have an `End` label.
            self.inst_builder.resolve_label(frame.end_label());
        }
        if self.is_reachable() {
            if self.control_frames.is_empty() {
                // If the control flow frames stack is empty at this point
                // we know that we have just popped the function body `block`
                // frame and therefore we have to return from the function.
                //
                // TODO: properly calculate DropKeep of returning at this point
                let drop_keep = DropKeep::new(0, 0);
                self.inst_builder.push_inst(Instruction::Return(drop_keep));
            }
        } else {
            // We reset the reachability if the popped control flow
            // frame was reachable to begin with.
            self.reachable = frame.is_reachable();
        }
        self.value_stack.shrink_to(frame.stack_height());
        Ok(())
    }

    /// Translates a Wasm `br` control flow operator.
    pub fn translate_br(&mut self, relative_depth: u32) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let (end_label, drop_keep) = builder.acquire_target(relative_depth);
            let dst_pc = builder.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
            builder
                .inst_builder
                .push_inst(Instruction::Br(Target::new(dst_pc, drop_keep)));
            builder.reachable = false;
            Ok(())
        })
    }

    /// Translates a Wasm `br_if` control flow operator.
    pub fn translate_br_if(&mut self, relative_depth: u32) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            builder.value_stack.pop1();
            let (end_label, drop_keep) = builder.acquire_target(relative_depth);
            let dst_pc = builder.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
            builder
                .inst_builder
                .push_inst(Instruction::BrIfNez(Target::new(dst_pc, drop_keep)));
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
            builder.value_stack.pop1();

            let mut compute_target = |n: usize, depth: RelativeDepth| {
                let (label, drop_keep) = builder.acquire_target(depth.into_u32());
                let dst_pc = builder.try_resolve_label(label, |pc| Reloc::BrTable {
                    inst_idx: pc,
                    target_idx: n,
                });
                Target::new(dst_pc, drop_keep)
            };

            let targets = targets
                .into_iter()
                .enumerate()
                .map(|(n, depth)| compute_target(n, depth))
                .collect::<Vec<_>>();
            let len_targets = targets.len();
            let default = compute_target(len_targets, default);
            builder
                .inst_builder
                .push_inst(Instruction::BrTable { len_targets });
            for target in targets {
                builder
                    .inst_builder
                    .push_inst(Instruction::BrTableTarget(target));
            }
            builder
                .inst_builder
                .push_inst(Instruction::BrTableTarget(default));
            builder.reachable = false;
            Ok(())
        })
    }

    /// Translates a Wasm `return` control flow operator.
    pub fn translate_return(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `call` instruction.
    pub fn translate_call(&mut self, func_idx: FuncIdx) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `call_indirect` instruction.
    pub fn translate_call_indirect(
        &mut self,
        func_type_idx: FuncTypeIdx,
        table_idx: TableIdx,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `drop` instruction.
    pub fn translate_drop(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `select` instruction.
    pub fn translate_select(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `local.get` instruction.
    pub fn translate_local_get(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `local.set` instruction.
    pub fn translate_local_set(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `local.tee` instruction.
    pub fn translate_local_tee(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `global.get` instruction.
    pub fn translate_global_get(&mut self, global_idx: GlobalIdx) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `global.set` instruction.
    pub fn translate_global_set(&mut self, global_idx: GlobalIdx) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32.load` instruction.
    pub fn translate_i32_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.load` instruction.
    pub fn translate_i64_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32.load` instruction.
    pub fn translate_f32_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64.load` instruction.
    pub fn translate_f64_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32.load_i8` instruction.
    pub fn translate_i32_load_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32.load_u8` instruction.
    pub fn translate_i32_load_u8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32.load_i16` instruction.
    pub fn translate_i32_load_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32.load_u16` instruction.
    pub fn translate_i32_load_u16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.load_i8` instruction.
    pub fn translate_i64_load_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.load_u8` instruction.
    pub fn translate_i64_load_u8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.load_i16` instruction.
    pub fn translate_i64_load_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.load_u16` instruction.
    pub fn translate_i64_load_u16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.load_i32` instruction.
    pub fn translate_i64_load_i32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.load_u32` instruction.
    pub fn translate_i64_load_u32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32.store` instruction.
    pub fn translate_i32_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.store` instruction.
    pub fn translate_i64_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32.store` instruction.
    pub fn translate_f32_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64.store` instruction.
    pub fn translate_f64_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32.store_i8` instruction.
    pub fn translate_i32_store_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32.store_i16` instruction.
    pub fn translate_i32_store_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.store_i8` instruction.
    pub fn translate_i64_store_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.store_i16` instruction.
    pub fn translate_i64_store_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.store_i32` instruction.
    pub fn translate_i64_store_i32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `memory.size` instruction.
    pub fn translate_memory_size(&mut self, memory_idx: MemoryIdx) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `memory.grow` instruction.
    pub fn translate_memory_grow(&mut self, memory_idx: MemoryIdx) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32.const` instruction.
    pub fn translate_i32_const(&mut self, value: i32) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64.const` instruction.
    pub fn translate_i64_const(&mut self, value: i64) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32.const` instruction.
    pub fn translate_f32_const(&mut self, value: F32) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64.const` instruction.
    pub fn translate_f64_const(&mut self, value: F64) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_eqz` instruction.
    pub fn translate_i32_eqz(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_eq` instruction.
    pub fn translate_i32_eq(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_ne` instruction.
    pub fn translate_i32_ne(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_lt` instruction.
    pub fn translate_i32_lt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u32_lt` instruction.
    pub fn translate_u32_lt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_gt` instruction.
    pub fn translate_i32_gt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u32_gt` instruction.
    pub fn translate_u32_gt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_le` instruction.
    pub fn translate_i32_le(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u32_le` instruction.
    pub fn translate_u32_le(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_ge` instruction.
    pub fn translate_i32_ge(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u32_ge` instruction.
    pub fn translate_u32_ge(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_eqz` instruction.
    pub fn translate_i64_eqz(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_eq` instruction.
    pub fn translate_i64_eq(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_ne` instruction.
    pub fn translate_i64_ne(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_lt` instruction.
    pub fn translate_i64_lt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u64_lt` instruction.
    pub fn translate_u64_lt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_gt` instruction.
    pub fn translate_i64_gt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u64_gt` instruction.
    pub fn translate_u64_gt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_le` instruction.
    pub fn translate_i64_le(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u64_le` instruction.
    pub fn translate_u64_le(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_ge` instruction.
    pub fn translate_i64_ge(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u64_ge` instruction.
    pub fn translate_u64_ge(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_eq` instruction.
    pub fn translate_f32_eq(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_ne` instruction.
    pub fn translate_f32_ne(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_lt` instruction.
    pub fn translate_f32_lt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_gt` instruction.
    pub fn translate_f32_gt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_le` instruction.
    pub fn translate_f32_le(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_ge` instruction.
    pub fn translate_f32_ge(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_eq` instruction.
    pub fn translate_f64_eq(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_ne` instruction.
    pub fn translate_f64_ne(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_lt` instruction.
    pub fn translate_f64_lt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_gt` instruction.
    pub fn translate_f64_gt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_le` instruction.
    pub fn translate_f64_le(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_ge` instruction.
    pub fn translate_f64_ge(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_clz` instruction.
    pub fn translate_i32_clz(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_ctz` instruction.
    pub fn translate_i32_ctz(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_popcnt` instruction.
    pub fn translate_i32_popcnt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_add` instruction.
    pub fn translate_i32_add(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_sub` instruction.
    pub fn translate_i32_sub(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_mul` instruction.
    pub fn translate_i32_mul(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_div` instruction.
    pub fn translate_i32_div(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u32_div` instruction.
    pub fn translate_u32_div(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_rem` instruction.
    pub fn translate_i32_rem(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u32_rem` instruction.
    pub fn translate_u32_rem(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_and` instruction.
    pub fn translate_i32_and(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_or` instruction.
    pub fn translate_i32_or(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_xor` instruction.
    pub fn translate_i32_xor(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_shl` instruction.
    pub fn translate_i32_shl(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_shr` instruction.
    pub fn translate_i32_shr(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u32_shr` instruction.
    pub fn translate_u32_shr(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_rotl` instruction.
    pub fn translate_i32_rotl(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_rotr` instruction.
    pub fn translate_i32_rotr(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_clz` instruction.
    pub fn translate_i64_clz(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_ctz` instruction.
    pub fn translate_i64_ctz(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_popcnt` instruction.
    pub fn translate_i64_popcnt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_add` instruction.
    pub fn translate_i64_add(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_sub` instruction.
    pub fn translate_i64_sub(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_mul` instruction.
    pub fn translate_i64_mul(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_div` instruction.
    pub fn translate_i64_div(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u64_div` instruction.
    pub fn translate_u64_div(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_rem` instruction.
    pub fn translate_i64_rem(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u64_rem` instruction.
    pub fn translate_u64_rem(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_and` instruction.
    pub fn translate_i64_and(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_or` instruction.
    pub fn translate_i64_or(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_xor` instruction.
    pub fn translate_i64_xor(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_shl` instruction.
    pub fn translate_i64_shl(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_shr` instruction.
    pub fn translate_i64_shr(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u64_shr` instruction.
    pub fn translate_u64_shr(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_rotl` instruction.
    pub fn translate_i64_rotl(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_rotr` instruction.
    pub fn translate_i64_rotr(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_abs` instruction.
    pub fn translate_f32_abs(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_neg` instruction.
    pub fn translate_f32_neg(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_ceil` instruction.
    pub fn translate_f32_ceil(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_floor` instruction.
    pub fn translate_f32_floor(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_trunc` instruction.
    pub fn translate_f32_trunc(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_nearest` instruction.
    pub fn translate_f32_nearest(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_sqrt` instruction.
    pub fn translate_f32_sqrt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_add` instruction.
    pub fn translate_f32_add(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_sub` instruction.
    pub fn translate_f32_sub(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_mul` instruction.
    pub fn translate_f32_mul(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_div` instruction.
    pub fn translate_f32_div(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_min` instruction.
    pub fn translate_f32_min(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_max` instruction.
    pub fn translate_f32_max(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_copysign` instruction.
    pub fn translate_f32_copysign(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_abs` instruction.
    pub fn translate_f64_abs(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_neg` instruction.
    pub fn translate_f64_neg(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_ceil` instruction.
    pub fn translate_f64_ceil(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_floor` instruction.
    pub fn translate_f64_floor(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_trunc` instruction.
    pub fn translate_f64_trunc(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_nearest` instruction.
    pub fn translate_f64_nearest(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_sqrt` instruction.
    pub fn translate_f64_sqrt(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_add` instruction.
    pub fn translate_f64_add(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_sub` instruction.
    pub fn translate_f64_sub(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_mul` instruction.
    pub fn translate_f64_mul(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_div` instruction.
    pub fn translate_f64_div(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_min` instruction.
    pub fn translate_f64_min(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_max` instruction.
    pub fn translate_f64_max(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_copysign` instruction.
    pub fn translate_f64_copysign(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_wrap_i64` instruction.
    pub fn translate_i32_wrap_i64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_trunc_f32` instruction.
    pub fn translate_i32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u32_trunc_f32` instruction.
    pub fn translate_u32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_trunc_f64` instruction.
    pub fn translate_i32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u32_trunc_f64` instruction.
    pub fn translate_u32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_extend_i32` instruction.
    pub fn translate_i64_extend_i32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u64_extend_i32` instruction.
    pub fn translate_u64_extend_i32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_trunc_f32` instruction.
    pub fn translate_i64_trunc_f32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u64_trunc_f32` instruction.
    pub fn translate_u64_trunc_f32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_trunc_f64` instruction.
    pub fn translate_i64_trunc_f64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `u64_trunc_f64` instruction.
    pub fn translate_u64_trunc_f64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_convert_i32` instruction.
    pub fn translate_f32_convert_i32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_convert_u32` instruction.
    pub fn translate_f32_convert_u32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_convert_i64` instruction.
    pub fn translate_f32_convert_i64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_convert_u64` instruction.
    pub fn translate_f32_convert_u64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_demote_f64` instruction.
    pub fn translate_f32_demote_f64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_convert_i32` instruction.
    pub fn translate_f64_convert_i32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_convert_u32` instruction.
    pub fn translate_f64_convert_u32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_convert_i64` instruction.
    pub fn translate_f64_convert_i64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_convert_u64` instruction.
    pub fn translate_f64_convert_u64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_promote_f32` instruction.
    pub fn translate_f64_promote_f32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i32_reinterpret_f32` instruction.
    pub fn translate_i32_reinterpret_f32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `i64_reinterpret_f64` instruction.
    pub fn translate_i64_reinterpret_f64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f32_reinterpret_i32` instruction.
    pub fn translate_f32_reinterpret_i32(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `f64_reinterpret_i64` instruction.
    pub fn translate_f64_reinterpret_i64(&mut self) -> Result<(), ModuleError> {
        todo!()
    }
}
