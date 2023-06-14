//! Function translation for the register-machine bytecode based `wasmi` engine.

#![allow(unused_imports)] // TODO: remove

mod control_frame;
mod control_stack;
mod instr_encoder;
mod stack;
mod utils;
mod visit;

use self::{control_frame::BlockControlFrame, stack::ValueStack, utils::WasmInteger};
pub use self::{
    control_frame::{ControlFrame, ControlFrameKind},
    control_stack::ControlStack,
    instr_encoder::InstrEncoder,
    stack::{DefragRegister, Provider, ProviderStack, RegisterAlloc},
};
use crate::{
    engine::{
        bytecode::{
            self,
            AddressOffset,
            BranchOffset,
            BranchTableTargets,
            DataSegmentIdx,
            ElementSegmentIdx,
            SignatureIdx,
            TableIdx,
        },
        bytecode2::{Const16, Const32, Instruction, Register},
        config::FuelCosts,
        func_builder::TranslationErrorInner,
        CompiledFunc,
        DropKeep,
        Instr,
        RelativeDepth,
        TranslationError,
    },
    module::{
        BlockType,
        ConstExpr,
        FuncIdx,
        FuncTypeIdx,
        GlobalIdx,
        MemoryIdx,
        ModuleResources,
        DEFAULT_MEMORY_INDEX,
    },
    Engine,
    FuncType,
    GlobalType,
    Mutability,
};
use alloc::vec::Vec;
use wasmi_core::{TrapCode, UntypedValue, ValueType, F32, F64};
use wasmparser::VisitOperator;

/// Reusable allocations of a [`FuncTranslator`].
#[derive(Debug, Default)]
pub struct FuncTranslatorAllocations {
    /// The emulated value stack.
    stack: ValueStack,
    /// The instruction sequence encoder.
    instr_encoder: InstrEncoder,
    /// The control stack.
    control_stack: ControlStack,
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
        let block_frame = BlockControlFrame::new(block_type, end_label, 0, consume_fuel);
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
        lhs: UntypedValue,
        rhs: UntypedValue,
        consteval: fn(UntypedValue, UntypedValue) -> UntypedValue,
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

