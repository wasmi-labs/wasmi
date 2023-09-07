//! Function translation for the register-machine bytecode based `wasmi` engine.

mod control_frame;
mod control_stack;
mod instr_encoder;
mod result_mut;
mod stack;
mod typed_value;
mod utils;
mod visit;
mod visit_register;

use self::{
    control_frame::{
        BlockControlFrame,
        BlockHeight,
        IfControlFrame,
        LoopControlFrame,
        UnreachableControlFrame,
    },
    control_stack::AcquiredTarget,
    stack::ValueStack,
    typed_value::TypedValue,
    utils::{WasmFloat, WasmInteger},
};
pub use self::{
    control_frame::{ControlFrame, ControlFrameKind},
    control_stack::ControlStack,
    instr_encoder::InstrEncoder,
    stack::{FuncLocalConstsIter, ProviderStack, RegisterAlloc, TypedProvider},
};
use crate::{
    engine::{
        bytecode::SignatureIdx,
        config::FuelCosts,
        func_builder::TranslationErrorInner,
        regmach::bytecode::{
            AnyConst32,
            Const16,
            Const32,
            Instruction,
            Register,
            RegisterSpan,
            RegisterSpanIter,
            Sign,
        },
        CompiledFunc,
        Instr,
        TranslationError,
    },
    module::{BlockType, FuncIdx, FuncTypeIdx, ModuleResources},
    Engine,
    FuncType,
};
use alloc::vec::Vec;
use wasmi_core::{TrapCode, UntypedValue, ValueType, F32};
use wasmparser::MemArg;

/// Reusable allocations of a [`FuncTranslator`].
#[derive(Debug, Default)]
pub struct FuncTranslatorAllocations {
    /// The emulated value stack.
    stack: ValueStack,
    /// The instruction sequence encoder.
    instr_encoder: InstrEncoder,
    /// The control stack.
    control_stack: ControlStack,
    /// Buffer to store providers when popped from the [`ValueStack`] in bulk.
    buffer: Vec<TypedProvider>,
    /// Buffer to temporarily store `br_table` target depths.
    br_table_targets: Vec<u32>,
}

impl FuncTranslatorAllocations {
    /// Resets the [`FuncTranslatorAllocations`].
    fn reset(&mut self) {
        self.stack.reset();
        self.instr_encoder.reset();
        self.control_stack.reset();
        self.buffer.clear();
        self.br_table_targets.clear();
    }
}

/// Type concerned with translating from Wasm bytecode to `wasmi` bytecode.
pub struct FuncTranslator<'parser> {
    /// The reference to the Wasm module function under construction.
    func: FuncIdx,
    /// The reference to the compiled func allocated to the [`Engine`].
    compiled_func: CompiledFunc,
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
    /// The reusable data structures of the [`FuncTranslator`].
    alloc: FuncTranslatorAllocations,
}

/// Bail out early in case the current code is unreachable.
///
/// # Note
///
/// - This should be prepended to most Wasm operator translation procedures.
/// - If we are in unreachable code most Wasm translation is skipped. Only
///   certain control flow operators such as `End` are going through the
///   translation process. In particular the `End` operator may end unreachable
///   code blocks.
macro_rules! bail_unreachable {
    ($this:ident) => {{
        if !$this.is_reachable() {
            return Ok(());
        }
    }};
}
use bail_unreachable;

impl<'parser> FuncTranslator<'parser> {
    /// Creates a new [`FuncTranslator`].
    pub fn new(
        func: FuncIdx,
        compiled_func: CompiledFunc,
        res: ModuleResources<'parser>,
        alloc: FuncTranslatorAllocations,
    ) -> Result<Self, TranslationError> {
        Self {
            func,
            compiled_func,
            res,
            reachable: true,
            alloc,
        }
        .init()
    }

    /// Initializes a newly constructed [`FuncTranslator`].
    fn init(mut self) -> Result<Self, TranslationError> {
        self.alloc.reset();
        self.init_func_body_block()?;
        self.init_func_params()?;
        Ok(self)
    }

    /// Registers the `block` control frame surrounding the entire function body.
    fn init_func_body_block(&mut self) -> Result<(), TranslationError> {
        let func_type = self.res.get_type_of_func(self.func);
        let block_type = BlockType::func_type(func_type);
        let end_label = self.alloc.instr_encoder.new_label();
        let consume_fuel = self
            .is_fuel_metering_enabled()
            .then(|| {
                self.alloc
                    .instr_encoder
                    .push_consume_fuel_instr(self.fuel_costs().base)
            })
            .transpose()?;
        // Note: we use a dummy `RegisterSpan` as placeholder.
        //
        // We can do this since the branch parameters of the function enclosing block
        // are never used due to optimizations to directly return to the caller instead.
        let branch_params = RegisterSpan::new(Register::from_i16(0));
        let block_frame = BlockControlFrame::new(
            block_type,
            end_label,
            branch_params,
            BlockHeight::default(),
            consume_fuel,
        );
        self.alloc.control_stack.push_frame(block_frame);
        Ok(())
    }

    /// Registers the function parameters in the emulated value stack.
    fn init_func_params(&mut self) -> Result<(), TranslationError> {
        for _param_type in self.func_type().params() {
            self.alloc.stack.register_locals(1)?;
        }
        Ok(())
    }

    /// Registers an `amount` of local variables.
    ///
    /// # Panics
    ///
    /// If too many local variables have been registered.
    pub fn register_locals(&mut self, amount: u32) -> Result<(), TranslationError> {
        self.alloc.stack.register_locals(amount)
    }

    /// This informs the [`FuncTranslator`] that the function header translation is finished.
    ///
    /// # Note
    ///
    /// This was introduced to properly calculate the fuel costs for all local variables
    /// and function parameters. After this function call no more locals and parameters may
    /// be added to this function translation.
    pub fn finish_translate_locals(&mut self) -> Result<(), TranslationError> {
        self.alloc.stack.finish_register_locals();
        Ok(())
    }

    /// Finishes constructing the function and returns its [`CompiledFunc`].
    pub fn finish(&mut self) -> Result<(), TranslationError> {
        self.alloc
            .instr_encoder
            .defrag_registers(&mut self.alloc.stack)?;
        self.alloc.instr_encoder.update_branch_offsets()?;
        let len_registers = self.alloc.stack.len_registers();
        let len_results = u16::try_from(self.func_type().results().len())
            .map_err(|_| TranslationError::new(TranslationErrorInner::TooManyFunctionResults))?;
        let func_consts = self.alloc.stack.func_local_consts();
        let instrs = self.alloc.instr_encoder.drain_instrs();
        self.res.engine().init_func_2(
            self.compiled_func,
            len_registers,
            len_results,
            func_consts,
            instrs,
        );
        Ok(())
    }

    /// Returns a shared reference to the underlying [`Engine`].
    fn engine(&self) -> &Engine {
        self.res.engine()
    }

    /// Consumes `self` and returns the underlying reusable [`FuncTranslatorAllocations`].
    pub fn into_allocations(self) -> FuncTranslatorAllocations {
        self.alloc
    }

