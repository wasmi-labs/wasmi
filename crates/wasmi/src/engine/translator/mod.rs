//! Function translation for the register-machine bytecode based Wasmi engine.

mod control_frame;
mod control_stack;
mod driver;
mod error;
mod instr_encoder;
mod labels;
mod relink_result;
mod stack;
mod typed_value;
mod utils;
mod visit;
mod visit_register;

#[cfg(test)]
mod tests;

use self::{
    control_frame::{
        BlockControlFrame,
        BlockHeight,
        IfControlFrame,
        LoopControlFrame,
        UnreachableControlFrame,
    },
    control_stack::AcquiredTarget,
    labels::{LabelRef, LabelRegistry},
    stack::ValueStack,
    typed_value::TypedValue,
    utils::{WasmFloat, WasmInteger},
};
pub use self::{
    control_frame::{ControlFrame, ControlFrameKind},
    control_stack::ControlStack,
    driver::FuncTranslationDriver,
    error::TranslationError,
    instr_encoder::{Instr, InstrEncoder},
    stack::TypedProvider,
};
use super::code_map::CompiledFuncEntity;
use crate::{
    engine::{
        bytecode::{
            AnyConst32,
            Const16,
            Const32,
            Instruction,
            Register,
            RegisterSpan,
            RegisterSpanIter,
            Sign,
            SignatureIdx,
        },
        config::FuelCosts,
        BlockType,
        CompiledFunc,
    },
    module::{FuncIdx, FuncTypeIdx, ModuleHeader},
    Engine,
    Error,
    FuncType,
};
use core::fmt;
use std::vec::Vec;
use wasmi_core::{TrapCode, UntypedValue, ValueType};
use wasmparser::{
    BinaryReaderError,
    FuncToValidate,
    FuncValidatorAllocations,
    MemArg,
    ValidatorResources,
    VisitOperator,
};

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
    /// Buffer to temporarily hold a bunch of preserved [`Register`] locals.
    preserved: Vec<PreservedLocal>,
}

/// A pair of local [`Register`] and its preserved [`Register`].
#[derive(Debug, Copy, Clone)]
pub struct PreservedLocal {
    local: Register,
    preserved: Register,
}

impl PreservedLocal {
    /// Creates a new [`PreservedLocal`].
    pub fn new(local: Register, preserved: Register) -> Self {
        Self { local, preserved }
    }
}

impl TranslationBuffers {
    fn reset(&mut self) {
        self.providers.clear();
        self.br_table_targets.clear();
        self.preserved.clear();
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
}

/// The used function validator type.
type FuncValidator = wasmparser::FuncValidator<wasmparser::ValidatorResources>;

/// A Wasm to Wasmi IR function translator that also validates its input.
pub struct ValidatingFuncTranslator<T> {
    /// The current position in the Wasm binary while parsing operators.
    pos: usize,
    /// The Wasm function validator.
    validator: FuncValidator,
    /// The chosen function translator.
    translator: T,
}

/// Reusable heap allocations for function validation and translation.
#[derive(Default)]
pub struct ReusableAllocations<T> {
    pub translation: T,
    pub validation: FuncValidatorAllocations,
}

/// A WebAssembly (Wasm) function translator.
pub trait WasmTranslator<'parser>: VisitOperator<'parser, Output = Result<(), Error>> {
    /// The reusable allocations required by the [`WasmTranslator`].
    ///
    /// # Note
    ///
    /// Those allocations can be cached on the caller side for reusability
    /// in order to avoid frequent memory allocations and deallocations.
    type Allocations: Default;

    /// Sets up the translation process for the Wasm `bytes` and Wasm `module` header.
    ///
    /// - Returns `true` if the [`WasmTranslator`] is done with the translation process.
    /// - Returns `false` if the [`WasmTranslator`] demands the translation driver to
    ///   proceed with the process of parsing the Wasm module and feeding parse pieces
    ///   to the [`WasmTranslator`].
    ///
    /// # Note
    ///
    /// - This method requires `bytes` to be the slice of bytes that make up the entire
    ///   Wasm function body (including local variables).
    /// - Also `module` must be a reference to the Wasm module header that is going to be
    ///   used for translation of the Wasm function body.
    fn setup(&mut self, bytes: &[u8]) -> Result<bool, Error>;

    /// Translates the given local variables for the translated function.
    fn translate_locals(
        &mut self,
        amount: u32,
        value_type: wasmparser::ValType,
    ) -> Result<(), Error>;

    /// Informs the [`WasmTranslator`] that the Wasm function header translation is finished.
    ///
    /// # Note
    ///
    /// - After this function call no more locals and parameters may be registered
    ///   to the [`WasmTranslator`] via [`WasmTranslator::translate_locals`].
    /// - After this function call the [`WasmTranslator`] expects its [`VisitOperator`]
    ///   trait methods to be called for translating the Wasm operators of the
    ///   translated function.
    ///
    /// # Dev. Note
    ///
    /// This got introduced to properly calculate the fuel costs for all local variables
    /// and function parameters.
    fn finish_translate_locals(&mut self) -> Result<(), Error>;

    /// Updates the [`WasmTranslator`] about the current byte position within translated Wasm binary.
    ///
    /// # Note
    ///
    /// This information is mainly required for properly locating translation errors.
    fn update_pos(&mut self, pos: usize);

