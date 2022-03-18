#![allow(dead_code, unused_imports)]

mod control_frame;
mod control_stack;
mod inst_builder;
mod locals_registry;
mod providers;
mod translate;

use core::cmp::PartialOrd;
pub use self::inst_builder::{Instr, LabelIdx, RelativeDepth, Reloc};
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
    inst_builder::InstructionsBuilder,
    locals_registry::LocalsRegistry,
    providers::{Provider, ProviderSlice, ProviderSliceArena, Providers, Register},
};
use super::{
    bytecode::Offset,
    register::RegisterEntry,
    Engine,
    ExecInstruction,
    FromRegisterEntry,
    FuncBody,
    Instruction,
    InstructionTypes,
};
use crate::{
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
    FuncType,
    ModuleError,
    Mutability,
};
use core::ops;
use wasmi_core::{Float, SignExtendFrom, TrapCode, Value, ValueType, F32, F64};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OpaqueTypes {}

impl InstructionTypes for OpaqueTypes {
    type Register = Register;
    type Provider = Provider;
    type ProviderSlice = ProviderSlice;
    type RegisterSlice = ProviderSlice;
}

pub type OpaqueInstruction = Instruction<OpaqueTypes>;

/// TODO: remove again when done
const DUMMY_INSTRUCTION: OpaqueInstruction = Instruction::Trap {
    trap_code: TrapCode::Unreachable,
};

/// TODO: remove again when done
fn make_dummy_instruction(_offset: Offset) -> OpaqueInstruction {
    DUMMY_INSTRUCTION
}

/// The interface to translate a `wasmi` bytecode function using Wasm bytecode.
#[derive(Debug)]
pub struct FunctionBuilder<'parser> {
    /// The [`Engine`] for which the function is translated.
    engine: Engine,
    /// The function under construction.
    func: FuncIdx,
    /// The immutable `wasmi` module resources.
    res: ModuleResources<'parser>,
    /// The control flow frame stack that represents the Wasm control flow.
    control_frames: ControlFlowStack,
    /// The emulated value stack.
    providers: Providers,
    /// Arena for register slices.
    reg_slices: ProviderSliceArena,
    /// The instruction builder.
    ///
    /// # Note
    ///
    /// Allows to incrementally construct the instruction of a function.
    inst_builder: InstructionsBuilder,
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

impl<'parser> FunctionBuilder<'parser> {
    /// Creates a new [`FunctionBuilder`].
    pub fn new(engine: &Engine, func: FuncIdx, res: ModuleResources<'parser>) -> Self {
        let mut inst_builder = InstructionsBuilder::default();
        let mut control_frames = ControlFlowStack::default();
        let mut providers = Providers::default();
        let reg_slices = ProviderSliceArena::default();
        Self::register_func_body_block(func, res, &mut inst_builder, &mut control_frames);
        Self::register_func_params(func, res, &mut providers);
        Self {
            engine: engine.clone(),
            func,
            res,
            control_frames,
            inst_builder,
            reachable: true,
            providers,
            reg_slices,
        }
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
        providers: &mut Providers,
    ) -> usize {
        let dedup_func_type = res.get_type_of_func(func);
        let func_type = res
            .engine()
            .resolve_func_type(dedup_func_type, Clone::clone);
        let params = func_type.params();
        for param_type in params {
            providers.register_locals(*param_type, 1);
        }
        params.len()
    }

    /// Translates the given local variables for the translated function.
    pub fn translate_locals(
        &mut self,
        amount: u32,
        value_type: ValueType,
    ) -> Result<(), ModuleError> {
        self.providers.register_locals(value_type, amount);
        Ok(())
    }

    /// Finishes constructing the function and returns its [`FuncBody`].
    pub fn finish(mut self) -> FuncBody {
        self.inst_builder
            .finish(&self.engine, &self.reg_slices, &self.providers)
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

    /// Compute the provider slice for the return statement.
    ///
    /// # Panics
    ///
    /// - If the control flow frame stack is empty.
    /// - If the value stack is underflown.
    fn return_provider_slice(&mut self) -> ProviderSlice {
        debug_assert!(self.is_reachable());
        assert!(
            !self.control_frames.is_empty(),
            "cannot create return provider slice with an empty stack of frames"
        );
        let func_type = self.res.get_type_of_func(self.func);
        let len_results = self
            .engine
            .resolve_func_type(func_type, |func_type| func_type.results().len());
        let providers = self.providers.pop_n(len_results);
        self.reg_slices.alloc(providers)
    }
}

impl<'parser> FunctionBuilder<'parser> {
    /// Translates a Wasm `unreachable` instruction.
    pub fn translate_unreachable(&mut self) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     builder.inst_builder.push_inst(Instruction::Unreachable);
        //     builder.reachable = false;
        //     Ok(())
        // })
        todo!()
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
        // let len_params = block_type.len_params(self.engine);
        // let stack_height = self.value_stack.len();
        // stack_height.checked_sub(len_params).unwrap_or_else(|| {
        //     panic!(
        //         "encountered emulated value stack underflow with \
        //          stack height {} and {} block parameters",
        //         stack_height, len_params
        //     )
        // })
        todo!()
    }

    /// Translates a Wasm `block` control flow operator.
    pub fn translate_block(&mut self, block_type: BlockType) -> Result<(), ModuleError> {
        // let stack_height = self.frame_stack_height(block_type);
        // if self.is_reachable() {
        //     let end_label = self.inst_builder.new_label();
        //     self.control_frames.push_frame(BlockControlFrame::new(
        //         block_type,
        //         end_label,
        //         stack_height,
        //     ));
        // } else {
        //     self.control_frames.push_frame(UnreachableControlFrame::new(
        //         ControlFrameKind::Block,
        //         block_type,
        //         stack_height,
        //     ));
        // }
        // Ok(())
        todo!()
    }

    /// Translates a Wasm `loop` control flow operator.
    pub fn translate_loop(&mut self, block_type: BlockType) -> Result<(), ModuleError> {
        // let stack_height = self.frame_stack_height(block_type);
        // if self.is_reachable() {
        //     let header = self.inst_builder.new_label();
        //     self.inst_builder.resolve_label(header);
        //     self.control_frames
        //         .push_frame(LoopControlFrame::new(block_type, header, stack_height));
        // } else {
        //     self.control_frames.push_frame(UnreachableControlFrame::new(
        //         ControlFrameKind::Loop,
        //         block_type,
        //         stack_height,
        //     ));
        // }
        // Ok(())
        todo!()
    }

    /// Translates a Wasm `if` control flow operator.
    pub fn translate_if(&mut self, block_type: BlockType) -> Result<(), ModuleError> {
        // if self.is_reachable() {
        //     let condition = self.value_stack.pop1();
        //     debug_assert_eq!(condition, ValueType::I32);
        //     let stack_height = self.frame_stack_height(block_type);
        //     let else_label = self.inst_builder.new_label();
        //     let end_label = self.inst_builder.new_label();
        //     self.control_frames.push_frame(IfControlFrame::new(
        //         block_type,
        //         end_label,
        //         else_label,
        //         stack_height,
        //     ));
        //     let dst_pc = self.try_resolve_label(else_label, |pc| Reloc::Br { inst_idx: pc });
        //     let branch_target = Target::new(dst_pc, DropKeep::new(0, 0));
        //     self.inst_builder
        //         .push_inst(Instruction::BrIfEqz(branch_target));
        // } else {
        //     let stack_height = self.frame_stack_height(block_type);
        //     self.control_frames.push_frame(UnreachableControlFrame::new(
        //         ControlFrameKind::If,
        //         block_type,
        //         stack_height,
        //     ));
        // }
        // Ok(())
        todo!()
    }