    /// Returns the [`FuncType`] of the function that is currently translated.
    fn func_type(&self) -> FuncType {
        let dedup_func_type = self.res.get_type_of_func(self.func);
        self.engine()
            .resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Resolves the [`FuncType`] of the given [`FuncTypeIdx`].
    fn func_type_at(&self, func_type_index: SignatureIdx) -> FuncType {
        let func_type_index = FuncTypeIdx::from(func_type_index.to_u32()); // TODO: use the same type
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

    /// Returns `true` if the code at the current translation position is reachable.
    fn is_reachable(&self) -> bool {
        self.reachable
    }

    /// Returns `true` if fuel metering is enabled for the [`Engine`].
    ///
    /// # Note
    ///
    /// This is important for the [`FuncTranslator`] to know since it
    /// has to create [`Instruction::ConsumeFuel`] instructions on the start
    /// of basic blocks such as Wasm `block`, `if` and `loop` that account
    /// for all the instructions that are going to be executed within their
    /// respective scope.
    fn is_fuel_metering_enabled(&self) -> bool {
        self.engine().config().get_consume_fuel()
    }

    /// Returns the configured [`FuelCosts`] of the [`Engine`].
    fn fuel_costs(&self) -> &FuelCosts {
        self.engine().config().fuel_costs()
    }

    /// Creates an [`Instruction::ConsumeFuel`] with base costs.
    fn make_consume_fuel_base(&self) -> Instruction {
        Instruction::consume_fuel(self.fuel_costs().base).expect("base fuel costs must be valid")
    }

    /// Returns the most recent [`Instruction::ConsumeFuel`] in the translation process.
    ///
    /// Returns `None` if gas metering is disabled.
    fn consume_fuel_instr(&self) -> Option<Instr> {
        self.alloc.control_stack.last().consume_fuel_instr()
    }

    /// Adds fuel to the most recent [`Instruction::ConsumeFuel`] in the translation process.
    ///
    /// Does nothing if gas metering is disabled.
    fn bump_fuel_consumption(&mut self, delta: u64) -> Result<(), TranslationError> {
        if let Some(instr) = self.consume_fuel_instr() {
            self.alloc
                .instr_encoder
                .bump_fuel_consumption(instr, delta)?;
        }
        Ok(())
    }

    /// Convenience function to copy the parameters when branching to a control frame.
    fn translate_copy_branch_params(
        &mut self,
        branch_params: RegisterSpanIter,
    ) -> Result<(), TranslationError> {
        if branch_params.is_empty() {
            // If the block does not have branch parameters there is no need to copy anything.
            return Ok(());
        }
        let params = &mut self.alloc.buffer;
        self.alloc.stack.pop_n(branch_params.len(), params);
        self.alloc.instr_encoder.encode_copies(
            &mut self.alloc.stack,
            branch_params,
            &self.alloc.buffer[..],
        )?;
        Ok(())
    }

    /// Translates the `end` of a Wasm `block` control frame.
    fn translate_end_block(&mut self, frame: BlockControlFrame) -> Result<(), TranslationError> {
        if self.alloc.control_stack.is_empty() {
            bail_unreachable!(self);
            // We dropped the Wasm `block` that encloses the function itself so we can return.
            return self.translate_return();
        }
        if self.reachable && frame.is_branched_to() {
            // If the end of the `block` is reachable AND
            // there are branches to the end of the `block`
            // prior, we need to copy the results to the
            // block result registers.
            //
            // # Note
            //
            // We can skip this step if the above condition is
            // not met since the code at this point is either
            // unreachable OR there is only one source of results
            // and thus there is no need to copy the results around.
            self.translate_copy_branch_params(frame.branch_params(self.engine()))?;
        }
        // Since the `block` is now sealed we can pin its end label.
        self.alloc.instr_encoder.pin_label(frame.end_label());
        if frame.is_branched_to() {
            // Case: branches to this block exist so we cannot treat the
            //       basic block as a no-op and instead have to put its
            //       block results on top of the stack.
            self.alloc
                .stack
                .trunc(frame.block_height().into_u16() as usize);
            for result in frame.branch_params(self.engine()) {
                self.alloc.stack.push_register(result)?;
            }
        }
        // We reset reachability in case the end of the `block` was reachable.
        self.reachable = self.reachable || frame.is_branched_to();
        Ok(())
    }

    /// Translates the `end` of a Wasm `loop` control frame.
    fn translate_end_loop(&mut self, _frame: LoopControlFrame) -> Result<(), TranslationError> {
        debug_assert!(
            !self.alloc.control_stack.is_empty(),
            "control stack must not be empty since its first element is always a `block`"
        );
        // # Note
        //
        // There is no need to copy the top of the stack over
        // to the `loop` result registers because a Wasm `loop`
        // only has exactly one exit point right at their end.
        //
        // If Wasm validation succeeds we can simply take whatever
        // is on top of the provider stack at that point to continue
        // translation or in other words: we do nothing.
        Ok(())
    }

    /// Translates the `end` of a Wasm `if` control frame.
    fn translate_end_if(&mut self, frame: IfControlFrame) -> Result<(), TranslationError> {
        debug_assert!(
            !self.alloc.control_stack.is_empty(),
            "control stack must not be empty since its first element is always a `block`"
        );
        match (frame.is_then_reachable(), frame.is_else_reachable()) {
            (true, true) => self.translate_end_if_then_else(frame),
            (true, false) => self.translate_end_if_then_only(frame),
            (false, true) => self.translate_end_if_else_only(frame),
            (false, false) => unreachable!("at least one of `then` or `else` must be reachable"),
        }
    }

    /// Translates the `end` of a Wasm `if` [`ControlFrame`] were only `then` is reachable.
    ///
    /// # Example
    ///
    /// This is used for translating
    ///
    /// ```wasm
    /// (if X
    ///     (then ..)
    ///     (else ..)
    /// )
    /// ```
    ///
    /// or
    ///
    /// ```wasm
    /// (if X
    ///     (then ..)
    /// )
    /// ```
    ///
    /// where `X` is a constant value that evaluates to `true` such as `(i32.const 1)`.
    fn translate_end_if_then_only(
        &mut self,
        frame: IfControlFrame,
    ) -> Result<(), TranslationError> {
        debug_assert!(frame.is_then_reachable());
        debug_assert!(!frame.is_else_reachable());
        debug_assert!(frame.else_label().is_none());
        // Note: `if_end_of_then_reachable` returns `None` if `else` was never visited.
        let end_of_then_reachable = frame.is_end_of_then_reachable().unwrap_or(self.reachable);
        if end_of_then_reachable && frame.is_branched_to() {
            // If the end of the `if` is reachable AND
            // there are branches to the end of the `block`
            // prior, we need to copy the results to the
            // block result registers.
            //
            // # Note
            //
            // We can skip this step if the above condition is
            // not met since the code at this point is either
            // unreachable OR there is only one source of results
            // and thus there is no need to copy the results around.
            self.translate_copy_branch_params(frame.branch_params(self.engine()))?;
        }
        // Since the `if` is now sealed we can pin its `end` label.
        self.alloc.instr_encoder.pin_label(frame.end_label());
        if frame.is_branched_to() {
            // Case: branches to this block exist so we cannot treat the
            //       basic block as a no-op and instead have to put its
            //       block results on top of the stack.
            self.alloc
                .stack
                .trunc(frame.block_height().into_u16() as usize);
            for result in frame.branch_params(self.engine()) {
                self.alloc.stack.push_register(result)?;
            }
        }
        // We reset reachability in case the end of the `block` was reachable.
        self.reachable = end_of_then_reachable || frame.is_branched_to();
        Ok(())
    }

    /// Translates the `end` of a Wasm `if` [`ControlFrame`] were only `else` is reachable.
    ///
    /// # Example
    ///
    /// This is used for translating
    ///
    /// ```wasm
    /// (if X
    ///     (then ..)
    ///     (else ..)
    /// )
    /// ```
    ///
    /// or
    ///
    /// ```wasm
    /// (if X
    ///     (then ..)
    /// )
    /// ```
    ///
    /// where `X` is a constant value that evaluates to `false` such as `(i32.const 0)`.
    fn translate_end_if_else_only(
        &mut self,
        frame: IfControlFrame,
    ) -> Result<(), TranslationError> {
        debug_assert!(!frame.is_then_reachable());
        debug_assert!(frame.is_else_reachable());
        debug_assert!(frame.else_label().is_none());
        // Note: `if_end_of_then_reachable` returns `None` if `else` was never visited.
        let end_of_else_reachable = self.reachable || !frame.has_visited_else();
        if end_of_else_reachable && frame.is_branched_to() {
            // If the end of the `if` is reachable AND
            // there are branches to the end of the `block`
            // prior, we need to copy the results to the
            // block result registers.
            //
            // # Note
            //
            // We can skip this step if the above condition is
            // not met since the code at this point is either
            // unreachable OR there is only one source of results
            // and thus there is no need to copy the results around.
            self.translate_copy_branch_params(frame.branch_params(self.engine()))?;
        }
        // Since the `if` is now sealed we can pin its `end` label.
        self.alloc.instr_encoder.pin_label(frame.end_label());
        if frame.is_branched_to() {
            // Case: branches to this block exist so we cannot treat the
            //       basic block as a no-op and instead have to put its
            //       block results on top of the stack.
            self.alloc
                .stack
                .trunc(frame.block_height().into_u16() as usize);
            for result in frame.branch_params(self.engine()) {
                self.alloc.stack.push_register(result)?;
            }
        }
        // We reset reachability in case the end of the `block` was reachable.
        self.reachable = end_of_else_reachable || frame.is_branched_to();
        Ok(())
    }

    /// Translates the `end` of a Wasm `if` [`ControlFrame`] were both `then` and `else` are reachable.
    fn translate_end_if_then_else(
        &mut self,
        frame: IfControlFrame,
    ) -> Result<(), TranslationError> {
        debug_assert!(frame.is_then_reachable());
        debug_assert!(frame.is_else_reachable());
        match frame.has_visited_else() {
            true => self.translate_end_if_then_with_else(frame),
            false => self.translate_end_if_then_missing_else(frame),
        }
    }

    /// Variant of [`Self::translate_end_if_then_else`] were the `else` block exists.
    fn translate_end_if_then_with_else(
        &mut self,
        frame: IfControlFrame,
    ) -> Result<(), TranslationError> {
        debug_assert!(frame.has_visited_else());
        let end_of_then_reachable = frame
            .is_end_of_then_reachable()
            .expect("must be set since `else` was visited");
        let end_of_else_reachable = self.reachable;
        let reachable = match (end_of_then_reachable, end_of_else_reachable) {
            (false, false) => frame.is_branched_to(),
            _ => true,
        };
        self.alloc.instr_encoder.pin_label_if_unpinned(
            frame
                .else_label()
                .expect("must have `else` label since `else` is reachable"),
        );
        let if_height = frame.block_height().into_u16() as usize;
        if end_of_else_reachable {
            // Since the end of `else` is reachable we need to properly
            // write the `else` block results back to were the `if` expects
            // its results to reside upon exit.
            self.translate_copy_branch_params(frame.branch_params(self.engine()))?;
        }
        // After `else` parameters have been copied we can finally pin the `end` label.
        self.alloc.instr_encoder.pin_label(frame.end_label());
        if reachable {
            // In case the code following the `if` is reachable we need
            // to clean up and prepare the value stack.
            self.alloc.stack.trunc(if_height);
            for result in frame.branch_params(self.engine()) {
                self.alloc.stack.push_register(result)?;
            }
        }
        self.reachable = reachable;
        Ok(())
    }

    /// Variant of [`Self::translate_end_if_then_else`] were the `else` block does not exist.
    ///
    /// # Note
    ///
    /// A missing `else` block forwards the [`IfControlFrame`] inputs like an identity function.
    fn translate_end_if_then_missing_else(
        &mut self,
        frame: IfControlFrame,
    ) -> Result<(), TranslationError> {
        debug_assert!(!frame.has_visited_else());
        let end_of_then_reachable = self.reachable;
        let has_results = frame.block_type().len_results(self.engine()) >= 1;
        if end_of_then_reachable && has_results {
            // Since the `else` block is missing we need to write the results
            // from the `then` block back to were the `if` control frame expects
            // its results afterwards.
            // Furthermore we need to encode the branch to the `if` end label.
            self.translate_copy_branch_params(frame.branch_params(self.engine()))?;
            let end_offset = self
                .alloc
                .instr_encoder
                .try_resolve_label(frame.end_label())?;
            self.alloc
                .instr_encoder
                .push_instr(Instruction::branch(end_offset))?;
        }
        self.alloc.instr_encoder.pin_label_if_unpinned(
            frame
                .else_label()
                .expect("must have `else` label since `else` is reachable"),
        );
        let if_height = frame.block_height().into_u16() as usize;
        if has_results {
            // We haven't visited the `else` block and thus the `else`
            // providers are still on the auxiliary stack and need to
            // be popped. We use them to restore the stack to the state
            // when entering the `if` block so that we can properly copy
            // the `else` results to were they are expected.
            self.alloc.stack.trunc(if_height);
            for provider in self.alloc.control_stack.pop_else_providers() {
                self.alloc.stack.push_provider(provider)?;
            }
            self.translate_copy_branch_params(frame.branch_params(self.engine()))?;
        }
        // After `else` parameters have been copied we can finally pin the `end` label.
        self.alloc.instr_encoder.pin_label(frame.end_label());
        // Without `else` block the code after the `if` is always reachable and
        // thus we need to clean up and prepare the value stack for the following code.
        self.alloc.stack.trunc(if_height);
        for result in frame.branch_params(self.engine()) {
            self.alloc.stack.push_register(result)?;
        }
        self.reachable = true;
        Ok(())
    }

    /// Translates the `end` of an unreachable control frame.
    fn translate_end_unreachable(
        &mut self,
        _frame: UnreachableControlFrame,
    ) -> Result<(), TranslationError> {
        Ok(())
    }

    /// Allocate control flow block branch parameters.
    ///
    /// # Note
    ///
    /// The naive description of this algorithm is as follows:
    ///
    /// 1. Pop off all block parameters of the control flow block from
    ///    the stack and store them temporarily in the `buffer`.
    /// 2. For each branch parameter dynamically allocate a register.
    ///    - Note: All dynamically allocated registers must be contiguous.
    ///    - These registers serve as the registers and to hold the branch
    ///      parameters upon branching to the control flow block and are
    ///      going to be returned via [`RegisterSpan`].
    /// 3. Drop all dynamically allocated branch parameter registers again.
    /// 4. Push the block parameters stored in the `buffer` back onto the stack.
    /// 5. Return the result registers of step 2.
    ///
    /// The `buffer` will be empty after this operation.
    ///
    /// # Dev. Note
    ///
    /// The current implementation is naive and rather inefficient
    /// for the purpose of simplicity and correctness and should be
    /// optimized if it turns out to be a bottleneck.
    ///
    /// # Errors
    ///
    /// If this procedure would allocate more registers than are available.
    fn alloc_branch_params(
        &mut self,
        len_block_params: usize,
        len_branch_params: usize,
    ) -> Result<RegisterSpan, TranslationError> {
        let params = &mut self.alloc.buffer;
        // Pop the block parameters off the stack.
        self.alloc.stack.pop_n(len_block_params, params);
        // Peek the branch parameter registers which are going to be returned.
        let branch_params = self.alloc.stack.peek_dynamic_n(len_branch_params)?;
        // Push the block parameters onto the stack again as if nothing happened.
        self.alloc.stack.push_n(params)?;
        params.clear();
        Ok(branch_params)
    }

    /// Pushes a binary instruction with two register inputs `lhs` and `rhs`.
    fn push_binary_instr(
        &mut self,
        lhs: Register,
        rhs: Register,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
    ) -> Result<(), TranslationError> {
        let result = self.alloc.stack.push_dynamic()?;
        self.alloc
            .instr_encoder
            .push_instr(make_instr(result, lhs, rhs))?;
        Ok(())
    }

    /// Pushes a binary instruction if the immediate operand can be encoded in 16 bits.
    ///
    /// # Note
    ///
    /// - Returns `Ok(true)` is the optmization was applied.
    /// - Returns `Ok(false)` is the optimization could not be applied.
    /// - Returns `Err(_)` if a translation error occured.
    fn try_push_binary_instr_imm16<T>(
        &mut self,
        lhs: Register,
        rhs: T,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16<T>) -> Instruction,
    ) -> Result<bool, TranslationError>
    where
        T: Copy + TryInto<Const16<T>>,
    {
        if let Ok(rhs) = rhs.try_into() {
            // Optimization: We can use a compact instruction for small constants.
            let result = self.alloc.stack.push_dynamic()?;
            self.alloc
                .instr_encoder
                .push_instr(make_instr_imm16(result, lhs, rhs))?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Variant of [`Self::try_push_binary_instr_imm16`] with swapped operands for `make_instr_imm16`.
    fn try_push_binary_instr_imm16_rev<T>(
        &mut self,
        lhs: T,
        rhs: Register,
        make_instr_imm16: fn(result: Register, lhs: Const16<T>, rhs: Register) -> Instruction,
    ) -> Result<bool, TranslationError>
    where
        T: Copy + TryInto<Const16<T>>,
    {
        if let Ok(lhs) = lhs.try_into() {
            // Optimization: We can use a compact instruction for small constants.
            let result = self.alloc.stack.push_dynamic()?;
            self.alloc
                .instr_encoder
                .push_instr(make_instr_imm16(result, lhs, rhs))?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Evaluates the constants and pushes the proper result to the value stack.
    fn push_binary_consteval(
        &mut self,
        lhs: TypedValue,
        rhs: TypedValue,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
    ) -> Result<(), TranslationError> {
        self.alloc.stack.push_const(consteval(lhs, rhs));
        Ok(())
    }

    /// Pushes a binary instruction with a generic immediate value.
    ///
    /// # Note
    ///
    /// The resulting binary instruction always takes up two instruction
    /// words for its encoding in the [`Instruction`] sequence.
    fn push_binary_instr_imm<T>(
        &mut self,
        lhs: Register,
        rhs: T,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
    ) -> Result<(), TranslationError>
    where
        T: Into<UntypedValue>,
    {
        let result = self.alloc.stack.push_dynamic()?;
        let rhs = self.alloc.stack.alloc_const(rhs)?;
        self.alloc
            .instr_encoder
            .push_instr(make_instr(result, lhs, rhs))?;
        Ok(())
    }

    /// Pushes a binary instruction with a generic immediate value.
    ///
    /// # Note
    ///
    /// The resulting binary instruction always takes up two instruction
    /// words for its encoding in the [`Instruction`] sequence.
    fn push_binary_instr_imm_rev<T>(
        &mut self,
        lhs: T,
        rhs: Register,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
    ) -> Result<(), TranslationError>
    where
        T: Into<UntypedValue>,
    {
        let result = self.alloc.stack.push_dynamic()?;
        let lhs = self.alloc.stack.alloc_const(lhs)?;
        self.alloc
            .instr_encoder
            .push_instr(make_instr(result, lhs, rhs))?;
        Ok(())
    }

    /// Translates a [`TrapCode`] as [`Instruction`].
    fn translate_trap(&mut self, trap_code: TrapCode) -> Result<(), TranslationError> {
        bail_unreachable!(self);
        self.alloc
            .instr_encoder
            .push_instr(Instruction::Trap(trap_code))?;
        self.reachable = false;
        Ok(())
    }

    /// Translate a non-commutative binary `wasmi` integer instruction.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for all non-commutative
    ///   binary instructions such as `i32.sub` or `i64.rotl`.
    /// - Its various function arguments allow it to be used generically for `i32` and `i64` types.
    /// - Applies constant evaluation if both operands are constant values.
    ///
    /// - The `make_instr_opt` closure allows to implement custom optimization
    ///   logic for the case that both operands are registers.
    /// - The `make_instr_reg_imm_opt` closure allows to implement custom optmization
    ///   logic for the case that the right-hand side operand is a constant value.
    /// - The `make_instr_imm_reg_opt` closure allows to implement custom optmization
    ///   logic for the case that the left-hand side operand is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to `wasmi` bytecode:
    ///
    /// - `{i32, i64}.{sub, lt_s, lt_u, le_s, le_u, gt_s, gt_u, ge_s, ge_u}`
    #[allow(clippy::too_many_arguments)]
    fn translate_binary<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16<T>) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16<T>, rhs: Register) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
        make_instr_reg_imm_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: T,
        ) -> Result<bool, TranslationError>,
        make_instr_imm_reg_opt: fn(
            &mut Self,
            lhs: T,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError>
    where
        T: Copy + From<TypedValue> + Into<TypedValue> + TryInto<Const16<T>>,
    {
        bail_unreachable!(self);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                if make_instr_opt(self, lhs, rhs)? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (TypedProvider::Register(lhs), TypedProvider::Const(rhs)) => {
                if make_instr_reg_imm_opt(self, lhs, T::from(rhs))? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                if self.try_push_binary_instr_imm16(lhs, T::from(rhs), make_instr_imm16)? {
                    // Optimization was applied: return early.
                    return Ok(());
                }
                self.push_binary_instr_imm(lhs, rhs, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Register(rhs)) => {
                if make_instr_imm_reg_opt(self, T::from(lhs), rhs)? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                if self.try_push_binary_instr_imm16_rev(T::from(lhs), rhs, make_instr_imm16_rev)? {
                    // Optimization was applied: return early.
                    return Ok(());
                }
                self.push_binary_instr_imm_rev(lhs, rhs, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => {
                self.push_binary_consteval(lhs, rhs, consteval)
            }
        }
    }

    /// Translate a non-commutative binary `wasmi` float instruction.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for all
    ///   non-commutative binary instructions.
    /// - Its various function arguments allow it to be used generically for `f32` and `f64` types.
    /// - Applies constant evaluation if both operands are constant values.
    ///
    /// - The `make_instr_opt` closure allows to implement custom optimization
    ///   logic for the case that both operands are registers.
    /// - The `make_instr_reg_imm_opt` closure allows to implement custom optmization
    ///   logic for the case that the right-hand side operand is a constant value.
    /// - The `make_instr_imm_reg_opt` closure allows to implement custom optmization
    ///   logic for the case that the left-hand side operand is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to `wasmi` bytecode:
    ///
    /// - `{f32, f64}.{sub, div}`
    #[allow(clippy::too_many_arguments)]
    fn translate_fbinary<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
        make_instr_reg_imm_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: T,
        ) -> Result<bool, TranslationError>,
        make_instr_imm_reg_opt: fn(
            &mut Self,
            lhs: T,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError>
    where
        T: WasmFloat,
    {
        bail_unreachable!(self);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                if make_instr_opt(self, lhs, rhs)? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (TypedProvider::Register(lhs), TypedProvider::Const(rhs)) => {
                if make_instr_reg_imm_opt(self, lhs, T::from(rhs))? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                if T::from(rhs).is_nan() {
                    // Optimization: non-canonicalized NaN propagation.
                    self.alloc.stack.push_const(rhs);
                    return Ok(());
                }
                self.push_binary_instr_imm(lhs, rhs, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Register(rhs)) => {
                if make_instr_imm_reg_opt(self, T::from(lhs), rhs)? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                if T::from(lhs).is_nan() {
                    // Optimization: non-canonicalized NaN propagation.
                    self.alloc.stack.push_const(lhs);
                    return Ok(());
                }
                self.push_binary_instr_imm_rev(lhs, rhs, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => {
                self.push_binary_consteval(lhs, rhs, consteval)
            }
        }
    }

    /// Translate `wasmi` float `{f32,f64}.copysign` instructions.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for copysign instructions.
    /// - Applies constant evaluation if both operands are constant values.
    fn translate_fcopysign<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm: fn(result: Register, lhs: Register, rhs: Sign) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
    ) -> Result<(), TranslationError>
    where
        T: WasmFloat,
    {
        bail_unreachable!(self);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                if lhs == rhs {
                    // Optimization: `copysign x x` is always just `x`
                    self.alloc.stack.push_register(lhs)?;
                    return Ok(());
                }
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (TypedProvider::Register(lhs), TypedProvider::Const(rhs)) => {
                let sign = T::from(rhs).sign();
                let result = self.alloc.stack.push_dynamic()?;
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr_imm(result, lhs, sign))?;
                Ok(())
            }
            (TypedProvider::Const(lhs), TypedProvider::Register(rhs)) => {
                self.push_binary_instr_imm_rev(lhs, rhs, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => {
                self.push_binary_consteval(lhs, rhs, consteval)
            }
        }
    }

    /// Translate a commutative binary `wasmi` integer instruction.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for all commutative
    ///   binary instructions such as `i32.add` or `i64.mul`.
    /// - Its various function arguments allow it to be used for `i32` and `i64` types.
    /// - Applies constant evaluation if both operands are constant values.
    ///
    /// - The `make_instr_opt` closure allows to implement custom optimization
    ///   logic for the case that both operands are registers.
    /// - The `make_instr_imm_opt` closure allows to implement custom optmization
    ///   logic for the case that one of the operands is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to `wasmi` bytecode:
    ///
    /// - `{i32, i64}.{eq, ne, add, mul, and, or, xor}`
    #[allow(clippy::too_many_arguments)]
    fn translate_binary_commutative<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16<T>) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
        make_instr_imm_opt: fn(&mut Self, lhs: Register, rhs: T) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError>
    where
        T: Copy + From<TypedValue> + TryInto<Const16<T>>,
    {
        bail_unreachable!(self);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                if make_instr_opt(self, lhs, rhs)? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (TypedProvider::Register(reg_in), TypedProvider::Const(imm_in))
            | (TypedProvider::Const(imm_in), TypedProvider::Register(reg_in)) => {
                if make_instr_imm_opt(self, reg_in, T::from(imm_in))? {
                    // Custom logic applied its optimization: return early.
                    return Ok(());
                }
                if self.try_push_binary_instr_imm16(reg_in, T::from(imm_in), make_instr_imm16)? {
                    // Optimization was applied: return early.
                    return Ok(());
                }
                self.push_binary_instr_imm(reg_in, imm_in, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => {
                self.push_binary_consteval(lhs, rhs, consteval)
            }
        }
    }

    /// Translate a commutative binary `wasmi` float instruction.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for all commutative
    ///   binary instructions such as `f32.add` or `f64.mul`.
    /// - Its various function arguments allow it to be used for `f32` and `f64` types.
    /// - Applies constant evaluation if both operands are constant values.
    ///
    /// - The `make_instr_opt` closure allows to implement custom optimization
    ///   logic for the case that both operands are registers.
    /// - The `make_instr_imm_opt` closure allows to implement custom optmization
    ///   logic for the case that one of the operands is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to `wasmi` bytecode:
    ///
    /// - `{f32, f64}.{add, mul, min, max}`
    #[allow(clippy::too_many_arguments)]
    fn translate_fbinary_commutative<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
        make_instr_imm_opt: fn(&mut Self, lhs: Register, rhs: T) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError>
    where
        T: WasmFloat,
    {
        bail_unreachable!(self);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                if make_instr_opt(self, lhs, rhs)? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (TypedProvider::Register(reg_in), TypedProvider::Const(imm_in))
            | (TypedProvider::Const(imm_in), TypedProvider::Register(reg_in)) => {
                if make_instr_imm_opt(self, reg_in, T::from(imm_in))? {
                    // Custom logic applied its optimization: return early.
                    return Ok(());
                }
                if T::from(imm_in).is_nan() {
                    // Optimization: non-canonicalized NaN propagation.
                    self.alloc.stack.push_const(T::from(imm_in));
                    return Ok(());
                }
                self.push_binary_instr_imm(reg_in, imm_in, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => {
                self.push_binary_consteval(lhs, rhs, consteval)
            }
        }
    }

    /// Translate a shift or rotate `wasmi` instruction.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for all shift or rotate instructions.
    /// - Its various function arguments allow it to be used for generic Wasm types.
    /// - Applies constant evaluation if both operands are constant values.
    ///
    /// - The `make_instr_imm_reg_opt` closure allows to implement custom optmization
    ///   logic for the case the shifted value operand is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to `wasmi` bytecode:
    ///
    /// - `{i32, i64}.{shl, shr_s, shr_u, rotl, rotr}`
    #[allow(clippy::too_many_arguments)]
    fn translate_shift<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm: fn(result: Register, lhs: Register, rhs: Const16<T>) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16<T>, rhs: Register) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_imm_reg_opt: fn(
            &mut Self,
            lhs: T,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError>
    where
        T: WasmInteger,
        Const16<T>: From<i16>,
    {
        bail_unreachable!(self);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (TypedProvider::Register(lhs), TypedProvider::Const(rhs)) => {
                let rhs = T::from(rhs).as_shift_amount();
                if rhs == 0 {
                    // Optimization: Shifting or rotating by zero bits is a no-op.
                    self.alloc.stack.push_register(lhs)?;
                    return Ok(());
                }
                let result = self.alloc.stack.push_dynamic()?;
                self.alloc.instr_encoder.push_instr(make_instr_imm(
                    result,
                    lhs,
                    <Const16<T>>::from(rhs),
                ))?;
                Ok(())
            }
            (TypedProvider::Const(lhs), TypedProvider::Register(rhs)) => {
                if make_instr_imm_reg_opt(self, T::from(lhs), rhs)? {
                    // Custom optimization was applied: return early
                    return Ok(());
                }
                if T::from(lhs).eq_zero() {
                    // Optimization: Shifting or rotating a zero value is a no-op.
                    self.alloc.stack.push_const(lhs);
                    return Ok(());
                }
                if self.try_push_binary_instr_imm16_rev(T::from(lhs), rhs, make_instr_imm16_rev)? {
                    // Optimization was applied: return early.
                    return Ok(());
                }
                self.push_binary_instr_imm_rev(lhs, rhs, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => {
                self.push_binary_consteval(lhs, rhs, consteval)
            }
        }
    }

    /// Translate an integer division or remainder `wasmi` instruction.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for all `div` or `rem` instructions.
    /// - Its various function arguments allow it to be used for `i32` and `i64` types.
    /// - Applies constant evaluation if both operands are constant values.
    ///
    /// - The `make_instr_reg_imm_opt` closure allows to implement custom optmization
    ///   logic for the case the right-hand side operand is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to `wasmi` bytecode:
    ///
    /// - `{i32, i64}.{div_u, div_s, rem_u, rem_s}`
    #[allow(clippy::too_many_arguments)]
    pub fn translate_divrem<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16<T>) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16<T>, rhs: Register) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> Result<TypedValue, TrapCode>,
        make_instr_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
        make_instr_reg_imm_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: T,
        ) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError>
    where
        T: WasmInteger,
    {
        bail_unreachable!(self);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                if make_instr_opt(self, lhs, rhs)? {
                    // Custom optimization was applied: return early
                    return Ok(());
                }
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (TypedProvider::Register(lhs), TypedProvider::Const(rhs)) => {
                if T::from(rhs).eq_zero() {
                    // Optimization: division by zero always traps
                    self.translate_trap(TrapCode::IntegerDivisionByZero)?;
                    return Ok(());
                }
                if make_instr_reg_imm_opt(self, lhs, T::from(rhs))? {
                    // Custom optimization was applied: return early
                    return Ok(());
                }
                if self.try_push_binary_instr_imm16(lhs, T::from(rhs), make_instr_imm16)? {
                    // Optimization was applied: return early.
                    return Ok(());
                }
                self.push_binary_instr_imm(lhs, rhs, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Register(rhs)) => {
                if self.try_push_binary_instr_imm16_rev(T::from(lhs), rhs, make_instr_imm16_rev)? {
                    // Optimization was applied: return early.
                    return Ok(());
                }
                self.push_binary_instr_imm_rev(lhs, rhs, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => match consteval(lhs, rhs) {
                Ok(result) => {
                    self.alloc.stack.push_const(result);
                    Ok(())
                }
                Err(trap_code) => self.translate_trap(trap_code),
            },
        }
    }

    /// Can be used for [`Self::translate_binary`] (and variants) if no custom optimization shall be applied.
    pub fn no_custom_opt<Lhs, Rhs>(
        &mut self,
        _lhs: Lhs,
        _rhs: Rhs,
    ) -> Result<bool, TranslationError> {
        Ok(false)
    }

    /// Translates a unary Wasm instruction to `wasmi` bytecode.
    pub fn translate_unary(
        &mut self,
        make_instr: fn(result: Register, input: Register) -> Instruction,
        consteval: fn(input: TypedValue) -> TypedValue,
    ) -> Result<(), TranslationError> {
        bail_unreachable!(self);
        match self.alloc.stack.pop() {
            TypedProvider::Register(input) => {
                let result = self.alloc.stack.push_dynamic()?;
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr(result, input))?;
                Ok(())
            }
            TypedProvider::Const(input) => {
                self.alloc.stack.push_const(consteval(input));
                Ok(())
            }
        }
    }

    /// Translates a fallible unary Wasm instruction to `wasmi` bytecode.
    pub fn translate_unary_fallible(
        &mut self,
        make_instr: fn(result: Register, input: Register) -> Instruction,
        consteval: fn(input: TypedValue) -> Result<TypedValue, TrapCode>,
    ) -> Result<(), TranslationError> {
        bail_unreachable!(self);
        match self.alloc.stack.pop() {
            TypedProvider::Register(input) => {
                let result = self.alloc.stack.push_dynamic()?;
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr(result, input))?;
                Ok(())
            }
            TypedProvider::Const(input) => match consteval(input) {
                Ok(result) => {
                    self.alloc.stack.push_const(result);
                    Ok(())
                }
                Err(trap_code) => self.translate_trap(trap_code),
            },
        }
    }

    /// Returns the 32-bit [`MemArg`] offset.
    ///
    /// # Panics
    ///
    /// If the [`MemArg`] offset is not 32-bit.
    fn memarg_offset(memarg: MemArg) -> u32 {
        u32::try_from(memarg.offset).unwrap_or_else(|_| {
            panic!(
                "encountered 64-bit memory load/store offset: {}",
                memarg.offset
            )
        })
    }

    /// Calculates the effective address `ptr+offset` and calls `f(address)` if valid.
    ///
    /// Encodes a [`TrapCode::MemoryOutOfBounds`] trap instruction if the effective address is invalid.
    fn effective_address_and(
        &mut self,
        ptr: TypedValue,
        offset: u32,
        f: impl FnOnce(&mut Self, u32) -> Result<(), TranslationError>,
    ) -> Result<(), TranslationError> {
        match u32::from(ptr).checked_add(offset) {
            Some(address) => f(self, address),
            None => self.translate_trap(TrapCode::MemoryOutOfBounds),
        }
    }

    /// Translates a Wasm `load` instruction to `wasmi` bytecode.
    ///
    /// # Note
    ///
    /// This chooses the right encoding for the given `load` instruction.
    /// If `ptr+offset` is a constant value the address is pre-calculated.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to `wasmi` bytecode:
    ///
    /// - `{i32, i64, f32, f64}.load`
    /// - `i32.{load8_s, load8_u, load16_s, load16_u}`
    /// - `i64.{load8_s, load8_u, load16_s, load16_u load32_s, load32_u}`
    pub fn translate_load(
        &mut self,
        memarg: MemArg,
        make_instr: fn(result: Register, ptr: Register) -> Instruction,
        make_instr_offset16: fn(
            result: Register,
            ptr: Register,
            offset: Const16<u32>,
        ) -> Instruction,
        make_instr_at: fn(result: Register, address: Const32<u32>) -> Instruction,
    ) -> Result<(), TranslationError> {
        bail_unreachable!(self);
        let offset = Self::memarg_offset(memarg);
        match self.alloc.stack.pop() {
            TypedProvider::Register(ptr) => {
                if let Some(offset) = <Const16<u32>>::from_u32(offset) {
                    let result = self.alloc.stack.push_dynamic()?;
                    self.alloc
                        .instr_encoder
                        .push_instr(make_instr_offset16(result, ptr, offset))?;
                    return Ok(());
                }
                let result = self.alloc.stack.push_dynamic()?;
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr(result, ptr))?;
                self.alloc
                    .instr_encoder
                    .append_instr(Instruction::const32(offset))?;
                Ok(())
            }
            TypedProvider::Const(ptr) => {
                self.effective_address_and(ptr, offset, |this, address| {
                    let result = this.alloc.stack.push_dynamic()?;
                    this.alloc
                        .instr_encoder
                        .push_instr(make_instr_at(result, Const32::from(address)))?;
                    Ok(())
                })
            }
        }
    }

    /// Translates Wasm integer `store` and `storeN` instructions to `wasmi` bytecode.
    ///
    /// # Note
    ///
    /// This chooses the most efficient encoding for the given `store` instruction.
    /// If `ptr+offset` is a constant value the pointer address is pre-calculated.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to `wasmi` bytecode:
    ///
    /// - `{i32, i64}.{store, store8, store16, store32}`
    fn translate_istore<T, U>(
        &mut self,
        memarg: MemArg,
        make_instr: fn(ptr: Register, offset: Const32<u32>) -> Instruction,
        make_instr_offset16: fn(ptr: Register, offset: u16, value: Register) -> Instruction,
        make_instr_offset16_imm: fn(ptr: Register, offset: u16, value: U) -> Instruction,
        make_instr_at: fn(address: Const32<u32>, value: Register) -> Instruction,
        make_instr_at_imm: fn(address: Const32<u32>, value: U) -> Instruction,
    ) -> Result<(), TranslationError>
    where
        T: Copy + From<TypedValue>,
        U: TryFrom<T>,
    {
        bail_unreachable!(self);
        let offset = Self::memarg_offset(memarg);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(ptr), TypedProvider::Register(value)) => {
                if let Ok(offset) = u16::try_from(offset) {
                    self.alloc
                        .instr_encoder
                        .push_instr(make_instr_offset16(ptr, offset, value))?;
                    Ok(())
                } else {
                    self.alloc
                        .instr_encoder
                        .push_instr(make_instr(ptr, Const32::from(offset)))?;
                    self.alloc
                        .instr_encoder
                        .append_instr(Instruction::Register(value))?;
                    Ok(())
                }
            }
            (TypedProvider::Register(ptr), TypedProvider::Const(value)) => {
                let offset16 = u16::try_from(offset);
                let value16 = U::try_from(T::from(value));
                match (offset16, value16) {
                    (Ok(offset), Ok(value)) => {
                        self.alloc
                            .instr_encoder
                            .push_instr(make_instr_offset16_imm(ptr, offset, value))?;
                        Ok(())
                    }
                    (Ok(offset), Err(_)) => {
                        let value = self.alloc.stack.alloc_const(value)?;
                        self.alloc
                            .instr_encoder
                            .push_instr(make_instr_offset16(ptr, offset, value))?;
                        Ok(())
                    }
                    (Err(_), _) => {
                        self.alloc
                            .instr_encoder
                            .push_instr(make_instr(ptr, Const32::from(offset)))?;
                        self.alloc
                            .instr_encoder
                            .append_instr(Instruction::Register(
                                self.alloc.stack.alloc_const(value)?,
                            ))?;
                        Ok(())
                    }
                }
            }
            (TypedProvider::Const(ptr), TypedProvider::Register(value)) => self
                .effective_address_and(ptr, offset, |this, address| {
                    this.alloc
                        .instr_encoder
                        .push_instr(make_instr_at(Const32::from(address), value))?;
                    Ok(())
                }),
            (TypedProvider::Const(ptr), TypedProvider::Const(value)) => {
                self.effective_address_and(ptr, offset, |this, address| {
                    if let Ok(value) = U::try_from(T::from(value)) {
                        this.alloc
                            .instr_encoder
                            .push_instr(make_instr_at_imm(Const32::from(address), value))?;
                        Ok(())
                    } else {
                        let value = this.alloc.stack.alloc_const(value)?;
                        this.alloc
                            .instr_encoder
                            .push_instr(make_instr_at(Const32::from(address), value))?;
                        Ok(())
                    }
                })
            }
        }
    }

    /// Translates Wasm float `store` instructions to `wasmi` bytecode.
    ///
    /// # Note
    ///
    /// This chooses the most efficient encoding for the given `store` instruction.
    /// If `ptr+offset` is a constant value the pointer address is pre-calculated.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to `wasmi` bytecode:
    ///
    /// - `{f32, f64}.store`
    fn translate_fstore(
        &mut self,
        memarg: MemArg,
        make_instr: fn(ptr: Register, offset: Const32<u32>) -> Instruction,
        make_instr_offset16: fn(ptr: Register, offset: u16, value: Register) -> Instruction,
        make_instr_at: fn(address: Const32<u32>, value: Register) -> Instruction,
    ) -> Result<(), TranslationError> {
        bail_unreachable!(self);
        let offset = Self::memarg_offset(memarg);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(ptr), TypedProvider::Register(value)) => {
                if let Ok(offset) = u16::try_from(offset) {
                    self.alloc
                        .instr_encoder
                        .push_instr(make_instr_offset16(ptr, offset, value))?;
                    Ok(())
                } else {
                    self.alloc
                        .instr_encoder
                        .push_instr(make_instr(ptr, Const32::from(offset)))?;
                    self.alloc
                        .instr_encoder
                        .append_instr(Instruction::Register(value))?;
                    Ok(())
                }
            }
            (TypedProvider::Register(ptr), TypedProvider::Const(value)) => {
                let offset16 = u16::try_from(offset);
                match offset16 {
                    Ok(offset) => {
                        let value = self.alloc.stack.alloc_const(value)?;
                        self.alloc
                            .instr_encoder
                            .push_instr(make_instr_offset16(ptr, offset, value))?;
                        Ok(())
                    }
                    Err(_) => {
                        self.alloc
                            .instr_encoder
                            .push_instr(make_instr(ptr, Const32::from(offset)))?;
                        self.alloc
                            .instr_encoder
                            .append_instr(Instruction::Register(
                                self.alloc.stack.alloc_const(value)?,
                            ))?;
                        Ok(())
                    }
                }
            }
            (TypedProvider::Const(ptr), TypedProvider::Register(value)) => self
                .effective_address_and(ptr, offset, |this, address| {
                    this.alloc
                        .instr_encoder
                        .push_instr(make_instr_at(Const32::from(address), value))?;
                    Ok(())
                }),
            (TypedProvider::Const(ptr), TypedProvider::Const(value)) => {
                self.effective_address_and(ptr, offset, |this, address| {
                    let value = this.alloc.stack.alloc_const(value)?;
                    this.alloc
                        .instr_encoder
                        .push_instr(make_instr_at(Const32::from(address), value))?;
                    Ok(())
                })
            }
        }
    }