    /// Finishes constructing the Wasm function translation.
    ///
    /// # Note
    ///
    /// - Initialized the [`CompiledFunc`] in the [`Engine`].
    /// - Returns the allocations used for translation.
    fn finish(self, finalize: impl FnOnce(CompiledFuncEntity)) -> Result<Self::Allocations, Error>;
}

impl<T> ValidatingFuncTranslator<T> {
    /// Creates a new [`ValidatingFuncTranslator`].
    pub fn new(validator: FuncValidator, translator: T) -> Result<Self, Error> {
        Ok(Self {
            pos: 0,
            validator,
            translator,
        })
    }

    /// Returns the current position within the Wasm binary while parsing operators.
    fn current_pos(&self) -> usize {
        self.pos
    }

    /// Translates into Wasmi bytecode if the current code path is reachable.
    fn validate_then_translate<Validate, Translate>(
        &mut self,
        validate: Validate,
        translate: Translate,
    ) -> Result<(), Error>
    where
        Validate: FnOnce(&mut FuncValidator) -> Result<(), BinaryReaderError>,
        Translate: FnOnce(&mut T) -> Result<(), Error>,
    {
        validate(&mut self.validator)?;
        translate(&mut self.translator)?;
        Ok(())
    }
}

impl<'parser, T> WasmTranslator<'parser> for ValidatingFuncTranslator<T>
where
    T: WasmTranslator<'parser>,
{
    type Allocations = ReusableAllocations<T::Allocations>;

    fn setup(&mut self, bytes: &[u8]) -> Result<bool, Error> {
        self.translator.setup(bytes)?;
        // Note: Wasm validation always need to be driven, therefore returning `Ok(false)`
        //       even if the underlying Wasm translator does not need a translation driver.
        Ok(false)
    }

    fn translate_locals(
        &mut self,
        amount: u32,
        value_type: wasmparser::ValType,
    ) -> Result<(), Error> {
        self.validator
            .define_locals(self.current_pos(), amount, value_type)?;
        self.translator.translate_locals(amount, value_type)?;
        Ok(())
    }

    fn finish_translate_locals(&mut self) -> Result<(), Error> {
        self.translator.finish_translate_locals()?;
        Ok(())
    }

    fn update_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    fn finish(
        mut self,
        finalize: impl FnOnce(CompiledFuncEntity),
    ) -> Result<Self::Allocations, Error> {
        let pos = self.current_pos();
        self.validator.finish(pos)?;
        let translation = self.translator.finish(finalize)?;
        let validation = self.validator.into_allocations();
        let allocations = ReusableAllocations {
            translation,
            validation,
        };
        Ok(allocations)
    }
}

macro_rules! impl_visit_operator {
    ( @mvp BrTable { $arg:ident: $argty:ty } => $visit:ident $($rest:tt)* ) => {
        // We need to special case the `BrTable` operand since its
        // arguments (a.k.a. `BrTable<'a>`) are not `Copy` which all
        // the other impls make use of.
        fn $visit(&mut self, $arg: $argty) -> Self::Output {
            let offset = self.current_pos();
            self.validate_then_translate(
                |validator| validator.visitor(offset).$visit($arg.clone()),
                |translator| translator.$visit($arg.clone()),
            )
        }
        impl_visit_operator!($($rest)*);
    };
    ( @mvp $($rest:tt)* ) => {
        impl_visit_operator!(@@supported $($rest)*);
    };
    ( @sign_extension $($rest:tt)* ) => {
        impl_visit_operator!(@@supported $($rest)*);
    };
    ( @saturating_float_to_int $($rest:tt)* ) => {
        impl_visit_operator!(@@supported $($rest)*);
    };
    ( @bulk_memory $($rest:tt)* ) => {
        impl_visit_operator!(@@supported $($rest)*);
    };
    ( @reference_types $($rest:tt)* ) => {
        impl_visit_operator!(@@supported $($rest)*);
    };
    ( @tail_call $($rest:tt)* ) => {
        impl_visit_operator!(@@supported $($rest)*);
    };
    ( @@supported $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        fn $visit(&mut self $($(,$arg: $argty)*)?) -> Self::Output {
            let offset = self.current_pos();
            self.validate_then_translate(
                move |validator| validator.visitor(offset).$visit($($($arg),*)?),
                move |translator| translator.$visit($($($arg),*)?),
            )
        }
        impl_visit_operator!($($rest)*);
    };
    ( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // Wildcard match arm for all the other (yet) unsupported Wasm proposals.
        fn $visit(&mut self $($(, $arg: $argty)*)?) -> Self::Output {
            let offset = self.current_pos();
            self.validator.visitor(offset).$visit($($($arg),*)?).map_err(::core::convert::Into::into)
        }
        impl_visit_operator!($($rest)*);
    };
    () => {};
}

impl<'a, T> VisitOperator<'a> for ValidatingFuncTranslator<T>
where
    T: WasmTranslator<'a>,
{
    type Output = Result<(), Error>;

    wasmparser::for_each_operator!(impl_visit_operator);
}

/// A lazy Wasm function translator that defers translation when the function is first used.
pub struct LazyFuncTranslator {
    /// The index of the lazily compiled function within its module.
    func_idx: FuncIdx,
    /// The identifier of the to be compiled function.
    compiled_func: CompiledFunc,
    /// The Wasm module header information used for translation.
    module: ModuleHeader,
    /// Optional information about lazy Wasm validation.
    func_to_validate: Option<FuncToValidate<ValidatorResources>>,
}

impl fmt::Debug for LazyFuncTranslator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LazyFuncTranslator")
            .field("func_idx", &self.func_idx)
            .field("compiled_func", &self.compiled_func)
            .field("module", &self.module)
            .field("validate", &self.func_to_validate.is_some())
            .finish()
    }
}

