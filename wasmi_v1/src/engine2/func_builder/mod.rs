#![allow(dead_code, unused_imports)]

mod control_frame;
mod control_stack;
mod inst_builder;
mod locals_registry;
mod providers;
mod translate;

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
use core::{
    cmp::PartialOrd,
    ops,
    ops::{Shl, Shr},
};
use wasmi_core::{
    ArithmeticOps,
    ExtendInto,
    Float,
    Integer,
    SignExtendFrom,
    TrapCode,
    TruncateSaturateInto,
    TryTruncateInto,
    Value,
    ValueType,
    WrapInto,
    F32,
    F64,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OpaqueTypes {}

impl InstructionTypes for OpaqueTypes {
    type Register = Register;
    type Provider = Provider;
    type ProviderSlice = ProviderSlice;
    type RegisterSlice = ProviderSlice;
}

pub type OpaqueInstruction = Instruction<OpaqueTypes>;

macro_rules! make_op {
    ( $name:ident ) => {{
        |result, lhs, rhs| Instruction::$name { result, lhs, rhs }
    }};
}

macro_rules! unary_op {
    ( $name:ident ) => {{
        |result, input| Instruction::$name { result, input }
    }};
}

macro_rules! load_op {
    ( $name:ident ) => {{
        |result, ptr, offset| Instruction::$name {
            result,
            ptr,
            offset,
        }
    }};
}

macro_rules! store_op {
    ( $name:ident ) => {{
        |ptr, offset, value| Instruction::$name { ptr, offset, value }
    }};
}

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
    /// - `{i32, i64, f32, f64}.load`
    /// - `{i32, i64}.load_i8`
    /// - `{i32, i64}.load_u8`
    /// - `{i32, i64}.load_i16`
    /// - `{i32, i64}.load_u16`
    /// - `i64.load_i32`
    /// - `i64.load_u32`
    fn translate_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
        make_inst: fn(Register, Register, Offset) -> OpaqueInstruction,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
            let ptr = builder.providers.pop();
            let result = builder.providers.push_dynamic();
            let offset = Offset::from(offset);
            match ptr {
                Provider::Register(ptr) => {
                    builder
                        .inst_builder
                        .push_inst(make_inst(result, ptr, offset));
                }
                Provider::Immediate(ptr) => {
                    builder.inst_builder.push_inst(Instruction::Copy {
                        result,
                        input: ptr.into(),
                    });
                    builder
                        .inst_builder
                        .push_inst(make_inst(result, result, offset));
                }
            }
            Ok(())
        })
    }

    /// Translate a Wasm `i32.load` instruction.
    pub fn translate_i32_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I32Load))
    }

    /// Translate a Wasm `i64.load` instruction.
    pub fn translate_i64_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I64Load))
    }

    /// Translate a Wasm `f32.load` instruction.
    pub fn translate_f32_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(F32Load))
    }

    /// Translate a Wasm `f64.load` instruction.
    pub fn translate_f64_load(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(F64Load))
    }

    /// Translate a Wasm `i32.load_i8` instruction.
    pub fn translate_i32_load_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I32Load8S))
    }

    /// Translate a Wasm `i32.load_u8` instruction.
    pub fn translate_i32_load_u8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I32Load8U))
    }

    /// Translate a Wasm `i32.load_i16` instruction.
    pub fn translate_i32_load_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I32Load16S))
    }

    /// Translate a Wasm `i32.load_u16` instruction.
    pub fn translate_i32_load_u16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I32Load16U))
    }

    /// Translate a Wasm `i64.load_i8` instruction.
    pub fn translate_i64_load_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I64Load8S))
    }

    /// Translate a Wasm `i64.load_u8` instruction.
    pub fn translate_i64_load_u8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I64Load8U))
    }

    /// Translate a Wasm `i64.load_i16` instruction.
    pub fn translate_i64_load_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I64Load16S))
    }

    /// Translate a Wasm `i64.load_u16` instruction.
    pub fn translate_i64_load_u16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I64Load16U))
    }

    /// Translate a Wasm `i64.load_i32` instruction.
    pub fn translate_i64_load_i32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I64Load32S))
    }

    /// Translate a Wasm `i64.load_u32` instruction.
    pub fn translate_i64_load_u32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_load(memory_idx, offset, load_op!(I64Load32U))
    }

    /// Translate a Wasm `<ty>.store` instruction.
    ///
    /// # Note
    ///
    /// This is used as the translation backend of the following Wasm instructions:
    ///
    /// - `{i32, i64, f32, f64}.store`
    /// - `{i32, i64}.store_i8`
    /// - `{i32, i64}.store_i16`
    /// - `i64.store_i32`
    fn translate_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
        make_inst: fn(Register, Offset, Provider) -> OpaqueInstruction,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
            let offset = Offset::from(offset);
            let (ptr, value) = builder.providers.pop2();
            match ptr {
                Provider::Register(ptr) => {
                    builder
                        .inst_builder
                        .push_inst(make_inst(ptr, offset, value));
                }
                Provider::Immediate(ptr) => {
                    // Note: store the constant pointer value into a temporary
                    //       `temp` register and use the register instead since
                    //       otherwise we cannot construct the `wasmi` bytecode
                    //       that expects the pointer value to be provided in
                    //       a register.
                    //       After the store instruction we immediate have to
                    //       pop the temporarily used register again.
                    let temp = builder.providers.push_dynamic();
                    builder.inst_builder.push_inst(Instruction::Copy {
                        result: temp,
                        input: ptr.into(),
                    });
                    builder
                        .inst_builder
                        .push_inst(make_inst(temp, offset, value));
                    builder.providers.pop();
                }
            };
            Ok(())
        })
    }

    /// Translate a Wasm `i32.store` instruction.
    pub fn translate_i32_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, store_op!(I32Store))
    }

    /// Translate a Wasm `i64.store` instruction.
    pub fn translate_i64_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, store_op!(I64Store))
    }

    /// Translate a Wasm `f32.store` instruction.
    pub fn translate_f32_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, store_op!(F32Store))
    }

    /// Translate a Wasm `f64.store` instruction.
    pub fn translate_f64_store(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, store_op!(F64Store))
    }

    /// Translate a Wasm `i32.store_i8` instruction.
    pub fn translate_i32_store_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, store_op!(I32Store8))
    }

    /// Translate a Wasm `i32.store_i16` instruction.
    pub fn translate_i32_store_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, store_op!(I32Store16))
    }

    /// Translate a Wasm `i64.store_i8` instruction.
    pub fn translate_i64_store_i8(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, store_op!(I64Store8))
    }

    /// Translate a Wasm `i64.store_i16` instruction.
    pub fn translate_i64_store_i16(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, store_op!(I64Store16))
    }

    /// Translate a Wasm `i64.store_i32` instruction.
    pub fn translate_i64_store_i32(
        &mut self,
        memory_idx: MemoryIdx,
        offset: u32,
    ) -> Result<(), ModuleError> {
        self.translate_store(memory_idx, offset, store_op!(I64Store32))
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
    fn translate_binary_cmp<O, E, T>(&mut self, make_op: O, exec_op: E) -> Result<(), ModuleError>
    where
        O: FnOnce(Register, Register, Provider) -> OpaqueInstruction,
        E: FnOnce(T, T) -> bool,
        T: FromRegisterEntry,
    {
        self.translate_if_reachable(|builder| {
            let (lhs, rhs) = builder.providers.pop2();
            match (lhs, rhs) {
                (Provider::Register(lhs), rhs) => {
                    let result = builder.providers.push_dynamic();
                    builder.inst_builder.push_inst(make_op(result, lhs, rhs));
                }
                (lhs @ Provider::Immediate(_), Provider::Register(rhs)) => {
                    // Note: we can swap the operands to avoid having to copy `rhs`.
                    let result = builder.providers.push_dynamic();
                    builder.inst_builder.push_inst(make_op(result, rhs, lhs));
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

    /// Translate a Wasm `i32.eq` instruction.
    pub fn translate_i32_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(I32Eq), |lhs: i32, rhs: i32| lhs == rhs)
    }

    /// Translate a Wasm `i32.ne` instruction.
    pub fn translate_i32_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(I32Ne), |lhs: i32, rhs: i32| lhs != rhs)
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
            match (lhs, rhs) {
                (Provider::Register(lhs), rhs) => {
                    let result = builder.providers.push_dynamic();
                    builder.inst_builder.push_inst(make_op(result, lhs, rhs));
                }
                (lhs @ Provider::Immediate(_), Provider::Register(rhs)) => {
                    // Note: we can swap the operands to avoid having to copy `rhs`.
                    let result = builder.providers.push_dynamic();
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
        self.translate_ord(make_op!(I32LtS), make_op!(I32GtS), |lhs: i32, rhs: i32| {
            lhs < rhs
        })
    }

    /// Translate a Wasm `u32.lt` instruction.
    pub fn translate_u32_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32LtU), make_op!(I32GtU), |lhs: u32, rhs: u32| {
            lhs < rhs
        })
    }

    /// Translate a Wasm `i32.gt` instruction.
    pub fn translate_i32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32GtS), make_op!(I32LtS), |lhs: i32, rhs: i32| {
            lhs > rhs
        })
    }

    /// Translate a Wasm `u32.gt` instruction.
    pub fn translate_u32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32GtU), make_op!(I32LtU), |lhs: u32, rhs: u32| {
            lhs > rhs
        })
    }

    /// Translate a Wasm `i32.le` instruction.
    pub fn translate_i32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32LeS), make_op!(I32GeS), |lhs: i32, rhs: i32| {
            lhs <= rhs
        })
    }

    /// Translate a Wasm `u32.le` instruction.
    pub fn translate_u32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32LeU), make_op!(I32GeU), |lhs: u32, rhs: u32| {
            lhs <= rhs
        })
    }

    /// Translate a Wasm `i32.ge` instruction.
    pub fn translate_i32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32GeS), make_op!(I32LeS), |lhs: i32, rhs: i32| {
            lhs >= rhs
        })
    }

    /// Translate a Wasm `u32.ge` instruction.
    pub fn translate_u32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32GeU), make_op!(I32LeU), |lhs: u32, rhs: u32| {
            lhs >= rhs
        })
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
        self.translate_binary_cmp(make_op!(I64Eq), |lhs: i64, rhs: i64| lhs == rhs)
    }

    /// Translate a Wasm `i64.ne` instruction.
    pub fn translate_i64_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(I64Ne), |lhs: i64, rhs: i64| lhs != rhs)
    }

    /// Translate a Wasm `i64.lt` instruction.
    pub fn translate_i64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64LtS), make_op!(I64GtS), |lhs: i64, rhs: i64| {
            lhs < rhs
        })
    }

    /// Translate a Wasm `u64.lt` instruction.
    pub fn translate_u64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64LtU), make_op!(I64GtU), |lhs: u64, rhs: u64| {
            lhs < rhs
        })
    }

    /// Translate a Wasm `i64.gt` instruction.
    pub fn translate_i64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64GtS), make_op!(I64LtS), |lhs: i64, rhs: i64| {
            lhs > rhs
        })
    }

    /// Translate a Wasm `u64.gt` instruction.
    pub fn translate_u64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64GtU), make_op!(I64LtU), |lhs: u64, rhs: u64| {
            lhs > rhs
        })
    }

    /// Translate a Wasm `i64.le` instruction.
    pub fn translate_i64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64LeS), make_op!(I64GeS), |lhs: i64, rhs: i64| {
            lhs <= rhs
        })
    }

    /// Translate a Wasm `u64.le` instruction.
    pub fn translate_u64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64LeU), make_op!(I64GeU), |lhs: u64, rhs: u64| {
            lhs <= rhs
        })
    }

    /// Translate a Wasm `i64.ge` instruction.
    pub fn translate_i64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64GeS), make_op!(I64LeS), |lhs: i64, rhs: i64| {
            lhs >= rhs
        })
    }

    /// Translate a Wasm `u64.ge` instruction.
    pub fn translate_u64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64GeU), make_op!(I64LeU), |lhs: u64, rhs: u64| {
            lhs >= rhs
        })
    }

    /// Translate a Wasm `f32.eq` instruction.
    pub fn translate_f32_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(F32Eq), |lhs: f32, rhs: f32| lhs == rhs)
    }

    /// Translate a Wasm `f32.ne` instruction.
    pub fn translate_f32_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(F32Ne), |lhs: f32, rhs: f32| lhs != rhs)
    }

    /// Translate a Wasm `f32.lt` instruction.
    pub fn translate_f32_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F32Lt), make_op!(F32Gt), |lhs: f32, rhs: f32| {
            lhs < rhs
        })
    }

    /// Translate a Wasm `f32.gt` instruction.
    pub fn translate_f32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F32Gt), make_op!(F32Lt), |lhs: f32, rhs: f32| {
            lhs > rhs
        })
    }

    /// Translate a Wasm `f32.le` instruction.
    pub fn translate_f32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F32Le), make_op!(F32Ge), |lhs: f32, rhs: f32| {
            lhs <= rhs
        })
    }

    /// Translate a Wasm `f32.ge` instruction.
    pub fn translate_f32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F32Ge), make_op!(F32Le), |lhs: f32, rhs: f32| {
            lhs >= rhs
        })
    }

    /// Translate a Wasm `f64.eq` instruction.
    pub fn translate_f64_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(F64Eq), |lhs: f64, rhs: f64| lhs == rhs)
    }

    /// Translate a Wasm `f64.ne` instruction.
    pub fn translate_f64_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(F64Ne), |lhs: f64, rhs: f64| lhs != rhs)
    }

    /// Translate a Wasm `f64.lt` instruction.
    pub fn translate_f64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F64Lt), make_op!(F64Gt), |lhs: f64, rhs: f64| {
            lhs < rhs
        })
    }

    /// Translate a Wasm `f64.gt` instruction.
    pub fn translate_f64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F64Gt), make_op!(F64Lt), |lhs: f64, rhs: f64| {
            lhs > rhs
        })
    }

    /// Translate a Wasm `f64.le` instruction.
    pub fn translate_f64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F64Le), make_op!(F64Ge), |lhs: f64, rhs: f64| {
            lhs <= rhs
        })
    }

    /// Translate a Wasm `f64.ge` instruction.
    pub fn translate_f64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F64Ge), make_op!(F64Le), |lhs: f64, rhs: f64| {
            lhs >= rhs
        })
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
    fn translate_unary_operation<F, E, T, R>(
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

    /// Translate a non-commutative binary Wasm instruction.
    ///
    /// - `{i32, i64, f32, f64}.sub`
    /// - `{i32, i64}.shl`
    /// - `{i32, u32, i64, u64}.shr`
    /// - `{i32, i64}.rotl`
    /// - `{i32, i64}.rotr`
    /// - `{f32, f64}.copysign`
    pub fn translate_binary_operation<F, E, T, R>(
        &mut self,
        make_op: F,
        exec_op: E,
    ) -> Result<(), ModuleError>
    where
        F: FnOnce(Register, Register, Provider) -> OpaqueInstruction,
        E: FnOnce(T, T) -> R,
        T: FromRegisterEntry,
        R: Into<Value>,
    {
        self.translate_if_reachable(|builder| {
            let (lhs, rhs) = builder.providers.pop2();
            match (lhs, rhs) {
                (Provider::Register(lhs), rhs) => {
                    let result = builder.providers.push_dynamic();
                    builder.inst_builder.push_inst(make_op(result, lhs, rhs));
                }
                (lhs @ Provider::Immediate(_), Provider::Register(rhs)) => {
                    // Note: this case is a bit tricky for non-commutative operations.
                    //
                    // In order to be able to represent the constant left-hand side
                    // operand for the instruction we need to `copy` it into a register
                    // first.
                    let result = builder.providers.push_dynamic();
                    builder
                        .inst_builder
                        .push_inst(Instruction::Copy { result, input: lhs });
                    builder
                        .inst_builder
                        .push_inst(make_op(result, result, rhs.into()));
                }
                (Provider::Immediate(lhs), Provider::Immediate(rhs)) => {
                    // Note: both operands are constant so we can evaluate the result.
                    let lhs = T::from_stack_entry(RegisterEntry::from(lhs));
                    let rhs = T::from_stack_entry(RegisterEntry::from(rhs));
                    let result = exec_op(lhs, rhs);
                    builder.providers.push_const(result.into());
                }
            }
            Ok(())
        })
    }

    /// Translate a fallible non-commutative binary Wasm instruction.
    ///
    /// - `{i32, u32, i64, u64, f32, f64}.div`
    /// - `{i32, u32, i64, u64}.rem`
    pub fn translate_fallible_binary_operation<F, E, T>(
        &mut self,
        make_op: F,
        exec_op: E,
    ) -> Result<(), ModuleError>
    where
        F: FnOnce(Register, Register, Provider) -> OpaqueInstruction,
        E: FnOnce(T, T) -> Result<T, TrapCode>,
        T: FromRegisterEntry + Into<Value>,
    {
        self.translate_if_reachable(|builder| {
            let (lhs, rhs) = builder.providers.pop2();
            match (lhs, rhs) {
                (Provider::Register(lhs), rhs) => {
                    let result = builder.providers.push_dynamic();
                    builder.inst_builder.push_inst(make_op(result, lhs, rhs));
                }
                (lhs @ Provider::Immediate(_), Provider::Register(rhs)) => {
                    // Note: this case is a bit tricky for non-commutative operations.
                    //
                    // In order to be able to represent the constant left-hand side
                    // operand for the instruction we need to `copy` it into a register
                    // first.
                    let result = builder.providers.push_dynamic();
                    builder
                        .inst_builder
                        .push_inst(Instruction::Copy { result, input: lhs });
                    builder
                        .inst_builder
                        .push_inst(make_op(result, result, rhs.into()));
                }
                (Provider::Immediate(lhs), Provider::Immediate(rhs)) => {
                    // Note: both operands are constant so we can evaluate the result.
                    let lhs = T::from_stack_entry(RegisterEntry::from(lhs));
                    let rhs = T::from_stack_entry(RegisterEntry::from(rhs));
                    match exec_op(lhs, rhs) {
                        Ok(result) => {
                            builder.providers.push_const(result.into());
                        }
                        Err(trap_code) => {
                            builder
                                .inst_builder
                                .push_inst(Instruction::Trap { trap_code });
                            builder.reachable = false;
                        }
                    }
                }
            }
            Ok(())
        })
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
    fn translate_commutative_binary_operation<F, E, T, R>(
        &mut self,
        make_op: F,
        exec_op: E,
    ) -> Result<(), ModuleError>
    where
        F: FnOnce(Register, Register, Provider) -> OpaqueInstruction,
        E: FnOnce(T, T) -> R,
        T: FromRegisterEntry,
        R: Into<Value>,
    {
        self.translate_if_reachable(|builder| {
            let (lhs, rhs) = builder.providers.pop2();
            match (lhs, rhs) {
                (Provider::Register(lhs), rhs) => {
                    let result = builder.providers.push_dynamic();
                    builder.inst_builder.push_inst(make_op(result, lhs, rhs));
                }
                (lhs @ Provider::Immediate(_), Provider::Register(rhs)) => {
                    // Note: due to commutativity of the operation we can swap the parameters.
                    let result = builder.providers.push_dynamic();
                    builder.inst_builder.push_inst(make_op(result, rhs, lhs));
                }
                (Provider::Immediate(lhs), Provider::Immediate(rhs)) => {
                    let lhs = T::from_stack_entry(RegisterEntry::from(lhs));
                    let rhs = T::from_stack_entry(RegisterEntry::from(rhs));
                    let result = exec_op(lhs, rhs);
                    builder.providers.push_const(result.into());
                }
            };
            Ok(())
        })
    }

    /// Translate a Wasm `i32.add` instruction.
    pub fn translate_i32_add(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I32Add), i32::wrapping_add)
    }

    /// Translate a Wasm `i32.sub` instruction.
    pub fn translate_i32_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32Sub), i32::wrapping_sub)
    }

    /// Translate a Wasm `i32.mul` instruction.
    pub fn translate_i32_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I32Mul), i32::wrapping_mul)
    }

    /// Translate a Wasm `i32.div` instruction.
    pub fn translate_i32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I32DivS), i32::div)
    }

    /// Translate a Wasm `u32.div` instruction.
    pub fn translate_u32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I32DivU), u32::div)
    }

    /// Translate a Wasm `i32.rem` instruction.
    pub fn translate_i32_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I32RemS), i32::rem)
    }

    /// Translate a Wasm `u32.rem` instruction.
    pub fn translate_u32_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I32RemU), u32::rem)
    }

    /// Translate a Wasm `i32.and` instruction.
    pub fn translate_i32_and(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I32And), |lhs: i32, rhs: i32| {
            lhs & rhs
        })
    }

    /// Translate a Wasm `i32.or` instruction.
    pub fn translate_i32_or(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I32Or), |lhs: i32, rhs: i32| lhs | rhs)
    }

    /// Translate a Wasm `i32.xor` instruction.
    pub fn translate_i32_xor(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I32Xor), |lhs: i32, rhs: i32| {
            lhs ^ rhs
        })
    }

    /// Translate a Wasm `i32.shl` instruction.
    pub fn translate_i32_shl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32Shl), |lhs: i32, rhs: i32| lhs.shl(rhs & 0x1F))
    }

    /// Translate a Wasm `i32.shr` instruction.
    pub fn translate_i32_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32ShrS), |lhs: i32, rhs: i32| lhs.shr(rhs & 0x1F))
    }

    /// Translate a Wasm `u32.shr` instruction.
    pub fn translate_u32_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32ShrU), |lhs: u32, rhs: u32| lhs.shr(rhs & 0x1F))
    }

    /// Translate a Wasm `i32.rotl` instruction.
    pub fn translate_i32_rotl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32Rotl), i32::rotl)
    }

    /// Translate a Wasm `i32.rotr` instruction.
    pub fn translate_i32_rotr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32Rotr), i32::rotr)
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
        self.translate_commutative_binary_operation(make_op!(I64Add), i64::wrapping_add)
    }

    /// Translate a Wasm `i64.sub` instruction.
    pub fn translate_i64_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64Sub), i64::wrapping_sub)
    }

    /// Translate a Wasm `i64.mul` instruction.
    pub fn translate_i64_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I64Mul), i64::wrapping_mul)
    }

    /// Translate a Wasm `i64.div` instruction.
    pub fn translate_i64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I64DivS), i64::div)
    }

    /// Translate a Wasm `u64.div` instruction.
    pub fn translate_u64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I64DivU), u64::div)
    }

    /// Translate a Wasm `i64.rem` instruction.
    pub fn translate_i64_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I64RemS), i64::rem)
    }

    /// Translate a Wasm `u64.rem` instruction.
    pub fn translate_u64_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I64RemU), u64::rem)
    }

    /// Translate a Wasm `i64.and` instruction.
    pub fn translate_i64_and(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I64And), |lhs: i64, rhs: i64| {
            lhs & rhs
        })
    }

    /// Translate a Wasm `i64.or` instruction.
    pub fn translate_i64_or(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I64Or), |lhs: i64, rhs: i64| lhs | rhs)
    }

    /// Translate a Wasm `i64.xor` instruction.
    pub fn translate_i64_xor(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I64Xor), |lhs: i64, rhs: i64| {
            lhs ^ rhs
        })
    }

    /// Translate a Wasm `i64.shl` instruction.
    pub fn translate_i64_shl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64Shl), |lhs: i64, rhs: i64| lhs.shl(rhs & 0x3F))
    }

    /// Translate a Wasm `i64.shr` instruction.
    pub fn translate_i64_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64ShrS), |lhs: i64, rhs: i64| lhs.shr(rhs & 0x3F))
    }

    /// Translate a Wasm `u64.shr` instruction.
    pub fn translate_u64_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64ShrU), |lhs: u64, rhs: u64| lhs.shr(rhs & 0x3F))
    }

    /// Translate a Wasm `i64.rotl` instruction.
    pub fn translate_i64_rotl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64Rotl), i64::rotl)
    }

    /// Translate a Wasm `i64.rotr` instruction.
    pub fn translate_i64_rotr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64Rotr), i64::rotr)
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
        self.translate_commutative_binary_operation(make_op!(F32Add), |lhs: F32, rhs: F32| {
            lhs + rhs
        })
    }

    /// Translate a Wasm `f32.sub` instruction.
    pub fn translate_f32_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(F32Sub), F32::sub)
    }

    /// Translate a Wasm `f32.mul` instruction.
    pub fn translate_f32_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F32Mul), |lhs: F32, rhs: F32| {
            lhs * rhs
        })
    }

    /// Translate a Wasm `f32.div` instruction.
    pub fn translate_f32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(F32Div), F32::div)
    }

    /// Translate a Wasm `f32.min` instruction.
    pub fn translate_f32_min(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F32Min), F32::min)
    }

    /// Translate a Wasm `f32.max` instruction.
    pub fn translate_f32_max(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F32Max), F32::max)
    }

    /// Translate a Wasm `f32.copysign` instruction.
    pub fn translate_f32_copysign(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(F32Copysign), F32::copysign)
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
        self.translate_commutative_binary_operation(make_op!(F64Add), |lhs: F64, rhs: F64| {
            lhs + rhs
        })
    }

    /// Translate a Wasm `f64.sub` instruction.
    pub fn translate_f64_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(F64Sub), F64::sub)
    }

    /// Translate a Wasm `f64.mul` instruction.
    pub fn translate_f64_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F64Mul), |lhs: F64, rhs: F64| {
            lhs * rhs
        })
    }

    /// Translate a Wasm `f64.div` instruction.
    pub fn translate_f64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(F64Div), F64::div)
    }

    /// Translate a Wasm `f64.min` instruction.
    pub fn translate_f64_min(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F64Min), F64::min)
    }

    /// Translate a Wasm `f64.max` instruction.
    pub fn translate_f64_max(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F64Max), F64::max)
    }

    /// Translate a Wasm `f64.copysign` instruction.
    pub fn translate_f64_copysign(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(F64Copysign), F64::copysign)
    }

    /// Translate an infallible Wasm conversion instruction.
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
    fn translate_conversion<F, E, T, R>(
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
        self.translate_unary_operation(make_op, exec_op)
    }

    /// Translate an infallible Wasm conversion instruction.
    ///
    /// - `{i32, u32}.trunc_f32
    /// - `{i32, u32}.trunc_f64`
    /// - `{i64, u64}.trunc_f32`
    /// - `{i64, u64}.trunc_f64`
    /// - `f32.convert_{i32, u32, i64, u64}`
    /// - `f64.convert_{i32, u32, i64, u64}`
    fn translate_fallible_conversion<F, E, T, R>(
        &mut self,
        make_op: F,
        exec_op: E,
    ) -> Result<(), ModuleError>
    where
        F: FnOnce(Register, Register) -> OpaqueInstruction,
        E: FnOnce(T) -> Result<R, TrapCode>,
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
                Provider::Immediate(input) => match exec_op(T::from_stack_entry(input.into())) {
                    Ok(result) => {
                        builder.providers.push_const(result.into());
                    }
                    Err(trap_code) => {
                        builder
                            .inst_builder
                            .push_inst(Instruction::Trap { trap_code });
                        builder.reachable = false;
                    }
                },
            }
            Ok(())
        })
    }

    /// Translate a Wasm `i32.wrap_i64` instruction.
    pub fn translate_i32_wrap_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(I32WrapI64), <i64 as WrapInto<i32>>::wrap_into)
    }

    /// Translate a Wasm `i32.trunc_f32` instruction.
    pub fn translate_i32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(
            unary_op!(I32TruncSF32),
            <f32 as TryTruncateInto<i32, TrapCode>>::try_truncate_into,
        )
    }

    /// Translate a Wasm `u32.trunc_f32` instruction.
    pub fn translate_u32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(
            unary_op!(I32TruncUF32),
            <f32 as TryTruncateInto<u32, TrapCode>>::try_truncate_into,
        )
    }

    /// Translate a Wasm `i32.trunc_f64` instruction.
    pub fn translate_i32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(
            unary_op!(I32TruncSF64),
            <f64 as TryTruncateInto<i32, TrapCode>>::try_truncate_into,
        )
    }

    /// Translate a Wasm `u32.trunc_f64` instruction.
    pub fn translate_u32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(
            unary_op!(I32TruncUF64),
            <f64 as TryTruncateInto<u32, TrapCode>>::try_truncate_into,
        )
    }

    /// Translate a Wasm `i64.extend_i32` instruction.
    pub fn translate_i64_extend_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I64ExtendSI32),
            <i32 as ExtendInto<i64>>::extend_into,
        )
    }

    /// Translate a Wasm `u64.extend_i32` instruction.
    pub fn translate_u64_extend_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I64ExtendUI32),
            <u32 as ExtendInto<i64>>::extend_into,
        )
    }

    /// Translate a Wasm `i64.trunc_f32` instruction.
    pub fn translate_i64_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(
            unary_op!(I64TruncSF32),
            <f32 as TryTruncateInto<i64, TrapCode>>::try_truncate_into,
        )
    }

    /// Translate a Wasm `u64.trunc_f32` instruction.
    pub fn translate_u64_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(
            unary_op!(I64TruncUF32),
            <f32 as TryTruncateInto<u64, TrapCode>>::try_truncate_into,
        )
    }

    /// Translate a Wasm `i64.trunc_f64` instruction.
    pub fn translate_i64_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(
            unary_op!(I64TruncSF64),
            <f64 as TryTruncateInto<i64, TrapCode>>::try_truncate_into,
        )
    }

    /// Translate a Wasm `u64.trunc_f64` instruction.
    pub fn translate_u64_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(
            unary_op!(I64TruncUF64),
            <f64 as TryTruncateInto<u64, TrapCode>>::try_truncate_into,
        )
    }

    /// Translate a Wasm `f32.convert_i32` instruction.
    pub fn translate_f32_convert_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(F32ConvertSI32),
            <i32 as ExtendInto<F32>>::extend_into,
        )
    }

    /// Translate a Wasm `f32.convert_u32` instruction.
    pub fn translate_f32_convert_u32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(F32ConvertUI32),
            <u32 as ExtendInto<F32>>::extend_into,
        )
    }

    /// Translate a Wasm `f32.convert_i64` instruction.
    pub fn translate_f32_convert_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F32ConvertSI64), <i64 as WrapInto<F32>>::wrap_into)
    }

    /// Translate a Wasm `f32.convert_u64` instruction.
    pub fn translate_f32_convert_u64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F32ConvertUI64), <u64 as WrapInto<F32>>::wrap_into)
    }

    /// Translate a Wasm `f32.demote_f64` instruction.
    pub fn translate_f32_demote_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F32DemoteF64), <F64 as WrapInto<F32>>::wrap_into)
    }

    /// Translate a Wasm `f64.convert_i32` instruction.
    pub fn translate_f64_convert_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(F64ConvertSI32),
            <i32 as ExtendInto<F64>>::extend_into,
        )
    }

    /// Translate a Wasm `f64.convert_u32` instruction.
    pub fn translate_f64_convert_u32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(F64ConvertUI32),
            <u32 as ExtendInto<F64>>::extend_into,
        )
    }

    /// Translate a Wasm `f64.convert_i64` instruction.
    pub fn translate_f64_convert_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(F64ConvertSI64),
            <i64 as ExtendInto<F64>>::extend_into,
        )
    }

    /// Translate a Wasm `f64.convert_u64` instruction.
    pub fn translate_f64_convert_u64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(F64ConvertUI64),
            <u64 as ExtendInto<F64>>::extend_into,
        )
    }

    /// Translate a Wasm `f64.promote_f32` instruction.
    pub fn translate_f64_promote_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(F64PromoteF32),
            <F32 as ExtendInto<F64>>::extend_into,
        )
    }

    /// Translate a Wasm `i32.reinterpret_f32` instruction.
    pub fn translate_i32_reinterpret_f32(&mut self) -> Result<(), ModuleError> {
        // Note: since `wasmi` engine treats its values internally as untyped
        //       bits we have to do nothing for reinterpret casts.
        //
        // -> `i32.reinterpret_f32` compiles to a no-op.
        Ok(())
    }

    /// Translate a Wasm `i64.reinterpret_f64` instruction.
    pub fn translate_i64_reinterpret_f64(&mut self) -> Result<(), ModuleError> {
        // Note: since `wasmi` engine treats its values internally as untyped
        //       bits we have to do nothing for reinterpret casts.
        //
        // -> `i64.reinterpret_f64` compiles to a no-op.
        Ok(())
    }

    /// Translate a Wasm `f32.reinterpret_i32` instruction.
    pub fn translate_f32_reinterpret_i32(&mut self) -> Result<(), ModuleError> {
        // Note: since `wasmi` engine treats its values internally as untyped
        //       bits we have to do nothing for reinterpret casts.
        //
        // -> `f32.reinterpret_i32` compiles to a no-op.
        Ok(())
    }

    /// Translate a Wasm `f64.reinterpret_i64` instruction.
    pub fn translate_f64_reinterpret_i64(&mut self) -> Result<(), ModuleError> {
        // Note: since `wasmi` engine treats its values internally as untyped
        //       bits we have to do nothing for reinterpret casts.
        //
        // -> `f64.reinterpret_i64` compiles to a no-op.
        Ok(())
    }

    /// Translate a Wasm `i32.extend_8s` instruction.
    pub fn translate_i32_sign_extend8(&mut self) -> Result<(), ModuleError> {
        // Note: we do not consider sign-extend operations to be conversion
        //       routines since they do not alter the type of the operand.
        self.translate_unary_operation(
            unary_op!(I32Extend8S),
            <i32 as SignExtendFrom<i8>>::sign_extend_from,
        )
    }

    /// Translate a Wasm `i32.extend_16s` instruction.
    pub fn translate_i32_sign_extend16(&mut self) -> Result<(), ModuleError> {
        // Note: we do not consider sign-extend operations to be conversion
        //       routines since they do not alter the type of the operand.
        self.translate_unary_operation(
            unary_op!(I32Extend16S),
            <i32 as SignExtendFrom<i16>>::sign_extend_from,
        )
    }

    /// Translate a Wasm `i64.extend_8s` instruction.
    pub fn translate_i64_sign_extend8(&mut self) -> Result<(), ModuleError> {
        // Note: we do not consider sign-extend operations to be conversion
        //       routines since they do not alter the type of the operand.
        self.translate_unary_operation(
            unary_op!(I64Extend8S),
            <i64 as SignExtendFrom<i8>>::sign_extend_from,
        )
    }

    /// Translate a Wasm `i64.extend_16s` instruction.
    pub fn translate_i64_sign_extend16(&mut self) -> Result<(), ModuleError> {
        // Note: we do not consider sign-extend operations to be conversion
        //       routines since they do not alter the type of the operand.
        self.translate_unary_operation(
            unary_op!(I64Extend16S),
            <i64 as SignExtendFrom<i16>>::sign_extend_from,
        )
    }

    /// Translate a Wasm `i64.extend_32s` instruction.
    pub fn translate_i64_sign_extend32(&mut self) -> Result<(), ModuleError> {
        // Note: we do not consider sign-extend operations to be conversion
        //       routines since they do not alter the type of the operand.
        self.translate_unary_operation(
            unary_op!(I64Extend32S),
            <i64 as SignExtendFrom<i32>>::sign_extend_from,
        )
    }

    /// Translate a Wasm `i32.truncate_sat_f32` instruction.
    pub fn translate_i32_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I32TruncSatF32S),
            <F32 as TruncateSaturateInto<i32>>::truncate_saturate_into,
        )
    }

    /// Translate a Wasm `u32.truncate_sat_f32` instruction.
    pub fn translate_u32_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I32TruncSatF32U),
            <F32 as TruncateSaturateInto<u32>>::truncate_saturate_into,
        )
    }

    /// Translate a Wasm `i32.truncate_sat_f64` instruction.
    pub fn translate_i32_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I32TruncSatF64S),
            <F64 as TruncateSaturateInto<i32>>::truncate_saturate_into,
        )
    }

    /// Translate a Wasm `u32.truncate_sat_f64` instruction.
    pub fn translate_u32_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I32TruncSatF64U),
            <F64 as TruncateSaturateInto<u32>>::truncate_saturate_into,
        )
    }

    /// Translate a Wasm `i64.truncate_sat_f32` instruction.
    pub fn translate_i64_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I64TruncSatF32S),
            <F32 as TruncateSaturateInto<i64>>::truncate_saturate_into,
        )
    }

    /// Translate a Wasm `u64.truncate_sat_f32` instruction.
    pub fn translate_u64_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I64TruncSatF32U),
            <F32 as TruncateSaturateInto<u64>>::truncate_saturate_into,
        )
    }

    /// Translate a Wasm `i64.truncate_sat_f64` instruction.
    pub fn translate_i64_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I64TruncSatF64S),
            <F64 as TruncateSaturateInto<i64>>::truncate_saturate_into,
        )
    }

    /// Translate a Wasm `u64.truncate_sat_f64` instruction.
    pub fn translate_u64_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I64TruncSatF64U),
            <F64 as TruncateSaturateInto<u64>>::truncate_saturate_into,
        )
    }
}
