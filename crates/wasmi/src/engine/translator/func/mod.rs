//! Function translation for the register-machine bytecode based Wasmi engine.

mod control_frame;
mod control_stack;
mod instr_encoder;
mod provider;
mod stack;
#[macro_use]
mod utils;
mod visit;

#[cfg(feature = "simd")]
mod simd;

pub use self::{
    control_frame::ControlFrame,
    control_stack::ControlStack,
    instr_encoder::InstrEncoder,
    stack::TypedProvider,
};
use self::{
    control_frame::{
        BlockControlFrame,
        BlockHeight,
        ControlFrameBase,
        IfControlFrame,
        LoopControlFrame,
        UnreachableControlFrame,
    },
    control_stack::AcquiredTarget,
    provider::{Provider, ProviderSliceStack, UntypedProvider},
    stack::ValueStack,
    utils::FromProviders as _,
};
use crate::{
    core::{FuelCostsProvider, TrapCode, TypedVal, UntypedVal, ValType},
    engine::{
        code_map::CompiledFuncEntity,
        translator::{
            labels::{LabelRef, LabelRegistry},
            utils::{FuelInfo, Instr, WasmFloat, WasmInteger, Wrap},
            WasmTranslator,
        },
        BlockType,
    },
    ir::{
        index,
        Address,
        Address32,
        AnyConst16,
        BoundedRegSpan,
        BranchOffset,
        Const16,
        Const32,
        FixedRegSpan,
        Instruction,
        IntoShiftAmount,
        Offset16,
        Offset64,
        Offset64Lo,
        Reg,
        RegSpan,
        Sign,
    },
    module::{FuncIdx, FuncTypeIdx, MemoryIdx, ModuleHeader, TableIdx},
    Engine,
    Error,
    FuncType,
};
use alloc::vec::Vec;
use core::mem;
use stack::RegisterSpace;
use wasmparser::{MemArg, WasmFeatures};

/// Reusable allocations of a [`FuncTranslator`].
#[derive(Debug, Default)]
pub struct FuncTranslatorAllocations {
    /// The emulated value stack.
    stack: ValueStack,
    /// The instruction sequence encoder.
    instr_encoder: InstrEncoder,
    /// The control stack.
    control_stack: ControlStack,
    /// Some reusable buffers for translation purposes.
    buffer: TranslationBuffers,
}

/// Reusable allocations for utility buffers.
#[derive(Debug, Default)]
pub struct TranslationBuffers {
    /// Buffer to temporarily hold a bunch of [`TypedProvider`] when bulk-popped from the [`ValueStack`].
    providers: Vec<TypedProvider>,
    /// Buffer to temporarily hold `br_table` target depths.
    br_table_targets: Vec<u32>,
    /// Buffer to temporarily hold a bunch of preserved [`Reg`] locals.
    preserved: Vec<PreservedLocal>,
}

/// A pair of local [`Reg`] and its preserved [`Reg`].
#[derive(Debug, Copy, Clone)]
pub struct PreservedLocal {
    local: Reg,
    preserved: Reg,
}

impl PreservedLocal {
    /// Creates a new [`PreservedLocal`].
    pub fn new(local: Reg, preserved: Reg) -> Self {
        Self { local, preserved }
    }
}

impl TranslationBuffers {
    /// Resets the [`TranslationBuffers`].
    fn reset(&mut self) {
        self.providers.clear();
        self.br_table_targets.clear();
        self.preserved.clear();
    }

    /// Resets `self` and returns it.
    fn into_reset(mut self) -> Self {
        self.reset();
        self
    }
}

impl FuncTranslatorAllocations {
    /// Resets the [`FuncTranslatorAllocations`].
    fn reset(&mut self) {
        self.stack.reset();
        self.instr_encoder.reset();
        self.control_stack.reset();
        self.buffer.reset();
    }

    /// Resets `self` and returns it.
    fn into_reset(mut self) -> Self {
        self.reset();
        self
    }
}

/// Type concerned with translating from Wasm bytecode to Wasmi bytecode.
pub struct FuncTranslator {
    /// The reference to the Wasm module function under construction.
    func: FuncIdx,
    /// The engine for which the function is compiled.
    ///
    /// # Note
    ///
    /// Technically this is not needed since the information is redundant given via
    /// the `module` field. However, this acts like a faster access since `module`
    /// only holds a weak reference to the engine.
    engine: Engine,
    /// The immutable Wasmi module resources.
    module: ModuleHeader,
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
    /// Fuel costs for fuel metering.
    ///
    /// `None` if fuel metering is disabled.
    fuel_costs: Option<FuelCostsProvider>,
    /// The emulated value stack.
    stack: ValueStack,
    /// The instruction sequence encoder.
    instr_encoder: InstrEncoder,
    /// The control stack.
    control_stack: ControlStack,
    /// Buffer to temporarily hold a bunch of [`TypedProvider`] when bulk-popped from the [`ValueStack`].
    providers: Vec<TypedProvider>,
    /// Buffer to temporarily hold `br_table` target depths.
    br_table_targets: Vec<u32>,
    /// Buffer to temporarily hold a bunch of preserved [`Reg`] locals.
    preserved: Vec<PreservedLocal>,
}

impl WasmTranslator<'_> for FuncTranslator {
    type Allocations = FuncTranslatorAllocations;

    fn setup(&mut self, _bytes: &[u8]) -> Result<bool, Error> {
        Ok(false)
    }

    #[inline]
    fn features(&self) -> WasmFeatures {
        self.engine.config().wasm_features()
    }

    fn translate_locals(
        &mut self,
        amount: u32,
        _value_type: wasmparser::ValType,
    ) -> Result<(), Error> {
        self.stack.register_locals(amount)
    }

    fn finish_translate_locals(&mut self) -> Result<(), Error> {
        self.stack.finish_register_locals();
        Ok(())
    }

    fn update_pos(&mut self, _pos: usize) {}

    fn finish(
        mut self,
        finalize: impl FnOnce(CompiledFuncEntity),
    ) -> Result<Self::Allocations, Error> {
        self.instr_encoder.defrag_registers(&mut self.stack)?;
        self.instr_encoder.update_branch_offsets(&mut self.stack)?;
        let len_registers = self.stack.len_registers();
        if let Some(fuel_costs) = self.fuel_costs() {
            // Note: Fuel metering is enabled so we need to bump the fuel
            //       of the function enclosing Wasm `block` by an amount
            //       that depends on the total number of registers used by
            //       the compiled function.
            // Note: The function enclosing block fuel instruction is always
            //       the instruction at the 0th index if fuel metering is enabled.
            let fuel_instr = Instr::from(0_u32);
            let fuel_info = FuelInfo::some(fuel_costs.clone(), fuel_instr);
            self.instr_encoder
                .bump_fuel_consumption(&fuel_info, |costs| {
                    costs.fuel_for_copying_values(u64::from(len_registers))
                })?;
        }
        let func_consts = self.stack.func_local_consts();
        let instrs = self.instr_encoder.drain_instrs();
        finalize(CompiledFuncEntity::new(len_registers, instrs, func_consts));
        Ok(self.into_allocations())
    }
}

impl FuncTranslator {
    /// Creates a new [`FuncTranslator`].
    pub fn new(
        func: FuncIdx,
        res: ModuleHeader,
        alloc: FuncTranslatorAllocations,
    ) -> Result<Self, Error> {
        let Some(engine) = res.engine().upgrade() else {
            panic!(
                "cannot compile function since engine does no longer exist: {:?}",
                res.engine()
            )
        };
        let config = engine.config();
        let fuel_costs = config
            .get_consume_fuel()
            .then(|| config.fuel_costs())
            .cloned();
        let FuncTranslatorAllocations {
            stack,
            instr_encoder,
            control_stack,
            buffer,
        } = alloc.into_reset();
        let TranslationBuffers {
            providers,
            br_table_targets,
            preserved,
        } = buffer.into_reset();
        Self {
            func,
            engine,
            module: res,
            reachable: true,
            fuel_costs,
            stack,
            instr_encoder,
            control_stack,
            providers,
            br_table_targets,
            preserved,
        }
        .init()
    }

    /// Returns the [`Engine`] for which the function is compiled.
    fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Initializes a newly constructed [`FuncTranslator`].
    fn init(mut self) -> Result<Self, Error> {
        self.init_func_body_block()?;
        self.init_func_params()?;
        Ok(self)
    }

    /// Registers the `block` control frame surrounding the entire function body.
    fn init_func_body_block(&mut self) -> Result<(), Error> {
        let func_type = self.module.get_type_of_func(self.func);
        let block_type = BlockType::func_type(func_type);
        let end_label = self.instr_encoder.new_label();
        let consume_fuel = self.make_fuel_instr()?;
        // Note: we use a dummy `RegSpan` as placeholder since the function enclosing
        //       control block never has branch parameters.
        let branch_params = RegSpan::new(Reg::from(0));
        let block_frame = BlockControlFrame::new(
            block_type,
            end_label,
            branch_params,
            BlockHeight::default(),
            consume_fuel,
        );
        self.control_stack.push_frame(block_frame);
        Ok(())
    }

    /// Registers the function parameters in the emulated value stack.
    fn init_func_params(&mut self) -> Result<(), Error> {
        for _param_type in self.func_type().params() {
            self.stack.register_locals(1)?;
        }
        Ok(())
    }

