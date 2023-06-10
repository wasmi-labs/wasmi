//! Function translation for the register-machine bytecode based `wasmi` engine.

#![allow(dead_code, unused_imports)] // TODO: remove

mod control_frame;
mod control_stack;
mod instr_encoder;
mod stack;
mod visit;

use self::{control_frame::BlockControlFrame, stack::ValueStack};
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
use wasmi_core::{UntypedValue, ValueType, F32, F64};
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
    #[allow(clippy::too_many_arguments)]
    fn translate_binary<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_rev: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_rhs: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
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
        let rhs = self.alloc.stack.pop();
        let lhs = self.alloc.stack.pop();
        match (lhs, rhs) {
            (Provider::Register(lhs), Provider::Register(rhs)) => {
                if make_instr_opt(self, lhs, rhs)? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                let result = self.alloc.stack.push_dynamic()?;
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr(result, lhs, rhs))?;
                Ok(())
            }
            (Provider::Register(lhs), Provider::Const(rhs)) => {
                if make_instr_reg_imm_opt(self, lhs, T::from(rhs))? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                if let Ok(rhs) = T::from(rhs).try_into() {
                    // Optimization: We can use a compact instruction for small constants.
                    let result = self.alloc.stack.push_dynamic()?;
                    self.alloc
                        .instr_encoder
                        .push_instr(make_instr_imm16(result, lhs, rhs))?;
                    return Ok(());
                }
                let result = self.alloc.stack.push_dynamic()?;
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr_imm(result, lhs))?;
                let rhs_instr = make_instr_imm_rhs(self, T::from(rhs))?;
                self.alloc.instr_encoder.push_instr(rhs_instr)?;
                Ok(())
            }
            (Provider::Const(lhs), Provider::Register(rhs)) => {
                if make_instr_imm_reg_opt(self, T::from(lhs), rhs)? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                if let Ok(lhs) = T::from(lhs).try_into() {
                    // Optimization: We can use a compact instruction for small constants.
                    let result = self.alloc.stack.push_dynamic()?;
                    self.alloc
                        .instr_encoder
                        .push_instr(make_instr_imm16_rev(result, lhs, rhs))?;
                    return Ok(());
                }
                let result = self.alloc.stack.push_dynamic()?;
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr_imm_rev(result, rhs))?;
                let rhs_instr = make_instr_imm_rhs(self, T::from(lhs))?;
                self.alloc.instr_encoder.push_instr(rhs_instr)?;
                Ok(())
            }
            (Provider::Const(lhs), Provider::Const(rhs)) => {
                self.alloc.stack.push_const(consteval(lhs, rhs));
                Ok(())
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
            Self::make_instr_imm_rhs_i32,
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
    #[allow(clippy::too_many_arguments)]
    fn translate_binary_commutative<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm_rhs: fn(&mut Self, value: T) -> Result<Instruction, TranslationError>,
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
        let rhs = self.alloc.stack.pop();
        let lhs = self.alloc.stack.pop();
        match (lhs, rhs) {
            (Provider::Register(lhs), Provider::Register(rhs)) => {
                if make_instr_opt(self, lhs, rhs)? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                let result = self.alloc.stack.push_dynamic()?;
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr(result, lhs, rhs))?;
                Ok(())
            }
            (Provider::Register(reg_in), Provider::Const(imm_in))
            | (Provider::Const(imm_in), Provider::Register(reg_in)) => {
                if make_instr_imm_opt(self, reg_in, T::from(imm_in))? {
                    // Case: the custom logic applied its optimization and we can return.
                    return Ok(());
                }
                if let Ok(rhs) = T::from(imm_in).try_into() {
                    // Optimization: We can use a compact instruction for small constants.
                    let result = self.alloc.stack.push_dynamic()?;
                    self.alloc
                        .instr_encoder
                        .push_instr(make_instr_imm16(result, reg_in, rhs))?;
                    return Ok(());
                }
                let result = self.alloc.stack.push_dynamic()?;
                self.alloc
                    .instr_encoder
                    .push_instr(make_instr_imm(result, reg_in))?;
                let rhs_instr = make_instr_imm_rhs(self, T::from(imm_in))?;
                self.alloc.instr_encoder.push_instr(rhs_instr)?;
                Ok(())
            }
            (Provider::Const(lhs), Provider::Const(rhs)) => {
                self.alloc.stack.push_const(consteval(lhs, rhs));
                Ok(())
            }
        }
    }

    /// Convenience method for [`Self::translate_binary_commutative`] when translating `i32` instructions.
    #[allow(clippy::too_many_arguments)]
    fn translate_binary_commutative_i32(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
        consteval: fn(UntypedValue, UntypedValue) -> UntypedValue,
        make_instr_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
        make_instr_imm_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: i32,
        ) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError> {
        self.translate_binary_commutative::<i32>(
            make_instr,
            make_instr_imm,
            Self::make_instr_imm_rhs_i32,
            make_instr_imm16,
            consteval,
            make_instr_opt,
            make_instr_imm_opt,
        )
    }

    /// Convenience method for [`Self::translate_binary_commutative`] when translating `i64` instructions.
    #[allow(clippy::too_many_arguments)]
    fn translate_binary_commutative_i64(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm: fn(result: Register, lhs: Register) -> Instruction,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
        consteval: fn(UntypedValue, UntypedValue) -> UntypedValue,
        make_instr_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: Register,
        ) -> Result<bool, TranslationError>,
        make_instr_imm_opt: fn(
            &mut Self,
            lhs: Register,
            rhs: i64,
        ) -> Result<bool, TranslationError>,
    ) -> Result<(), TranslationError> {
        self.translate_binary_commutative::<i64>(
            make_instr,
            make_instr_imm,
            Self::make_instr_imm_rhs_i64,
            make_instr_imm16,
            consteval,
            make_instr_opt,
            make_instr_imm_opt,
        )
    }

    /// Can be used for [`Self::translate_binary_commutative`] if no custom optimization shall be applied.
    pub fn no_custom_opt<Lhs, Rhs>(
        &mut self,
        _lhs: Lhs,
        _rhs: Rhs,
    ) -> Result<bool, TranslationError> {
        Ok(false)
    }

    /// Can be used for [`Self::translate_binary_commutative`] to create immediate `i32` instructions.
    pub fn make_instr_imm_rhs_i32(&mut self, value: i32) -> Result<Instruction, TranslationError> {
        Ok(Instruction::const32(value))
    }

    /// Can be used for [`Self::translate_binary_commutative`] to create immediate `i64` instructions.
    pub fn make_instr_imm_rhs_i64(&mut self, value: i64) -> Result<Instruction, TranslationError> {
        let cref = self.engine().alloc_const(UntypedValue::from(value))?;
        Ok(Instruction::ConstRef(cref))
    }
}