    /// Translate a non-commutative binary `wasmi` instruction.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for all non-commutative
    ///   binary instructions such as `i32.sub` or `i64.rotl`.
    /// - Its various function arguments allow it to be used generically for `i32`
    ///   instructions as well as for `i64`, `f32` and `f64` types.
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
    /// - `{i32, i64}.sub`
    /// - `{f32, f64}.{sub, div, copysign}`
    #[allow(clippy::too_many_arguments)]
    fn translate_binary<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_rev: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_param: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16, rhs: Register) -> Instruction,
        consteval: fn(UntypedValue, UntypedValue) -> UntypedValue,
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
        T: Copy + From<UntypedValue> + Into<UntypedValue> + TryInto<Const16>,
    {
        bail_unreachable!(self);
        match self.alloc.stack.pop2() {
            (Provider::Register(lhs), Provider::Register(rhs)) => {
                if make_instr_opt(self, lhs, rhs)? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (Provider::Register(lhs), Provider::Const(rhs)) => {
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
            (Provider::Const(lhs), Provider::Register(rhs)) => {
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
            (Provider::Const(lhs), Provider::Const(rhs)) => {
                self.push_binary_consteval(lhs, rhs, consteval)
            }
        }
    }

    /// Convenience method for [`Self::translate_binary`] when translating `i32` instructions.
    #[allow(clippy::too_many_arguments)]
    fn translate_binary_i32(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_rev: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16, rhs: Register) -> Instruction,
        consteval: fn(UntypedValue, UntypedValue) -> UntypedValue,
        make_instr_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
        make_instr_reg_imm_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: i32,
        ) -> Result<bool, TranslationError>,
        make_instr_imm_reg_opt: fn(
            &mut Self,
            lhs: i32,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError> {
        self.translate_binary::<i32>(
            make_instr,
            make_instr_imm,
            make_instr_imm_rev,
            Self::make_instr_imm_param_i32,
            make_instr_imm16,
            make_instr_imm16_rev,
            consteval,
            make_instr_opt,
            make_instr_reg_imm_opt,
            make_instr_imm_reg_opt,
        )
    }

    /// Convenience method for [`Self::translate_binary`] when translating `i64` instructions.
    #[allow(clippy::too_many_arguments)]
    fn translate_binary_i64(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_rev: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16, rhs: Register) -> Instruction,
        consteval: fn(UntypedValue, UntypedValue) -> UntypedValue,
        make_instr_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
        make_instr_reg_imm_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: i64,
        ) -> Result<bool, TranslationError>,
        make_instr_imm_reg_opt: fn(
            &mut Self,
            lhs: i64,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError> {
        self.translate_binary::<i64>(
            make_instr,
            make_instr_imm,
            make_instr_imm_rev,
            Self::make_instr_imm_param_i64,
            make_instr_imm16,
            make_instr_imm16_rev,
            consteval,
            make_instr_opt,
            make_instr_reg_imm_opt,
            make_instr_imm_reg_opt,
        )
    }

    /// Translate a commutative binary `wasmi` instruction.
    ///
    /// # Note
    ///
    /// - This applies several optimization that are valid for all commutative
    ///   binary instructions such as `i32.add` or `i64.mul`.
    /// - Its various function arguments allow it to be used generically for `i32`
    ///   instructions as well as for `i64`, `f32` and `f64` types.
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
    /// - `{i32, i64}.{add, mul, and, or, xor}`
    /// - `{f32, f64}.{add, mul, min, max}`
    #[allow(clippy::too_many_arguments)]
    fn translate_binary_commutative<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_param: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
        consteval: fn(UntypedValue, UntypedValue) -> UntypedValue,
        make_instr_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
        make_instr_imm_opt: fn(&mut Self, lhs: Register, rhs: T) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError>
    where
        T: Copy + From<UntypedValue> + Into<UntypedValue> + TryInto<Const16>,
    {
        bail_unreachable!(self);
        match self.alloc.stack.pop2() {
            (Provider::Register(lhs), Provider::Register(rhs)) => {
                if make_instr_opt(self, lhs, rhs)? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (Provider::Register(reg_in), Provider::Const(imm_in))
            | (Provider::Const(imm_in), Provider::Register(reg_in)) => {
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
            (Provider::Const(lhs), Provider::Const(rhs)) => {
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
        consteval: fn(UntypedValue, UntypedValue) -> UntypedValue,
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
            (Provider::Register(lhs), Provider::Register(rhs)) => {
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (Provider::Register(lhs), Provider::Const(rhs)) => {
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
            (Provider::Const(lhs), Provider::Register(rhs)) => {
                if make_instr_imm_reg_opt(self, T::from(lhs), rhs)? {
                    // Custom optimization was applied: return early
                    return Ok(());
                }
                if T::from(lhs).eq_zero() {
                    // Optimization: Shifting or rotating a zero value is a no-op.
                    self.alloc.stack.push_const(UntypedValue::from(0_i32));
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
            (Provider::Const(lhs), Provider::Const(rhs)) => {
                self.push_binary_consteval(lhs, rhs, consteval)
            }
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

    /// Can be used for [`Self::translate_binary`] (and variants) to create immediate `i32` instructions.
    pub fn make_instr_imm_param_i32(
        &mut self,
        value: i32,
    ) -> Result<Instruction, TranslationError> {
        Ok(Instruction::const32(value))
    }

    /// Can be used for [`Self::translate_binary`] (and variants) to create immediate `i64` instructions.
    pub fn make_instr_imm_param_i64(
        &mut self,
        value: i64,
    ) -> Result<Instruction, TranslationError> {
        let cref = self.engine().alloc_const(UntypedValue::from(value))?;
        Ok(Instruction::ConstRef(cref))
    }
}