    /// Translates a Wasm `else` control flow operator.
    pub fn translate_else(&mut self) -> Result<(), ModuleError> {
        // let mut if_frame = match self.control_frames.pop_frame() {
        //     ControlFrame::If(if_frame) => if_frame,
        //     ControlFrame::Unreachable(frame) if matches!(frame.kind(), ControlFrameKind::If) => {
        //         // Encountered `Else` block for unreachable `If` block.
        //         //
        //         // In this case we can simply ignore the entire `Else` block
        //         // since it is unreachable anyways.
        //         self.control_frames.push_frame(frame);
        //         return Ok(());
        //     }
        //     unexpected => panic!(
        //         "expected `if` control flow frame on top for `else` but found: {:?}",
        //         unexpected,
        //     ),
        // };
        // let reachable = self.is_reachable();
        // // At this point we know if the end of the `then` block of the paren
        // // `if` block is reachable so we update the parent `if` frame.
        // //
        // // Note: This information is important to decide whether code is
        // //       reachable after the `if` block (including `else`) ends.
        // if_frame.update_end_of_then_reachability(reachable);
        // // Create the jump from the end of the `then` block to the `if`
        // // block's end label in case the end of `then` is reachable.
        // if reachable {
        //     let dst_pc =
        //         self.try_resolve_label(if_frame.end_label(), |pc| Reloc::Br { inst_idx: pc });
        //     let target = Target::new(dst_pc, DropKeep::new(0, 0));
        //     self.inst_builder.push_inst(Instruction::Br(target));
        // }
        // // Now resolve labels for the instructions of the `else` block
        // self.inst_builder.resolve_label(if_frame.else_label());
        // // We need to reset the value stack to exactly how it has been
        // // when entering the `if` in the first place so that the `else`
        // // block has the same parameters on top of the stack.
        // self.value_stack.shrink_to(if_frame.stack_height());
        // if_frame.block_type().foreach_param(self.engine, |param| {
        //     self.value_stack.push(param);
        // });
        // self.control_frames.push_frame(if_frame);
        // // We can reset reachability now since the parent `if` block was reachable.
        // self.reachable = true;
        // Ok(())
        todo!()
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
        self.providers.shrink_to(frame_stack_height);
        let frame = self.control_frames.pop_frame();
        frame.block_type().foreach_result(&self.engine, |_result| {
            self.providers.push_dynamic();
        });
        Ok(())
    }

    /// Translates a Wasm `br` control flow operator.
    pub fn translate_br(&mut self, relative_depth: u32) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     match builder.acquire_target(relative_depth) {
        //         AquiredTarget::Branch(end_label, drop_keep) => {
        //             let dst_pc =
        //                 builder.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
        //             builder
        //                 .inst_builder
        //                 .push_inst(Instruction::Br(Target::new(dst_pc, drop_keep)));
        //         }
        //         AquiredTarget::Return(drop_keep) => {
        //             // In this case the `br` can be directly translated as `return`.
        //             builder.translate_return()?;
        //         }
        //     }
        //     builder.reachable = false;
        //     Ok(())
        // })
        todo!()
    }

    /// Translates a Wasm `br_if` control flow operator.
    pub fn translate_br_if(&mut self, relative_depth: u32) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     let condition = builder.value_stack.pop1();
        //     debug_assert_eq!(condition, ValueType::I32);
        //     match builder.acquire_target(relative_depth) {
        //         AquiredTarget::Branch(end_label, drop_keep) => {
        //             let dst_pc =
        //                 builder.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
        //             builder
        //                 .inst_builder
        //                 .push_inst(Instruction::BrIfNez(Target::new(dst_pc, drop_keep)));
        //         }
        //         AquiredTarget::Return(drop_keep) => {
        //             builder
        //                 .inst_builder
        //                 .push_inst(Instruction::ReturnIfNez(drop_keep));
        //         }
        //     }
        //     Ok(())
        // })
        todo!()
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
        // self.translate_if_reachable(|builder| {
        //     let case = builder.value_stack.pop1();
        //     debug_assert_eq!(case, ValueType::I32);

        //     fn compute_inst(
        //         builder: &mut FunctionBuilder,
        //         n: usize,
        //         depth: RelativeDepth,
        //     ) -> Instruction {
        //         match builder.acquire_target(depth.into_u32()) {
        //             AquiredTarget::Branch(label_idx, drop_keep) => {
        //                 let dst_pc = builder.try_resolve_label(label_idx, |pc| Reloc::BrTable {
        //                     inst_idx: pc,
        //                     target_idx: n,
        //                 });
        //                 Instruction::Br(Target::new(dst_pc, drop_keep))
        //             }
        //             AquiredTarget::Return(drop_keep) => Instruction::Return(drop_keep),
        //         }
        //     }