    /// Translates a Wasm `select` or `select <ty>` instruction.
    ///
    /// # Note
    ///
    /// - This applies constant propagation in case `condition` is a constant value.
    /// - If both `lhs` and `rhs` are equal registers or constant values `lhs` is forwarded.
    /// - Properly chooses the correct `select` instruction encoding and optimizes for
    ///   cases with 32-bit constant values.
    fn translate_select(&mut self, type_hint: Option<ValueType>) -> Result<(), TranslationError> {
        /// Convenience function to encode a `select` instruction.
        ///
        /// # Note
        ///
        /// Helper for `select` instructions where one of `lhs` and `rhs`
        /// is a [`Register`] and the other a funtion local constant value.
        fn encode_select_imm(
            this: &mut FuncTranslator<'_>,
            result: Register,
            condition: Register,
            reg_in: Register,
            imm_in: impl Into<UntypedValue>,
            make_instr: fn(
                result: Register,
                condition: Register,
                lhs_or_rhs: Register,
            ) -> Instruction,
        ) -> Result<(), TranslationError> {
            this.alloc
                .instr_encoder
                .push_instr(make_instr(result, condition, reg_in))?;
            let rhs = this.alloc.stack.alloc_const(imm_in)?;
            this.alloc
                .instr_encoder
                .append_instr(Instruction::Register(rhs))?;
            Ok(())
        }

        /// Convenience function to encode a `select` instruction.
        ///
        /// # Note
        ///
        /// Helper for `select` instructions where one of `lhs` and `rhs`
        /// is a [`Register`] and the other a 32-bit constant value.
        fn encode_select_imm32(
            this: &mut FuncTranslator<'_>,
            result: Register,
            condition: Register,
            reg_in: Register,
            imm_in: impl Into<AnyConst32>,
            make_instr: fn(
                result: Register,
                condition: Register,
                lhs_or_rhs: Register,
            ) -> Instruction,
        ) -> Result<(), TranslationError> {
            this.alloc
                .instr_encoder
                .push_instr(make_instr(result, condition, reg_in))?;
            this.alloc
                .instr_encoder
                .append_instr(Instruction::const32(imm_in))?;
            Ok(())
        }

        /// Convenience function to encode a `select` instruction.
        ///
        /// # Note
        ///
        /// Helper for `select` instructions where one of `lhs` and `rhs`
        /// is a [`Register`] and the other a 64-bit constant value.
        fn encode_select_imm64<T>(
            this: &mut FuncTranslator<'_>,
            result: Register,
            condition: Register,
            reg_in: Register,
            imm_in: T,
            make_instr: fn(
                result: Register,
                condition: Register,
                lhs_or_rhs: Register,
            ) -> Instruction,
            make_instr_param: fn(Const32<T>) -> Instruction,
        ) -> Result<(), TranslationError>
        where
            T: Copy + Into<UntypedValue>,
            Const32<T>: TryFrom<T>,
        {
            match <Const32<T>>::try_from(imm_in) {
                Ok(imm_in) => {
                    this.alloc
                        .instr_encoder
                        .push_instr(make_instr(result, condition, reg_in))?;
                    this.alloc
                        .instr_encoder
                        .append_instr(make_instr_param(imm_in))?;
                }
                Err(_) => {
                    encode_select_imm(this, result, condition, reg_in, imm_in, make_instr)?;
                }
            }
            Ok(())
        }

        bail_unreachable!(self);
        let (lhs, rhs, condition) = self.alloc.stack.pop3();
        match condition {
            TypedProvider::Const(condition) => match (bool::from(condition), lhs, rhs) {
                // # Optimization
                //
                // Since the `condition` is a constant value we can forward `lhs` or `rhs` statically.
                (true, TypedProvider::Register(reg), _)
                | (false, _, TypedProvider::Register(reg)) => {
                    self.alloc.stack.push_register(reg)?;
                    Ok(())
                }
                (true, TypedProvider::Const(value), _)
                | (false, _, TypedProvider::Const(value)) => {
                    self.alloc.stack.push_const(value);
                    Ok(())
                }
            },
            TypedProvider::Register(condition) => match (lhs, rhs) {
                (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                    if lhs == rhs {
                        // # Optimization
                        //
                        // Both `lhs` and `rhs` are equal registers
                        // and thus will always yield the same value.
                        self.alloc.stack.push_register(lhs)?;
                        return Ok(());
                    }
                    let result = self.alloc.stack.push_dynamic()?;
                    self.alloc
                        .instr_encoder
                        .push_instr(Instruction::select(result, condition, lhs))?;
                    self.alloc
                        .instr_encoder
                        .append_instr(Instruction::Register(rhs))?;
                    Ok(())
                }
                (TypedProvider::Register(lhs), TypedProvider::Const(rhs)) => {
                    if let Some(type_hint) = type_hint {
                        debug_assert_eq!(rhs.ty(), type_hint);
                    }
                    let result = self.alloc.stack.push_dynamic()?;
                    match rhs.ty() {
                        ValueType::I32 => encode_select_imm32(
                            self,
                            result,
                            condition,
                            lhs,
                            i32::from(rhs),
                            Instruction::select,
                        ),
                        ValueType::F32 => encode_select_imm32(
                            self,
                            result,
                            condition,
                            lhs,
                            f32::from(rhs),
                            Instruction::select,
                        ),
                        ValueType::I64 => encode_select_imm64(
                            self,
                            result,
                            condition,
                            lhs,
                            i64::from(rhs),
                            Instruction::select,
                            Instruction::i64const32,
                        ),
                        ValueType::F64 => encode_select_imm64(
                            self,
                            result,
                            condition,
                            lhs,
                            f64::from(rhs),
                            Instruction::select,
                            Instruction::f64const32,
                        ),
                        ValueType::FuncRef | ValueType::ExternRef => encode_select_imm(
                            self,
                            result,
                            condition,
                            lhs,
                            rhs,
                            Instruction::select,
                        ),
                    }
                }
                (TypedProvider::Const(lhs), TypedProvider::Register(rhs)) => {
                    if let Some(type_hint) = type_hint {
                        debug_assert_eq!(lhs.ty(), type_hint);
                    }
                    let result = self.alloc.stack.push_dynamic()?;
                    match lhs.ty() {
                        ValueType::I32 => encode_select_imm32(
                            self,
                            result,
                            condition,
                            rhs,
                            i32::from(lhs),
                            Instruction::select_rev,
                        ),
                        ValueType::F32 => encode_select_imm32(
                            self,
                            result,
                            condition,
                            rhs,
                            f32::from(lhs),
                            Instruction::select_rev,
                        ),
                        ValueType::I64 => encode_select_imm64(
                            self,
                            result,
                            condition,
                            rhs,
                            i64::from(lhs),
                            Instruction::select_rev,
                            Instruction::i64const32,
                        ),
                        ValueType::F64 => encode_select_imm64(
                            self,
                            result,
                            condition,
                            rhs,
                            f64::from(lhs),
                            Instruction::select_rev,
                            Instruction::f64const32,
                        ),
                        ValueType::FuncRef | ValueType::ExternRef => encode_select_imm(
                            self,
                            result,
                            condition,
                            rhs,
                            lhs,
                            Instruction::select_rev,
                        ),
                    }
                }
                (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => {
                    /// Convenience function to encode a `select` instruction.
                    ///
                    /// # Note
                    ///
                    /// Helper for `select` instructions where both
                    /// `lhs` and `rhs` are 32-bit constant values.
                    fn encode_select_imm32<T: Into<AnyConst32>>(
                        this: &mut FuncTranslator<'_>,
                        result: Register,
                        condition: Register,
                        lhs: T,
                        rhs: T,
                    ) -> Result<(), TranslationError> {
                        this.alloc
                            .instr_encoder
                            .push_instr(Instruction::select_imm32(result, lhs))?;
                        this.alloc
                            .instr_encoder
                            .append_instr(Instruction::select_imm32(condition, rhs))?;
                        Ok(())
                    }

                    /// Convenience function to encode a `select` instruction.
                    ///
                    /// # Note
                    ///
                    /// Helper for `select` instructions where both
                    /// `lhs` and `rhs` are 64-bit constant values.
                    fn encode_select_imm64<T>(
                        this: &mut FuncTranslator<'_>,
                        result: Register,
                        condition: Register,
                        lhs: T,
                        rhs: T,
                        make_instr: fn(
                            result_or_condition: Register,
                            lhs_or_rhs: Const32<T>,
                        ) -> Instruction,
                        make_param: fn(Const32<T>) -> Instruction,
                    ) -> Result<(), TranslationError>
                    where
                        T: Copy + Into<UntypedValue>,
                        Const32<T>: TryFrom<T>,
                    {
                        let lhs32 = <Const32<T>>::try_from(lhs).ok();
                        let rhs32 = <Const32<T>>::try_from(rhs).ok();
                        match (lhs32, rhs32) {
                            (Some(lhs), Some(rhs)) => {
                                this.alloc
                                    .instr_encoder
                                    .push_instr(make_instr(result, lhs))?;
                                this.alloc
                                    .instr_encoder
                                    .append_instr(make_instr(condition, rhs))?;
                                Ok(())
                            }
                            (Some(lhs), None) => {
                                let rhs = this.alloc.stack.alloc_const(rhs)?;
                                this.alloc
                                    .instr_encoder
                                    .push_instr(Instruction::select_rev(result, condition, rhs))?;
                                this.alloc.instr_encoder.append_instr(make_param(lhs))?;
                                Ok(())
                            }
                            (None, Some(rhs)) => {
                                let lhs = this.alloc.stack.alloc_const(lhs)?;
                                this.alloc
                                    .instr_encoder
                                    .push_instr(Instruction::select(result, condition, lhs))?;
                                this.alloc.instr_encoder.append_instr(make_param(rhs))?;
                                Ok(())
                            }
                            (None, None) => encode_select_imm(this, result, condition, lhs, rhs),
                        }
                    }

                    /// Convenience function to encode a `select` instruction.
                    ///
                    /// # Note
                    ///
                    /// Helper for `select` instructions where both `lhs`
                    /// and `rhs` are function local constant values.
                    fn encode_select_imm<T>(
                        this: &mut FuncTranslator<'_>,
                        result: Register,
                        condition: Register,
                        lhs: T,
                        rhs: T,
                    ) -> Result<(), TranslationError>
                    where
                        T: Into<UntypedValue>,
                    {
                        let lhs = this.alloc.stack.alloc_const(lhs)?;
                        let rhs = this.alloc.stack.alloc_const(rhs)?;
                        this.alloc
                            .instr_encoder
                            .push_instr(Instruction::select(result, condition, lhs))?;
                        this.alloc
                            .instr_encoder
                            .append_instr(Instruction::Register(rhs))?;
                        Ok(())
                    }

                    debug_assert_eq!(lhs.ty(), rhs.ty());
                    if let Some(type_hint) = type_hint {
                        debug_assert_eq!(lhs.ty(), type_hint);
                    }
                    if lhs == rhs {
                        // # Optimization
                        //
                        // Both `lhs` and `rhs` are equal constant values
                        // and thus will always yield the same value.
                        self.alloc.stack.push_const(lhs);
                        return Ok(());
                    }
                    let result = self.alloc.stack.push_dynamic()?;
                    match lhs.ty() {
                        ValueType::I32 => {
                            encode_select_imm32(
                                self,
                                result,
                                condition,
                                i32::from(lhs),
                                i32::from(rhs),
                            )?;
                            Ok(())
                        }
                        ValueType::F32 => {
                            encode_select_imm32(
                                self,
                                result,
                                condition,
                                f32::from(lhs),
                                f32::from(rhs),
                            )?;
                            Ok(())
                        }
                        ValueType::I64 => encode_select_imm64(
                            self,
                            result,
                            condition,
                            i64::from(lhs),
                            i64::from(rhs),
                            Instruction::select_i64imm32,
                            Instruction::i64const32,
                        ),
                        ValueType::F64 => encode_select_imm64(
                            self,
                            result,
                            condition,
                            f64::from(lhs),
                            f64::from(rhs),
                            Instruction::select_f64imm32,
                            Instruction::f64const32,
                        ),
                        ValueType::FuncRef | ValueType::ExternRef => {
                            encode_select_imm(self, result, condition, lhs, rhs)
                        }
                    }
                }
            },
        }
    }

    /// Translates a Wasm `reinterpret` instruction.
    pub fn translate_reinterpret(&mut self, ty: ValueType) -> Result<(), TranslationError> {
        bail_unreachable!(self);
        match self.alloc.stack.pop() {
            TypedProvider::Register(reg) => {
                // Nothing to do in this case so we simply push the popped register back.
                self.alloc.stack.push_register(reg)?;
                Ok(())
            }
            TypedProvider::Const(value) => {
                // In case of a constant value we have to adjust for its new type and push it back.
                self.alloc.stack.push_const(value.reinterpret(ty));
                Ok(())
            }
        }
    }

    /// Translates an unconditional `return` instruction.
    pub fn translate_return(&mut self) -> Result<(), TranslationError> {
        let func_type = self.func_type();
        let results = func_type.results();
        let values = &mut self.alloc.buffer;
        self.alloc.stack.pop_n(results.len(), values);
        self.alloc
            .instr_encoder
            .encode_return(&mut self.alloc.stack, results, values)?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a conditional `br_if` that targets the function enclosing `block`.
    pub fn translate_return_if(&mut self, condition: Register) -> Result<(), TranslationError> {
        bail_unreachable!(self);
        let instr = match self.func_type().results() {
            [] => {
                // Case: Function returns nothing therefore all return statements must return nothing.
                Instruction::return_nez(condition)
            }
            [ValueType::I32] => match self.alloc.stack.peek() {
                // Case: Function returns a single `i32` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_nez_reg(condition, value),
                TypedProvider::Const(value) => {
                    Instruction::return_nez_imm32(condition, i32::from(value))
                }
            },
            [ValueType::I64] => match self.alloc.stack.peek() {
                // Case: Function returns a single `i64` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_nez_reg(condition, value),
                TypedProvider::Const(value) => {
                    if let Some(value) = <Const32<i64>>::from_i64(i64::from(value)) {
                        Instruction::return_nez_i64imm32(condition, value)
                    } else {
                        Instruction::return_nez_reg(condition, self.alloc.stack.alloc_const(value)?)
                    }
                }
            },
            [ValueType::F32] => match self.alloc.stack.peek() {
                // Case: Function returns a single `f32` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_nez_reg(condition, value),
                TypedProvider::Const(value) => {
                    Instruction::return_nez_imm32(condition, F32::from(value))
                }
            },
            [ValueType::F64] => match self.alloc.stack.peek() {
                // Case: Function returns a single `f64` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_nez_reg(condition, value),
                TypedProvider::Const(value) => {
                    if let Some(value) = <Const32<f64>>::from_f64(f64::from(value)) {
                        Instruction::return_nez_f64imm32(condition, value)
                    } else {
                        Instruction::return_nez_reg(condition, self.alloc.stack.alloc_const(value)?)
                    }
                }
            },
            [ValueType::FuncRef | ValueType::ExternRef] => {
                // Case: Function returns a single `externref` or `funcref` value which allows for special operator.
                match self.alloc.stack.peek() {
                    TypedProvider::Register(value) => Instruction::return_nez_reg(condition, value),
                    TypedProvider::Const(value) => {
                        Instruction::return_nez_reg(condition, self.alloc.stack.alloc_const(value)?)
                    }
                }
            }
            results => {
                let providers = &mut self.alloc.buffer;
                self.alloc.stack.pop_n(results.len(), providers);
                let values = self
                    .alloc
                    .instr_encoder
                    .encode_conditional_branch_params(&mut self.alloc.stack, providers)?;
                Instruction::return_nez_many(condition, values)
            }
        };
        self.alloc.instr_encoder.push_instr(instr)?;
        Ok(())
    }
}
