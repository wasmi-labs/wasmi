mod control_frame;
mod control_stack;
mod inst_builder;
mod inst_result;
mod locals_registry;
mod providers;

use self::{
    control_frame::{
        BlockControlFrame,
        ControlFrame,
        ControlFrameKind,
        IfControlFrame,
        IfReachability,
        LoopControlFrame,
        UnreachableControlFrame,
    },
    control_stack::ControlFlowStack,
    inst_builder::InstructionsBuilder,
    locals_registry::LocalsRegistry,
    providers::{ProviderSliceArena, Providers},
};
pub use self::{
    inst_builder::{Instr, LabelIdx, RelativeDepth, Reloc},
    providers::{IrProvider, IrProviderSlice, IrRegister, IrRegisterSlice},
};
use super::{
    bytecode::Offset,
    Engine,
    ExecRegister,
    FuncBody,
    Instruction,
    InstructionTypes,
    Target,
};
use crate::{
    engine::bytecode::Global,
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
};
use core::mem;
use wasmi_core::{TrapCode, UntypedValue, ValueType, F32, F64};

pub struct CompileContext<'a> {
    provider_slices: &'a ProviderSliceArena,
    providers: &'a Providers,
}

impl CompileContext<'_> {
    pub fn len_registers(&self) -> u16 {
        self.providers.len_required_registers()
    }

    pub fn resolve_provider_slice(&self, slice: IrProviderSlice) -> &[IrProvider] {
        self.provider_slices.resolve(slice)
    }

    pub fn compile_register(&self, register: IrRegister) -> ExecRegister {
        let index = match register {
            IrRegister::Local(index) => index,
            IrRegister::Dynamic(index) => self.providers.locals.len_registered() as usize + index,
            IrRegister::Preserved(index) => {
                self.providers.locals.len_registered() as usize
                    + self.providers.stacks.max_dynamic()
                    + index
            }
        };
        let bounded = index.try_into().unwrap_or_else(|error| {
            panic!(
                "encountered out of bounds register index ({}): {}",
                index, error
            )
        });
        ExecRegister::from_inner(bounded)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IrTypes {}

impl InstructionTypes for IrTypes {
    type Register = IrRegister;
    type Provider = IrProvider;
    type ProviderSlice = IrProviderSlice;
    type RegisterSlice = IrRegisterSlice;
}

pub type IrInstruction = Instruction<IrTypes>;

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
    provider_slices: ProviderSliceArena,
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
    /// Allows `local.set` to override the `result` register of the last
    /// `wasmi` instruction.
    /// This optimization may only be applied if the `local.set` immediately
    /// follow the instruction of which the `result` register shall be replaced
    /// and must not be applied if for example a `i32.const` or similar was
    /// in between.
    allow_set_local_override: bool,
}

impl<'parser> FunctionBuilder<'parser> {
    /// Creates a new [`FunctionBuilder`].
    pub fn new(engine: &Engine, func: FuncIdx, res: ModuleResources<'parser>) -> Self {
        let mut inst_builder = InstructionsBuilder::default();
        let mut control_frames = ControlFlowStack::default();
        let mut providers = Providers::default();
        let reg_slices = ProviderSliceArena::default();
        Self::register_func_body_block(
            func,
            res,
            &mut inst_builder,
            &mut control_frames,
            &mut providers,
        );
        Self::register_func_params(func, res, &mut providers);
        Self {
            engine: engine.clone(),
            func,
            res,
            control_frames,
            inst_builder,
            reachable: true,
            providers,
            provider_slices: reg_slices,
            allow_set_local_override: false,
        }
    }

    /// Updates `allow_set_local_override` to `allow` and returns its old value.
    fn update_allow_set_local_override(&mut self, allow: bool) -> bool {
        mem::replace(&mut self.allow_set_local_override, allow)
    }

    /// Registers the `block` control frame surrounding the entire function body.
    fn register_func_body_block(
        func: FuncIdx,
        res: ModuleResources<'parser>,
        inst_builder: &mut InstructionsBuilder,
        control_frames: &mut ControlFlowStack,
        providers: &mut Providers,
    ) {
        let func_type = res.get_type_of_func(func);
        let block_type = BlockType::func_type(func_type);
        let end_label = inst_builder.new_label();
        let results = providers.peek_dynamic_many(block_type.len_results(res.engine()) as usize);
        let block_frame = BlockControlFrame::new(results, block_type, end_label, 0);
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

    /// Try to resolve the given label.
    ///
    /// In case the label cannot yet be resolved register the [`Reloc`] as its user.
    fn try_resolve_label<F>(&mut self, label: LabelIdx, reloc_provider: F) -> Instr
    where
        F: FnOnce(Instr) -> Reloc,
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
        self.providers.register_locals(value_type, amount);
        Ok(())
    }