        //     let branches = targets
        //         .into_iter()
        //         .enumerate()
        //         .map(|(n, depth)| compute_inst(builder, n, depth))
        //         .collect::<Vec<_>>();
        //     // We include the default target in `len_branches`.
        //     let len_branches = branches.len();
        //     let default_branch = compute_inst(builder, len_branches, default);
        //     builder.inst_builder.push_inst(Instruction::BrTable {
        //         len_targets: len_branches + 1,
        //     });
        //     for branch in branches {
        //         builder.inst_builder.push_inst(branch);
        //     }
        //     builder.inst_builder.push_inst(default_branch);
        //     builder.reachable = false;
        //     Ok(())
        // })
        todo!()
    }

    /// Translates a Wasm `return` control flow operator.
    pub fn translate_return(&mut self) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let results = builder.return_provider_slice();
            builder
                .inst_builder
                .push_inst(Instruction::Return { results });
            builder.reachable = false;
            Ok(())
        })
    }

    /// Adjusts the emulated [`ValueStack`] given the [`FuncType`] of the call.
    fn adjust_value_stack_for_call(&mut self, func_type: &FuncType) {
        // let (params, results) = func_type.params_results();
        // for param in params.iter().rev() {
        //     let popped = self.value_stack.pop1();
        //     debug_assert_eq!(popped, *param);
        // }
        // for result in results {
        //     self.value_stack.push(*result);
        // }
        todo!()
    }

    /// Translates a Wasm `call` instruction.
    pub fn translate_call(&mut self, func_idx: FuncIdx) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     let func_type = builder.func_type_of(func_idx);
        //     builder.adjust_value_stack_for_call(&func_type);
        //     let func_idx = func_idx.into_u32().into();
        //     builder.inst_builder.push_inst(Instruction::Call(func_idx));
        //     Ok(())
        // })
        todo!()
    }

    /// Translates a Wasm `call_indirect` instruction.
    pub fn translate_call_indirect(
        &mut self,
        func_type_idx: FuncTypeIdx,
        table_idx: TableIdx,
    ) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     /// The default Wasm MVP table index.
        //     const DEFAULT_TABLE_INDEX: u32 = 0;
        //     assert_eq!(table_idx.into_u32(), DEFAULT_TABLE_INDEX);
        //     let func_type_offset = builder.value_stack.pop1();
        //     debug_assert_eq!(func_type_offset, ValueType::I32);
        //     let func_type = builder.func_type_at(func_type_idx);
        //     builder.adjust_value_stack_for_call(&func_type);
        //     let func_type_idx = func_type_idx.into_u32().into();
        //     builder
        //         .inst_builder
        //         .push_inst(Instruction::CallIndirect(func_type_idx));
        //     Ok(())
        // })
        todo!()
    }

    /// Translates a Wasm `drop` instruction.
    pub fn translate_drop(&mut self) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     builder.value_stack.pop1();
        //     builder.inst_builder.push_inst(Instruction::Drop);
        //     Ok(())
        // })
        todo!()
    }

    /// Translates a Wasm `select` instruction.
    pub fn translate_select(&mut self) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     let (v0, v1, selector) = builder.value_stack.pop3();
        //     debug_assert_eq!(selector, ValueType::I32);
        //     debug_assert_eq!(v0, v1);
        //     builder.value_stack.push(v0);
        //     builder.inst_builder.push_inst(Instruction::Select);
        //     Ok(())
        // })
        todo!()
    }

    /// Translate a Wasm `local.get` instruction.
    pub fn translate_local_get(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            builder.providers.push_local(local_idx);
            Ok(())
        })
    }

    /// Translate a Wasm `local.set` instruction.
    pub fn translate_local_set(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     let actual = builder.value_stack.pop1();
        //     let local_depth = builder.relative_local_depth(local_idx);
        //     builder
        //         .inst_builder
        //         .push_inst(Instruction::local_set(local_depth));
        //     let expected = builder
        //         .locals
        //         .resolve_local(local_idx)
        //         .unwrap_or_else(|| panic!("failed to resolve local {}", local_idx));
        //     debug_assert_eq!(actual, expected);
        //     Ok(())
        // })
        todo!()
    }

    /// Translate a Wasm `local.tee` instruction.
    pub fn translate_local_tee(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     let local_depth = builder.relative_local_depth(local_idx);
        //     builder
        //         .inst_builder
        //         .push_inst(Instruction::local_tee(local_depth));
        //     let expected = builder
        //         .locals
        //         .resolve_local(local_idx)
        //         .unwrap_or_else(|| panic!("failed to resolve local {}", local_idx));
        //     let actual = builder.value_stack.top();
        //     debug_assert_eq!(actual, expected);
        //     Ok(())
        // })
        todo!()
    }

    /// Translate a Wasm `global.get` instruction.
    pub fn translate_global_get(&mut self, global_idx: GlobalIdx) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     let global_type = builder.res.get_type_of_global(global_idx);
        //     builder.value_stack.push(global_type.value_type());
        //     let global_idx = global_idx.into_u32().into();
        //     builder
        //         .inst_builder
        //         .push_inst(Instruction::GetGlobal(global_idx));
        //     Ok(())
        // })
        todo!()
    }

    /// Translate a Wasm `global.set` instruction.
    pub fn translate_global_set(&mut self, global_idx: GlobalIdx) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     let global_type = builder.res.get_type_of_global(global_idx);
        //     debug_assert_eq!(global_type.mutability(), Mutability::Mutable);
        //     let expected = global_type.value_type();
        //     let actual = builder.value_stack.pop1();
        //     debug_assert_eq!(actual, expected);
        //     let global_idx = global_idx.into_u32().into();
        //     builder
        //         .inst_builder
        //         .push_inst(Instruction::SetGlobal(global_idx));
        //     Ok(())
        // })
        todo!()
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
        make_inst: fn(Offset) -> OpaqueInstruction,
    ) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
        //     let pointer = builder.value_stack.pop1();
        //     debug_assert_eq!(pointer, ValueType::I32);
        //     builder.value_stack.push(loaded_type);
        //     let offset = Offset::from(offset);
        //     builder.inst_builder.push_inst(make_inst(offset));
        //     Ok(())
        // })
        todo!()
    }

    /// Translate a Wasm `i32.load` instruction.
    pub fn translate_i32_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I32,
            make_dummy_instruction, /* Instruction::I32Load */
        )
    }

    /// Translate a Wasm `i64.load` instruction.
    pub fn translate_i64_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I64,
            make_dummy_instruction, /* Instruction::I64Load */
        )
    }

    /// Translate a Wasm `f32.load` instruction.
    pub fn translate_f32_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::F32,
            make_dummy_instruction, /* Instruction::F32Load */
        )
    }

    /// Translate a Wasm `f64.load` instruction.
    pub fn translate_f64_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::F64,
            make_dummy_instruction, /* Instruction::F64Load */
        )
    }

    /// Translate a Wasm `i32.load_i8` instruction.
    pub fn translate_i32_load_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I32,
            make_dummy_instruction, /* Instruction::I32Load8S */
        )
    }

    /// Translate a Wasm `i32.load_u8` instruction.
    pub fn translate_i32_load_u8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I32,
            make_dummy_instruction, /* Instruction::I32Load8U */
        )
    }

    /// Translate a Wasm `i32.load_i16` instruction.
    pub fn translate_i32_load_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I32,
            make_dummy_instruction, /* Instruction::I32Load16S */
        )
    }

    /// Translate a Wasm `i32.load_u16` instruction.
    pub fn translate_i32_load_u16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I32,
            make_dummy_instruction, /* Instruction::I32Load16U */
        )
    }

    /// Translate a Wasm `i64.load_i8` instruction.
    pub fn translate_i64_load_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I64,
            make_dummy_instruction, /* Instruction::I64Load8S */
        )
    }

    /// Translate a Wasm `i64.load_u8` instruction.
    pub fn translate_i64_load_u8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I64,
            make_dummy_instruction, /* Instruction::I64Load8U */
        )
    }

    /// Translate a Wasm `i64.load_i16` instruction.
    pub fn translate_i64_load_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I64,
            make_dummy_instruction, /* Instruction::I64Load16S */
        )
    }

    /// Translate a Wasm `i64.load_u16` instruction.
    pub fn translate_i64_load_u16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I64,
            make_dummy_instruction, /* Instruction::I64Load16U */
        )
    }

    /// Translate a Wasm `i64.load_i32` instruction.
    pub fn translate_i64_load_i32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I64,
            make_dummy_instruction, /* Instruction::I64Load32S */
        )
    }

    /// Translate a Wasm `i64.load_u32` instruction.
    pub fn translate_i64_load_u32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(
            memory_idx,
            offset,
            ValueType::I64,
            make_dummy_instruction, /* Instruction::I64Load32U */
        )
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
        make_inst: fn(Offset) -> OpaqueInstruction,
    ) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
        //     let (pointer, stored) = builder.value_stack.pop2();
        //     debug_assert_eq!(pointer, ValueType::I32);
        //     assert_eq!(stored_value, stored);
        //     let offset = Offset::from(offset);
        //     builder.inst_builder.push_inst(make_inst(offset));
        //     Ok(())
        // })
        todo!()
    }

    /// Translate a Wasm `i32.store` instruction.
    pub fn translate_i32_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(
            memory_idx,
            offset,
            ValueType::I32,
            make_dummy_instruction, /* Instruction::I32Store */
        )
    }

    /// Translate a Wasm `i64.store` instruction.
    pub fn translate_i64_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(
            memory_idx,
            offset,
            ValueType::I64,
            make_dummy_instruction, /* Instruction::I64Store */
        )
    }

    /// Translate a Wasm `f32.store` instruction.
    pub fn translate_f32_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(
            memory_idx,
            offset,
            ValueType::F32,
            make_dummy_instruction, /* Instruction::F32Store */
        )
    }

    /// Translate a Wasm `f64.store` instruction.
    pub fn translate_f64_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(
            memory_idx,
            offset,
            ValueType::F64,
            make_dummy_instruction, /* Instruction::F64Store */
        )
    }

    /// Translate a Wasm `i32.store_i8` instruction.
    pub fn translate_i32_store_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(
            memory_idx,
            offset,
            ValueType::I32,
            make_dummy_instruction, /* Instruction::I32Store8 */
        )
    }

    /// Translate a Wasm `i32.store_i16` instruction.
    pub fn translate_i32_store_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(
            memory_idx,
            offset,
            ValueType::I32,
            make_dummy_instruction, /* Instruction::I32Store16 */
        )
    }

    /// Translate a Wasm `i64.store_i8` instruction.
    pub fn translate_i64_store_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(
            memory_idx,
            offset,
            ValueType::I64,
            make_dummy_instruction, /* Instruction::I64Store8 */
        )
    }

    /// Translate a Wasm `i64.store_i16` instruction.
    pub fn translate_i64_store_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(
            memory_idx,
            offset,
            ValueType::I64,
            make_dummy_instruction, /* Instruction::I64Store16 */
        )
    }

    /// Translate a Wasm `i64.store_i32` instruction.
    pub fn translate_i64_store_i32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(
            memory_idx,
            offset,
            ValueType::I64,
            make_dummy_instruction, /* Instruction::I64Store32 */
        )
    }

    /// Translate a Wasm `memory.size` instruction.
    pub fn translate_memory_size(&mut self, memory_idx: MemoryIdx) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
        //     builder.value_stack.push(ValueType::I32);
        //     builder.inst_builder.push_inst(Instruction::CurrentMemory);
        //     Ok(())
        // })
        todo!()
    }

    /// Translate a Wasm `memory.grow` instruction.
    pub fn translate_memory_grow(&mut self, memory_idx: MemoryIdx) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
        //     debug_assert_eq!(builder.value_stack.top(), ValueType::I32);
        //     builder.inst_builder.push_inst(Instruction::GrowMemory);
        //     Ok(())
        // })
        todo!()
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
            builder.providers.push_const(value);
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

    /// Translate a Wasm `i32.eqz` instruction.
    pub fn translate_i32_eqz(&mut self) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            builder.translate_const(0_i32)?;
            builder.translate_i32_eq()?;
            Ok(())
        })
    }

    /// Translate a Wasm binary comparison instruction.
    ///
    /// # Note
    ///
    /// This is used to translate the following Wasm instructions:
    ///
    /// - `{i32, i64}.eqz`
    /// - `{i32, i64, f32, f64}.eq`
    /// - `{i32, i64, f32, f64}.ne`
    fn translate_binary_cmp<T, F>(&mut self, make_op: T, exec_op: F) -> Result<(), ModuleError>
    where
        T: FnOnce(Register, Register, Provider) -> OpaqueInstruction,
        F: FnOnce(RegisterEntry, RegisterEntry) -> bool,
    {
        self.translate_if_reachable(|builder| {
            let (lhs, rhs) = builder.providers.pop2();
            let result = builder.providers.push_dynamic();
            match (lhs, rhs) {
                (Provider::Register(lhs), rhs @ Provider::Register(_))
                | (Provider::Register(lhs), rhs @ Provider::Immediate(_)) => {
                    builder.inst_builder.push_inst(make_op(result, lhs, rhs));
                }
                (lhs @ Provider::Immediate(_), Provider::Register(rhs)) => {
                    // Note: we can swap the operands to avoid having to copy `rhs`.
                    builder.inst_builder.push_inst(make_op(result, rhs, lhs));
                }
                (Provider::Immediate(lhs), Provider::Immediate(rhs)) => {
                    // Note: precompute result and push onto provider stack
                    let lhs = RegisterEntry::from(lhs);
                    let rhs = RegisterEntry::from(rhs);
                    let result = RegisterEntry::from(exec_op(lhs, rhs)).with_type(ValueType::I32);
                    builder.providers.push_const(result);
                }
            };
            Ok(())
        })
    }

    /// Translate a Wasm `i32.eq` instruction.
    pub fn translate_i32_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(
            |result, lhs, rhs| Instruction::I32Eq { result, lhs, rhs },
            |lhs, rhs| i32::from_stack_entry(lhs) == i32::from_stack_entry(rhs),
        )
    }

    /// Translate a Wasm `i32.ne` instruction.
    pub fn translate_i32_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(
            |result, lhs, rhs| Instruction::I32Ne { result, lhs, rhs },
            |lhs, rhs| i32::from_stack_entry(lhs) != i32::from_stack_entry(rhs),
        )
    }

    /// Translate a Wasm binary ordering instruction.
    ///
    /// # Note
    ///
    /// This is used to translate the following Wasm instructions:
    ///
    /// - `{i32, u32, i64, u64, f32, f64}.lt`
    /// - `{i32, u32, i64, u64, f32, f64}.le`
    /// - `{i32, u32, i64, u64, f32, f64}.gt`
    /// - `{i32, u32, i64, u64, f32, f64}.ge`
    fn translate_ord<O1, O2, E, T>(
        &mut self,
        make_op: O1,
        swap_op: O2,
        exec_op: E,
    ) -> Result<(), ModuleError>
    where
        O1: FnOnce(Register, Register, Provider) -> OpaqueInstruction,
        O2: FnOnce(Register, Register, Provider) -> OpaqueInstruction,
        E: FnOnce(T, T) -> bool,
        T: FromRegisterEntry,
    {
        self.translate_if_reachable(|builder| {
            let (lhs, rhs) = builder.providers.pop2();
            let result = builder.providers.push_dynamic();
            match (lhs, rhs) {
                (Provider::Register(lhs), rhs @ Provider::Register(_))
                | (Provider::Register(lhs), rhs @ Provider::Immediate(_)) => {
                    builder.inst_builder.push_inst(make_op(result, lhs, rhs));
                }
                (lhs @ Provider::Immediate(_), Provider::Register(rhs)) => {
                    // Note: we can swap the operands to avoid having to copy `rhs`.
                    builder.inst_builder.push_inst(swap_op(result, rhs, lhs));
                }
                (Provider::Immediate(lhs), Provider::Immediate(rhs)) => {
                    // Note: precompute result and push onto provider stack
                    let lhs = T::from_stack_entry(RegisterEntry::from(lhs));
                    let rhs = T::from_stack_entry(RegisterEntry::from(rhs));
                    let result = RegisterEntry::from(exec_op(lhs, rhs)).with_type(ValueType::I32);
                    builder.providers.push_const(result);
                }
            };
            Ok(())
        })
    }

    /// Translate a Wasm `i32.lt` instruction.
    pub fn translate_i32_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I32LtS { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I32GtS { result, lhs, rhs },
            |lhs: i32, rhs: i32| lhs < rhs,
        )
    }

    /// Translate a Wasm `u32.lt` instruction.
    pub fn translate_u32_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I32LtU { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I32GtU { result, lhs, rhs },
            |lhs: u32, rhs: u32| lhs < rhs,
        )
    }

    /// Translate a Wasm `i32.gt` instruction.
    pub fn translate_i32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I32GtS { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I32LtS { result, lhs, rhs },
            |lhs: i32, rhs: i32| lhs > rhs,
        )
    }

    /// Translate a Wasm `u32.gt` instruction.
    pub fn translate_u32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I32GtU { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I32LtU { result, lhs, rhs },
            |lhs: u32, rhs: u32| lhs > rhs,
        )
    }

    /// Translate a Wasm `i32.le` instruction.
    pub fn translate_i32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I32LeS { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I32GeS { result, lhs, rhs },
            |lhs: i32, rhs: i32| lhs <= rhs,
        )
    }

    /// Translate a Wasm `u32.le` instruction.
    pub fn translate_u32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I32LeU { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I32GeU { result, lhs, rhs },
            |lhs: u32, rhs: u32| lhs <= rhs,
        )
    }

    /// Translate a Wasm `i32.ge` instruction.
    pub fn translate_i32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I32GeS { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I32LeS { result, lhs, rhs },
            |lhs: i32, rhs: i32| lhs >= rhs,
        )
    }

    /// Translate a Wasm `u32.ge` instruction.
    pub fn translate_u32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I32GeU { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I32LeU { result, lhs, rhs },
            |lhs: u32, rhs: u32| lhs >= rhs,
        )
    }

    /// Translate a Wasm `i64.eqz` instruction.
    pub fn translate_i64_eqz(&mut self) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            builder.translate_const(0_i64)?;
            builder.translate_i64_eq()?;
            Ok(())
        })
    }

    /// Translate a Wasm `i64.eq` instruction.
    pub fn translate_i64_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(
            |result, lhs, rhs| Instruction::I64Eq { result, lhs, rhs },
            |lhs, rhs| i64::from_stack_entry(lhs) == i64::from_stack_entry(rhs),
        )
    }

    /// Translate a Wasm `i64.ne` instruction.
    pub fn translate_i64_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(
            |result, lhs, rhs| Instruction::I64Ne { result, lhs, rhs },
            |lhs, rhs| i64::from_stack_entry(lhs) != i64::from_stack_entry(rhs),
        )
    }

    /// Translate a Wasm `i64.lt` instruction.
    pub fn translate_i64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I64LtS { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I64GtS { result, lhs, rhs },
            |lhs: i64, rhs: i64| lhs < rhs,
        )
    }

    /// Translate a Wasm `u64.lt` instruction.
    pub fn translate_u64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I64LtU { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I64GtU { result, lhs, rhs },
            |lhs: u64, rhs: u64| lhs < rhs,
        )
    }

    /// Translate a Wasm `i64.gt` instruction.
    pub fn translate_i64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I64GtS { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I64LtS { result, lhs, rhs },
            |lhs: i64, rhs: i64| lhs > rhs,
        )
    }

    /// Translate a Wasm `u64.gt` instruction.
    pub fn translate_u64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I64GtU { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I64LtU { result, lhs, rhs },
            |lhs: u64, rhs: u64| lhs > rhs,
        )
    }

    /// Translate a Wasm `i64.le` instruction.
    pub fn translate_i64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I64LeS { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I64GeS { result, lhs, rhs },
            |lhs: i64, rhs: i64| lhs <= rhs,
        )
    }

    /// Translate a Wasm `u64.le` instruction.
    pub fn translate_u64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I64LeU { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I64GeU { result, lhs, rhs },
            |lhs: u64, rhs: u64| lhs <= rhs,
        )
    }

    /// Translate a Wasm `i64.ge` instruction.
    pub fn translate_i64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I64GeS { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I64LeS { result, lhs, rhs },
            |lhs: i64, rhs: i64| lhs >= rhs,
        )
    }

    /// Translate a Wasm `u64.ge` instruction.
    pub fn translate_u64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::I64GeU { result, lhs, rhs },
            |result, lhs, rhs| Instruction::I64LeU { result, lhs, rhs },
            |lhs: u64, rhs: u64| lhs >= rhs,
        )
    }

    /// Translate a Wasm `f32.eq` instruction.
    pub fn translate_f32_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(
            |result, lhs, rhs| Instruction::F32Eq { result, lhs, rhs },
            |lhs, rhs| f32::from_stack_entry(lhs) == f32::from_stack_entry(rhs),
        )
    }

    /// Translate a Wasm `f32.ne` instruction.
    pub fn translate_f32_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(
            |result, lhs, rhs| Instruction::F32Ne { result, lhs, rhs },
            |lhs, rhs| f32::from_stack_entry(lhs) != f32::from_stack_entry(rhs),
        )
    }

    /// Translate a Wasm `f32.lt` instruction.
    pub fn translate_f32_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::F32Lt { result, lhs, rhs },
            |result, lhs, rhs| Instruction::F32Gt { result, lhs, rhs },
            |lhs: f32, rhs: f32| lhs < rhs,
        )
    }

    /// Translate a Wasm `f32.gt` instruction.
    pub fn translate_f32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::F32Gt { result, lhs, rhs },
            |result, lhs, rhs| Instruction::F32Lt { result, lhs, rhs },
            |lhs: f32, rhs: f32| lhs > rhs,
        )
    }

    /// Translate a Wasm `f32.le` instruction.
    pub fn translate_f32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::F32Le { result, lhs, rhs },
            |result, lhs, rhs| Instruction::F32Ge { result, lhs, rhs },
            |lhs: f32, rhs: f32| lhs <= rhs,
        )
    }

    /// Translate a Wasm `f32.ge` instruction.
    pub fn translate_f32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::F32Ge { result, lhs, rhs },
            |result, lhs, rhs| Instruction::F32Le { result, lhs, rhs },
            |lhs: f32, rhs: f32| lhs >= rhs,
        )
    }

    /// Translate a Wasm `f64.eq` instruction.
    pub fn translate_f64_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(
            |result, lhs, rhs| Instruction::F64Eq { result, lhs, rhs },
            |lhs, rhs| f64::from_stack_entry(lhs) == f64::from_stack_entry(rhs),
        )
    }

    /// Translate a Wasm `f64.ne` instruction.
    pub fn translate_f64_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(
            |result, lhs, rhs| Instruction::F64Ne { result, lhs, rhs },
            |lhs, rhs| f64::from_stack_entry(lhs) != f64::from_stack_entry(rhs),
        )
    }

    /// Translate a Wasm `f64.lt` instruction.
    pub fn translate_f64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::F64Lt { result, lhs, rhs },
            |result, lhs, rhs| Instruction::F64Gt { result, lhs, rhs },
            |lhs: f64, rhs: f64| lhs < rhs,
        )
    }

    /// Translate a Wasm `f64.gt` instruction.
    pub fn translate_f64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::F64Gt { result, lhs, rhs },
            |result, lhs, rhs| Instruction::F64Lt { result, lhs, rhs },
            |lhs: f64, rhs: f64| lhs > rhs,
        )
    }

    /// Translate a Wasm `f64.le` instruction.
    pub fn translate_f64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::F64Le { result, lhs, rhs },
            |result, lhs, rhs| Instruction::F64Ge { result, lhs, rhs },
            |lhs: f64, rhs: f64| lhs <= rhs,
        )
    }

    /// Translate a Wasm `f64.ge` instruction.
    pub fn translate_f64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(
            |result, lhs, rhs| Instruction::F64Ge { result, lhs, rhs },
            |result, lhs, rhs| Instruction::F64Le { result, lhs, rhs },
            |lhs: f64, rhs: f64| lhs >= rhs,
        )
    }

    /// Translate a unary Wasm instruction.
    ///
    /// # Note
    ///
    /// This is used to translate the following Wasm instructions:
    ///
    /// - `{i32, i64}.clz`
    /// - `{i32, i64}.ctz`
    /// - `{i32, i64}.popcnt`
    /// - `{i32, i64}.extend_8s`
    /// - `{i32, i64}.extend_16s`
    /// - `i64.extend_32s`
    /// - `{f32, f64}.abs`
    /// - `{f32, f64}.neg`
    /// - `{f32, f64}.ceil`
    /// - `{f32, f64}.floor`
    /// - `{f32, f64}.trunc`
    /// - `{f32, f64}.nearest`
    /// - `{f32, f64}.sqrt`
    pub fn translate_unary_operation<F, E, T, R>(
        &mut self,
        make_op: F,
        exec_op: E,
    ) -> Result<(), ModuleError>
    where
        F: FnOnce(Register, Register) -> OpaqueInstruction,
        E: FnOnce(T) -> R,
        T: FromRegisterEntry,
        R: Into<Value>,
    {
        self.translate_if_reachable(|builder| {
            let input = builder.providers.pop();
            match input {
                Provider::Register(input) => {
                    let result = builder.providers.push_dynamic();
                    builder.inst_builder.push_inst(make_op(result, input));
                }
                Provider::Immediate(input) => {
                    let result = exec_op(T::from_stack_entry(input.into()));
                    builder.providers.push_const(result.into());
                }
            }
            Ok(())
        })
    }

    /// Translate a Wasm `i32.clz` instruction.
    pub fn translate_i32_clz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I32Clz { result, input },
            i32::leading_zeros,
        )
    }

    /// Translate a Wasm `i32.ctz` instruction.
    pub fn translate_i32_ctz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I32Ctz { result, input },
            i32::trailing_zeros,
        )
    }

    /// Translate a Wasm `i32.popcnt` instruction.
    pub fn translate_i32_popcnt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I32Popcnt { result, input },
            i32::count_ones,
        )
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
        inst: OpaqueInstruction,
    ) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     let (v0, v1) = builder.value_stack.pop2();
        //     debug_assert_eq!(v0, v1);
        //     debug_assert_eq!(v0, value_type);
        //     builder.value_stack.push(value_type);
        //     builder.inst_builder.push_inst(inst);
        //     Ok(())
        // })
        todo!()
    }

    /// Translates commutative binary Wasm operators.
    ///
    /// This uses the commutativity of the instruction in order to
    /// swap operands in cases where the `wasmi` bytecode would otherwise
    /// not be able to represent the instruction.
    ///
    /// # Note
    ///
    /// This includes the following Wasm instructions:
    ///
    /// - `{i32, i64, f32, f64}.add`
    /// - `{i32, i64, f32, f64}.mul`
    /// - `{i32, i64}.and`
    /// - `{i32, i64}.or`
    /// - `{i32, i64}.xor`
    /// - `{f32, f64}.min`
    /// - `{f32, f64}.max`
    fn translate_commutative_binary_operation<F, E, R>(
        &mut self,
        make_op: F,
        exec_op: E,
    ) -> Result<(), ModuleError>
    where
        F: FnOnce(Register, Register, Provider) -> OpaqueInstruction,
        E: FnOnce(RegisterEntry, RegisterEntry) -> R,
        R: Into<Value>,
    {
        self.translate_if_reachable(|builder| {
            let (lhs, rhs) = builder.providers.pop2();
            let result = builder.providers.push_dynamic();
            match (lhs, rhs) {
                (Provider::Register(lhs), rhs @ Provider::Register(_))
                | (Provider::Register(lhs), rhs @ Provider::Immediate(_)) => {
                    builder.inst_builder.push_inst(make_op(result, lhs, rhs));
                }
                (lhs @ Provider::Immediate(_), Provider::Register(rhs)) => {
                    // Note: due to commutativity of the operation we can swap the parameters.
                    builder.inst_builder.push_inst(make_op(result, rhs, lhs));
                }
                (Provider::Immediate(lhs), Provider::Immediate(rhs)) => {
                    let lhs = RegisterEntry::from(lhs);
                    let rhs = RegisterEntry::from(rhs);
                    let result = exec_op(lhs, rhs);
                    builder.providers.push_const(result.into());
                }
            };
            Ok(())
        })
    }

    /// Translate a Wasm `i32.add` instruction.
    pub fn translate_i32_add(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::I32Add { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = i32::from_stack_entry(lhs);
                let rhs = i32::from_stack_entry(rhs);
                lhs.wrapping_add(rhs)
            },
        )
    }

    /// Translate a Wasm `i32.sub` instruction.
    pub fn translate_i32_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32Sub */
        )
    }

    /// Translate a Wasm `i32.mul` instruction.
    pub fn translate_i32_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::I32Mul { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = i32::from_stack_entry(lhs);
                let rhs = i32::from_stack_entry(rhs);
                lhs.wrapping_mul(rhs)
            },
        )
    }

    /// Translate a Wasm `i32.div` instruction.
    pub fn translate_i32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32DivS */
        )
    }

    /// Translate a Wasm `u32.div` instruction.
    pub fn translate_u32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32DivU */
        )
    }

    /// Translate a Wasm `i32.rem` instruction.
    pub fn translate_i32_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32RemS */
        )
    }

    /// Translate a Wasm `u32.rem` instruction.
    pub fn translate_u32_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32RemU */
        )
    }

    /// Translate a Wasm `i32.and` instruction.
    pub fn translate_i32_and(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::I32And { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = i32::from_stack_entry(lhs);
                let rhs = i32::from_stack_entry(rhs);
                lhs & rhs
            },
        )
    }

    /// Translate a Wasm `i32.or` instruction.
    pub fn translate_i32_or(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::I32Or { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = i32::from_stack_entry(lhs);
                let rhs = i32::from_stack_entry(rhs);
                lhs | rhs
            },
        )
    }

    /// Translate a Wasm `i32.xor` instruction.
    pub fn translate_i32_xor(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::I32Xor { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = i32::from_stack_entry(lhs);
                let rhs = i32::from_stack_entry(rhs);
                lhs ^ rhs
            },
        )
    }

    /// Translate a Wasm `i32.shl` instruction.
    pub fn translate_i32_shl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32Shl */
        )
    }

    /// Translate a Wasm `i32.shr` instruction.
    pub fn translate_i32_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32ShrS */
        )
    }

    /// Translate a Wasm `u32.shr` instruction.
    pub fn translate_u32_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32ShrU */
        )
    }

    /// Translate a Wasm `i32.rotl` instruction.
    pub fn translate_i32_rotl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32Rotl */
        )
    }

    /// Translate a Wasm `i32.rotr` instruction.
    pub fn translate_i32_rotr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32Rotr */
        )
    }

    /// Translate a Wasm `i64.clz` instruction.
    pub fn translate_i64_clz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I64Clz { result, input },
            i64::leading_zeros,
        )
    }

    /// Translate a Wasm `i64.ctz` instruction.
    pub fn translate_i64_ctz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I64Ctz { result, input },
            i64::trailing_zeros,
        )
    }

    /// Translate a Wasm `i64.popcnt` instruction.
    pub fn translate_i64_popcnt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I64Popcnt { result, input },
            i64::count_ones,
        )
    }

    /// Translate a Wasm `i64.add` instruction.
    pub fn translate_i64_add(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::I64Add { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = i64::from_stack_entry(lhs);
                let rhs = i64::from_stack_entry(rhs);
                lhs.wrapping_add(rhs)
            },
        )
    }

    /// Translate a Wasm `i64.sub` instruction.
    pub fn translate_i64_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64Sub */
        )
    }

    /// Translate a Wasm `i64.mul` instruction.
    pub fn translate_i64_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::I64Mul { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = i64::from_stack_entry(lhs);
                let rhs = i64::from_stack_entry(rhs);
                lhs.wrapping_mul(rhs)
            },
        )
    }

    /// Translate a Wasm `i64.div` instruction.
    pub fn translate_i64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64DivS */
        )
    }

    /// Translate a Wasm `u64.div` instruction.
    pub fn translate_u64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64DivU */
        )
    }

    /// Translate a Wasm `i64.rem` instruction.
    pub fn translate_i64_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64RemS */
        )
    }

    /// Translate a Wasm `u64.rem` instruction.
    pub fn translate_u64_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64RemU */
        )
    }

    /// Translate a Wasm `i64.and` instruction.
    pub fn translate_i64_and(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::I64And { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = i64::from_stack_entry(lhs);
                let rhs = i64::from_stack_entry(rhs);
                lhs & rhs
            },
        )
    }

    /// Translate a Wasm `i64.or` instruction.
    pub fn translate_i64_or(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::I64Or { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = i64::from_stack_entry(lhs);
                let rhs = i64::from_stack_entry(rhs);
                lhs | rhs
            },
        )
    }

    /// Translate a Wasm `i64.xor` instruction.
    pub fn translate_i64_xor(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::I64Xor { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = i64::from_stack_entry(lhs);
                let rhs = i64::from_stack_entry(rhs);
                lhs ^ rhs
            },
        )
    }

    /// Translate a Wasm `i64.shl` instruction.
    pub fn translate_i64_shl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64Shl */
        )
    }

    /// Translate a Wasm `i64.shr` instruction.
    pub fn translate_i64_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64ShrS */
        )
    }

    /// Translate a Wasm `u64.shr` instruction.
    pub fn translate_u64_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64ShrU */
        )
    }

    /// Translate a Wasm `i64.rotl` instruction.
    pub fn translate_i64_rotl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64Rotl */
        )
    }

    /// Translate a Wasm `i64.rotr` instruction.
    pub fn translate_i64_rotr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64Rotr */
        )
    }

    /// Translate a Wasm `f32.abs` instruction.
    pub fn translate_f32_abs(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Abs { result, input },
            F32::abs,
        )
    }

    /// Translate a Wasm `f32.neg` instruction.
    pub fn translate_f32_neg(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Neg { result, input },
            <F32 as ops::Neg>::neg,
        )
    }

    /// Translate a Wasm `f32.ceil` instruction.
    pub fn translate_f32_ceil(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Ceil { result, input },
            F32::ceil,
        )
    }

    /// Translate a Wasm `f32.floor` instruction.
    pub fn translate_f32_floor(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Floor { result, input },
            F32::floor,
        )
    }

    /// Translate a Wasm `f32.trunc` instruction.
    pub fn translate_f32_trunc(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Trunc { result, input },
            F32::trunc,
        )
    }

    /// Translate a Wasm `f32.nearest` instruction.
    pub fn translate_f32_nearest(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Nearest { result, input },
            F32::nearest,
        )
    }

    /// Translate a Wasm `f32.sqrt` instruction.
    pub fn translate_f32_sqrt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Sqrt { result, input },
            F32::sqrt,
        )
    }

    /// Translate a Wasm `f32.add` instruction.
    pub fn translate_f32_add(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::F32Add { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = F32::from_stack_entry(lhs);
                let rhs = F32::from_stack_entry(rhs);
                lhs + rhs
            },
        )
    }

    /// Translate a Wasm `f32.sub` instruction.
    pub fn translate_f32_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::F32,
            DUMMY_INSTRUCTION, /* Instruction::F32Sub */
        )
    }

    /// Translate a Wasm `f32.mul` instruction.
    pub fn translate_f32_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::F32Mul { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = F32::from_stack_entry(lhs);
                let rhs = F32::from_stack_entry(rhs);
                lhs * rhs
            },
        )
    }

    /// Translate a Wasm `f32.div` instruction.
    pub fn translate_f32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::F32,
            DUMMY_INSTRUCTION, /* Instruction::F32Div */
        )
    }

    /// Translate a Wasm `f32.min` instruction.
    pub fn translate_f32_min(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::F32Min { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = F32::from_stack_entry(lhs);
                let rhs = F32::from_stack_entry(rhs);
                lhs.min(rhs)
            },
        )
    }

    /// Translate a Wasm `f32.max` instruction.
    pub fn translate_f32_max(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::F32Max { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = F32::from_stack_entry(lhs);
                let rhs = F32::from_stack_entry(rhs);
                lhs.max(rhs)
            },
        )
    }

    /// Translate a Wasm `f32.copysign` instruction.
    pub fn translate_f32_copysign(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::F32,
            DUMMY_INSTRUCTION, /* Instruction::F32Copysign */
        )
    }

    /// Translate a Wasm `f64.abs` instruction.
    pub fn translate_f64_abs(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Abs { result, input },
            F64::abs,
        )
    }

    /// Translate a Wasm `f64.neg` instruction.
    pub fn translate_f64_neg(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Neg { result, input },
            <F64 as ops::Neg>::neg,
        )
    }

    /// Translate a Wasm `f64.ceil` instruction.
    pub fn translate_f64_ceil(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Ceil { result, input },
            F64::ceil,
        )
    }

    /// Translate a Wasm `f64.floor` instruction.
    pub fn translate_f64_floor(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Floor { result, input },
            F64::floor,
        )
    }

    /// Translate a Wasm `f64.trunc` instruction.
    pub fn translate_f64_trunc(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Trunc { result, input },
            F64::trunc,
        )
    }

    /// Translate a Wasm `f64.nearest` instruction.
    pub fn translate_f64_nearest(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Nearest { result, input },
            F64::nearest,
        )
    }

    /// Translate a Wasm `f64.sqrt` instruction.
    pub fn translate_f64_sqrt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Sqrt { result, input },
            F64::sqrt,
        )
    }

    /// Translate a Wasm `f64.add` instruction.
    pub fn translate_f64_add(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::F64Add { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = F64::from_stack_entry(lhs);
                let rhs = F64::from_stack_entry(rhs);
                lhs + rhs
            },
        )
    }

    /// Translate a Wasm `f64.sub` instruction.
    pub fn translate_f64_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::F64,
            DUMMY_INSTRUCTION, /* Instruction::F64Sub */
        )
    }

    /// Translate a Wasm `f64.mul` instruction.
    pub fn translate_f64_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::F64Mul { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = F64::from_stack_entry(lhs);
                let rhs = F64::from_stack_entry(rhs);
                lhs * rhs
            },
        )
    }

    /// Translate a Wasm `f64.div` instruction.
    pub fn translate_f64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::F64,
            DUMMY_INSTRUCTION, /* Instruction::F64Div */
        )
    }

    /// Translate a Wasm `f64.min` instruction.
    pub fn translate_f64_min(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::F64Min { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = F64::from_stack_entry(lhs);
                let rhs = F64::from_stack_entry(rhs);
                lhs.min(rhs)
            },
        )
    }

    /// Translate a Wasm `f64.max` instruction.
    pub fn translate_f64_max(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(
            |result, lhs, rhs| Instruction::F64Max { result, lhs, rhs },
            |lhs, rhs| {
                let lhs = F64::from_stack_entry(lhs);
                let rhs = F64::from_stack_entry(rhs);
                lhs.max(rhs)
            },
        )
    }

    /// Translate a Wasm `f64.copysign` instruction.
    pub fn translate_f64_copysign(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(
            ValueType::F64,
            DUMMY_INSTRUCTION, /* Instruction::F64Copysign */
        )
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
        inst: OpaqueInstruction,
    ) -> Result<(), ModuleError> {
        // self.translate_if_reachable(|builder| {
        //     let input = builder.value_stack.pop1();
        //     debug_assert_eq!(input, input_type);
        //     builder.value_stack.push(output_type);
        //     builder.inst_builder.push_inst(inst);
        //     Ok(())
        // })
        todo!()
    }

    /// Translate a Wasm `i32.wrap_i64` instruction.
    pub fn translate_i32_wrap_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I64,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32WrapI64 */
        )
    }

    /// Translate a Wasm `i32.trunc_f32` instruction.
    pub fn translate_i32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F32,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32TruncSF32 */
        )
    }

    /// Translate a Wasm `u32.trunc_f32` instruction.
    pub fn translate_u32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F32,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32TruncUF32 */
        )
    }

    /// Translate a Wasm `i32.trunc_f64` instruction.
    pub fn translate_i32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F64,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32TruncSF64 */
        )
    }

    /// Translate a Wasm `u32.trunc_f64` instruction.
    pub fn translate_u32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F64,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32TruncUF64 */
        )
    }

    /// Translate a Wasm `i64.extend_i32` instruction.
    pub fn translate_i64_extend_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I32,
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64ExtendSI32 */
        )
    }

    /// Translate a Wasm `u64.extend_i32` instruction.
    pub fn translate_u64_extend_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I32,
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64ExtendUI32 */
        )
    }

    /// Translate a Wasm `i64.trunc_f32` instruction.
    pub fn translate_i64_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F32,
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64TruncSF32 */
        )
    }

    /// Translate a Wasm `u64.trunc_f32` instruction.
    pub fn translate_u64_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F32,
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64TruncUF32 */
        )
    }

    /// Translate a Wasm `i64.trunc_f64` instruction.
    pub fn translate_i64_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F64,
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64TruncSF64 */
        )
    }

    /// Translate a Wasm `u64.trunc_f64` instruction.
    pub fn translate_u64_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F64,
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64TruncUF64 */
        )
    }

    /// Translate a Wasm `f32.convert_i32` instruction.
    pub fn translate_f32_convert_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I32,
            ValueType::F32,
            DUMMY_INSTRUCTION, /* Instruction::F32ConvertSI32 */
        )
    }

    /// Translate a Wasm `f32.convert_u32` instruction.
    pub fn translate_f32_convert_u32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I32,
            ValueType::F32,
            DUMMY_INSTRUCTION, /* Instruction::F32ConvertUI32 */
        )
    }

    /// Translate a Wasm `f32.convert_i64` instruction.
    pub fn translate_f32_convert_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I64,
            ValueType::F32,
            DUMMY_INSTRUCTION, /* Instruction::F32ConvertSI64 */
        )
    }

    /// Translate a Wasm `f32.convert_u64` instruction.
    pub fn translate_f32_convert_u64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I64,
            ValueType::F32,
            DUMMY_INSTRUCTION, /* Instruction::F32ConvertUI64 */
        )
    }

    /// Translate a Wasm `f32.demote_f64` instruction.
    pub fn translate_f32_demote_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F64,
            ValueType::F32,
            DUMMY_INSTRUCTION, /* Instruction::F32DemoteF64 */
        )
    }

    /// Translate a Wasm `f64.convert_i32` instruction.
    pub fn translate_f64_convert_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I32,
            ValueType::F64,
            DUMMY_INSTRUCTION, /* Instruction::F64ConvertSI32 */
        )
    }

    /// Translate a Wasm `f64.convert_u32` instruction.
    pub fn translate_f64_convert_u32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I32,
            ValueType::F64,
            DUMMY_INSTRUCTION, /* Instruction::F64ConvertUI32 */
        )
    }

    /// Translate a Wasm `f64.convert_i64` instruction.
    pub fn translate_f64_convert_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I64,
            ValueType::F64,
            DUMMY_INSTRUCTION, /* Instruction::F64ConvertSI64 */
        )
    }

    /// Translate a Wasm `f64.convert_u64` instruction.
    pub fn translate_f64_convert_u64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I64,
            ValueType::F64,
            DUMMY_INSTRUCTION, /* Instruction::F64ConvertUI64 */
        )
    }

    /// Translate a Wasm `f64.promote_f32` instruction.
    pub fn translate_f64_promote_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F32,
            ValueType::F64,
            DUMMY_INSTRUCTION, /* Instruction::F64PromoteF32 */
        )
    }

    /// Translate a Wasm `i32.reinterpret_f32` instruction.
    pub fn translate_i32_reinterpret_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F32,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32ReinterpretF32 */
        )
    }

    /// Translate a Wasm `i64.reinterpret_f64` instruction.
    pub fn translate_i64_reinterpret_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F64,
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64ReinterpretF64 */
        )
    }

    /// Translate a Wasm `f32.reinterpret_i32` instruction.
    pub fn translate_f32_reinterpret_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I32,
            ValueType::F32,
            DUMMY_INSTRUCTION, /* Instruction::F32ReinterpretI32 */
        )
    }

    /// Translate a Wasm `f64.reinterpret_i64` instruction.
    pub fn translate_f64_reinterpret_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::I64,
            ValueType::F64,
            DUMMY_INSTRUCTION, /* Instruction::F64ReinterpretI64 */
        )
    }

    /// Translate a Wasm `i32.extend_8s` instruction.
    pub fn translate_i32_sign_extend8(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I32Extend8S { result, input },
            <i32 as SignExtendFrom<i8>>::sign_extend_from,
        )
    }

    /// Translate a Wasm `i32.extend_16s` instruction.
    pub fn translate_i32_sign_extend16(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I32Extend16S { result, input },
            <i32 as SignExtendFrom<i16>>::sign_extend_from,
        )
    }

    /// Translate a Wasm `i64.extend_8s` instruction.
    pub fn translate_i64_sign_extend8(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I64Extend8S { result, input },
            <i64 as SignExtendFrom<i8>>::sign_extend_from,
        )
    }

    /// Translate a Wasm `i64.extend_16s` instruction.
    pub fn translate_i64_sign_extend16(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I64Extend16S { result, input },
            <i64 as SignExtendFrom<i16>>::sign_extend_from,
        )
    }

    /// Translate a Wasm `i64.extend_32s` instruction.
    pub fn translate_i64_sign_extend32(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I64Extend32S { result, input },
            <i64 as SignExtendFrom<i32>>::sign_extend_from,
        )
    }

    /// Translate a Wasm `i32.truncate_sat_f32` instruction.
    pub fn translate_i32_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F32,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32TruncSatF32S */
        )
    }

    /// Translate a Wasm `u32.truncate_sat_f32` instruction.
    pub fn translate_u32_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F32,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32TruncSatF32U */
        )
    }

    /// Translate a Wasm `i32.truncate_sat_f64` instruction.
    pub fn translate_i32_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F64,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32TruncSatF64S */
        )
    }

    /// Translate a Wasm `u32.truncate_sat_f64` instruction.
    pub fn translate_u32_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F64,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I32TruncSatF64U */
        )
    }

    /// Translate a Wasm `i64.truncate_sat_f32` instruction.
    pub fn translate_i64_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F32,
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64TruncSatF32S */
        )
    }

    /// Translate a Wasm `u64.truncate_sat_f32` instruction.
    pub fn translate_u64_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F32,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I64TruncSatF32U */
        )
    }

    /// Translate a Wasm `i64.truncate_sat_f64` instruction.
    pub fn translate_i64_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F64,
            ValueType::I64,
            DUMMY_INSTRUCTION, /* Instruction::I64TruncSatF64S */
        )
    }

    /// Translate a Wasm `u64.truncate_sat_f64` instruction.
    pub fn translate_u64_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            ValueType::F64,
            ValueType::I32,
            DUMMY_INSTRUCTION, /* Instruction::I64TruncSatF64U */
        )
    }
}
