//! Function translation for the register-machine bytecode based `wasmi` engine.

mod control_frame;
mod control_stack;
mod instr_encoder;
mod stack;
mod typed_value;
mod utils;
mod visit;

use self::{
    control_frame::{
        BlockControlFrame,
        BlockHeight,
        IfControlFrame,
        LoopControlFrame,
        UnreachableControlFrame,
    },
    stack::ValueStack,
    typed_value::TypedValue,
    utils::{WasmFloat, WasmInteger},
};
pub use self::{
    control_frame::{ControlFrame, ControlFrameKind},
    control_stack::ControlStack,
    instr_encoder::InstrEncoder,
    stack::{DefragRegister, ProviderStack, RegisterAlloc, TypedProvider},
};
use crate::{
    engine::{
        bytecode::SignatureIdx,
        bytecode2::{
            Const16,
            Const32,
            Instruction,
            Register,
            RegisterSlice,
            RegisterSliceIter,
            Sign,
        },
        config::FuelCosts,
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
use wasmparser::{MemArg, VisitOperator};

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
}

impl FuncTranslatorAllocations {
    /// Resets the [`FuncTranslatorAllocations`].
    fn reset(&mut self) {
        self.stack.reset();
        self.instr_encoder.reset();
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
        // Note: we use a dummy `RegisterSlice` as placeholder.
        //
        // We can do this since the branch parameters of the function enclosing block
        // are never used due to optimizations to directly return to the caller instead.
        let branch_params = RegisterSlice::new(Register::from_u16(0));
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
        self.alloc.stack.defrag(&mut self.alloc.instr_encoder);
        self.alloc.instr_encoder.update_branch_offsets()?;
        let len_registers = self.alloc.stack.len_registers();
        let instrs = self.alloc.instr_encoder.drain_instrs();
        self.res
            .engine()
            .init_func_2(self.compiled_func, len_registers, instrs);
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
        mut branch_params: RegisterSliceIter,
    ) -> Result<(), TranslationError> {
        if branch_params.len() == 0 {
            // If the block does not have branch parameters there is no need to copy anything.
            return Ok(());
        }
        self.alloc
            .stack
            .pop_n(branch_params.len(), &mut self.alloc.buffer);
        let engine = self.res.engine();
        for provider in self.alloc.buffer.iter().copied() {
            let result = self.alloc.stack.push_dynamic()?;
            debug_assert_eq!(branch_params.next(), Some(result));
            self.alloc
                .instr_encoder
                .encode_copy(engine, result, provider)?;
        }
        Ok(())
    }

    /// Translates the `end` of a Wasm `block` control frame.
    fn translate_end_block(&mut self, frame: BlockControlFrame) -> Result<(), TranslationError> {
        if self.alloc.control_stack.is_empty() {
            // We dropped the Wasm `block` that encloses the function itself so we can return.
            return self.visit_return();
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
        if self.reachable || frame.is_branched_to() {
            // We reset reachability in case the end of the `block` was reachable.
            self.reachable = true;
        }
        Ok(())
    }

    /// Translates the `end` of a Wasm `loop` control frame.
    fn translate_end_loop(&mut self, _frame: LoopControlFrame) -> Result<(), TranslationError> {
        todo!()
    }

    /// Translates the `end` of a Wasm `if` control frame.
    fn translate_end_if(&mut self, _frame: IfControlFrame) -> Result<(), TranslationError> {
        todo!()
    }

    /// Translates the `end` of an unreachable control frame.
    fn translate_end_unreachable(
        &mut self,
        _frame: UnreachableControlFrame,
    ) -> Result<(), TranslationError> {
        todo!()
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
    ///      going to be returned via [`RegisterSlice`].
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
    ) -> Result<RegisterSlice, TranslationError> {
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
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
    ) -> Result<bool, TranslationError>
    where
        T: Copy + TryInto<Const16>,
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
        make_instr_imm16: fn(result: Register, lhs: Const16, rhs: Register) -> Instruction,
    ) -> Result<bool, TranslationError>
    where
        T: Copy + TryInto<Const16>,
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
        reg_in: Register,
        imm_in: T,
        make_instr_imm: fn(result: Register, reg_in: Register) -> Instruction,
        make_instr_imm_param: fn(&mut Self, imm_in: T) -> Result<Instruction, TranslationError>,
    ) -> Result<(), TranslationError> {
        let result = self.alloc.stack.push_dynamic()?;
        self.alloc
            .instr_encoder
            .push_instr(make_instr_imm(result, reg_in))?;
        let rhs_instr = make_instr_imm_param(self, imm_in)?;
        self.alloc.instr_encoder.push_instr(rhs_instr)?;
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
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_rev: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_param: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16, rhs: Register) -> Instruction,
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
        T: Copy + From<TypedValue> + Into<TypedValue> + TryInto<Const16>,
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
                self.push_binary_instr_imm(lhs, T::from(rhs), make_instr_imm, make_instr_imm_param)
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
                self.push_binary_instr_imm(
                    rhs,
                    T::from(lhs),
                    make_instr_imm_rev,
                    make_instr_imm_param,
                )
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
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_rev: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_param: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
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
                self.push_binary_instr_imm(lhs, T::from(rhs), make_instr_imm, make_instr_imm_param)
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
                self.push_binary_instr_imm(
                    rhs,
                    T::from(lhs),
                    make_instr_imm_rev,
                    make_instr_imm_param,
                )
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
        make_instr_imm_rev: fn(result: Register, rhs: Register) -> Instruction,
        make_instr_imm_param: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
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
            (TypedProvider::Const(lhs), TypedProvider::Register(rhs)) => self
                .push_binary_instr_imm(rhs, T::from(lhs), make_instr_imm_rev, make_instr_imm_param),
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
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_param: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
        make_instr_imm_opt: fn(&mut Self, lhs: Register, rhs: T) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError>
    where
        T: Copy + From<TypedValue> + Into<TypedValue> + TryInto<Const16>,
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
                self.push_binary_instr_imm(
                    reg_in,
                    T::from(imm_in),
                    make_instr_imm,
                    make_instr_imm_param,
                )
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
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_param: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
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
                self.push_binary_instr_imm(
                    reg_in,
                    T::from(imm_in),
                    make_instr_imm,
                    make_instr_imm_param,
                )
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
        make_instr_imm: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
        make_instr_imm_param: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
        make_instr_imm_rev: fn(result: Register, rhs: Register) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16, rhs: Register) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_imm_reg_opt: fn(
            &mut Self,
            lhs: T,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError>
    where
        T: WasmInteger,
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
                    Const16::from_i16(rhs),
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
                self.push_binary_instr_imm(
                    rhs,
                    T::from(lhs),
                    make_instr_imm_rev,
                    make_instr_imm_param,
                )
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
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_rev: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_param: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16, rhs: Register) -> Instruction,
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
                self.push_binary_instr_imm(lhs, T::from(rhs), make_instr_imm, make_instr_imm_param)
            }
            (TypedProvider::Const(lhs), TypedProvider::Register(rhs)) => {
                if self.try_push_binary_instr_imm16_rev(T::from(lhs), rhs, make_instr_imm16_rev)? {
                    // Optimization was applied: return early.
                    return Ok(());
                }
                self.push_binary_instr_imm(
                    rhs,
                    T::from(lhs),
                    make_instr_imm_rev,
                    make_instr_imm_param,
                )
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

    /// Can be used for [`Self::translate_binary`] (and variants) to create 32-bit immediate instructions.
    pub fn make_instr_imm_param_32<T>(&mut self, value: T) -> Result<Instruction, TranslationError>
    where
        T: Into<Const32>,
    {
        Ok(Instruction::const32(value))
    }

    /// Can be used for [`Self::translate_binary`] (and variants) to create 64-bit immediate instructions.
    pub fn make_instr_imm_param_64<T>(&mut self, value: T) -> Result<Instruction, TranslationError>
    where
        T: Into<UntypedValue>,
    {
        let cref = self.engine().alloc_const(value.into())?;
        Ok(Instruction::ConstRef(cref))
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
        make_instr_offset16: fn(result: Register, ptr: Register, offset: Const16) -> Instruction,
        make_instr_at: fn(result: Register, address: Const32) -> Instruction,
    ) -> Result<(), TranslationError> {
        bail_unreachable!(self);
        let offset = Self::memarg_offset(memarg);
        match self.alloc.stack.pop() {
            TypedProvider::Register(ptr) => {
                if let Ok(offset) = Const16::try_from(offset) {
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
                    .push_instr(Instruction::const32(offset))?;
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

    /// Translates a Wasm `store` instruction to `wasmi` bytecode.
    ///
    /// # Note
    ///
    /// This chooses the right encoding for the given `store` instruction.
    /// If `ptr+offset` is a constant value the address is pre-calculated.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to `wasmi` bytecode:
    ///
    /// - `{i32, i64, f32, f64}.store`
    /// - `i64.store32`
    ///
    /// Not used for these Wasm `store` instructions:
    ///
    /// - `{i32, i64}.{store8, store16}`
    fn translate_store<T>(
        &mut self,
        memarg: MemArg,
        make_instr: fn(ptr: Register, offset: Const32) -> Instruction,
        make_instr_imm: fn(ptr: Register, offset: Const32) -> Instruction,
        make_instr_imm_param: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
        make_instr_at: fn(address: Const32, value: Register) -> Instruction,
        make_instr_imm_at: fn(address: Const32) -> Instruction,
    ) -> Result<(), TranslationError>
    where
        T: Copy + From<TypedValue>,
    {
        bail_unreachable!(self);
        let offset = Self::memarg_offset(memarg);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(ptr), TypedProvider::Register(value)) => {
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr(ptr, Const32::from(offset)))?;
                self.alloc
                    .instr_encoder
                    .push_instr(Instruction::Register(value))?;
                Ok(())
            }
            (TypedProvider::Register(ptr), TypedProvider::Const(value)) => {
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr_imm(ptr, Const32::from(offset)))?;
                let param = make_instr_imm_param(self, T::from(value))?;
                self.alloc.instr_encoder.push_instr(param)?;
                Ok(())
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
                    this.alloc
                        .instr_encoder
                        .push_instr(make_instr_imm_at(Const32::from(address)))?;
                    let param = make_instr_imm_param(this, T::from(value))?;
                    this.alloc.instr_encoder.push_instr(param)?;
                    Ok(())
                })
            }
        }
    }

    /// Translates a Wasm `storeN` instruction to `wasmi` bytecode.
    ///
    /// # Note
    ///
    /// This chooses the right encoding for the given `store` instruction.
    /// If `ptr+offset` is a constant value the address is pre-calculated.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to `wasmi` bytecode:
    ///
    /// - `{i32, i64}.{store8, store16}`
    fn translate_store_trunc<T>(
        &mut self,
        memarg: MemArg,
        make_instr: fn(ptr: Register, offset: Const32) -> Instruction,
        make_instr_imm: fn(ptr: Register, offset: Const32) -> Instruction,
        make_instr_imm_param: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
        make_instr_at: fn(address: Const32, value: Register) -> Instruction,
        make_instr_imm_at: fn(address: Const32, value: T) -> Instruction,
    ) -> Result<(), TranslationError>
    where
        T: Copy + From<TypedValue>,
    {
        bail_unreachable!(self);
        let offset = Self::memarg_offset(memarg);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(ptr), TypedProvider::Register(value)) => {
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr(ptr, Const32::from(offset)))?;
                self.alloc
                    .instr_encoder
                    .push_instr(Instruction::Register(value))?;
                Ok(())
            }
            (TypedProvider::Register(ptr), TypedProvider::Const(value)) => {
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr_imm(ptr, Const32::from(offset)))?;
                let param = make_instr_imm_param(self, T::from(value))?;
                self.alloc.instr_encoder.push_instr(param)?;
                Ok(())
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
                    this.alloc
                        .instr_encoder
                        .push_instr(make_instr_imm_at(Const32::from(address), T::from(value)))?;
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
                        .push_instr(Instruction::Register(rhs))?;
                    Ok(())
                }
                (TypedProvider::Register(lhs), TypedProvider::Const(rhs)) => {
                    fn push_select_imm32_rhs(
                        this: &mut FuncTranslator<'_>,
                        result: Register,
                        condition: Register,
                        lhs: Register,
                        rhs: impl Into<Const32>,
                    ) -> Result<(), TranslationError> {
                        this.alloc
                            .instr_encoder
                            .push_instr(Instruction::select_imm32_rhs(result, condition, lhs))?;
                        this.alloc
                            .instr_encoder
                            .push_instr(Instruction::const32(rhs))?;
                        Ok(())
                    }

                    if let Some(type_hint) = type_hint {
                        debug_assert_eq!(rhs.ty(), type_hint);
                    }
                    let result = self.alloc.stack.push_dynamic()?;
                    match rhs.ty() {
                        ValueType::I32 => {
                            push_select_imm32_rhs(self, result, condition, lhs, i32::from(rhs))
                        }
                        ValueType::F32 => {
                            push_select_imm32_rhs(self, result, condition, lhs, f32::from(rhs))
                        }
                        ValueType::I64
                        | ValueType::F64
                        | ValueType::FuncRef
                        | ValueType::ExternRef => {
                            let rhs_cref = self.engine().alloc_const(rhs)?;
                            self.alloc
                                .instr_encoder
                                .push_instr(Instruction::select_imm_rhs(result, condition, lhs))?;
                            self.alloc
                                .instr_encoder
                                .push_instr(Instruction::const_ref(rhs_cref))?;
                            Ok(())
                        }
                    }
                }
                (TypedProvider::Const(lhs), TypedProvider::Register(rhs)) => {
                    fn push_select_imm32_lhs(
                        this: &mut FuncTranslator<'_>,
                        result: Register,
                        condition: Register,
                        lhs: impl Into<Const32>,
                        rhs: Register,
                    ) -> Result<(), TranslationError> {
                        this.alloc
                            .instr_encoder
                            .push_instr(Instruction::select_imm32_lhs(result, condition, rhs))?;
                        this.alloc
                            .instr_encoder
                            .push_instr(Instruction::const32(lhs))?;
                        Ok(())
                    }

                    if let Some(type_hint) = type_hint {
                        debug_assert_eq!(lhs.ty(), type_hint);
                    }
                    let result = self.alloc.stack.push_dynamic()?;
                    match lhs.ty() {
                        ValueType::I32 => {
                            push_select_imm32_lhs(self, result, condition, i32::from(lhs), rhs)
                        }
                        ValueType::F32 => {
                            push_select_imm32_lhs(self, result, condition, f32::from(lhs), rhs)
                        }
                        ValueType::I64
                        | ValueType::F64
                        | ValueType::FuncRef
                        | ValueType::ExternRef => {
                            let lhs_cref = self.engine().alloc_const(lhs)?;
                            self.alloc
                                .instr_encoder
                                .push_instr(Instruction::select_imm_lhs(result, condition, rhs))?;
                            self.alloc
                                .instr_encoder
                                .push_instr(Instruction::const_ref(lhs_cref))?;
                            Ok(())
                        }
                    }
                }
                (TypedProvider::Const(lhs), TypedProvider::Const(rhs)) => {
                    fn push_select_imm32(
                        this: &mut FuncTranslator<'_>,
                        reg: Register,
                        value: impl Into<Const32>,
                    ) -> Result<(), TranslationError> {
                        this.alloc
                            .instr_encoder
                            .push_instr(Instruction::select_imm32(reg, value))?;
                        Ok(())
                    }

                    debug_assert_eq!(lhs.ty(), rhs.ty());
                    if let Some(type_hint) = type_hint {
                        debug_assert_eq!(lhs.ty(), type_hint);
                    }
                    if lhs == rhs {
                        // # Optimization
                        //
                        // Both `lhs` and `rhs` are equal registers
                        // and thus will always yield the same value.
                        self.alloc.stack.push_const(lhs);
                        return Ok(());
                    }
                    let result = self.alloc.stack.push_dynamic()?;
                    match lhs.ty() {
                        ValueType::I32 => {
                            push_select_imm32(self, result, i32::from(lhs))?;
                            push_select_imm32(self, condition, i32::from(rhs))?;
                            Ok(())
                        }
                        ValueType::F32 => {
                            push_select_imm32(self, result, f32::from(lhs))?;
                            push_select_imm32(self, condition, f32::from(rhs))?;
                            Ok(())
                        }
                        ValueType::I64
                        | ValueType::F64
                        | ValueType::FuncRef
                        | ValueType::ExternRef => {
                            let lhs_cref = self.engine().alloc_const(lhs)?;
                            let rhs_cref = self.engine().alloc_const(rhs)?;
                            self.alloc
                                .instr_encoder
                                .push_instr(Instruction::select_imm(result, lhs_cref))?;
                            self.alloc
                                .instr_encoder
                                .push_instr(Instruction::select_imm(condition, rhs_cref))?;
                            Ok(())
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
        let instr = match self.func_type().results() {
            [] => {
                // Case: Function returns nothing therefore all return statements must return nothing.
                Instruction::Return
            }
            [ValueType::I32] => match self.alloc.stack.pop() {
                // Case: Function returns a single `i32` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_reg(value),
                TypedProvider::Const(value) => Instruction::return_imm32(i32::from(value)),
            },
            [ValueType::I64] => match self.alloc.stack.pop() {
                // Case: Function returns a single `i64` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_reg(value),
                TypedProvider::Const(value) => {
                    if let Ok(value) = i32::try_from(i64::from(value)) {
                        Instruction::return_i64imm32(value)
                    } else {
                        Instruction::return_imm(self.engine().alloc_const(value)?)
                    }
                }
            },
            [ValueType::F32] => match self.alloc.stack.pop() {
                // Case: Function returns a single `f32` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_reg(value),
                TypedProvider::Const(value) => Instruction::return_imm32(F32::from(value)),
            },
            [ValueType::F64 | ValueType::FuncRef | ValueType::ExternRef] => {
                match self.alloc.stack.pop() {
                    // Case: Function returns a single `f64` value which allows for special operator.
                    TypedProvider::Register(value) => Instruction::return_reg(value),
                    TypedProvider::Const(value) => {
                        Instruction::return_imm(self.engine().alloc_const(value)?)
                    }
                }
            }
            results => {
                self.alloc
                    .stack
                    .pop_n(results.len(), &mut self.alloc.buffer);
                let providers = self
                    .alloc
                    .buffer
                    .iter()
                    .copied()
                    .map(TypedProvider::into_untyped);
                let sref = self.res.engine().alloc_providers(providers)?;
                Instruction::return_many(sref)
            }
        };
        self.alloc.instr_encoder.push_instr(instr)?;
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
            [ValueType::I32] => match self.alloc.stack.pop() {
                // Case: Function returns a single `i32` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_nez_reg(condition, value),
                TypedProvider::Const(value) => {
                    Instruction::return_nez_imm32(condition, i32::from(value))
                }
            },
            [ValueType::I64] => match self.alloc.stack.pop() {
                // Case: Function returns a single `i64` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_nez_reg(condition, value),
                TypedProvider::Const(value) => {
                    if let Ok(value) = i32::try_from(i64::from(value)) {
                        Instruction::return_nez_i64imm32(condition, value)
                    } else {
                        Instruction::return_nez_imm(condition, self.engine().alloc_const(value)?)
                    }
                }
            },
            [ValueType::F32] => match self.alloc.stack.pop() {
                // Case: Function returns a single `f32` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_nez_reg(condition, value),
                TypedProvider::Const(value) => {
                    Instruction::return_nez_imm32(condition, F32::from(value))
                }
            },
            [ValueType::F64 | ValueType::FuncRef | ValueType::ExternRef] => {
                match self.alloc.stack.pop() {
                    // Case: Function returns a single `f64` value which allows for special operator.
                    TypedProvider::Register(value) => Instruction::return_nez_reg(condition, value),
                    TypedProvider::Const(value) => {
                        Instruction::return_nez_imm(condition, self.engine().alloc_const(value)?)
                    }
                }
            }
            results => {
                self.alloc
                    .stack
                    .pop_n(results.len(), &mut self.alloc.buffer);
                let providers = self
                    .alloc
                    .buffer
                    .iter()
                    .copied()
                    .map(TypedProvider::into_untyped);
                let sref = self.res.engine().alloc_providers(providers)?;
                Instruction::return_nez_many(condition, sref)
            }
        };
        self.alloc.instr_encoder.push_instr(instr)?;
        self.reachable = false;
        Ok(())
    }
}