impl LazyFuncTranslator {
    /// Create a new [`LazyFuncTranslator`].
    pub fn new(
        func_idx: FuncIdx,
        compiled_func: CompiledFunc,
        module: ModuleHeader,
        func_to_validate: Option<FuncToValidate<ValidatorResources>>,
    ) -> Self {
        Self {
            func_idx,
            compiled_func,
            module,
            func_to_validate,
        }
    }
}

impl<'parser> WasmTranslator<'parser> for LazyFuncTranslator {
    type Allocations = ();

    fn setup(&mut self, bytes: &[u8]) -> Result<bool, Error> {
        self.module
            .engine()
            .upgrade()
            .unwrap_or_else(|| {
                panic!(
                    "engine does no longer exist for lazy compilation setup: {:?}",
                    self.module.engine()
                )
            })
            .init_lazy_func(
                self.func_idx,
                self.compiled_func,
                bytes,
                &self.module,
                self.func_to_validate.take(),
            );
        Ok(true)
    }

    #[inline]
    fn translate_locals(
        &mut self,
        _amount: u32,
        _value_type: wasmparser::ValType,
    ) -> Result<(), Error> {
        Ok(())
    }

    #[inline]
    fn finish_translate_locals(&mut self) -> Result<(), Error> {
        Ok(())
    }

    #[inline]
    fn update_pos(&mut self, _pos: usize) {}

    #[inline]
    fn finish(
        self,
        _finalize: impl FnOnce(CompiledFuncEntity),
    ) -> Result<Self::Allocations, Error> {
        Ok(())
    }
}

macro_rules! impl_visit_operator {
    ( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        #[inline]
        fn $visit(&mut self $($(, $arg: $argty)*)?) -> Self::Output {
            Ok(())
        }
        impl_visit_operator!($($rest)*);
    };
    () => {};
}

impl<'a> VisitOperator<'a> for LazyFuncTranslator {
    type Output = Result<(), Error>;

    wasmparser::for_each_operator!(impl_visit_operator);
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
    fuel_costs: Option<FuelCosts>,
    /// The reusable data structures of the [`FuncTranslator`].
    alloc: FuncTranslatorAllocations,
}

impl<'parser> WasmTranslator<'parser> for FuncTranslator {
    type Allocations = FuncTranslatorAllocations;

    fn setup(&mut self, _bytes: &[u8]) -> Result<bool, Error> {
        Ok(false)
    }

    fn translate_locals(
        &mut self,
        amount: u32,
        _value_type: wasmparser::ValType,
    ) -> Result<(), Error> {
        self.alloc.stack.register_locals(amount)
    }

    fn finish_translate_locals(&mut self) -> Result<(), Error> {
        self.alloc.stack.finish_register_locals();
        Ok(())
    }

    fn update_pos(&mut self, _pos: usize) {}

    fn finish(
        mut self,
        finalize: impl FnOnce(CompiledFuncEntity),
    ) -> Result<Self::Allocations, Error> {
        self.alloc
            .instr_encoder
            .defrag_registers(&mut self.alloc.stack)?;
        self.alloc
            .instr_encoder
            .update_branch_offsets(&mut self.alloc.stack)?;
        let len_registers = self.alloc.stack.len_registers();
        if let Some(fuel_costs) = self.fuel_costs() {
            // Note: Fuel metering is enabled so we need to bump the fuel
            //       of the function enclosing Wasm `block` by an amount
            //       that depends on the total number of registers used by
            //       the compiled function.
            // Note: The function enclosing block fuel instruction is always
            //       the instruction at the 0th index if fuel metering is enabled.
            let fuel_instr = Instr::from_u32(0);
            let fuel_info = FuelInfo::some(*fuel_costs, fuel_instr);
            self.alloc
                .instr_encoder
                .bump_fuel_consumption(fuel_info, |costs| {
                    costs.fuel_for_copies(u64::from(len_registers))
                })?;
        }
        let func_consts = self.alloc.stack.func_local_consts();
        let instrs = self.alloc.instr_encoder.drain_instrs();
        finalize(CompiledFuncEntity::new(len_registers, instrs, func_consts));
        Ok(self.into_allocations())
    }
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

/// Fuel metering information for a certain translation state.
#[derive(Debug, Copy, Clone)]
pub enum FuelInfo {
    /// Fuel metering is disabled.
    None,
    /// Fuel metering is enabled with the following information.
    Some {
        /// The [`FuelCosts`] for the function translation.
        costs: FuelCosts,
        /// Index to the current [`Instruction::ConsumeFuel`] of a parent [`ControlFrame`].
        instr: Instr,
    },
}

impl FuelInfo {
    /// Create a new [`FuelInfo`] for enabled fuel metering.
    pub fn some(costs: FuelCosts, instr: Instr) -> Self {
        Self::Some { costs, instr }
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
            .copied();
        Self {
            func,
            engine,
            module: res,
            reachable: true,
            fuel_costs,
            alloc,
        }
        .init()
    }

    /// Returns the [`Engine`] for which the function is compiled.
    fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Initializes a newly constructed [`FuncTranslator`].
    fn init(mut self) -> Result<Self, Error> {
        self.alloc.reset();
        self.init_func_body_block()?;
        self.init_func_params()?;
        Ok(self)
    }