    /// Consumes `self` and returns the underlying reusable [`FuncTranslatorAllocations`].
    fn into_allocations(self) -> FuncTranslatorAllocations {
        FuncTranslatorAllocations {
            stack: self.stack,
            instr_encoder: self.instr_encoder,
            control_stack: self.control_stack,
            buffer: TranslationBuffers {
                providers: self.providers,
                br_table_targets: self.br_table_targets,
                preserved: self.preserved,
            },
        }
    }

    /// Returns the [`FuncType`] of the function that is currently translated.
    fn func_type(&self) -> FuncType {
        let dedup_func_type = self.module.get_type_of_func(self.func);
        self.engine()
            .resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Resolves the [`FuncType`] of the given [`FuncTypeIdx`].
    fn func_type_at(&self, func_type_index: index::FuncType) -> FuncType {
        let func_type_index = FuncTypeIdx::from(u32::from(func_type_index));
        let dedup_func_type = self.module.get_func_type(func_type_index);
        self.engine()
            .resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Resolves the [`FuncType`] of the given [`FuncIdx`].
    fn func_type_of(&self, func_index: FuncIdx) -> FuncType {
        let dedup_func_type = self.module.get_type_of_func(func_index);
        self.engine()
            .resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Returns `true` if the code at the current translation position is reachable.
    fn is_reachable(&self) -> bool {
        self.reachable
    }

    /// Returns the configured [`FuelCostsProvider`] of the [`Engine`] if any.
    ///
    /// Returns `None` if fuel metering is disabled.
    fn fuel_costs(&self) -> Option<&FuelCostsProvider> {
        self.fuel_costs.as_ref()
    }

    /// Returns the most recent [`Instruction::ConsumeFuel`] in the translation process.
    ///
    /// Returns `None` if fuel metering is disabled.
    fn fuel_instr(&self) -> Option<Instr> {
        self.control_stack.last().consume_fuel_instr()
    }

    /// Returns the [`FuelInfo`] for the current translation state.
    ///
    /// Returns [`FuelInfo::None`] if fuel metering is disabled.
    fn fuel_info(&self) -> FuelInfo {
        self.fuel_info_with(FuncTranslator::fuel_instr)
    }

    /// Returns the [`FuelInfo`] for the given `frame`.
    ///
    /// Returns [`FuelInfo::None`] if fuel metering is disabled.
    fn fuel_info_for(&self, frame: &impl ControlFrameBase) -> FuelInfo {
        self.fuel_info_with(|_| frame.consume_fuel_instr())
    }

    /// Returns the [`FuelInfo`] for the current translation state.
    ///
    /// Returns [`FuelInfo::None`] if fuel metering is disabled.
    fn fuel_info_with(
        &self,
        fuel_instr: impl FnOnce(&FuncTranslator) -> Option<Instr>,
    ) -> FuelInfo {
        let Some(fuel_costs) = self.fuel_costs() else {
            // Fuel metering is disabled so we can bail out.
            return FuelInfo::None;
        };
        let fuel_instr = fuel_instr(self)
            .expect("fuel metering is enabled but there is no Instruction::ConsumeFuel");
        FuelInfo::some(fuel_costs.clone(), fuel_instr)
    }

    /// Pushes a [`Instruction::ConsumeFuel`] with base costs if fuel metering is enabled.
    ///
    /// Returns `None` if fuel metering is disabled.
    fn make_fuel_instr(&mut self) -> Result<Option<Instr>, Error> {
        self.instr_encoder.push_fuel_instr(self.fuel_costs.as_ref())
    }

    /// Utility function for pushing a new [`Instruction`] with fuel costs.
    ///
    /// # Note
    ///
    /// Fuel metering is only encoded or adjusted if it is enabled.
    fn push_fueled_instr<F>(&mut self, instr: Instruction, f: F) -> Result<Instr, Error>
    where
        F: FnOnce(&FuelCostsProvider) -> u64,
    {
        let fuel_info = self.fuel_info();
        self.instr_encoder.push_fueled_instr(instr, &fuel_info, f)
    }

    /// Convenience method for appending an [`Instruction`] parameter.
    fn append_instr(&mut self, instr: Instruction) -> Result<(), Error> {
        self.instr_encoder.append_instr(instr)?;
        Ok(())
    }

    /// Utility function for pushing a new [`Instruction`] with basic fuel costs.
    ///
    /// # Note
    ///
    /// Fuel metering is only encoded or adjusted if it is enabled.
    fn push_base_instr(&mut self, instr: Instruction) -> Result<Instr, Error> {
        self.push_fueled_instr(instr, FuelCostsProvider::base)
    }

    /// Preserve all locals that are currently on the emulated stack.
    ///
    /// # Note
    ///
    /// This is required for correctness upon entering the compilation
    /// of a Wasm control flow structure such as a Wasm `block`, `if` or `loop`.
    /// Locals on the stack might be manipulated conditionally witihn the
    /// control flow structure and therefore need to be preserved before
    /// this might happen.
    /// For efficiency reasons all locals are preserved independent of their
    /// actual use in the entered control flow structure since the analysis
    /// of their uses would be too costly.
    fn preserve_locals(&mut self) -> Result<(), Error> {
        let fuel_info = self.fuel_info();
        let preserved = &mut self.preserved;
        preserved.clear();
        self.stack.preserve_all_locals(|preserved_local| {
            preserved.push(preserved_local);
            Ok(())
        })?;
        preserved.reverse();
        let copy_groups = preserved.chunk_by(|a, b| {
            // Note: we group copies into groups with continuous result register indices
            //       because this is what allows us to fuse single `Copy` instructions into
            //       more efficient `Copy2` or `CopyManyNonOverlapping` instructions.
            //
            // At the time of this writing the author was not sure if all result registers
            // of all preserved locals are always continuous so this can be understood as
            // a safety guard.
            (i16::from(b.preserved) - i16::from(a.preserved)) == 1
        });
        for copy_group in copy_groups {
            let len = u16::try_from(copy_group.len()).unwrap_or_else(|error| {
                panic!(
                    "too many ({}) registers in copy group: {}",
                    copy_group.len(),
                    error
                )
            });
            let results = BoundedRegSpan::new(RegSpan::new(copy_group[0].preserved), len);
            let providers = &mut self.providers;
            providers.clear();
            providers.extend(
                copy_group
                    .iter()
                    .map(|p| p.local)
                    .map(TypedProvider::Register),
            );
            let instr = self.instr_encoder.encode_copies(
                &mut self.stack,
                results,
                &providers[..],
                &fuel_info,
            )?;
            if let Some(instr) = instr {
                self.instr_encoder.notify_preserved_register(instr)
            }
        }
        Ok(())
    }

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

    /// Convenience function to copy the parameters when branching to a control frame.
    fn translate_copy_branch_params(&mut self, frame: &impl ControlFrameBase) -> Result<(), Error> {
        let branch_params = frame.branch_params(&self.engine);
        let consume_fuel_instr = frame.consume_fuel_instr();
        self.translate_copy_branch_params_impl(branch_params, consume_fuel_instr)
    }

    /// Convenience function to copy the parameters when branching to a control frame.
    fn translate_copy_branch_params_impl(
        &mut self,
        branch_params: BoundedRegSpan,
        consume_fuel_instr: Option<Instr>,
    ) -> Result<(), Error> {
        if branch_params.is_empty() {
            // If the block does not have branch parameters there is no need to copy anything.
            return Ok(());
        }
        let fuel_info = self.fuel_info_with(|_| consume_fuel_instr);
        let params = &mut self.providers;
        self.stack.pop_n(usize::from(branch_params.len()), params);
        self.instr_encoder.encode_copies(
            &mut self.stack,
            branch_params,
            &self.providers[..],
            &fuel_info,
        )?;
        Ok(())
    }

    /// Translates the `end` of a Wasm `block` control frame.
    fn translate_end_block(&mut self, frame: BlockControlFrame) -> Result<(), Error> {
        let is_func_block = self.control_stack.is_empty();
        if self.reachable && frame.is_branched_to() {
            self.translate_copy_branch_params(&frame)?;
        }
        // Since the `block` is now sealed we can pin its end label.
        self.instr_encoder.pin_label(frame.end_label());
        if frame.is_branched_to() {
            // Case: branches to this block exist so we cannot treat the
            //       basic block as a no-op and instead have to put its
            //       block results on top of the stack.
            self.stack
                .trunc(usize::from(frame.block_height().into_u16()));
            for result in frame.branch_params(self.engine()) {
                self.stack.push_register(result)?;
            }
        }
        self.reachable |= frame.is_branched_to();
        if self.reachable && is_func_block {
            // We dropped the Wasm `block` that encloses the function itself so we can return.
            self.translate_return_with(&frame)?;
        }
        Ok(())
    }

    /// Translates the `end` of a Wasm `loop` control frame.
    fn translate_end_loop(&mut self, _frame: LoopControlFrame) -> Result<(), Error> {
        debug_assert!(
            !self.control_stack.is_empty(),
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
    fn translate_end_if(&mut self, frame: IfControlFrame) -> Result<(), Error> {
        debug_assert!(
            !self.control_stack.is_empty(),
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
    fn translate_end_if_then_only(&mut self, frame: IfControlFrame) -> Result<(), Error> {
        debug_assert!(frame.is_then_reachable());
        debug_assert!(!frame.is_else_reachable());
        debug_assert!(frame.else_label().is_none());
        let end_of_then_reachable = frame.is_end_of_then_reachable().unwrap_or(self.reachable);
        self.translate_end_if_then_or_else_only(frame, end_of_then_reachable)
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
    fn translate_end_if_else_only(&mut self, frame: IfControlFrame) -> Result<(), Error> {
        debug_assert!(!frame.is_then_reachable());
        debug_assert!(frame.is_else_reachable());
        debug_assert!(frame.else_label().is_none());
        let end_of_else_reachable = self.reachable || !frame.has_visited_else();
        self.translate_end_if_then_or_else_only(frame, end_of_else_reachable)
    }

    /// Translates the `end` of a Wasm `if` [`ControlFrame`] were only `then` xor `else` is reachable.
    fn translate_end_if_then_or_else_only(
        &mut self,
        frame: IfControlFrame,
        end_is_reachable: bool,
    ) -> Result<(), Error> {
        if end_is_reachable && frame.is_branched_to() {
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
            self.translate_copy_branch_params(&frame)?;
        }
        // Since the `if` is now sealed we can pin its `end` label.
        self.instr_encoder.pin_label(frame.end_label());
        if frame.is_branched_to() {
            // Case: branches to this block exist so we cannot treat the
            //       basic block as a no-op and instead have to put its
            //       block results on top of the stack.
            self.stack
                .trunc(usize::from(frame.block_height().into_u16()));
            for result in frame.branch_params(self.engine()) {
                self.stack.push_register(result)?;
            }
        }
        // We reset reachability in case the end of the `block` was reachable.
        self.reachable = end_is_reachable || frame.is_branched_to();
        Ok(())
    }

    /// Translates the `end` of a Wasm `if` [`ControlFrame`] were both `then` and `else` are reachable.
    fn translate_end_if_then_else(&mut self, frame: IfControlFrame) -> Result<(), Error> {
        debug_assert!(frame.is_then_reachable());
        debug_assert!(frame.is_else_reachable());
        match frame.has_visited_else() {
            true => self.translate_end_if_then_with_else(frame),
            false => self.translate_end_if_then_missing_else(frame),
        }
    }

    /// Variant of [`Self::translate_end_if_then_else`] were the `else` block exists.
    fn translate_end_if_then_with_else(&mut self, frame: IfControlFrame) -> Result<(), Error> {
        debug_assert!(frame.has_visited_else());
        let end_of_then_reachable = frame
            .is_end_of_then_reachable()
            .expect("must be set since `else` was visited");
        let end_of_else_reachable = self.reachable;
        let reachable = match (end_of_then_reachable, end_of_else_reachable) {
            (false, false) => frame.is_branched_to(),
            _ => true,
        };
        self.instr_encoder.pin_label_if_unpinned(
            frame
                .else_label()
                .expect("must have `else` label since `else` is reachable"),
        );
        let if_height = usize::from(frame.block_height().into_u16());
        if end_of_else_reachable {
            // Since the end of `else` is reachable we need to properly
            // write the `else` block results back to were the `if` expects
            // its results to reside upon exit.
            self.translate_copy_branch_params(&frame)?;
        }
        // After `else` parameters have been copied we can finally pin the `end` label.
        self.instr_encoder.pin_label(frame.end_label());
        if reachable {
            // In case the code following the `if` is reachable we need
            // to clean up and prepare the value stack.
            self.stack.trunc(if_height);
            for result in frame.branch_params(self.engine()) {
                self.stack.push_register(result)?;
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
    fn translate_end_if_then_missing_else(&mut self, frame: IfControlFrame) -> Result<(), Error> {
        debug_assert!(!frame.has_visited_else());
        let end_of_then_reachable = self.reachable;
        let has_results = frame.block_type().len_results(self.engine()) >= 1;
        if end_of_then_reachable && has_results {
            // Since the `else` block is missing we need to write the results
            // from the `then` block back to were the `if` control frame expects
            // its results afterwards.
            // Furthermore we need to encode the branch to the `if` end label.
            self.translate_copy_branch_params(&frame)?;
            let end_offset = self.instr_encoder.try_resolve_label(frame.end_label())?;
            self.push_fueled_instr(Instruction::branch(end_offset), FuelCostsProvider::base)?;
        }
        self.instr_encoder.pin_label_if_unpinned(
            frame
                .else_label()
                .expect("must have `else` label since `else` is reachable"),
        );
        let engine = self.engine().clone();
        let if_height = usize::from(frame.block_height().into_u16());
        let else_providers = self.control_stack.pop_else_providers();
        if has_results {
            // We haven't visited the `else` block and thus the `else`
            // providers are still on the auxiliary stack and need to
            // be popped. We use them to restore the stack to the state
            // when entering the `if` block so that we can properly copy
            // the `else` results to were they are expected.
            self.stack.trunc(if_height);
            for provider in else_providers {
                self.stack.push_provider(provider)?;
                if let TypedProvider::Register(register) = provider {
                    self.stack.dec_register_usage(register);
                }
            }
            self.translate_copy_branch_params(&frame)?;
        }
        // After `else` parameters have been copied we can finally pin the `end` label.
        self.instr_encoder.pin_label(frame.end_label());
        // Without `else` block the code after the `if` is always reachable and
        // thus we need to clean up and prepare the value stack for the following code.
        self.stack.trunc(if_height);
        for result in frame.branch_params(&engine) {
            self.stack.push_register(result)?;
        }
        self.reachable = true;
        Ok(())
    }

    /// Translates the `end` of an unreachable control frame.
    fn translate_end_unreachable(&mut self, _frame: UnreachableControlFrame) -> Result<(), Error> {
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
    ///      going to be returned via [`RegSpan`].
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
        len_block_params: u16,
        len_branch_params: u16,
    ) -> Result<RegSpan, Error> {
        let params = &mut self.providers;
        // Pop the block parameters off the stack.
        self.stack.pop_n(usize::from(len_block_params), params);
        // Peek the branch parameter registers which are going to be returned.
        let branch_params = self.stack.peek_dynamic_n(usize::from(len_branch_params))?;
        // Push the block parameters onto the stack again as if nothing happened.
        self.stack.push_n(params)?;
        params.clear();
        Ok(branch_params)
    }

    /// Pushes a binary instruction with two register inputs `lhs` and `rhs`.
    fn push_binary_instr(
        &mut self,
        lhs: Reg,
        rhs: Reg,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
    ) -> Result<(), Error> {
        let result = self.stack.push_dynamic()?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCostsProvider::base)?;
        Ok(())
    }

    /// Pushes a binary instruction if the immediate operand can be encoded in 16 bits.
    ///
    /// # Note
    ///
    /// - Returns `Ok(true)` is the optimization was applied.
    /// - Returns `Ok(false)` is the optimization could not be applied.
    /// - Returns `Err(_)` if a translation error occurred.
    fn try_push_binary_instr_imm16<T>(
        &mut self,
        lhs: Reg,
        rhs: T,
        make_instr_imm16: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
    ) -> Result<bool, Error>
    where
        T: Copy + TryInto<Const16<T>>,
    {
        if let Ok(rhs) = rhs.try_into() {
            // Optimization: We can use a compact instruction for small constants.
            let result = self.stack.push_dynamic()?;
            self.push_fueled_instr(make_instr_imm16(result, lhs, rhs), FuelCostsProvider::base)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Variant of [`Self::try_push_binary_instr_imm16`] with swapped operands for `make_instr_imm16`.
    fn try_push_binary_instr_imm16_rev<T>(
        &mut self,
        lhs: T,
        rhs: Reg,
        make_instr_imm16: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
    ) -> Result<bool, Error>
    where
        T: Copy + TryInto<Const16<T>>,
    {
        if let Ok(lhs) = lhs.try_into() {
            // Optimization: We can use a compact instruction for small constants.
            let result = self.stack.push_dynamic()?;
            self.push_fueled_instr(make_instr_imm16(result, lhs, rhs), FuelCostsProvider::base)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Evaluates the constants and pushes the proper result to the value stack.
    fn push_binary_consteval<T, R>(
        &mut self,
        lhs: TypedVal,
        rhs: TypedVal,
        consteval: fn(T, T) -> R,
    ) -> Result<(), Error>
    where
        T: From<TypedVal>,
        R: Into<TypedVal>,
    {
        self.stack
            .push_const(consteval(lhs.into(), rhs.into()).into());
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
        lhs: Reg,
        rhs: T,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
    ) -> Result<(), Error>
    where
        T: Into<UntypedVal>,
    {
        let result = self.stack.push_dynamic()?;
        let rhs = self.stack.alloc_const(rhs)?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCostsProvider::base)?;
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
        rhs: Reg,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
    ) -> Result<(), Error>
    where
        T: Into<UntypedVal>,
    {
        let result = self.stack.push_dynamic()?;
        let lhs = self.stack.alloc_const(lhs)?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCostsProvider::base)?;
        Ok(())
    }

    /// Translates a [`TrapCode`] as [`Instruction`].
    fn translate_trap(&mut self, trap_code: TrapCode) -> Result<(), Error> {
        bail_unreachable!(self);
        self.push_fueled_instr(Instruction::trap(trap_code), FuelCostsProvider::base)?;
        self.reachable = false;
        Ok(())
    }

    /// Translate a non-commutative binary Wasmi integer instruction.
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
    /// - The `make_instr_reg_imm_opt` closure allows to implement custom optimization
    ///   logic for the case that the right-hand side operand is a constant value.
    /// - The `make_instr_imm_reg_opt` closure allows to implement custom optimization
    ///   logic for the case that the left-hand side operand is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to Wasmi bytecode:
    ///
    /// - `{i32, i64}.{sub, lt_s, lt_u, le_s, le_u, gt_s, gt_u, ge_s, ge_u}`
    #[allow(clippy::too_many_arguments)]
    fn translate_binary<T, R>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm16_rhs: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
        make_instr_imm16_lhs: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
        consteval: fn(T, T) -> R,
        make_instr_opt: fn(&mut Self, lhs: Reg, rhs: Reg) -> Result<bool, Error>,
        make_instr_reg_imm_opt: fn(&mut Self, lhs: Reg, rhs: T) -> Result<bool, Error>,
        make_instr_imm_reg_opt: fn(&mut Self, lhs: T, rhs: Reg) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: Copy + From<TypedVal> + TryInto<Const16<T>>,
        R: Into<TypedVal>,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
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
                if self.try_push_binary_instr_imm16(lhs, T::from(rhs), make_instr_imm16_rhs)? {
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
                if self.try_push_binary_instr_imm16_rev(T::from(lhs), rhs, make_instr_imm16_lhs)? {
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

    /// Translate a non-commutative binary Wasmi float instruction.
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
    /// - The `make_instr_reg_imm_opt` closure allows to implement custom optimization
    ///   logic for the case that the right-hand side operand is a constant value.
    /// - The `make_instr_imm_reg_opt` closure allows to implement custom optimization
    ///   logic for the case that the left-hand side operand is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to Wasmi bytecode:
    ///
    /// - `{f32, f64}.{sub, div}`
    #[allow(clippy::too_many_arguments)]
    fn translate_fbinary<T, R>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        consteval: fn(T, T) -> R,
        make_instr_opt: fn(&mut Self, lhs: Reg, rhs: Reg) -> Result<bool, Error>,
        make_instr_reg_imm_opt: fn(&mut Self, lhs: Reg, rhs: T) -> Result<bool, Error>,
        make_instr_imm_reg_opt: fn(&mut Self, lhs: T, rhs: Reg) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: WasmFloat,
        R: Into<TypedVal>,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
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
                    self.stack.push_const(rhs);
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
                    self.stack.push_const(lhs);
                    return Ok(());
                }
                self.push_binary_instr_imm_rev(lhs, rhs, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => {
                self.push_binary_consteval(lhs, rhs, consteval)
            }
        }
    }

    /// Translate Wasmi float `{f32,f64}.copysign` instructions.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for copysign instructions.
    /// - Applies constant evaluation if both operands are constant values.
    fn translate_fcopysign<T>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm: fn(result: Reg, lhs: Reg, rhs: Sign<T>) -> Instruction,
        consteval: fn(T, T) -> T,
    ) -> Result<(), Error>
    where
        T: WasmFloat,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                if lhs == rhs {
                    // Optimization: `copysign x x` is always just `x`
                    self.stack.push_register(lhs)?;
                    return Ok(());
                }
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (TypedProvider::Register(lhs), TypedProvider::Const(rhs)) => {
                let sign = T::from(rhs).sign();
                let result = self.stack.push_dynamic()?;
                self.push_fueled_instr(make_instr_imm(result, lhs, sign), FuelCostsProvider::base)?;
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

    /// Translate a commutative binary Wasmi integer instruction.
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
    /// - The `make_instr_imm_opt` closure allows to implement custom optimization
    ///   logic for the case that one of the operands is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to Wasmi bytecode:
    ///
    /// - `{i32, i64}.{eq, ne, add, mul, and, or, xor}`
    #[allow(clippy::too_many_arguments)]
    fn translate_binary_commutative<T, R>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm16: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
        consteval: fn(T, T) -> R,
        make_instr_opt: fn(&mut Self, lhs: Reg, rhs: Reg) -> Result<bool, Error>,
        make_instr_imm_opt: fn(&mut Self, lhs: Reg, rhs: T) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: Copy + From<TypedVal> + TryInto<Const16<T>>,
        R: Into<TypedVal>,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
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
                self.push_binary_consteval::<T, R>(lhs, rhs, consteval)
            }
        }
    }

    /// Translate a commutative binary Wasmi float instruction.
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
    /// - The `make_instr_imm_opt` closure allows to implement custom optimization
    ///   logic for the case that one of the operands is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to Wasmi bytecode:
    ///
    /// - `{f32, f64}.{add, mul, min, max}`
    #[allow(clippy::too_many_arguments)]
    fn translate_fbinary_commutative<T, R>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        consteval: fn(T, T) -> R,
        make_instr_opt: fn(&mut Self, lhs: Reg, rhs: Reg) -> Result<bool, Error>,
        make_instr_imm_opt: fn(&mut Self, lhs: Reg, rhs: T) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: WasmFloat,
        R: Into<TypedVal>,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
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
                    self.stack.push_const(T::from(imm_in));
                    return Ok(());
                }
                self.push_binary_instr_imm(reg_in, imm_in, make_instr)
            }
            (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => {
                self.push_binary_consteval(lhs, rhs, consteval)
            }
        }
    }

    /// Translate a shift or rotate Wasmi instruction.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for all shift or rotate instructions.
    /// - Its various function arguments allow it to be used for generic Wasm types.
    /// - Applies constant evaluation if both operands are constant values.
    ///
    /// - The `make_instr_imm_reg_opt` closure allows to implement custom optimization
    ///   logic for the case the shifted value operand is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to Wasmi bytecode:
    ///
    /// - `{i32, i64}.{shl, shr_s, shr_u, rotl, rotr}`
    #[allow(clippy::too_many_arguments)]
    fn translate_shift<T>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_by: fn(
            result: Reg,
            lhs: Reg,
            rhs: <T as IntoShiftAmount>::Output,
        ) -> Instruction,
        make_instr_imm16: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
        consteval: fn(T, T) -> T,
        make_instr_imm_reg_opt: fn(&mut Self, lhs: T, rhs: Reg) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: WasmInteger + IntoShiftAmount<Input: From<TypedVal>>,
        Const16<T>: From<i16>,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (TypedProvider::Register(lhs), TypedProvider::Const(rhs)) => {
                let Some(rhs) = T::into_shift_amount(rhs.into()) else {
                    // Optimization: Shifting or rotating by zero bits is a no-op.
                    self.stack.push_register(lhs)?;
                    return Ok(());
                };
                let result = self.stack.push_dynamic()?;
                self.push_fueled_instr(make_instr_by(result, lhs, rhs), FuelCostsProvider::base)?;
                Ok(())
            }
            (TypedProvider::Const(lhs), TypedProvider::Register(rhs)) => {
                if make_instr_imm_reg_opt(self, T::from(lhs), rhs)? {
                    // Custom optimization was applied: return early
                    return Ok(());
                }
                if T::from(lhs).eq_zero() {
                    // Optimization: Shifting or rotating a zero value is a no-op.
                    self.stack.push_const(lhs);
                    return Ok(());
                }
                if self.try_push_binary_instr_imm16_rev(T::from(lhs), rhs, make_instr_imm16)? {
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

    /// Translate an integer division or remainder Wasmi instruction.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for all `div` or `rem` instructions.
    /// - Its various function arguments allow it to be used for `i32` and `i64` types.
    /// - Applies constant evaluation if both operands are constant values.
    ///
    /// - The `make_instr_reg_imm_opt` closure allows to implement custom optimization
    ///   logic for the case the right-hand side operand is a constant value.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to Wasmi bytecode:
    ///
    /// - `{i32, i64}.{div_u, div_s, rem_u, rem_s}`
    #[allow(clippy::too_many_arguments)]
    fn translate_divrem<T>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm16: fn(result: Reg, lhs: Reg, rhs: Const16<T::NonZero>) -> Instruction,
        make_instr_imm16_rev: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
        consteval: fn(T, T) -> Result<T, TrapCode>,
        make_instr_opt: fn(&mut Self, lhs: Reg, rhs: Reg) -> Result<bool, Error>,
        make_instr_reg_imm_opt: fn(&mut Self, lhs: Reg, rhs: T) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: WasmInteger,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                if make_instr_opt(self, lhs, rhs)? {
                    // Custom optimization was applied: return early
                    return Ok(());
                }
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (TypedProvider::Register(lhs), TypedProvider::Const(rhs)) => {
                let Some(non_zero_rhs) = <T as WasmInteger>::non_zero(T::from(rhs)) else {
                    // Optimization: division by zero always traps
                    self.translate_trap(TrapCode::IntegerDivisionByZero)?;
                    return Ok(());
                };
                if make_instr_reg_imm_opt(self, lhs, T::from(rhs))? {
                    // Custom optimization was applied: return early
                    return Ok(());
                }
                if self.try_push_binary_instr_imm16(lhs, non_zero_rhs, make_instr_imm16)? {
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
            (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => {
                match consteval(lhs.into(), rhs.into()) {
                    Ok(result) => {
                        self.stack.push_const(result);
                        Ok(())
                    }
                    Err(trap_code) => self.translate_trap(trap_code),
                }
            }
        }
    }

    /// Can be used for [`Self::translate_binary`] (and variants) if no custom optimization shall be applied.
    fn no_custom_opt<Lhs, Rhs>(&mut self, _lhs: Lhs, _rhs: Rhs) -> Result<bool, Error> {
        Ok(false)
    }

    /// Translates a unary Wasm instruction to Wasmi bytecode.
    fn translate_unary<T, R>(
        &mut self,
        make_instr: fn(result: Reg, input: Reg) -> Instruction,
        consteval: fn(input: T) -> R,
    ) -> Result<(), Error>
    where
        T: From<TypedVal>,
        R: Into<TypedVal>,
    {
        bail_unreachable!(self);
        match self.stack.pop() {
            TypedProvider::Register(input) => {
                let result = self.stack.push_dynamic()?;
                self.push_fueled_instr(make_instr(result, input), FuelCostsProvider::base)?;
                Ok(())
            }
            TypedProvider::Const(input) => {
                self.stack.push_const(consteval(input.into()).into());
                Ok(())
            }
        }
    }

    /// Translates a fallible unary Wasm instruction to Wasmi bytecode.
    fn translate_unary_fallible<T, R>(
        &mut self,
        make_instr: fn(result: Reg, input: Reg) -> Instruction,
        consteval: fn(input: T) -> Result<R, TrapCode>,
    ) -> Result<(), Error>
    where
        T: From<TypedVal>,
        R: Into<TypedVal>,
    {
        bail_unreachable!(self);
        match self.stack.pop() {
            TypedProvider::Register(input) => {
                let result = self.stack.push_dynamic()?;
                self.push_fueled_instr(make_instr(result, input), FuelCostsProvider::base)?;
                Ok(())
            }
            TypedProvider::Const(input) => match consteval(input.into()) {
                Ok(result) => {
                    self.stack.push_const(result);
                    Ok(())
                }
                Err(trap_code) => self.translate_trap(trap_code),
            },
        }
    }

    /// Returns the [`MemArg`] linear `memory` index and load/store `offset`.
    ///
    /// # Panics
    ///
    /// If the [`MemArg`] offset is not 32-bit.
    fn decode_memarg(memarg: MemArg) -> (index::Memory, u64) {
        let memory = index::Memory::from(memarg.memory);
        (memory, memarg.offset)
    }

    /// Returns the effective address `ptr+offset` if it is valid.
    fn effective_address(&self, mem: index::Memory, ptr: TypedVal, offset: u64) -> Option<Address> {
        let memory_type = *self
            .module
            .get_type_of_memory(MemoryIdx::from(u32::from(mem)));
        let ptr = match memory_type.is_64() {
            true => u64::from(ptr),
            false => u64::from(u32::from(ptr)),
        };
        let Some(address) = ptr.checked_add(offset) else {
            // Case: address overflows any legal memory index.
            return None;
        };
        if let Some(max) = memory_type.maximum() {
            // The memory's maximum size in bytes.
            let max_size = max << memory_type.page_size_log2();
            if address > max_size {
                // Case: address overflows the memory's maximum size.
                return None;
            }
        }
        if !memory_type.is_64() && address >= 1 << 32 {
            // Case: address overflows the 32-bit memory index.
            return None;
        }
        let Ok(address) = Address::try_from(address) else {
            // Case: address is too big for the system to handle properly.
            return None;
        };
        Some(address)
    }

    /// Translates a Wasm `load` instruction to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This chooses the right encoding for the given `load` instruction.
    /// If `ptr+offset` is a constant value the address is pre-calculated.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to Wasmi bytecode:
    ///
    /// - `{i32, i64, f32, f64}.load`
    /// - `i32.{load8_s, load8_u, load16_s, load16_u}`
    /// - `i64.{load8_s, load8_u, load16_s, load16_u load32_s, load32_u}`
    fn translate_load(
        &mut self,
        memarg: MemArg,
        make_instr: fn(result: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_offset16: fn(result: Reg, ptr: Reg, offset: Offset16) -> Instruction,
        make_instr_at: fn(result: Reg, address: Address32) -> Instruction,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg);
        let ptr = self.stack.pop();
        let (ptr, offset) = match ptr {
            Provider::Register(ptr) => (ptr, offset),
            Provider::Const(ptr) => {
                let Some(address) = self.effective_address(memory, ptr, offset) else {
                    return self.translate_trap(TrapCode::MemoryOutOfBounds);
                };
                if let Ok(address) = Address32::try_from(address) {
                    let result = self.stack.push_dynamic()?;
                    self.push_fueled_instr(
                        make_instr_at(result, address),
                        FuelCostsProvider::load,
                    )?;
                    if !memory.is_default() {
                        self.instr_encoder
                            .append_instr(Instruction::memory_index(memory))?;
                    }
                    return Ok(());
                }
                // Case: we cannot use specialized encoding and thus have to fall back
                //       to the general case where `ptr` is zero and `offset` stores the
                //       `ptr+offset` address value.
                let zero_ptr = self.stack.alloc_const(0_u64)?;
                (zero_ptr, u64::from(address))
            }
        };
        let result = self.stack.push_dynamic()?;
        if memory.is_default() {
            if let Ok(offset) = Offset16::try_from(offset) {
                self.push_fueled_instr(
                    make_instr_offset16(result, ptr, offset),
                    FuelCostsProvider::load,
                )?;
                return Ok(());
            }
        }
        let (offset_hi, offset_lo) = Offset64::split(offset);
        self.push_fueled_instr(make_instr(result, offset_lo), FuelCostsProvider::load)?;
        self.instr_encoder
            .append_instr(Instruction::register_and_offset_hi(ptr, offset_hi))?;
        if !memory.is_default() {
            self.instr_encoder
                .append_instr(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    /// Translates non-wrapping Wasm integer `store` to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// Convenience method that simply forwards to [`Self::translate_istore_wrap`].
    #[allow(clippy::too_many_arguments)]
    fn translate_istore<Src, Field>(
        &mut self,
        memarg: MemArg,
        make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_imm: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_offset16: fn(ptr: Reg, offset: Offset16, value: Reg) -> Instruction,
        make_instr_offset16_imm: fn(ptr: Reg, offset: Offset16, value: Field) -> Instruction,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
        make_instr_at_imm: fn(value: Field, address: Address32) -> Instruction,
    ) -> Result<(), Error>
    where
        Src: Copy + From<TypedVal>,
        Field: TryFrom<Src> + Into<AnyConst16>,
    {
        self.translate_istore_wrap::<Src, Src, Field>(
            memarg,
            make_instr,
            make_instr_imm,
            make_instr_offset16,
            make_instr_offset16_imm,
            make_instr_at,
            make_instr_at_imm,
        )
    }

    /// Translates Wasm integer `store` and `storeN` instructions to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This chooses the most efficient encoding for the given `store` instruction.
    /// If `ptr+offset` is a constant value the pointer address is pre-calculated.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to Wasmi bytecode:
    ///
    /// - `{i32, i64}.{store, store8, store16, store32}`
    #[allow(clippy::too_many_arguments)]
    fn translate_istore_wrap<Src, Wrapped, Field>(
        &mut self,
        memarg: MemArg,
        make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_imm: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_offset16: fn(ptr: Reg, offset: Offset16, value: Reg) -> Instruction,
        make_instr_offset16_imm: fn(ptr: Reg, offset: Offset16, value: Field) -> Instruction,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
        make_instr_at_imm: fn(value: Field, address: Address32) -> Instruction,
    ) -> Result<(), Error>
    where
        Src: Copy + Wrap<Wrapped> + From<TypedVal>,
        Field: TryFrom<Wrapped> + Into<AnyConst16>,
    {
        bail_unreachable!(self);
        let (ptr, value) = self.stack.pop2();
        self.translate_istore_wrap_impl::<Src, Wrapped, Field>(
            memarg,
            ptr,
            value,
            make_instr,
            make_instr_imm,
            make_instr_offset16,
            make_instr_offset16_imm,
            make_instr_at,
            make_instr_at_imm,
        )
    }

    /// Implementation of [`Self::translate_istore_wrap`] without emulation stack popping.
    #[allow(clippy::too_many_arguments)]
    fn translate_istore_wrap_impl<Src, Wrapped, Field>(
        &mut self,
        memarg: MemArg,
        ptr: TypedProvider,
        value: TypedProvider,
        make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_imm: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_offset16: fn(ptr: Reg, offset: Offset16, value: Reg) -> Instruction,
        make_instr_offset16_imm: fn(ptr: Reg, offset: Offset16, value: Field) -> Instruction,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
        make_instr_at_imm: fn(value: Field, address: Address32) -> Instruction,
    ) -> Result<(), Error>
    where
        Src: Copy + Wrap<Wrapped> + From<TypedVal>,
        Field: TryFrom<Wrapped> + Into<AnyConst16>,
    {
        let (memory, offset) = Self::decode_memarg(memarg);
        let (ptr, offset) = match ptr {
            Provider::Register(ptr) => (ptr, offset),
            Provider::Const(ptr) => {
                let Some(address) = self.effective_address(memory, ptr, offset) else {
                    return self.translate_trap(TrapCode::MemoryOutOfBounds);
                };
                if let Ok(address) = Address32::try_from(address) {
                    return self.translate_istore_wrap_at::<Src, Wrapped, Field>(
                        memory,
                        address,
                        value,
                        make_instr_at,
                        make_instr_at_imm,
                    );
                }
                // Case: we cannot use specialized encoding and thus have to fall back
                //       to the general case where `ptr` is zero and `offset` stores the
                //       `ptr+offset` address value.
                let zero_ptr = self.stack.alloc_const(0_u64)?;
                (zero_ptr, u64::from(address))
            }
        };
        if memory.is_default() {
            if let Some(_instr) = self.translate_istore_wrap_mem0::<Src, Wrapped, Field>(
                ptr,
                offset,
                value,
                make_instr_offset16,
                make_instr_offset16_imm,
            )? {
                return Ok(());
            }
        }
        let (offset_hi, offset_lo) = Offset64::split(offset);
        let (instr, param) = match value {
            TypedProvider::Register(value) => (
                make_instr(ptr, offset_lo),
                Instruction::register_and_offset_hi(value, offset_hi),
            ),
            TypedProvider::Const(value) => match Field::try_from(Src::from(value).wrap()).ok() {
                Some(value) => (
                    make_instr_imm(ptr, offset_lo),
                    Instruction::imm16_and_offset_hi(value, offset_hi),
                ),
                None => (
                    make_instr(ptr, offset_lo),
                    Instruction::register_and_offset_hi(self.stack.alloc_const(value)?, offset_hi),
                ),
            },
        };
        self.push_fueled_instr(instr, FuelCostsProvider::store)?;
        self.append_instr(param)?;
        if !memory.is_default() {
            self.instr_encoder
                .append_instr(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    /// Translates Wasm integer `store` and `storeN` instructions to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This is used in cases where the `ptr` is a known constant value.
    fn translate_istore_wrap_at<Src, Wrapped, Field>(
        &mut self,
        memory: index::Memory,
        address: Address32,
        value: TypedProvider,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
        make_instr_at_imm: fn(value: Field, address: Address32) -> Instruction,
    ) -> Result<(), Error>
    where
        Src: Copy + From<TypedVal> + Wrap<Wrapped>,
        Field: TryFrom<Wrapped>,
    {
        match value {
            Provider::Register(value) => {
                self.push_fueled_instr(make_instr_at(value, address), FuelCostsProvider::store)?;
            }
            Provider::Const(value) => {
                if let Ok(value) = Field::try_from(Src::from(value).wrap()) {
                    self.push_fueled_instr(
                        make_instr_at_imm(value, address),
                        FuelCostsProvider::store,
                    )?;
                } else {
                    let value = self.stack.alloc_const(value)?;
                    self.push_fueled_instr(
                        make_instr_at(value, address),
                        FuelCostsProvider::store,
                    )?;
                }
            }
        }
        if !memory.is_default() {
            self.instr_encoder
                .append_instr(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    /// Translates Wasm integer `store` and `storeN` instructions to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This optimizes for cases where the Wasm linear memory that is operated on is known
    /// to be the default memory.
    /// Returns `Some` in case the optimized instructions have been encoded.
    fn translate_istore_wrap_mem0<Src, Wrapped, Field>(
        &mut self,
        ptr: Reg,
        offset: u64,
        value: TypedProvider,
        make_instr_offset16: fn(Reg, Offset16, Reg) -> Instruction,
        make_instr_offset16_imm: fn(Reg, Offset16, Field) -> Instruction,
    ) -> Result<Option<Instr>, Error>
    where
        Src: Copy + From<TypedVal> + Wrap<Wrapped>,
        Field: TryFrom<Wrapped>,
    {
        let Ok(offset16) = Offset16::try_from(offset) else {
            return Ok(None);
        };
        let instr = match value {
            Provider::Register(value) => self.push_fueled_instr(
                make_instr_offset16(ptr, offset16, value),
                FuelCostsProvider::store,
            )?,
            Provider::Const(value) => match Field::try_from(Src::from(value).wrap()) {
                Ok(value) => self.push_fueled_instr(
                    make_instr_offset16_imm(ptr, offset16, value),
                    FuelCostsProvider::store,
                )?,
                Err(_) => {
                    let value = self.stack.alloc_const(value)?;
                    self.push_fueled_instr(
                        make_instr_offset16(ptr, offset16, value),
                        FuelCostsProvider::store,
                    )?
                }
            },
        };
        Ok(Some(instr))
    }

    /// Translates a general Wasm `store` instruction to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This chooses the most efficient encoding for the given `store` instruction.
    /// If `ptr+offset` is a constant value the pointer address is pre-calculated.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to Wasmi bytecode:
    ///
    /// - `{f32, f64, v128}.store`
    fn translate_store(
        &mut self,
        memarg: MemArg,
        make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_offset16: fn(ptr: Reg, offset: Offset16, value: Reg) -> Instruction,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg);
        let (ptr, value) = self.stack.pop2();
        let (ptr, offset) = match ptr {
            Provider::Register(ptr) => (ptr, offset),
            Provider::Const(ptr) => {
                let Some(address) = self.effective_address(memory, ptr, offset) else {
                    return self.translate_trap(TrapCode::MemoryOutOfBounds);
                };
                if let Ok(address) = Address32::try_from(address) {
                    return self.translate_fstore_at(memory, address, value, make_instr_at);
                }
                let zero_ptr = self.stack.alloc_const(0_u64)?;
                (zero_ptr, u64::from(address))
            }
        };
        let (offset_hi, offset_lo) = Offset64::split(offset);
        let value = self.stack.provider2reg(&value)?;
        if memory.is_default() {
            if let Ok(offset) = Offset16::try_from(offset) {
                self.push_fueled_instr(
                    make_instr_offset16(ptr, offset, value),
                    FuelCostsProvider::store,
                )?;
                return Ok(());
            }
        }
        self.push_fueled_instr(make_instr(ptr, offset_lo), FuelCostsProvider::store)?;
        self.instr_encoder
            .append_instr(Instruction::register_and_offset_hi(value, offset_hi))?;
        if !memory.is_default() {
            self.instr_encoder
                .append_instr(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    /// Translates a general Wasm `store` instruction to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This is used in cases where the `ptr` is a known constant value.
    fn translate_fstore_at(
        &mut self,
        memory: index::Memory,
        address: Address32,
        value: TypedProvider,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
    ) -> Result<(), Error> {
        let value = self.stack.provider2reg(&value)?;
        self.push_fueled_instr(make_instr_at(value, address), FuelCostsProvider::store)?;
        if !memory.is_default() {
            self.instr_encoder
                .append_instr(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    /// Translates a Wasm `select` or `select <ty>` instruction.
    ///
    /// # Note
    ///
    /// - This applies constant propagation in case `condition` is a constant value.
    /// - If both `lhs` and `rhs` are equal registers or constant values `lhs` is forwarded.
    /// - Fuses compare instructions with the associated select instructions if possible.
    fn translate_select(&mut self, _type_hint: Option<ValType>) -> Result<(), Error> {
        bail_unreachable!(self);
        let (true_val, false_val, condition) = self.stack.pop3();
        if true_val == false_val {
            // Optimization: both `lhs` and `rhs` either are the same register or constant values and
            //               thus `select` will always yield this same value irrespective of the condition.
            //
            // TODO: we could technically look through registers representing function local constants and
            //       check whether they are equal to a given constant in cases where `lhs` and `rhs` are referring
            //       to a function local register and a constant value or vice versa.
            self.stack.push_provider(true_val)?;
            return Ok(());
        }
        let condition = match condition {
            Provider::Register(condition) => condition,
            Provider::Const(condition) => {
                // Optimization: since condition is a constant value we can const-fold the `select`
                //               instruction and simply push the selected value back to the provider stack.
                let selected = match i32::from(condition) != 0 {
                    true => true_val,
                    false => false_val,
                };
                if let Provider::Register(reg) = selected {
                    if matches!(
                        self.stack.get_register_space(reg),
                        RegisterSpace::Dynamic | RegisterSpace::Preserve
                    ) {
                        // Case: constant propagating a dynamic or preserved register might overwrite it in
                        //       future instruction translation steps and thus we may require a copy instruction
                        //       to prevent this from happening.
                        let result = self.stack.push_dynamic()?;
                        let fuel_info = self.fuel_info();
                        self.instr_encoder.encode_copy(
                            &mut self.stack,
                            result,
                            selected,
                            &fuel_info,
                        )?;
                        return Ok(());
                    }
                }
                self.stack.push_provider(selected)?;
                return Ok(());
            }
        };
        let mut true_val = self.stack.provider2reg(&true_val)?;
        let mut false_val = self.stack.provider2reg(&false_val)?;
        let result = self.stack.push_dynamic()?;
        match self
            .instr_encoder
            .try_fuse_select(&mut self.stack, result, condition)
        {
            Some((_, swap_operands)) => {
                if swap_operands {
                    mem::swap(&mut true_val, &mut false_val);
                }
            }
            None => {
                let select_instr = Instruction::select_i32_eq_imm16(result, condition, 0_i16);
                self.push_fueled_instr(select_instr, FuelCostsProvider::base)?;
                mem::swap(&mut true_val, &mut false_val);
            }
        };
        self.append_instr(Instruction::register2_ext(true_val, false_val))?;
        Ok(())
    }

    /// Translates a Wasm `reinterpret` instruction.
    fn translate_reinterpret(&mut self, ty: ValType) -> Result<(), Error> {
        bail_unreachable!(self);
        if let TypedProvider::Register(_) = self.stack.peek() {
            // Nothing to do.
            //
            // We try to not manipulate the emulation stack if not needed.
            return Ok(());
        }
        // Case: At this point we know that the top-most stack item is a constant value.
        //       We pop it, change its type and push it back onto the stack.
        let TypedProvider::Const(value) = self.stack.pop() else {
            panic!("the top-most stack item was asserted to be a constant value but a register was found")
        };
        self.stack.push_const(value.reinterpret(ty));
        Ok(())
    }

    /// Translates a Wasm `i64.extend_i32_u` instruction.
    fn translate_i64_extend_i32_u(&mut self) -> Result<(), Error> {
        bail_unreachable!(self);
        if let TypedProvider::Register(_) = self.stack.peek() {
            // Nothing to do.
            //
            // We try to not manipulate the emulation stack if not needed.
            return Ok(());
        }
        // Case: At this point we know that the top-most stack item is a constant value.
        //       We pop it, change its type and push it back onto the stack.
        let TypedProvider::Const(value) = self.stack.pop() else {
            panic!("the top-most stack item was asserted to be a constant value but a register was found")
        };
        debug_assert_eq!(value.ty(), ValType::I32);
        self.stack.push_const(u64::from(u32::from(value)));
        Ok(())
    }

    /// Translates an unconditional `return` instruction.
    fn translate_return(&mut self) -> Result<(), Error> {
        let fuel_info = self.fuel_info();
        self.translate_return_impl(&fuel_info)
    }

    /// Translates an unconditional `return` instruction given fuel information.
    fn translate_return_with(&mut self, frame: &impl ControlFrameBase) -> Result<(), Error> {
        let fuel_info = self.fuel_info_for(frame);
        self.translate_return_impl(&fuel_info)
    }

    /// Translates an unconditional `return` instruction given fuel information.
    fn translate_return_impl(&mut self, fuel_info: &FuelInfo) -> Result<(), Error> {
        let func_type = self.func_type();
        let results = func_type.results();
        let values = &mut self.providers;
        self.stack.pop_n(results.len(), values);
        self.instr_encoder
            .encode_return(&mut self.stack, values, fuel_info)?;
        self.reachable = false;
        Ok(())
    }

    /// Create either [`Instruction::CallIndirectParams`] or [`Instruction::CallIndirectParamsImm16`] depending on the inputs.
    fn call_indirect_params(
        &mut self,
        index: Provider<TypedVal>,
        table_index: u32,
    ) -> Result<Instruction, Error> {
        let table_type = *self.module.get_type_of_table(TableIdx::from(table_index));
        let index = self.as_index_type_const16(index, table_type.index_ty())?;
        let instr = match index {
            Provider::Register(index) => Instruction::call_indirect_params(index, table_index),
            Provider::Const(index) => Instruction::call_indirect_params_imm16(index, table_index),
        };
        Ok(instr)
    }

    /// Translates a Wasm `br` instruction with its `relative_depth`.
    fn translate_br(&mut self, relative_depth: u32) -> Result<(), Error> {
        let engine = self.engine().clone();
        match self.control_stack.acquire_target(relative_depth) {
            AcquiredTarget::Return(_frame) => self.translate_return(),
            AcquiredTarget::Branch(frame) => {
                frame.branch_to();
                let branch_dst = frame.branch_destination();
                let branch_params = frame.branch_params(&engine);
                let consume_fuel_instr = frame.consume_fuel_instr();
                self.translate_copy_branch_params_impl(branch_params, consume_fuel_instr)?;
                let branch_offset = self.instr_encoder.try_resolve_label(branch_dst)?;
                self.push_base_instr(Instruction::branch(branch_offset))?;
                self.reachable = false;
                Ok(())
            }
        }
    }

    /// Populate the `buffer` with the `table` targets including the `table` default target.
    ///
    /// Returns a shared slice to the `buffer` after it has been filled.
    ///
    /// # Note
    ///
    /// The `table` default target is pushed last to the `buffer`.
    fn populate_br_table_buffer<'a>(
        buffer: &'a mut Vec<u32>,
        table: &wasmparser::BrTable,
    ) -> Result<&'a [u32], Error> {
        let default_target = table.default();
        buffer.clear();
        for target in table.targets() {
            buffer.push(target?);
        }
        buffer.push(default_target);
        Ok(buffer)
    }

    /// Convenience method to allow inspecting the provider buffer while manipulating `self` circumventing the borrow checker.
    fn apply_providers_buffer<R>(&mut self, f: impl FnOnce(&mut Self, &[TypedProvider]) -> R) -> R {
        let values = mem::take(&mut self.providers);
        let result = f(self, &values[..]);
        let _ = mem::replace(&mut self.providers, values);
        result
    }

    /// Translates a Wasm `br_table` instruction with its branching targets.
    fn translate_br_table(&mut self, table: wasmparser::BrTable) -> Result<(), Error> {
        let engine = self.engine().clone();
        let index = self.stack.pop();
        let default_target = table.default();
        if table.is_empty() {
            // Case: the `br_table` only has a single target `t` which is equal to a `br t`.
            return self.translate_br(default_target);
        }
        let index: Reg = match index {
            TypedProvider::Register(index) => index,
            TypedProvider::Const(index) => {
                // Case: the `br_table` index is a constant value, therefore always taking the same branch.
                let chosen_index = u32::from(index) as usize;
                let chosen_target = table
                    .targets()
                    .nth(chosen_index)
                    .transpose()?
                    .unwrap_or(default_target);
                return self.translate_br(chosen_target);
            }
        };
        let targets = &mut self.br_table_targets;
        Self::populate_br_table_buffer(targets, &table)?;
        if targets.iter().all(|&target| target == default_target) {
            // Case: all targets are the same and thus the `br_table` is equal to a `br`.
            return self.translate_br(default_target);
        }
        // Note: The Wasm spec mandates that all `br_table` targets manipulate the
        //       Wasm value stack the same. This implies for Wasmi that all `br_table`
        //       targets have the same branch parameter arity.
        let branch_params = self
            .control_stack
            .acquire_target(default_target)
            .control_frame()
            .branch_params(&engine);
        match branch_params.len() {
            0 => self.translate_br_table_0(index),
            1 => self.translate_br_table_1(index),
            2 => self.translate_br_table_2(index),
            3 => self.translate_br_table_3(index),
            n => self.translate_br_table_n(index, n),
        }
    }

    /// Translates the branching targets of a Wasm `br_table` instruction for simple cases without value copying.
    fn translate_br_table_targets_simple(&mut self, values: &[TypedProvider]) -> Result<(), Error> {
        self.translate_br_table_targets(values, |_, _| unreachable!())
    }

    /// Translates the branching targets of a Wasm `br_table` instruction.
    ///
    /// The `make_target` closure allows to define the branch table target instruction being used
    /// for each branch that copies 4 or more values to the destination.
    fn translate_br_table_targets(
        &mut self,
        values: &[TypedProvider],
        make_target: impl Fn(BoundedRegSpan, BranchOffset) -> Instruction,
    ) -> Result<(), Error> {
        let engine = self.engine().clone();
        let fuel_info = self.fuel_info();
        let targets = &self.br_table_targets;
        for &target in targets {
            match self.control_stack.acquire_target(target) {
                AcquiredTarget::Return(_) => {
                    self.instr_encoder
                        .encode_return(&mut self.stack, values, &fuel_info)?;
                }
                AcquiredTarget::Branch(frame) => {
                    frame.branch_to();
                    let branch_params = frame.branch_params(&engine);
                    let branch_dst = frame.branch_destination();
                    let branch_offset = self.instr_encoder.try_resolve_label(branch_dst)?;
                    let instr = match branch_params.len() {
                        0 => Instruction::branch(branch_offset),
                        1..=3 => {
                            Instruction::branch_table_target(branch_params.span(), branch_offset)
                        }
                        _ => make_target(branch_params, branch_offset),
                    };
                    self.instr_encoder.append_instr(instr)?;
                }
            }
        }
        Ok(())
    }

    /// Translates a Wasm `br_table` instruction without inputs.
    fn translate_br_table_0(&mut self, index: Reg) -> Result<(), Error> {
        let targets = &self.br_table_targets;
        let len_targets = targets.len() as u32;
        self.instr_encoder.push_fueled_instr(
            Instruction::branch_table_0(index, len_targets),
            &self.fuel_info(),
            FuelCostsProvider::base,
        )?;
        self.translate_br_table_targets_simple(&[])?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `br_table` instruction with a single input.
    fn translate_br_table_1(&mut self, index: Reg) -> Result<(), Error> {
        let targets = &self.br_table_targets;
        let len_targets = targets.len() as u32;
        let fuel_info = self.fuel_info();
        self.instr_encoder.push_fueled_instr(
            Instruction::branch_table_1(index, len_targets),
            &fuel_info,
            FuelCostsProvider::base,
        )?;
        let stack = &mut self.stack;
        let value = stack.pop();
        let param_instr = match value {
            TypedProvider::Register(register) => Instruction::register(register),
            TypedProvider::Const(immediate) => match immediate.ty() {
                ValType::I32 | ValType::F32 => Instruction::const32(u32::from(immediate.untyped())),
                ValType::I64 => match <Const32<i64>>::try_from(i64::from(immediate)) {
                    Ok(value) => Instruction::i64const32(value),
                    Err(_) => {
                        let register = self.stack.provider2reg(&value)?;
                        Instruction::register(register)
                    }
                },
                ValType::F64 => match <Const32<f64>>::try_from(f64::from(immediate)) {
                    Ok(value) => Instruction::f64const32(value),
                    Err(_) => {
                        let register = self.stack.provider2reg(&value)?;
                        Instruction::register(register)
                    }
                },
                ValType::V128 | ValType::ExternRef | ValType::FuncRef => {
                    let register = self.stack.provider2reg(&value)?;
                    Instruction::register(register)
                }
            },
        };
        self.append_instr(param_instr)?;
        self.translate_br_table_targets_simple(&[value])?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `br_table` instruction with exactly two inputs.
    fn translate_br_table_2(&mut self, index: Reg) -> Result<(), Error> {
        let targets = &self.br_table_targets;
        let len_targets = targets.len() as u32;
        let fuel_info = self.fuel_info();
        self.instr_encoder.push_fueled_instr(
            Instruction::branch_table_2(index, len_targets),
            &fuel_info,
            FuelCostsProvider::base,
        )?;
        let stack = &mut self.stack;
        let (v0, v1) = stack.pop2();
        self.instr_encoder.append_instr(Instruction::register2_ext(
            stack.provider2reg(&v0)?,
            stack.provider2reg(&v1)?,
        ))?;
        self.translate_br_table_targets_simple(&[v0, v1])?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `br_table` instruction with exactly three inputs.
    fn translate_br_table_3(&mut self, index: Reg) -> Result<(), Error> {
        let targets = &self.br_table_targets;
        let len_targets = targets.len() as u32;
        let fuel_info = self.fuel_info();
        self.instr_encoder.push_fueled_instr(
            Instruction::branch_table_3(index, len_targets),
            &fuel_info,
            FuelCostsProvider::base,
        )?;
        let stack = &mut self.stack;
        let (v0, v1, v2) = stack.pop3();
        self.instr_encoder.append_instr(Instruction::register3_ext(
            stack.provider2reg(&v0)?,
            stack.provider2reg(&v1)?,
            stack.provider2reg(&v2)?,
        ))?;
        self.translate_br_table_targets_simple(&[v0, v1, v2])?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `br_table` instruction with 4 or more inputs.
    fn translate_br_table_n(&mut self, index: Reg, len_values: u16) -> Result<(), Error> {
        debug_assert!(len_values > 3);
        let values = &mut self.providers;
        self.stack.pop_n(usize::from(len_values), values);
        match BoundedRegSpan::from_providers(values) {
            Some(span) => self.translate_br_table_span(index, span),
            None => self.translate_br_table_many(index),
        }
    }

    /// Translates a Wasm `br_table` instruction with 4 or more inputs that form a [`RegSpan`].
    fn translate_br_table_span(&mut self, index: Reg, values: BoundedRegSpan) -> Result<(), Error> {
        debug_assert!(values.len() > 3);
        let fuel_info = self.fuel_info();
        let targets = &mut self.br_table_targets;
        let len_targets = targets.len() as u32;
        self.instr_encoder.push_fueled_instr(
            Instruction::branch_table_span(index, len_targets),
            &fuel_info,
            FuelCostsProvider::base,
        )?;
        self.instr_encoder
            .append_instr(Instruction::register_span(values))?;
        self.apply_providers_buffer(|this, buffer| {
            this.translate_br_table_targets(buffer, |branch_params, branch_offset| {
                debug_assert_eq!(values.len(), branch_params.len());
                let len = values.len();
                let results = branch_params.span();
                let values = values.span();
                let make_instr =
                    match InstrEncoder::has_overlapping_copy_spans(results, values, len) {
                        true => Instruction::branch_table_target,
                        false => Instruction::branch_table_target_non_overlapping,
                    };
                make_instr(branch_params.span(), branch_offset)
            })
        })?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `br_table` instruction with 4 or more inputs that cannot form a [`RegSpan`].
    fn translate_br_table_many(&mut self, index: Reg) -> Result<(), Error> {
        let targets = &mut self.br_table_targets;
        let len_targets = targets.len() as u32;
        let fuel_info = self.fuel_info();
        self.instr_encoder.push_fueled_instr(
            Instruction::branch_table_many(index, len_targets),
            &fuel_info,
            FuelCostsProvider::base,
        )?;
        let stack = &mut self.stack;
        let values = &self.providers[..];
        debug_assert!(values.len() > 3);
        self.instr_encoder.encode_register_list(stack, values)?;
        self.apply_providers_buffer(|this, values| {
            this.translate_br_table_targets(&[], |branch_params, branch_offset| {
                let make_instr = match InstrEncoder::has_overlapping_copies(branch_params, values) {
                    true => Instruction::branch_table_target,
                    false => Instruction::branch_table_target_non_overlapping,
                };
                make_instr(branch_params.span(), branch_offset)
            })
        })?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `i64.binop128` instruction from the `wide-arithmetic` proposal.
    fn translate_i64_binop128(
        &mut self,
        make_instr: fn(results: [Reg; 2], lhs_lo: Reg) -> Instruction,
        const_eval: fn(lhs_lo: i64, lhs_hi: i64, rhs_lo: i64, rhs_hi: i64) -> (i64, i64),
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (rhs_lo, rhs_hi) = self.stack.pop2();
        let (lhs_lo, lhs_hi) = self.stack.pop2();
        if let (
            Provider::Const(lhs_lo),
            Provider::Const(lhs_hi),
            Provider::Const(rhs_lo),
            Provider::Const(rhs_hi),
        ) = (lhs_lo, lhs_hi, rhs_lo, rhs_hi)
        {
            let (result_lo, result_hi) =
                const_eval(lhs_lo.into(), lhs_hi.into(), rhs_lo.into(), rhs_hi.into());
            self.stack.push_const(result_lo);
            self.stack.push_const(result_hi);
            return Ok(());
        }
        let rhs_lo = match rhs_lo {
            Provider::Register(reg) => reg,
            Provider::Const(rhs_lo) => self.stack.alloc_const(rhs_lo)?,
        };
        let rhs_hi = match rhs_hi {
            Provider::Register(reg) => reg,
            Provider::Const(rhs_hi) => self.stack.alloc_const(rhs_hi)?,
        };
        let lhs_lo = match lhs_lo {
            Provider::Register(reg) => reg,
            Provider::Const(lhs_lo) => self.stack.alloc_const(lhs_lo)?,
        };
        let lhs_hi = match lhs_hi {
            Provider::Register(reg) => reg,
            Provider::Const(lhs_hi) => self.stack.alloc_const(lhs_hi)?,
        };
        let result_lo = self.stack.push_dynamic()?;
        let result_hi = self.stack.push_dynamic()?;
        self.push_fueled_instr(
            make_instr([result_lo, result_hi], lhs_lo),
            FuelCostsProvider::base,
        )?;
        self.instr_encoder
            .append_instr(Instruction::register3_ext(lhs_hi, rhs_lo, rhs_hi))?;
        Ok(())
    }

    /// Translates a Wasm `i64.mul_wide_sx` instruction from the `wide-arithmetic` proposal.
    fn translate_i64_mul_wide_sx(
        &mut self,
        make_instr: fn(results: FixedRegSpan<2>, lhs: Reg, rhs: Reg) -> Instruction,
        const_eval: fn(lhs: i64, rhs: i64) -> (i64, i64),
        signed: bool,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (lhs, rhs) = self.stack.pop2();
        let (lhs, rhs) = match (lhs, rhs) {
            (Provider::Register(lhs), Provider::Register(rhs)) => (lhs, rhs),
            (Provider::Register(lhs), Provider::Const(rhs)) => {
                if self.try_opt_i64_mul_wide_sx(lhs, rhs, signed)? {
                    return Ok(());
                }
                let rhs = self.stack.alloc_const(rhs)?;
                (lhs, rhs)
            }
            (Provider::Const(lhs), Provider::Register(rhs)) => {
                if self.try_opt_i64_mul_wide_sx(rhs, lhs, signed)? {
                    return Ok(());
                }
                let lhs = self.stack.alloc_const(lhs)?;
                (lhs, rhs)
            }
            (Provider::Const(lhs), Provider::Const(rhs)) => {
                let (result_lo, result_hi) = const_eval(lhs.into(), rhs.into());
                self.stack.push_const(result_lo);
                self.stack.push_const(result_hi);
                return Ok(());
            }
        };
        let results = self.stack.push_dynamic_n(2)?;
        let results = <FixedRegSpan<2>>::new(results).unwrap_or_else(|_| {
            panic!("`i64.mul_wide_sx` requires 2 results but found: {results:?}")
        });
        self.push_fueled_instr(make_instr(results, lhs, rhs), FuelCostsProvider::base)?;
        Ok(())
    }

    /// Try to optimize a `i64.mul_wide_sx` instruction with one [`Reg`] and one immediate input.
    ///
    /// - Returns `Ok(true)` if the optimiation was applied successfully.
    /// - Returns `Ok(false)` if no optimization was applied.
    fn try_opt_i64_mul_wide_sx(
        &mut self,
        reg_in: Reg,
        imm_in: TypedVal,
        signed: bool,
    ) -> Result<bool, Error> {
        let imm_in = i64::from(imm_in);
        if imm_in == 0 {
            // Case: `mul(x, 0)` or `mul(0, x)` always evaluates to 0.
            self.stack.push_const(0_i64); // lo-bits
            self.stack.push_const(0_i64); // hi-bits
            return Ok(true);
        }
        if imm_in == 1 && !signed {
            // Case: `mul(x, 1)` or `mul(1, x)` always evaluates to just `x`.
            // This is only valid if `x` is not a singed (negative) value.
            self.stack.push_register(reg_in)?; // lo-bits
            self.stack.push_const(0_i64); // hi-bits
            return Ok(true);
        }
        Ok(false)
    }
}