    /// Finishes constructing the function and returns its [`FuncBody`].
    pub fn finish(mut self) -> FuncBody {
        self.inst_builder
            .finish(&self.engine, &self.provider_slices, &self.providers)
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
    fn return_provider_slice(&mut self) -> IrProviderSlice {
        debug_assert!(self.is_reachable());
        let func_type = self.res.get_type_of_func(self.func);
        let len_results = self
            .engine
            .resolve_func_type(func_type, |func_type| func_type.results().len());
        let providers = self.providers.peek_n(len_results).iter().copied();
        self.provider_slices.alloc(providers)
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

    /// Returns the branching target at the given `depth`.
    ///
    /// # Panics
    ///
    /// - If the `depth` is greater than the current height of the control frame stack.
    /// - If the value stack underflowed.
    fn acquire_target<F>(&mut self, relative_depth: u32, reloc_provider: F) -> AquiredTarget
    where
        F: FnOnce(Instr) -> Reloc,
    {
        debug_assert!(self.is_reachable());
        if self.control_frames.is_root(relative_depth) {
            AquiredTarget::Return
        } else {
            let frame = self.control_frames.nth_back(relative_depth);
            let br_dst = frame.branch_destination();
            let results = frame.branch_results();
            let instr = self.try_resolve_label(br_dst, reloc_provider);
            let target = Target::from(instr);
            let returned = self.provider_slices.alloc(
                self.providers
                    .peek_n(results.len() as usize)
                    .iter()
                    .copied(),
            );
            AquiredTarget::Branch {
                target,
                results,
                returned,
            }
        }
    }
}

/// An aquired target for a branch instruction.
#[derive(Debug, Copy, Clone)]
pub enum AquiredTarget {
    /// The branching targets the entry to a control flow frame.
    ///
    /// # Note
    ///
    /// This is the usual case.
    Branch {
        /// The first instruction of the targeted control flow frame.
        target: Target,
        /// The registers where the targeted control flow frame expects
        /// its results.
        results: IrRegisterSlice,
        /// The providers copied over to the results of the targeted control
        /// flow frame upon taking the branch.
        returned: IrProviderSlice,
    },
    /// The branch is ending the called function.
    ///
    /// # Note
    ///
    /// This happens for example when branching to the function enclosing block.
    Return,
}

impl<'parser> FunctionBuilder<'parser> {
    /// Translates a Wasm `unreachable` instruction.
    pub fn translate_unreachable(&mut self) -> Result<(), ModuleError> {
        self.update_allow_set_local_override(false);
        self.translate_if_reachable(|builder| {
            builder.push_instr(Instruction::Trap {
                trap_code: TrapCode::Unreachable,
            });
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
        let stack_height = self.providers.len();
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
        self.update_allow_set_local_override(false);
        let stack_height = self.frame_stack_height(block_type);
        if self.is_reachable() {
            let end_label = self.inst_builder.new_label();
            let results = self
                .providers
                .peek_dynamic_many(block_type.len_results(&self.engine) as usize);
            self.control_frames.push_frame(BlockControlFrame::new(
                results,
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
        self.update_allow_set_local_override(false);
        let stack_height = self.frame_stack_height(block_type);
        if self.is_reachable() {
            let branch_results = self
                .providers
                .peek_dynamic_many(block_type.len_params(&self.engine) as usize);
            let end_results = self
                .providers
                .peek_dynamic_many(block_type.len_results(&self.engine) as usize);
            // Copy over the initial loop arguments from the provider stack.
            // Unlike with other blocks we do have to do this since loop headers
            // can be jumped to again for which they always need their inputs
            // in their expected registers.
            let len_params = branch_results.len() as usize;
            self.inst_builder.push_copy_many_instr(
                &mut self.provider_slices,
                branch_results,
                self.providers.pop_n(len_params).as_slice(),
            );
            self.providers.push_dynamic_many(len_params);
            // After putting all required copy intsructions we can now
            // resolve the loop header.
            let header = self.inst_builder.new_label();
            self.inst_builder.resolve_label(header);
            self.control_frames.push_frame(LoopControlFrame::new(
                branch_results,
                end_results,
                block_type,
                header,
                stack_height,
            ));
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
        self.update_allow_set_local_override(false);
        if self.is_reachable() {
            let condition = self.providers.pop();
            let stack_height = self.frame_stack_height(block_type);
            let results = self
                .providers
                .peek_dynamic_many(block_type.len_results(&self.engine) as usize);
            match condition {
                IrProvider::Register(condition) => {
                    // We duplicate the `if` parameters on the provider stack
                    // so that we do not have to store the `if` parameters for
                    // the optional `else` block somewhere else.
                    // We do this despite the fact that we do not know at this
                    // point if the `if` block actually has an `else` block.
                    // The `else_height` stores the height to which we have to
                    // prune the provider stack upon visiting the `else` block.
                    // The `stack_height` is still denoting the height of the
                    // provider stack upon entering the outer `if` block.
                    let len_params = block_type.len_params(&self.engine);
                    self.providers.duplicate_n(len_params as usize);
                    let else_height = self.frame_stack_height(block_type);
                    let else_label = self.inst_builder.new_label();
                    let end_label = self.inst_builder.new_label();
                    self.control_frames.push_frame(IfControlFrame::new(
                        results,
                        block_type,
                        end_label,
                        Some(else_label),
                        stack_height,
                        Some(else_height),
                        IfReachability::Both,
                    ));
                    let dst_pc =
                        self.try_resolve_label(else_label, |pc| Reloc::Br { inst_idx: pc });
                    let branch_target = Target::from(dst_pc);
                    self.push_instr(Instruction::BrEqz {
                        target: branch_target,
                        condition,
                        results: IrRegisterSlice::empty(), // TODO: proper inputs
                        returned: IrProviderSlice::empty(), // TODO: proper inputs
                    });
                }
                IrProvider::Immediate(condition) => {
                    // TODO: ideally we flatten the entire if-block
                    //       to its `then` block if the condition is true OR
                    //       to its `else` block if the condition is false.
                    //
                    // We have not yet implemented this `if` flattening since
                    // it is potentially a ton of complicated work.
                    let reachability = if bool::from(condition) {
                        IfReachability::OnlyThen
                    } else {
                        // Note in this case we know all code is unreachable
                        // until we enter the `else` block of this `if`.
                        self.reachable = false;
                        IfReachability::OnlyElse
                    };
                    // We are still in need of the `end_label` since jumps
                    // from within the `else` or `then` blocks might occur to
                    // the end of this `if`.
                    let end_label = self.inst_builder.new_label();
                    // Since in this case we know that only one of `then` or
                    // `else` are reachable we do not require to duplicate the
                    // `if` parameters on the provider stack as in the general
                    // case.
                    let else_height = None;
                    // We are not in need of an `else` label if either `then`
                    // or `else` are unreachable since there won't be a `br_nez`
                    // instruction that would usually target it.
                    let else_label = None;
                    self.control_frames.push_frame(IfControlFrame::new(
                        results,
                        block_type,
                        end_label,
                        else_label,
                        stack_height,
                        else_height,
                        reachability,
                    ));
                }
            }
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
        self.update_allow_set_local_override(false);
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
        let end_of_then_is_reachable = self.is_reachable();
        // At this point we know if the end of the `then` block of the paren
        // `if` block is reachable so we update the parent `if` frame.
        //
        // Note: This information is important to decide whether code is
        //       reachable after the `if` block (including `else`) ends.
        if_frame.update_end_of_then_reachability(end_of_then_is_reachable);
        // We need to check if the `else` block is known to be reachable.
        let then_reachable = if_frame.is_then_reachable();
        let else_reachable = if_frame.is_else_reachable();
        // Create the jump from the end of the `then` block to the `if`
        // block's end label in case the end of `then` is reachable.
        if then_reachable {
            // Return providers on the stack to where the `if` block expects
            // its results in case the `if` block has return values.
            let results = if_frame.branch_results();
            let returned = self.providers.pop_n(results.len() as usize);
            if else_reachable {
                // Case: both `then` and `else` are reachable
                let returned = self.provider_slices.alloc(returned);
                let dst_pc =
                    self.try_resolve_label(if_frame.end_label(), |pc| Reloc::Br { inst_idx: pc });
                let target = Target::from(dst_pc);
                self.push_instr(Instruction::Br {
                    target,
                    results,
                    returned,
                });
                // Now resolve labels for the instructions of the `else` block
                if let Some(else_label) = if_frame.else_label() {
                    self.inst_builder.resolve_label(else_label);
                }
            } else {
                // Case: only `then` is reachable
                self.inst_builder.push_copy_many_instr(
                    &mut self.provider_slices,
                    results,
                    returned.as_slice(),
                );
            }
        }
        if else_reachable {
            // We need to reset the value stack to exactly how it has been
            // when entering the `if` in the first place so that the `else`
            // block has the same parameters on top of the stack.
            if let Some(else_height) = if_frame.else_height() {
                self.providers.shrink_to(else_height);
            }
        }
        self.control_frames.push_frame(if_frame);
        // We can reset reachability now since the parent `if` block was reachable.
        self.reachable = else_reachable;
        Ok(())
    }

    /// Translates a Wasm `end` control flow operator.
    pub fn translate_end(&mut self) -> Result<(), ModuleError> {
        self.update_allow_set_local_override(false);
        match self.control_frames.pop_frame() {
            ControlFrame::Block(frame) => self.translate_end_block(frame),
            ControlFrame::Loop(frame) => self.translate_end_loop(frame),
            ControlFrame::If(frame) => self.translate_end_if(frame),
            ControlFrame::Unreachable(frame) => self.translate_end_unreachable(frame),
        }
    }

    /// Updates the value stack, removing all intermediate values used
    /// to execute the block and put its results on the stack.
    fn finalize_frame(&mut self, frame: ControlFrame) -> Result<(), ModuleError> {
        let frame_stack_height = frame.stack_height();
        let len_results = frame.block_type().len_results(&self.engine) as usize;
        self.providers.shrink_to(frame_stack_height);
        self.providers.push_dynamic_many(len_results);
        Ok(())
    }

    /// Inserts a `copy` or `copy_many` instruction to write back results to
    /// parent control flow block upon ending one of its children.
    fn copy_frame_results(&mut self, results: IrRegisterSlice) -> Result<(), ModuleError> {
        let returned = self.providers.peek_n(results.len() as usize);
        self.inst_builder
            .push_copy_many_instr(&mut self.provider_slices, results, returned);
        Ok(())
    }

    /// Translates a Wasm `end` control flow operator for a `block`.
    fn translate_end_block(&mut self, frame: BlockControlFrame) -> Result<(), ModuleError> {
        if self.is_reachable() && !self.control_frames.is_empty() {
            // Write back results to where the parent control flow frame
            // is expecting them.
            self.copy_frame_results(frame.end_results())?;
        }
        // At this point we can resolve the `End` labels.
        // Note that `loop` control frames do not have an `End` label.
        self.inst_builder.resolve_label(frame.end_label());
        // These bindings are required because of borrowing issues.
        if self.control_frames.is_empty() {
            // We are closing the function body block therefore we
            // are required to return the function execution results.
            self.translate_return()?;
        }
        // Code after a `block` ends is generally reachable again.
        self.reachable = true;
        self.finalize_frame(frame.into())
    }

    /// Translates a Wasm `end` control flow operator for a `loop`.
    fn translate_end_loop(&mut self, frame: LoopControlFrame) -> Result<(), ModuleError> {
        if self.is_reachable() {
            // Write back results to where the parent control flow frame
            // is expecting them.
            self.copy_frame_results(frame.end_results())?;
        }
        // Code after a `loop` ends is generally reachable again.
        self.reachable = true;
        self.finalize_frame(frame.into())
    }

    /// Translates a Wasm `end` control flow operator for an `if`.
    fn translate_end_if(&mut self, frame: IfControlFrame) -> Result<(), ModuleError> {
        let visited_else = frame.visited_else();
        let req_copy = match visited_else {
            true => self.is_reachable() && !self.control_frames.is_empty(),
            false => !self.control_frames.is_empty(),
        };
        let frame_results = frame.end_results();
        if req_copy && visited_else {
            self.copy_frame_results(frame_results)?;
        }
        // At this point we can resolve the `Else` label.
        //
        // Note: The `Else` label might have already been resolved
        //       in case there was an `Else` block.
        if let Some(else_label) = frame.else_label() {
            self.inst_builder.resolve_label_if_unresolved(else_label)
        }
        if req_copy && !visited_else {
            self.copy_frame_results(frame_results)?;
        }
        // At this point we can resolve the `End` labels.
        // Note that `loop` control frames do not have an `End` label.
        self.inst_builder.resolve_label(frame.end_label());
        if self.control_frames.is_empty() {
            // If the control flow frames stack is empty after this point
            // we know that we are ending the function body `block`
            // frame and therefore we have to return from the function.
            self.translate_return()?;
        }
        // Code after a `if` ends is generally reachable again.
        self.reachable = true;
        self.finalize_frame(frame.into())
    }

    /// Translates a Wasm `end` control flow operator for an unreachable frame.
    fn translate_end_unreachable(
        &mut self,
        _frame: UnreachableControlFrame,
    ) -> Result<(), ModuleError> {
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `br` control flow operator.
    pub fn translate_br(&mut self, relative_depth: u32) -> Result<(), ModuleError> {
        self.update_allow_set_local_override(false);
        self.translate_if_reachable(|builder| {
            match builder.acquire_target(relative_depth, |pc| Reloc::Br { inst_idx: pc }) {
                AquiredTarget::Branch {
                    target,
                    results,
                    returned,
                } => {
                    builder.push_instr(Instruction::Br {
                        target,
                        results,
                        returned,
                    });
                }
                AquiredTarget::Return => {
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
        self.update_allow_set_local_override(false);
        self.translate_if_reachable(|builder| {
            let condition = builder.providers.pop();
            match condition {
                IrProvider::Register(condition) => {
                    match builder.acquire_target(relative_depth, |pc| Reloc::Br { inst_idx: pc }) {
                        AquiredTarget::Branch {
                            target,
                            results,
                            returned,
                        } => {
                            builder.push_instr(Instruction::BrNez {
                                target,
                                condition,
                                results,
                                returned,
                            });
                        }
                        AquiredTarget::Return => {
                            let results = builder.return_provider_slice();
                            builder.push_instr(Instruction::ReturnNez { results, condition });
                        }
                    }
                }
                IrProvider::Immediate(condition) => {
                    if bool::from(condition) {
                        // In this case the branch always takes place and
                        // therefore is unconditional so we can replace it
                        // with a `br` instruction.
                        return builder.translate_br(relative_depth);
                    } else {
                        // In this case the branch never takes place and
                        // therefore is a no-op. We simply do nothing in this
                        // case.
                    }
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
        self.update_allow_set_local_override(false);
        self.translate_if_reachable(|builder| {
            fn make_branch_instr<F>(
                builder: &mut FunctionBuilder,
                depth: RelativeDepth,
                make_reloc: F,
            ) -> IrInstruction
            where
                F: FnOnce(Instr) -> Reloc,
            {
                match builder.acquire_target(depth.into_u32(), make_reloc) {
                    AquiredTarget::Branch {
                        target,
                        results,
                        returned,
                    } => Instruction::Br {
                        target,
                        results,
                        returned,
                    },
                    AquiredTarget::Return => {
                        let results = builder.return_provider_slice();
                        Instruction::Return { results }
                    }
                }
            }

            fn make_branch(
                builder: &mut FunctionBuilder,
                n: usize,
                depth: RelativeDepth,
            ) -> IrInstruction {
                make_branch_instr(builder, depth, |pc| Reloc::BrTable {
                    inst_idx: pc,
                    target_idx: n,
                })
            }

            fn make_const_branch(
                builder: &mut FunctionBuilder,
                depth: RelativeDepth,
            ) -> IrInstruction {
                make_branch_instr(builder, depth, |pc| Reloc::Br { inst_idx: pc })
            }

            let case = builder.providers.pop();
            match case {
                IrProvider::Register(case) => {
                    let branches = targets
                        .into_iter()
                        .enumerate()
                        .map(|(n, depth)| make_branch(builder, n, depth))
                        .collect::<Vec<_>>();
                    // We do not include the default target in `len_branches`.
                    let len_non_default_targets = branches.len();
                    let default_branch = make_branch(builder, len_non_default_targets, default);
                    // Note: We include the default branch in this amount.
                    let len_targets = len_non_default_targets + 1;
                    builder.push_instr(Instruction::BrTable { case, len_targets });
                    for branch in branches {
                        builder.push_instr(branch);
                    }
                    builder.push_instr(default_branch);
                    builder.reachable = false;
                }
                IrProvider::Immediate(case) => {
                    // In this case the case is a constant value and therefore
                    // it is possible to pre-compute the label which is going to
                    // be used for branching always.
                    let index = u32::from(case) as usize;
                    let branches = targets.into_iter().collect::<Vec<_>>();
                    let case = branches
                        .get(index)
                        .cloned()
                        .map(|depth| make_const_branch(builder, depth))
                        .unwrap_or_else(|| make_const_branch(builder, default));
                    builder.push_instr(case);
                    builder.reachable = false;
                }
            }
            Ok(())
        })
    }

    /// Translates a Wasm `return` control flow operator.
    pub fn translate_return(&mut self) -> Result<(), ModuleError> {
        self.update_allow_set_local_override(false);
        self.translate_if_reachable(|builder| {
            let results = builder.return_provider_slice();
            builder.push_instr(Instruction::Return { results });
            builder.reachable = false;
            Ok(())
        })
    }

    /// Adjusts the emulated provider stack given the [`FuncType`] of the call.
    fn adjust_provider_stack_for_call(
        &mut self,
        func_type: &FuncType,
    ) -> (IrProviderSlice, IrRegisterSlice) {
        let (params, results) = func_type.params_results();
        let params_providers = self.providers.pop_n(params.len());
        let params_slice = self.provider_slices.alloc(params_providers);
        let results_slice = self.providers.push_dynamic_many(results.len());
        (params_slice, results_slice)
    }

    /// Translates a Wasm `call` instruction.
    pub fn translate_call(&mut self, func_idx: FuncIdx) -> Result<(), ModuleError> {
        self.update_allow_set_local_override(false);
        self.translate_if_reachable(|builder| {
            let func_type = builder.func_type_of(func_idx);
            let (params, results) = builder.adjust_provider_stack_for_call(&func_type);
            builder.push_instr(Instruction::Call {
                func_idx,
                results,
                params,
            });
            Ok(())
        })
    }

    /// Translates a Wasm `call_indirect` instruction.
    pub fn translate_call_indirect(
        &mut self,
        func_type_idx: FuncTypeIdx,
        table_idx: TableIdx,
    ) -> Result<(), ModuleError> {
        self.update_allow_set_local_override(false);
        self.translate_if_reachable(|builder| {
            /// The default Wasm MVP table index.
            const DEFAULT_TABLE_INDEX: u32 = 0;
            assert_eq!(table_idx.into_u32(), DEFAULT_TABLE_INDEX);
            let index = builder.providers.pop();
            let func_type = builder.func_type_at(func_type_idx);
            let (params, results) = builder.adjust_provider_stack_for_call(&func_type);
            builder.push_instr(Instruction::CallIndirect {
                func_type_idx,
                results,
                index,
                params,
            });
            Ok(())
        })
    }

    /// Translates a Wasm `drop` instruction.
    pub fn translate_drop(&mut self) -> Result<(), ModuleError> {
        self.update_allow_set_local_override(false);
        self.translate_if_reachable(|builder| {
            // Note: there is no need to synthesize a `drop` instruction for
            //       the register machine based `wasmi` bytecode. It is enough
            //       to remove the emulated stack entry during compilation.
            builder.providers.pop();
            Ok(())
        })
    }

    /// Translate a `wasmi` `copy` instruction.
    ///
    /// # Note
    ///
    /// This filters out nop-copies which are copies where
    /// `result` and `input` registers are the same.
    fn translate_copy(&mut self, result: IrRegister, input: IrProvider) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            if let IrProvider::Register(input) = input {
                if input == result {
                    // Bail out to avoid nop copies.
                    return Ok(());
                }
            }
            builder
                .inst_builder
                .push_inst(Instruction::Copy { result, input });
            Ok(())
        })
    }

    /// Pushes the [`IrInstruction`] to the instruction builder.
    ///
    /// # Note
    ///
    /// This also automatically updates the `allow_set_local_override` flag.
    fn push_instr(&mut self, mut instr: IrInstruction) -> Instr {
        let has_result = instr.result_mut().is_some();
        self.update_allow_set_local_override(has_result);
        self.inst_builder.push_inst(instr)
    }

    /// Translates a Wasm `select` instruction.
    pub fn translate_select(&mut self) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let (v0, v1, selector) = builder.providers.pop3();
            let result = builder.providers.push_dynamic();
            match selector {
                IrProvider::Register(condition) => {
                    builder.push_instr(Instruction::Select {
                        result,
                        condition,
                        if_true: v0,
                        if_false: v1,
                    });
                }
                IrProvider::Immediate(condition) => {
                    // Note: if the condition is a constant we can replace the
                    //       `select` instruction with one of its arms.
                    let condition = bool::from(condition);
                    let input = if condition { v0 } else { v1 };
                    builder.translate_copy(result, input)?;
                }
            }
            Ok(())
        })
    }

    /// Translate a Wasm `local.get` instruction.
    pub fn translate_local_get(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        self.update_allow_set_local_override(false);
        self.translate_if_reachable(|builder| {
            builder.providers.push_local(local_idx);
            Ok(())
        })
    }

    /// Translate a Wasm `local.set` instruction.
    pub fn translate_local_set(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        let allow_override = self.update_allow_set_local_override(false);
        self.translate_if_reachable(|builder| {
            let input = builder.providers.pop();
            let result = IrRegister::Local(local_idx as usize);
            let copy_preserve =
                if let Some(preserved) = builder.providers.preserve_locals(local_idx) {
                    // Note: insert a `copy` instruction to preserve previous
                    //       values of all `local.get` calls to the same index.
                    let replace = IrRegister::Local(local_idx as usize);
                    debug_assert!(matches!(preserved, IrRegister::Preserved(_)));
                    Some(Instruction::Copy {
                        result: preserved,
                        input: replace.into(),
                    })
                } else {
                    None
                };
            let last_inst = builder.inst_builder.peek_mut();
            if let Some(last_inst) = last_inst {
                if let Some(last_result) = last_inst.result_mut() {
                    if !last_result.is_local() && allow_override {
                        // Note: - the last instruction emits a single result
                        //         value that we can simply alter.
                        //       - we prevent from re-overriding via the
                        //         `is_local` check above.
                        *last_result = result;
                        if let Some(copy_preserve) = copy_preserve {
                            builder.push_instr(copy_preserve);
                        }
                        return Ok(());
                    }
                }
            }
            if let Some(copy_preserve) = copy_preserve {
                builder.push_instr(copy_preserve);
            }
            // Note: we simply emit a `copy` instruction as a fall back.
            builder.translate_copy(result, input)?;
            Ok(())
        })
    }

    /// Translate a Wasm `local.tee` instruction.
    pub fn translate_local_tee(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            builder.translate_local_set(local_idx)?;
            builder.providers.push_local(local_idx);
            Ok(())
        })
    }

    /// Translate a Wasm `global.get` instruction.
    pub fn translate_global_get(&mut self, global_idx: GlobalIdx) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let result = builder.providers.push_dynamic();
            let global = Global::from(global_idx.into_u32());
            builder.push_instr(Instruction::GlobalGet { result, global });
            Ok(())
        })
    }

    /// Translate a Wasm `global.set` instruction.
    pub fn translate_global_set(&mut self, global_idx: GlobalIdx) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let global = Global::from(global_idx.into_u32());
            let value = builder.providers.pop();
            builder.push_instr(Instruction::GlobalSet { global, value });
            Ok(())
        })
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
        make_inst: fn(IrRegister, IrRegister, Offset) -> IrInstruction,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
            let ptr = builder.providers.pop();
            let result = builder.providers.push_dynamic();
            let offset = Offset::from(offset);
            match ptr {
                IrProvider::Register(ptr) => {
                    builder.push_instr(make_inst(result, ptr, offset));
                }
                IrProvider::Immediate(ptr) => {
                    builder.translate_copy(result, ptr.into())?;
                    builder.push_instr(make_inst(result, result, offset));
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
        make_inst: fn(IrRegister, Offset, IrProvider) -> IrInstruction,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
            let offset = Offset::from(offset);
            let copy_result = match builder.providers.peek2() {
                (IrProvider::Immediate(_), _) => Some(builder.providers.peek_dynamic()),
                _ => None,
            };
            let (ptr, value) = builder.providers.pop2();
            match ptr {
                IrProvider::Register(ptr) => {
                    builder.push_instr(make_inst(ptr, offset, value));
                }
                IrProvider::Immediate(ptr) => {
                    // Note: store the constant pointer value into a temporary
                    //       `temp` register and use the register instead since
                    //       otherwise we cannot construct the `wasmi` bytecode
                    //       that expects the pointer value to be provided in
                    //       a register.
                    //       After the store instruction we immediate have to
                    //       pop the temporarily used register again.
                    let copy_result = copy_result.expect("register for intermediate copy");
                    builder.translate_copy(copy_result, ptr.into())?;
                    builder.push_instr(make_inst(copy_result, offset, value));
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
        debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
        self.translate_if_reachable(|builder| {
            let result = builder.providers.push_dynamic();
            builder.push_instr(Instruction::MemorySize { result });
            Ok(())
        })
    }

    /// Translate a Wasm `memory.grow` instruction.
    pub fn translate_memory_grow(&mut self, memory_idx: MemoryIdx) -> Result<(), ModuleError> {
        debug_assert_eq!(memory_idx.into_u32(), DEFAULT_MEMORY_INDEX);
        self.translate_if_reachable(|builder| {
            let amount = builder.providers.pop();
            let result = builder.providers.push_dynamic();
            builder.push_instr(Instruction::MemoryGrow { result, amount });
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
        T: Into<UntypedValue>,
    {
        self.update_allow_set_local_override(false);
        self.translate_if_reachable(|builder| {
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
    fn translate_binary_cmp(
        &mut self,
        make_op: fn(IrRegister, IrRegister, IrProvider) -> IrInstruction,
        exec_op: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let (lhs, rhs) = builder.providers.pop2();
            match (lhs, rhs) {
                (IrProvider::Register(lhs), rhs) => {
                    let result = builder.providers.push_dynamic();
                    builder.push_instr(make_op(result, lhs, rhs));
                }
                (lhs @ IrProvider::Immediate(_), IrProvider::Register(rhs)) => {
                    // Note: we can swap the operands to avoid having to copy `rhs`.
                    let result = builder.providers.push_dynamic();
                    builder.push_instr(make_op(result, rhs, lhs));
                }
                (IrProvider::Immediate(lhs), IrProvider::Immediate(rhs)) => {
                    // Note: precompute result and push onto provider stack
                    builder.providers.push_const(exec_op(lhs, rhs));
                }
            };
            Ok(())
        })
    }

    /// Translate a Wasm `i32.eq` instruction.
    pub fn translate_i32_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(I32Eq), UntypedValue::i32_eq)
    }

    /// Translate a Wasm `i32.ne` instruction.
    pub fn translate_i32_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(I32Ne), UntypedValue::i32_ne)
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
    fn translate_ord(
        &mut self,
        make_op: fn(IrRegister, IrRegister, IrProvider) -> IrInstruction,
        swap_op: fn(IrRegister, IrRegister, IrProvider) -> IrInstruction,
        exec_op: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let (lhs, rhs) = builder.providers.pop2();
            match (lhs, rhs) {
                (IrProvider::Register(lhs), rhs) => {
                    let result = builder.providers.push_dynamic();
                    builder.push_instr(make_op(result, lhs, rhs));
                }
                (lhs @ IrProvider::Immediate(_), IrProvider::Register(rhs)) => {
                    // Note: we can swap the operands to avoid having to copy `rhs`.
                    let result = builder.providers.push_dynamic();
                    builder.push_instr(swap_op(result, rhs, lhs));
                }
                (IrProvider::Immediate(lhs), IrProvider::Immediate(rhs)) => {
                    // Note: precompute result and push onto provider stack
                    builder.providers.push_const(exec_op(lhs, rhs));
                }
            };
            Ok(())
        })
    }

    /// Translate a Wasm `i32.lt` instruction.
    pub fn translate_i32_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32LtS), make_op!(I32GtS), UntypedValue::i32_lt_s)
    }

    /// Translate a Wasm `u32.lt` instruction.
    pub fn translate_u32_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32LtU), make_op!(I32GtU), UntypedValue::i32_lt_u)
    }

    /// Translate a Wasm `i32.gt` instruction.
    pub fn translate_i32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32GtS), make_op!(I32LtS), UntypedValue::i32_gt_s)
    }

    /// Translate a Wasm `u32.gt` instruction.
    pub fn translate_u32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32GtU), make_op!(I32LtU), UntypedValue::i32_gt_u)
    }

    /// Translate a Wasm `i32.le` instruction.
    pub fn translate_i32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32LeS), make_op!(I32GeS), UntypedValue::i32_le_s)
    }

    /// Translate a Wasm `u32.le` instruction.
    pub fn translate_u32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32LeU), make_op!(I32GeU), UntypedValue::i32_le_u)
    }

    /// Translate a Wasm `i32.ge` instruction.
    pub fn translate_i32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32GeS), make_op!(I32LeS), UntypedValue::i32_ge_s)
    }

    /// Translate a Wasm `u32.ge` instruction.
    pub fn translate_u32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I32GeU), make_op!(I32LeU), UntypedValue::i32_ge_u)
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
        self.translate_binary_cmp(make_op!(I64Eq), UntypedValue::i64_eq)
    }

    /// Translate a Wasm `i64.ne` instruction.
    pub fn translate_i64_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(I64Ne), UntypedValue::i64_ne)
    }

    /// Translate a Wasm `i64.lt` instruction.
    pub fn translate_i64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64LtS), make_op!(I64GtS), UntypedValue::i64_lt_s)
    }

    /// Translate a Wasm `u64.lt` instruction.
    pub fn translate_u64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64LtU), make_op!(I64GtU), UntypedValue::i64_lt_u)
    }

    /// Translate a Wasm `i64.gt` instruction.
    pub fn translate_i64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64GtS), make_op!(I64LtS), UntypedValue::i64_gt_s)
    }

    /// Translate a Wasm `u64.gt` instruction.
    pub fn translate_u64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64GtU), make_op!(I64LtU), UntypedValue::i64_gt_u)
    }

    /// Translate a Wasm `i64.le` instruction.
    pub fn translate_i64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64LeS), make_op!(I64GeS), UntypedValue::i64_le_s)
    }

    /// Translate a Wasm `u64.le` instruction.
    pub fn translate_u64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64LeU), make_op!(I64GeU), UntypedValue::i64_le_u)
    }

    /// Translate a Wasm `i64.ge` instruction.
    pub fn translate_i64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64GeS), make_op!(I64LeS), UntypedValue::i64_ge_s)
    }

    /// Translate a Wasm `u64.ge` instruction.
    pub fn translate_u64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(I64GeU), make_op!(I64LeU), UntypedValue::i64_ge_u)
    }

    /// Translate a Wasm `f32.eq` instruction.
    pub fn translate_f32_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(F32Eq), UntypedValue::f32_eq)
    }

    /// Translate a Wasm `f32.ne` instruction.
    pub fn translate_f32_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(F32Ne), UntypedValue::f32_ne)
    }

    /// Translate a Wasm `f32.lt` instruction.
    pub fn translate_f32_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F32Lt), make_op!(F32Gt), UntypedValue::f32_lt)
    }

    /// Translate a Wasm `f32.gt` instruction.
    pub fn translate_f32_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F32Gt), make_op!(F32Lt), UntypedValue::f32_gt)
    }

    /// Translate a Wasm `f32.le` instruction.
    pub fn translate_f32_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F32Le), make_op!(F32Ge), UntypedValue::f32_le)
    }

    /// Translate a Wasm `f32.ge` instruction.
    pub fn translate_f32_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F32Ge), make_op!(F32Le), UntypedValue::f32_ge)
    }

    /// Translate a Wasm `f64.eq` instruction.
    pub fn translate_f64_eq(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(F64Eq), UntypedValue::f64_eq)
    }

    /// Translate a Wasm `f64.ne` instruction.
    pub fn translate_f64_ne(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_cmp(make_op!(F64Ne), UntypedValue::f64_ne)
    }

    /// Translate a Wasm `f64.lt` instruction.
    pub fn translate_f64_lt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F64Lt), make_op!(F64Gt), UntypedValue::f64_lt)
    }

    /// Translate a Wasm `f64.gt` instruction.
    pub fn translate_f64_gt(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F64Gt), make_op!(F64Lt), UntypedValue::f64_gt)
    }

    /// Translate a Wasm `f64.le` instruction.
    pub fn translate_f64_le(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F64Le), make_op!(F64Ge), UntypedValue::f64_le)
    }

    /// Translate a Wasm `f64.ge` instruction.
    pub fn translate_f64_ge(&mut self) -> Result<(), ModuleError> {
        self.translate_ord(make_op!(F64Ge), make_op!(F64Le), UntypedValue::f64_ge)
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
    fn translate_unary_operation(
        &mut self,
        make_op: fn(IrRegister, IrRegister) -> IrInstruction,
        exec_op: fn(UntypedValue) -> UntypedValue,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let input = builder.providers.pop();
            match input {
                IrProvider::Register(input) => {
                    let result = builder.providers.push_dynamic();
                    builder.push_instr(make_op(result, input));
                }
                IrProvider::Immediate(input) => {
                    builder.providers.push_const(exec_op(input));
                }
            }
            Ok(())
        })
    }

    /// Translate a Wasm `i32.clz` instruction.
    pub fn translate_i32_clz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I32Clz { result, input },
            UntypedValue::i32_clz,
        )
    }

    /// Translate a Wasm `i32.ctz` instruction.
    pub fn translate_i32_ctz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I32Ctz { result, input },
            UntypedValue::i32_ctz,
        )
    }

    /// Translate a Wasm `i32.popcnt` instruction.
    pub fn translate_i32_popcnt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I32Popcnt { result, input },
            UntypedValue::i32_popcnt,
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
    pub fn translate_binary_operation(
        &mut self,
        make_op: fn(IrRegister, IrRegister, IrProvider) -> IrInstruction,
        exec_op: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let copy_result = match builder.providers.peek2() {
                (IrProvider::Immediate(_), IrProvider::Register(_)) => {
                    Some(builder.providers.peek_dynamic())
                }
                _ => None,
            };
            let (lhs, rhs) = builder.providers.pop2();
            match (lhs, rhs) {
                (IrProvider::Register(lhs), rhs) => {
                    let result = builder.providers.push_dynamic();
                    builder.push_instr(make_op(result, lhs, rhs));
                }
                (lhs @ IrProvider::Immediate(_), IrProvider::Register(rhs)) => {
                    // Note: this case is a bit tricky for non-commutative operations.
                    //
                    // In order to be able to represent the constant left-hand side
                    // operand for the instruction we need to `copy` it into a register
                    // first.
                    let copy_result = copy_result.expect("register for intermediate copy");
                    let result = builder.providers.push_dynamic();
                    builder.translate_copy(copy_result, lhs)?;
                    builder.push_instr(make_op(result, copy_result, rhs.into()));
                }
                (IrProvider::Immediate(lhs), IrProvider::Immediate(rhs)) => {
                    // Note: both operands are constant so we can evaluate the result.
                    builder.update_allow_set_local_override(false);
                    builder.providers.push_const(exec_op(lhs, rhs));
                }
            }
            Ok(())
        })
    }

    /// Translate a fallible non-commutative binary Wasm instruction.
    ///
    /// - `{i32, u32, i64, u64, f32, f64}.div`
    /// - `{i32, u32, i64, u64}.rem`
    pub fn translate_fallible_binary_operation(
        &mut self,
        make_op: fn(IrRegister, IrRegister, IrProvider) -> IrInstruction,
        exec_op: fn(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let copy_result = match builder.providers.peek2() {
                (IrProvider::Immediate(_), IrProvider::Register(_)) => {
                    Some(builder.providers.peek_dynamic())
                }
                _ => None,
            };
            let (lhs, rhs) = builder.providers.pop2();
            match (lhs, rhs) {
                (IrProvider::Register(lhs), rhs) => {
                    let result = builder.providers.push_dynamic();
                    builder.push_instr(make_op(result, lhs, rhs));
                }
                (lhs @ IrProvider::Immediate(_), IrProvider::Register(rhs)) => {
                    // Note: this case is a bit tricky for non-commutative operations.
                    //
                    // In order to be able to represent the constant left-hand side
                    // operand for the instruction we need to `copy` it into a register
                    // first.
                    let copy_result = copy_result.expect("register for intermediate copy");
                    let result = builder.providers.push_dynamic();
                    builder.translate_copy(copy_result, lhs)?;
                    builder.push_instr(make_op(result, copy_result, rhs.into()));
                }
                (IrProvider::Immediate(lhs), IrProvider::Immediate(rhs)) => {
                    // Note: both operands are constant so we can evaluate the result.
                    match exec_op(lhs, rhs) {
                        Ok(result) => {
                            builder.update_allow_set_local_override(false);
                            builder.providers.push_const(result);
                        }
                        Err(trap_code) => {
                            builder.push_instr(Instruction::Trap { trap_code });
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
    fn translate_commutative_binary_operation(
        &mut self,
        make_op: fn(IrRegister, IrRegister, IrProvider) -> IrInstruction,
        exec_op: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let (lhs, rhs) = builder.providers.pop2();
            match (lhs, rhs) {
                (IrProvider::Register(lhs), rhs) => {
                    let result = builder.providers.push_dynamic();
                    builder.push_instr(make_op(result, lhs, rhs));
                }
                (lhs @ IrProvider::Immediate(_), IrProvider::Register(rhs)) => {
                    // Note: due to commutativity of the operation we can swap the parameters.
                    let result = builder.providers.push_dynamic();
                    builder.push_instr(make_op(result, rhs, lhs));
                }
                (IrProvider::Immediate(lhs), IrProvider::Immediate(rhs)) => {
                    builder.providers.push_const(exec_op(lhs, rhs));
                }
            };
            Ok(())
        })
    }

    /// Translate a Wasm `i32.add` instruction.
    pub fn translate_i32_add(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I32Add), UntypedValue::i32_add)
    }

    /// Translate a Wasm `i32.sub` instruction.
    pub fn translate_i32_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32Sub), UntypedValue::i32_sub)
    }

    /// Translate a Wasm `i32.mul` instruction.
    pub fn translate_i32_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I32Mul), UntypedValue::i32_mul)
    }

    /// Translate a Wasm `i32.div` instruction.
    pub fn translate_i32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I32DivS), UntypedValue::i32_div_s)
    }

    /// Translate a Wasm `u32.div` instruction.
    pub fn translate_u32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I32DivU), UntypedValue::i32_div_u)
    }

    /// Translate a Wasm `i32.rem` instruction.
    pub fn translate_i32_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I32RemS), UntypedValue::i32_rem_s)
    }

    /// Translate a Wasm `u32.rem` instruction.
    pub fn translate_u32_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I32RemU), UntypedValue::i32_rem_u)
    }

    /// Translate a Wasm `i32.and` instruction.
    pub fn translate_i32_and(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I32And), UntypedValue::i32_and)
    }

    /// Translate a Wasm `i32.or` instruction.
    pub fn translate_i32_or(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I32Or), UntypedValue::i32_or)
    }

    /// Translate a Wasm `i32.xor` instruction.
    pub fn translate_i32_xor(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I32Xor), UntypedValue::i32_xor)
    }

    /// Translate a Wasm `i32.shl` instruction.
    pub fn translate_i32_shl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32Shl), UntypedValue::i32_shl)
    }

    /// Translate a Wasm `i32.shr` instruction.
    pub fn translate_i32_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32ShrS), UntypedValue::i32_shr_s)
    }

    /// Translate a Wasm `u32.shr` instruction.
    pub fn translate_u32_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32ShrU), UntypedValue::i32_shr_u)
    }

    /// Translate a Wasm `i32.rotl` instruction.
    pub fn translate_i32_rotl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32Rotl), UntypedValue::i32_rotl)
    }

    /// Translate a Wasm `i32.rotr` instruction.
    pub fn translate_i32_rotr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I32Rotr), UntypedValue::i32_rotr)
    }

    /// Translate a Wasm `i64.clz` instruction.
    pub fn translate_i64_clz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I64Clz { result, input },
            UntypedValue::i64_clz,
        )
    }

    /// Translate a Wasm `i64.ctz` instruction.
    pub fn translate_i64_ctz(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I64Ctz { result, input },
            UntypedValue::i64_ctz,
        )
    }

    /// Translate a Wasm `i64.popcnt` instruction.
    pub fn translate_i64_popcnt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::I64Popcnt { result, input },
            UntypedValue::i64_popcnt,
        )
    }

    /// Translate a Wasm `i64.add` instruction.
    pub fn translate_i64_add(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I64Add), UntypedValue::i64_add)
    }

    /// Translate a Wasm `i64.sub` instruction.
    pub fn translate_i64_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64Sub), UntypedValue::i64_sub)
    }

    /// Translate a Wasm `i64.mul` instruction.
    pub fn translate_i64_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I64Mul), UntypedValue::i64_mul)
    }

    /// Translate a Wasm `i64.div` instruction.
    pub fn translate_i64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I64DivS), UntypedValue::i64_div_s)
    }

    /// Translate a Wasm `u64.div` instruction.
    pub fn translate_u64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I64DivU), UntypedValue::i64_div_u)
    }

    /// Translate a Wasm `i64.rem` instruction.
    pub fn translate_i64_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I64RemS), UntypedValue::i64_rem_s)
    }

    /// Translate a Wasm `u64.rem` instruction.
    pub fn translate_u64_rem(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(I64RemU), UntypedValue::i64_rem_u)
    }

    /// Translate a Wasm `i64.and` instruction.
    pub fn translate_i64_and(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I64And), UntypedValue::i64_and)
    }

    /// Translate a Wasm `i64.or` instruction.
    pub fn translate_i64_or(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I64Or), UntypedValue::i64_or)
    }

    /// Translate a Wasm `i64.xor` instruction.
    pub fn translate_i64_xor(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(I64Xor), UntypedValue::i64_xor)
    }

    /// Translate a Wasm `i64.shl` instruction.
    pub fn translate_i64_shl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64Shl), UntypedValue::i64_shl)
    }

    /// Translate a Wasm `i64.shr` instruction.
    pub fn translate_i64_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64ShrS), UntypedValue::i64_shr_s)
    }

    /// Translate a Wasm `u64.shr` instruction.
    pub fn translate_u64_shr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64ShrU), UntypedValue::i64_shr_u)
    }

    /// Translate a Wasm `i64.rotl` instruction.
    pub fn translate_i64_rotl(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64Rotl), UntypedValue::i64_rotl)
    }

    /// Translate a Wasm `i64.rotr` instruction.
    pub fn translate_i64_rotr(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(I64Rotr), UntypedValue::i64_rotr)
    }

    /// Translate a Wasm `f32.abs` instruction.
    pub fn translate_f32_abs(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Abs { result, input },
            UntypedValue::f32_abs,
        )
    }

    /// Translate a Wasm `f32.neg` instruction.
    pub fn translate_f32_neg(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Neg { result, input },
            UntypedValue::f32_neg,
        )
    }

    /// Translate a Wasm `f32.ceil` instruction.
    pub fn translate_f32_ceil(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Ceil { result, input },
            UntypedValue::f32_ceil,
        )
    }

    /// Translate a Wasm `f32.floor` instruction.
    pub fn translate_f32_floor(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Floor { result, input },
            UntypedValue::f32_floor,
        )
    }

    /// Translate a Wasm `f32.trunc` instruction.
    pub fn translate_f32_trunc(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Trunc { result, input },
            UntypedValue::f32_trunc,
        )
    }

    /// Translate a Wasm `f32.nearest` instruction.
    pub fn translate_f32_nearest(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Nearest { result, input },
            UntypedValue::f32_nearest,
        )
    }

    /// Translate a Wasm `f32.sqrt` instruction.
    pub fn translate_f32_sqrt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F32Sqrt { result, input },
            UntypedValue::f32_sqrt,
        )
    }

    /// Translate a Wasm `f32.add` instruction.
    pub fn translate_f32_add(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F32Add), UntypedValue::f32_add)
    }

    /// Translate a Wasm `f32.sub` instruction.
    pub fn translate_f32_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(F32Sub), UntypedValue::f32_sub)
    }

    /// Translate a Wasm `f32.mul` instruction.
    pub fn translate_f32_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F32Mul), UntypedValue::f32_mul)
    }

    /// Translate a Wasm `f32.div` instruction.
    pub fn translate_f32_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(F32Div), UntypedValue::f32_div)
    }

    /// Translate a Wasm `f32.min` instruction.
    pub fn translate_f32_min(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F32Min), UntypedValue::f32_min)
    }

    /// Translate a Wasm `f32.max` instruction.
    pub fn translate_f32_max(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F32Max), UntypedValue::f32_max)
    }

    /// Translate a Wasm `f32.copysign` instruction.
    pub fn translate_f32_copysign(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(F32Copysign), UntypedValue::f32_copysign)
    }

    /// Translate a Wasm `f64.abs` instruction.
    pub fn translate_f64_abs(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Abs { result, input },
            UntypedValue::f64_abs,
        )
    }

    /// Translate a Wasm `f64.neg` instruction.
    pub fn translate_f64_neg(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Neg { result, input },
            UntypedValue::f64_neg,
        )
    }

    /// Translate a Wasm `f64.ceil` instruction.
    pub fn translate_f64_ceil(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Ceil { result, input },
            UntypedValue::f64_ceil,
        )
    }

    /// Translate a Wasm `f64.floor` instruction.
    pub fn translate_f64_floor(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Floor { result, input },
            UntypedValue::f64_floor,
        )
    }

    /// Translate a Wasm `f64.trunc` instruction.
    pub fn translate_f64_trunc(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Trunc { result, input },
            UntypedValue::f64_trunc,
        )
    }

    /// Translate a Wasm `f64.nearest` instruction.
    pub fn translate_f64_nearest(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Nearest { result, input },
            UntypedValue::f64_nearest,
        )
    }

    /// Translate a Wasm `f64.sqrt` instruction.
    pub fn translate_f64_sqrt(&mut self) -> Result<(), ModuleError> {
        self.translate_unary_operation(
            |result, input| Instruction::F64Sqrt { result, input },
            UntypedValue::f64_sqrt,
        )
    }

    /// Translate a Wasm `f64.add` instruction.
    pub fn translate_f64_add(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F64Add), UntypedValue::f64_add)
    }

    /// Translate a Wasm `f64.sub` instruction.
    pub fn translate_f64_sub(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(F64Sub), UntypedValue::f64_sub)
    }

    /// Translate a Wasm `f64.mul` instruction.
    pub fn translate_f64_mul(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F64Mul), UntypedValue::f64_mul)
    }

    /// Translate a Wasm `f64.div` instruction.
    pub fn translate_f64_div(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_binary_operation(make_op!(F64Div), UntypedValue::f64_div)
    }

    /// Translate a Wasm `f64.min` instruction.
    pub fn translate_f64_min(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F64Min), UntypedValue::f64_min)
    }

    /// Translate a Wasm `f64.max` instruction.
    pub fn translate_f64_max(&mut self) -> Result<(), ModuleError> {
        self.translate_commutative_binary_operation(make_op!(F64Max), UntypedValue::f64_max)
    }

    /// Translate a Wasm `f64.copysign` instruction.
    pub fn translate_f64_copysign(&mut self) -> Result<(), ModuleError> {
        self.translate_binary_operation(make_op!(F64Copysign), UntypedValue::f64_copysign)
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
    fn translate_conversion(
        &mut self,
        make_op: fn(IrRegister, IrRegister) -> IrInstruction,
        exec_op: fn(UntypedValue) -> UntypedValue,
    ) -> Result<(), ModuleError> {
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
    fn translate_fallible_conversion(
        &mut self,
        make_op: fn(IrRegister, IrRegister) -> IrInstruction,
        exec_op: fn(UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), ModuleError> {
        self.translate_if_reachable(|builder| {
            let input = builder.providers.pop();
            match input {
                IrProvider::Register(input) => {
                    let result = builder.providers.push_dynamic();
                    builder.push_instr(make_op(result, input));
                }
                IrProvider::Immediate(input) => match exec_op(input) {
                    Ok(result) => {
                        builder.providers.push_const(result);
                    }
                    Err(trap_code) => {
                        builder.push_instr(Instruction::Trap { trap_code });
                        builder.reachable = false;
                    }
                },
            }
            Ok(())
        })
    }

    /// Translate a Wasm `i32.wrap_i64` instruction.
    pub fn translate_i32_wrap_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(I32WrapI64), UntypedValue::i32_wrap_i64)
    }

    /// Translate a Wasm `i32.trunc_f32` instruction.
    pub fn translate_i32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(unary_op!(I32TruncSF32), UntypedValue::i32_trunc_f32_s)
    }

    /// Translate a Wasm `u32.trunc_f32` instruction.
    pub fn translate_u32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(unary_op!(I32TruncUF32), UntypedValue::i32_trunc_f32_u)
    }

    /// Translate a Wasm `i32.trunc_f64` instruction.
    pub fn translate_i32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(unary_op!(I32TruncSF64), UntypedValue::i32_trunc_f64_s)
    }

    /// Translate a Wasm `u32.trunc_f64` instruction.
    pub fn translate_u32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(unary_op!(I32TruncUF64), UntypedValue::i32_trunc_f64_u)
    }

    /// Translate a Wasm `i64.extend_i32` instruction.
    pub fn translate_i64_extend_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(I64ExtendSI32), UntypedValue::i64_extend_i32_s)
    }

    /// Translate a Wasm `u64.extend_i32` instruction.
    pub fn translate_u64_extend_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(I64ExtendUI32), UntypedValue::i64_extend_i32_u)
    }

    /// Translate a Wasm `i64.trunc_f32` instruction.
    pub fn translate_i64_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(unary_op!(I64TruncSF32), UntypedValue::i64_trunc_f32_s)
    }

    /// Translate a Wasm `u64.trunc_f32` instruction.
    pub fn translate_u64_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(unary_op!(I64TruncUF32), UntypedValue::i64_trunc_f32_u)
    }

    /// Translate a Wasm `i64.trunc_f64` instruction.
    pub fn translate_i64_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(unary_op!(I64TruncSF64), UntypedValue::i64_trunc_f64_s)
    }

    /// Translate a Wasm `u64.trunc_f64` instruction.
    pub fn translate_u64_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_fallible_conversion(unary_op!(I64TruncUF64), UntypedValue::i64_trunc_f64_u)
    }

    /// Translate a Wasm `f32.convert_i32` instruction.
    pub fn translate_f32_convert_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F32ConvertSI32), UntypedValue::f32_convert_i32_s)
    }

    /// Translate a Wasm `f32.convert_u32` instruction.
    pub fn translate_f32_convert_u32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F32ConvertUI32), UntypedValue::f32_convert_i32_u)
    }

    /// Translate a Wasm `f32.convert_i64` instruction.
    pub fn translate_f32_convert_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F32ConvertSI64), UntypedValue::f32_convert_i64_s)
    }

    /// Translate a Wasm `f32.convert_u64` instruction.
    pub fn translate_f32_convert_u64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F32ConvertUI64), UntypedValue::f32_convert_i64_u)
    }

    /// Translate a Wasm `f32.demote_f64` instruction.
    pub fn translate_f32_demote_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F32DemoteF64), UntypedValue::f32_demote_f64)
    }

    /// Translate a Wasm `f64.convert_i32` instruction.
    pub fn translate_f64_convert_i32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F64ConvertSI32), UntypedValue::f64_convert_i32_s)
    }

    /// Translate a Wasm `f64.convert_u32` instruction.
    pub fn translate_f64_convert_u32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F64ConvertUI32), UntypedValue::f64_convert_i32_u)
    }

    /// Translate a Wasm `f64.convert_i64` instruction.
    pub fn translate_f64_convert_i64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F64ConvertSI64), UntypedValue::f64_convert_i64_s)
    }

    /// Translate a Wasm `f64.convert_u64` instruction.
    pub fn translate_f64_convert_u64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F64ConvertUI64), UntypedValue::f64_convert_i64_u)
    }

    /// Translate a Wasm `f64.promote_f32` instruction.
    pub fn translate_f64_promote_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(unary_op!(F64PromoteF32), UntypedValue::f64_promote_f32)
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
        self.translate_unary_operation(unary_op!(I32Extend8S), UntypedValue::i32_extend8_s)
    }

    /// Translate a Wasm `i32.extend_16s` instruction.
    pub fn translate_i32_sign_extend16(&mut self) -> Result<(), ModuleError> {
        // Note: we do not consider sign-extend operations to be conversion
        //       routines since they do not alter the type of the operand.
        self.translate_unary_operation(unary_op!(I32Extend16S), UntypedValue::i32_extend16_s)
    }

    /// Translate a Wasm `i64.extend_8s` instruction.
    pub fn translate_i64_sign_extend8(&mut self) -> Result<(), ModuleError> {
        // Note: we do not consider sign-extend operations to be conversion
        //       routines since they do not alter the type of the operand.
        self.translate_unary_operation(unary_op!(I64Extend8S), UntypedValue::i64_extend8_s)
    }

    /// Translate a Wasm `i64.extend_16s` instruction.
    pub fn translate_i64_sign_extend16(&mut self) -> Result<(), ModuleError> {
        // Note: we do not consider sign-extend operations to be conversion
        //       routines since they do not alter the type of the operand.
        self.translate_unary_operation(unary_op!(I64Extend16S), UntypedValue::i64_extend16_s)
    }

    /// Translate a Wasm `i64.extend_32s` instruction.
    pub fn translate_i64_sign_extend32(&mut self) -> Result<(), ModuleError> {
        // Note: we do not consider sign-extend operations to be conversion
        //       routines since they do not alter the type of the operand.
        self.translate_unary_operation(unary_op!(I64Extend32S), UntypedValue::i64_extend32_s)
    }

    /// Translate a Wasm `i32.truncate_sat_f32` instruction.
    pub fn translate_i32_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I32TruncSatF32S),
            UntypedValue::i32_trunc_sat_f32_s,
        )
    }

    /// Translate a Wasm `u32.truncate_sat_f32` instruction.
    pub fn translate_u32_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I32TruncSatF32U),
            UntypedValue::i32_trunc_sat_f32_u,
        )
    }

    /// Translate a Wasm `i32.truncate_sat_f64` instruction.
    pub fn translate_i32_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I32TruncSatF64S),
            UntypedValue::i32_trunc_sat_f64_s,
        )
    }

    /// Translate a Wasm `u32.truncate_sat_f64` instruction.
    pub fn translate_u32_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I32TruncSatF64U),
            UntypedValue::i32_trunc_sat_f64_u,
        )
    }

    /// Translate a Wasm `i64.truncate_sat_f32` instruction.
    pub fn translate_i64_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I64TruncSatF32S),
            UntypedValue::i64_trunc_sat_f32_s,
        )
    }

    /// Translate a Wasm `u64.truncate_sat_f32` instruction.
    pub fn translate_u64_truncate_saturate_f32(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I64TruncSatF32U),
            UntypedValue::i64_trunc_sat_f32_u,
        )
    }

    /// Translate a Wasm `i64.truncate_sat_f64` instruction.
    pub fn translate_i64_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I64TruncSatF64S),
            UntypedValue::i64_trunc_sat_f64_s,
        )
    }

    /// Translate a Wasm `u64.truncate_sat_f64` instruction.
    pub fn translate_u64_truncate_saturate_f64(&mut self) -> Result<(), ModuleError> {
        self.translate_conversion(
            unary_op!(I64TruncSatF64U),
            UntypedValue::i64_trunc_sat_f64_u,
        )
    }
}