    /// Registers the `block` control frame surrounding the entire function body.
    fn init_func_body_block(&mut self) -> Result<(), Error> {
        let func_type = self.module.get_type_of_func(self.func);
        let block_type = BlockType::func_type(func_type);
        let end_label = self.alloc.instr_encoder.new_label();
        let consume_fuel = self.make_fuel_instr()?;
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
    fn init_func_params(&mut self) -> Result<(), Error> {
        for _param_type in self.func_type().params() {
            self.alloc.stack.register_locals(1)?;
        }
        Ok(())
    }

    /// Consumes `self` and returns the underlying reusable [`FuncTranslatorAllocations`].
    fn into_allocations(self) -> FuncTranslatorAllocations {
        self.alloc
    }

    /// Returns the [`FuncType`] of the function that is currently translated.
    fn func_type(&self) -> FuncType {
        let dedup_func_type = self.module.get_type_of_func(self.func);
        self.engine()
            .resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Resolves the [`FuncType`] of the given [`FuncTypeIdx`].
    fn func_type_at(&self, func_type_index: SignatureIdx) -> FuncType {
        let func_type_index = FuncTypeIdx::from(func_type_index.to_u32()); // TODO: use the same type
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

    /// Returns the configured [`FuelCosts`] of the [`Engine`] if any.
    ///
    /// Returns `None` if fuel metering is disabled.
    fn fuel_costs(&self) -> Option<&FuelCosts> {
        self.fuel_costs.as_ref()
    }

    /// Returns the most recent [`Instruction::ConsumeFuel`] in the translation process.
    ///
    /// Returns `None` if fuel metering is disabled.
    fn fuel_instr(&self) -> Option<Instr> {
        self.alloc.control_stack.last().consume_fuel_instr()
    }

    /// Returns the [`FuelInfo`] for the current translation state.
    ///
    /// Returns [`FuelInfo::None`] if fuel metering is disabled.
    fn fuel_info(&self) -> FuelInfo {
        let Some(&fuel_costs) = self.fuel_costs() else {
            // Fuel metering is disabled so we can bail out.
            return FuelInfo::None;
        };
        let fuel_instr = self
            .fuel_instr()
            .expect("fuel metering is enabled but there is no Instruction::ConsumeFuel");
        FuelInfo::some(fuel_costs, fuel_instr)
    }

    /// Pushes a [`Instruction::ConsumeFuel`] with base costs if fuel metering is enabled.
    ///
    /// Returns `None` if fuel metering is disabled.
    fn make_fuel_instr(&mut self) -> Result<Option<Instr>, Error> {
        let Some(fuel_costs) = self.fuel_costs() else {
            // Fuel metering is disabled so there is no need to create an `Instruction::ConsumeFuel`.
            return Ok(None);
        };
        let fuel_instr = Instruction::consume_fuel(fuel_costs.base())
            .expect("base fuel must be valid for creating `Instruction::ConsumeFuel`");
        let instr = self.alloc.instr_encoder.push_instr(fuel_instr)?;
        Ok(Some(instr))
    }

    /// Bumps fuel consumption of the most recent [`Instruction::ConsumeFuel`] in the translation process.
    ///
    /// Does nothing if gas metering is disabled.
    fn bump_fuel_consumption<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnOnce(&FuelCosts) -> u64,
    {
        let fuel_info = self.fuel_info();
        self.alloc
            .instr_encoder
            .bump_fuel_consumption(fuel_info, f)?;
        Ok(())
    }

    /// Utility function for pushing a new [`Instruction`] with fuel costs.
    ///
    /// # Note
    ///
    /// Fuel metering is only encoded or adjusted if it is enabled.
    fn push_fueled_instr<F>(&mut self, instr: Instruction, f: F) -> Result<Instr, Error>
    where
        F: FnOnce(&FuelCosts) -> u64,
    {
        self.bump_fuel_consumption(f)?;
        self.alloc.instr_encoder.push_instr(instr)
    }

    /// Utility function for pushing a new [`Instruction`] with basic fuel costs.
    ///
    /// # Note
    ///
    /// Fuel metering is only encoded or adjusted if it is enabled.
    fn push_base_instr(&mut self, instr: Instruction) -> Result<Instr, Error> {
        self.push_fueled_instr(instr, FuelCosts::base)
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
        let preserved = &mut self.alloc.buffer.preserved;
        preserved.clear();
        self.alloc.stack.preserve_all_locals(|preserved_local| {
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
            (b.preserved.to_i16() - a.preserved.to_i16()) == 1
        });
        for copy_group in copy_groups {
            let len = copy_group.len();
            let results = RegisterSpan::new(copy_group[0].preserved).iter(len);
            let providers = &mut self.alloc.buffer.providers;
            providers.clear();
            providers.extend(
                copy_group
                    .iter()
                    .map(|p| p.local)
                    .map(TypedProvider::Register),
            );
            let instr = self.alloc.instr_encoder.encode_copies(
                &mut self.alloc.stack,
                results,
                &providers[..],
                fuel_info,
            )?;
            if let Some(instr) = instr {
                self.alloc.instr_encoder.notify_preserved_register(instr)
            }
        }
        Ok(())
    }

    /// Convenience function to copy the parameters when branching to a control frame.
    fn translate_copy_branch_params(
        &mut self,
        branch_params: RegisterSpanIter,
    ) -> Result<(), Error> {
        if branch_params.is_empty() {
            // If the block does not have branch parameters there is no need to copy anything.
            return Ok(());
        }
        let fuel_info = self.fuel_info();
        let params = &mut self.alloc.buffer.providers;
        self.alloc.stack.pop_n(branch_params.len(), params);
        self.alloc.instr_encoder.encode_copies(
            &mut self.alloc.stack,
            branch_params,
            &self.alloc.buffer.providers[..],
            fuel_info,
        )?;
        Ok(())
    }

    /// Translates the `end` of a Wasm `block` control frame.
    fn translate_end_block(&mut self, frame: BlockControlFrame) -> Result<(), Error> {
        if self.alloc.control_stack.is_empty() {
            bail_unreachable!(self);
            let fuel_info = match self.fuel_costs().copied() {
                None => FuelInfo::None,
                Some(fuel_costs) => {
                    let fuel_instr = frame
                        .consume_fuel_instr()
                        .expect("must have fuel instruction if fuel metering is enabled");
                    FuelInfo::some(fuel_costs, fuel_instr)
                }
            };
            // We dropped the Wasm `block` that encloses the function itself so we can return.
            return self.translate_return_with(fuel_info);
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
    fn translate_end_loop(&mut self, _frame: LoopControlFrame) -> Result<(), Error> {
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
    fn translate_end_if(&mut self, frame: IfControlFrame) -> Result<(), Error> {
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
    fn translate_end_if_then_only(&mut self, frame: IfControlFrame) -> Result<(), Error> {
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
    fn translate_end_if_else_only(&mut self, frame: IfControlFrame) -> Result<(), Error> {
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
    fn translate_end_if_then_missing_else(&mut self, frame: IfControlFrame) -> Result<(), Error> {
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
        let engine = self.engine().clone();
        let if_height = frame.block_height().into_u16() as usize;
        let else_providers = self.alloc.control_stack.pop_else_providers();
        if has_results {
            // We haven't visited the `else` block and thus the `else`
            // providers are still on the auxiliary stack and need to
            // be popped. We use them to restore the stack to the state
            // when entering the `if` block so that we can properly copy
            // the `else` results to were they are expected.
            self.alloc.stack.trunc(if_height);
            for provider in else_providers {
                self.alloc.stack.push_provider(provider)?;
                if let TypedProvider::Register(register) = provider {
                    self.alloc.stack.dec_register_usage(register);
                }
            }
            self.translate_copy_branch_params(frame.branch_params(&engine))?;
        }
        // After `else` parameters have been copied we can finally pin the `end` label.
        self.alloc.instr_encoder.pin_label(frame.end_label());
        // Without `else` block the code after the `if` is always reachable and
        // thus we need to clean up and prepare the value stack for the following code.
        self.alloc.stack.trunc(if_height);
        for result in frame.branch_params(&engine) {
            self.alloc.stack.push_register(result)?;
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
    ) -> Result<RegisterSpan, Error> {
        let params = &mut self.alloc.buffer.providers;
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
    ) -> Result<(), Error> {
        let result = self.alloc.stack.push_dynamic()?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCosts::base)?;
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
        lhs: Register,
        rhs: T,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16<T>) -> Instruction,
    ) -> Result<bool, Error>
    where
        T: Copy + TryInto<Const16<T>>,
    {
        if let Ok(rhs) = rhs.try_into() {
            // Optimization: We can use a compact instruction for small constants.
            let result = self.alloc.stack.push_dynamic()?;
            self.push_fueled_instr(make_instr_imm16(result, lhs, rhs), FuelCosts::base)?;
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
    ) -> Result<bool, Error>
    where
        T: Copy + TryInto<Const16<T>>,
    {
        if let Ok(lhs) = lhs.try_into() {
            // Optimization: We can use a compact instruction for small constants.
            let result = self.alloc.stack.push_dynamic()?;
            self.push_fueled_instr(make_instr_imm16(result, lhs, rhs), FuelCosts::base)?;
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
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error>
    where
        T: Into<UntypedValue>,
    {
        let result = self.alloc.stack.push_dynamic()?;
        let rhs = self.alloc.stack.alloc_const(rhs)?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCosts::base)?;
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
    ) -> Result<(), Error>
    where
        T: Into<UntypedValue>,
    {
        let result = self.alloc.stack.push_dynamic()?;
        let lhs = self.alloc.stack.alloc_const(lhs)?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCosts::base)?;
        Ok(())
    }

    /// Translates a [`TrapCode`] as [`Instruction`].
    fn translate_trap(&mut self, trap_code: TrapCode) -> Result<(), Error> {
        bail_unreachable!(self);
        self.push_fueled_instr(Instruction::Trap(trap_code), FuelCosts::base)?;
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
    fn translate_binary<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16<T>) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16<T>, rhs: Register) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_opt: fn(&mut Self, lhs: Register, rhs: Register) -> Result<bool, Error>,
        make_instr_reg_imm_opt: fn(&mut Self, lhs: Register, rhs: T) -> Result<bool, Error>,
        make_instr_imm_reg_opt: fn(&mut Self, lhs: T, rhs: Register) -> Result<bool, Error>,
    ) -> Result<(), Error>
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
    fn translate_fbinary<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_opt: fn(&mut Self, lhs: Register, rhs: Register) -> Result<bool, Error>,
        make_instr_reg_imm_opt: fn(&mut Self, lhs: Register, rhs: T) -> Result<bool, Error>,
        make_instr_imm_reg_opt: fn(&mut Self, lhs: T, rhs: Register) -> Result<bool, Error>,
    ) -> Result<(), Error>
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

    /// Translate Wasmi float `{f32,f64}.copysign` instructions.
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
    ) -> Result<(), Error>
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
    fn translate_binary_commutative<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm16: fn(result: Register, lhs: Register, rhs: Const16<T>) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_opt: fn(&mut Self, lhs: Register, rhs: Register) -> Result<bool, Error>,
        make_instr_imm_opt: fn(&mut Self, lhs: Register, rhs: T) -> Result<bool, Error>,
    ) -> Result<(), Error>
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
    fn translate_fbinary_commutative<T>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_opt: fn(&mut Self, lhs: Register, rhs: Register) -> Result<bool, Error>,
        make_instr_imm_opt: fn(&mut Self, lhs: Register, rhs: T) -> Result<bool, Error>,
    ) -> Result<(), Error>
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
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm: fn(result: Register, lhs: Register, rhs: Const16<T>) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16<T>, rhs: Register) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> TypedValue,
        make_instr_imm_reg_opt: fn(&mut Self, lhs: T, rhs: Register) -> Result<bool, Error>,
    ) -> Result<(), Error>
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
                self.push_fueled_instr(
                    make_instr_imm(result, lhs, <Const16<T>>::from(rhs)),
                    FuelCosts::base,
                )?;
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
    fn translate_divrem<T, NonZeroT>(
        &mut self,
        make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
        make_instr_imm16: fn(
            result: Register,
            lhs: Register,
            rhs: Const16<NonZeroT>,
        ) -> Instruction,
        make_instr_imm16_rev: fn(result: Register, lhs: Const16<T>, rhs: Register) -> Instruction,
        consteval: fn(TypedValue, TypedValue) -> Result<TypedValue, TrapCode>,
        make_instr_opt: fn(&mut Self, lhs: Register, rhs: Register) -> Result<bool, Error>,
        make_instr_reg_imm_opt: fn(&mut Self, lhs: Register, rhs: T) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: WasmInteger,
        NonZeroT: Copy + TryFrom<T> + TryInto<Const16<NonZeroT>>,
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
                let Some(non_zero_rhs) = NonZeroT::try_from(T::from(rhs)).ok() else {
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
    fn no_custom_opt<Lhs, Rhs>(&mut self, _lhs: Lhs, _rhs: Rhs) -> Result<bool, Error> {
        Ok(false)
    }

    /// Translates a unary Wasm instruction to Wasmi bytecode.
    fn translate_unary(
        &mut self,
        make_instr: fn(result: Register, input: Register) -> Instruction,
        consteval: fn(input: TypedValue) -> TypedValue,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        match self.alloc.stack.pop() {
            TypedProvider::Register(input) => {
                let result = self.alloc.stack.push_dynamic()?;
                self.push_fueled_instr(make_instr(result, input), FuelCosts::base)?;
                Ok(())
            }
            TypedProvider::Const(input) => {
                self.alloc.stack.push_const(consteval(input));
                Ok(())
            }
        }
    }

    /// Translates a fallible unary Wasm instruction to Wasmi bytecode.
    fn translate_unary_fallible(
        &mut self,
        make_instr: fn(result: Register, input: Register) -> Instruction,
        consteval: fn(input: TypedValue) -> Result<TypedValue, TrapCode>,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        match self.alloc.stack.pop() {
            TypedProvider::Register(input) => {
                let result = self.alloc.stack.push_dynamic()?;
                self.push_fueled_instr(make_instr(result, input), FuelCosts::base)?;
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
        f: impl FnOnce(&mut Self, u32) -> Result<(), Error>,
    ) -> Result<(), Error> {
        match u32::from(ptr).checked_add(offset) {
            Some(address) => f(self, address),
            None => self.translate_trap(TrapCode::MemoryOutOfBounds),
        }
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
        make_instr: fn(result: Register, ptr: Register) -> Instruction,
        make_instr_offset16: fn(
            result: Register,
            ptr: Register,
            offset: Const16<u32>,
        ) -> Instruction,
        make_instr_at: fn(result: Register, address: Const32<u32>) -> Instruction,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let offset = Self::memarg_offset(memarg);
        match self.alloc.stack.pop() {
            TypedProvider::Register(ptr) => {
                if let Ok(offset) = <Const16<u32>>::try_from(offset) {
                    let result = self.alloc.stack.push_dynamic()?;
                    self.push_fueled_instr(
                        make_instr_offset16(result, ptr, offset),
                        FuelCosts::load,
                    )?;
                    return Ok(());
                }
                let result = self.alloc.stack.push_dynamic()?;
                self.push_fueled_instr(make_instr(result, ptr), FuelCosts::load)?;
                self.alloc
                    .instr_encoder
                    .append_instr(Instruction::const32(offset))?;
                Ok(())
            }
            TypedProvider::Const(ptr) => {
                self.effective_address_and(ptr, offset, |this, address| {
                    let result = this.alloc.stack.push_dynamic()?;
                    this.push_fueled_instr(
                        make_instr_at(result, Const32::from(address)),
                        FuelCosts::load,
                    )?;
                    Ok(())
                })
            }
        }
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
    fn translate_istore<T, U>(
        &mut self,
        memarg: MemArg,
        make_instr: fn(ptr: Register, offset: Const32<u32>) -> Instruction,
        make_instr_offset16: fn(ptr: Register, offset: u16, value: Register) -> Instruction,
        make_instr_offset16_imm: fn(ptr: Register, offset: u16, value: U) -> Instruction,
        make_instr_at: fn(address: Const32<u32>, value: Register) -> Instruction,
        make_instr_at_imm: fn(address: Const32<u32>, value: U) -> Instruction,
    ) -> Result<(), Error>
    where
        T: Copy + From<TypedValue>,
        U: TryFrom<T>,
    {
        bail_unreachable!(self);
        let offset = Self::memarg_offset(memarg);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(ptr), TypedProvider::Register(value)) => {
                if let Ok(offset) = u16::try_from(offset) {
                    self.push_fueled_instr(
                        make_instr_offset16(ptr, offset, value),
                        FuelCosts::store,
                    )?;
                    Ok(())
                } else {
                    self.push_fueled_instr(
                        make_instr(ptr, Const32::from(offset)),
                        FuelCosts::store,
                    )?;
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
                        self.push_fueled_instr(
                            make_instr_offset16_imm(ptr, offset, value),
                            FuelCosts::store,
                        )?;
                        Ok(())
                    }
                    (Ok(offset), Err(_)) => {
                        let value = self.alloc.stack.alloc_const(value)?;
                        self.push_fueled_instr(
                            make_instr_offset16(ptr, offset, value),
                            FuelCosts::store,
                        )?;
                        Ok(())
                    }
                    (Err(_), _) => {
                        self.push_fueled_instr(
                            make_instr(ptr, Const32::from(offset)),
                            FuelCosts::store,
                        )?;
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
                    this.push_fueled_instr(
                        make_instr_at(Const32::from(address), value),
                        FuelCosts::store,
                    )?;
                    Ok(())
                }),
            (TypedProvider::Const(ptr), TypedProvider::Const(value)) => {
                self.effective_address_and(ptr, offset, |this, address| {
                    if let Ok(value) = U::try_from(T::from(value)) {
                        this.push_fueled_instr(
                            make_instr_at_imm(Const32::from(address), value),
                            FuelCosts::store,
                        )?;
                        Ok(())
                    } else {
                        let value = this.alloc.stack.alloc_const(value)?;
                        this.push_fueled_instr(
                            make_instr_at(Const32::from(address), value),
                            FuelCosts::store,
                        )?;
                        Ok(())
                    }
                })
            }
        }
    }

    /// Translates Wasm float `store` instructions to Wasmi bytecode.
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
    /// - `{f32, f64}.store`
    fn translate_fstore(
        &mut self,
        memarg: MemArg,
        make_instr: fn(ptr: Register, offset: Const32<u32>) -> Instruction,
        make_instr_offset16: fn(ptr: Register, offset: u16, value: Register) -> Instruction,
        make_instr_at: fn(address: Const32<u32>, value: Register) -> Instruction,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let offset = Self::memarg_offset(memarg);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(ptr), TypedProvider::Register(value)) => {
                if let Ok(offset) = u16::try_from(offset) {
                    self.push_fueled_instr(
                        make_instr_offset16(ptr, offset, value),
                        FuelCosts::store,
                    )?;
                    Ok(())
                } else {
                    self.push_fueled_instr(
                        make_instr(ptr, Const32::from(offset)),
                        FuelCosts::store,
                    )?;
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
                        self.push_fueled_instr(
                            make_instr_offset16(ptr, offset, value),
                            FuelCosts::store,
                        )?;
                        Ok(())
                    }
                    Err(_) => {
                        self.push_fueled_instr(
                            make_instr(ptr, Const32::from(offset)),
                            FuelCosts::store,
                        )?;
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
                    this.push_fueled_instr(
                        make_instr_at(Const32::from(address), value),
                        FuelCosts::store,
                    )?;
                    Ok(())
                }),
            (TypedProvider::Const(ptr), TypedProvider::Const(value)) => {
                self.effective_address_and(ptr, offset, |this, address| {
                    let value = this.alloc.stack.alloc_const(value)?;
                    this.push_fueled_instr(
                        make_instr_at(Const32::from(address), value),
                        FuelCosts::store,
                    )?;
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
    fn translate_select(&mut self, type_hint: Option<ValueType>) -> Result<(), Error> {
        /// Convenience function to encode a `select` instruction.
        ///
        /// # Note
        ///
        /// Helper for `select` instructions where one of `lhs` and `rhs`
        /// is a [`Register`] and the other a function local constant value.
        fn encode_select_imm(
            this: &mut FuncTranslator,
            result: Register,
            condition: Register,
            reg_in: Register,
            imm_in: impl Into<UntypedValue>,
            make_instr: fn(
                result: Register,
                condition: Register,
                lhs_or_rhs: Register,
            ) -> Instruction,
        ) -> Result<(), Error> {
            this.push_fueled_instr(make_instr(result, condition, reg_in), FuelCosts::base)?;
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
            this: &mut FuncTranslator,
            result: Register,
            condition: Register,
            reg_in: Register,
            imm_in: impl Into<AnyConst32>,
            make_instr: fn(
                result: Register,
                condition: Register,
                lhs_or_rhs: Register,
            ) -> Instruction,
        ) -> Result<(), Error> {
            this.push_fueled_instr(make_instr(result, condition, reg_in), FuelCosts::base)?;
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
            this: &mut FuncTranslator,
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
        ) -> Result<(), Error>
        where
            T: Copy + Into<UntypedValue>,
            Const32<T>: TryFrom<T>,
        {
            match <Const32<T>>::try_from(imm_in) {
                Ok(imm_in) => {
                    this.push_fueled_instr(make_instr(result, condition, reg_in), FuelCosts::base)?;
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
            TypedProvider::Register(condition) => {
                match (lhs, rhs) {
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
                        self.push_fueled_instr(
                            Instruction::select(result, condition, lhs),
                            FuelCosts::base,
                        )?;
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
                            this: &mut FuncTranslator,
                            result: Register,
                            condition: Register,
                            lhs: T,
                            rhs: T,
                        ) -> Result<(), Error> {
                            this.push_fueled_instr(
                                Instruction::select_imm32(result, lhs),
                                FuelCosts::base,
                            )?;
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
                            this: &mut FuncTranslator,
                            result: Register,
                            condition: Register,
                            lhs: T,
                            rhs: T,
                            make_instr: fn(
                                result_or_condition: Register,
                                lhs_or_rhs: Const32<T>,
                            ) -> Instruction,
                            make_param: fn(Const32<T>) -> Instruction,
                        ) -> Result<(), Error>
                        where
                            T: Copy + Into<UntypedValue>,
                            Const32<T>: TryFrom<T>,
                        {
                            let lhs32 = <Const32<T>>::try_from(lhs).ok();
                            let rhs32 = <Const32<T>>::try_from(rhs).ok();
                            match (lhs32, rhs32) {
                                (Some(lhs), Some(rhs)) => {
                                    this.push_fueled_instr(
                                        make_instr(result, lhs),
                                        FuelCosts::base,
                                    )?;
                                    this.alloc
                                        .instr_encoder
                                        .append_instr(make_instr(condition, rhs))?;
                                    Ok(())
                                }
                                (Some(lhs), None) => {
                                    let rhs = this.alloc.stack.alloc_const(rhs)?;
                                    this.push_fueled_instr(
                                        Instruction::select_rev(result, condition, rhs),
                                        FuelCosts::base,
                                    )?;
                                    this.alloc.instr_encoder.append_instr(make_param(lhs))?;
                                    Ok(())
                                }
                                (None, Some(rhs)) => {
                                    let lhs = this.alloc.stack.alloc_const(lhs)?;
                                    this.push_fueled_instr(
                                        Instruction::select(result, condition, lhs),
                                        FuelCosts::base,
                                    )?;
                                    this.alloc.instr_encoder.append_instr(make_param(rhs))?;
                                    Ok(())
                                }
                                (None, None) => {
                                    encode_select_imm(this, result, condition, lhs, rhs)
                                }
                            }
                        }

                        /// Convenience function to encode a `select` instruction.
                        ///
                        /// # Note
                        ///
                        /// Helper for `select` instructions where both `lhs`
                        /// and `rhs` are function local constant values.
                        fn encode_select_imm<T>(
                            this: &mut FuncTranslator,
                            result: Register,
                            condition: Register,
                            lhs: T,
                            rhs: T,
                        ) -> Result<(), Error>
                        where
                            T: Into<UntypedValue>,
                        {
                            let lhs = this.alloc.stack.alloc_const(lhs)?;
                            let rhs = this.alloc.stack.alloc_const(rhs)?;
                            this.push_fueled_instr(
                                Instruction::select(result, condition, lhs),
                                FuelCosts::base,
                            )?;
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
                }
            }
        }
    }

    /// Translates a Wasm `reinterpret` instruction.
    fn translate_reinterpret(&mut self, ty: ValueType) -> Result<(), Error> {
        bail_unreachable!(self);
        if let TypedProvider::Register(_) = self.alloc.stack.peek() {
            // Nothing to do.
            //
            // We try to not manipulate the emulation stack if not needed.
            return Ok(());
        }
        // Case: At this point we know that the top-most stack item is a constant value.
        //       We pop it, change its type and push it back onto the stack.
        let TypedProvider::Const(value) = self.alloc.stack.pop() else {
            panic!("the top-most stack item was asserted to be a constant value but a register was found")
        };
        self.alloc.stack.push_const(value.reinterpret(ty));
        Ok(())
    }

    /// Translates an unconditional `return` instruction.
    fn translate_return(&mut self) -> Result<(), Error> {
        let fuel_info = self.fuel_info();
        self.translate_return_with(fuel_info)
    }

    /// Translates an unconditional `return` instruction given fuel information.
    fn translate_return_with(&mut self, fuel_info: FuelInfo) -> Result<(), Error> {
        let func_type = self.func_type();
        let results = func_type.results();
        let values = &mut self.alloc.buffer.providers;
        self.alloc.stack.pop_n(results.len(), values);
        self.alloc
            .instr_encoder
            .encode_return(&mut self.alloc.stack, values, fuel_info)?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a conditional `br_if` that targets the function enclosing `block`.
    fn translate_return_if(&mut self, condition: Register) -> Result<(), Error> {
        bail_unreachable!(self);
        let len_results = self.func_type().results().len();
        let fuel_info = self.fuel_info();
        let values = &mut self.alloc.buffer.providers;
        self.alloc.stack.peek_n(len_results, values);
        self.alloc.instr_encoder.encode_return_nez(
            &mut self.alloc.stack,
            condition,
            values,
            fuel_info,
        )
    }
}
