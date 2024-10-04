//! Function translation for the register-machine bytecode based Wasmi engine.

mod control_frame;
mod control_stack;
mod driver;
mod error;
mod instr_encoder;
mod labels;
mod provider;
mod relink_result;
mod stack;
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
    provider::{Provider, ProviderSliceStack, UntypedProvider},
    stack::ValueStack,
    utils::{FromProviders as _, WasmFloat, WasmInteger},
};
pub use self::{
    control_frame::{ControlFrame, ControlFrameKind},
    control_stack::ControlStack,
    driver::FuncTranslationDriver,
    error::TranslationError,
    instr_encoder::{Instr, InstrEncoder},
    stack::TypedProvider,
};
use super::{
    bytecode::{index, BoundedRegSpan, BranchOffset},
    code_map::CompiledFuncEntity,
};
use crate::{
    core::{TrapCode, Typed, TypedVal, UntypedVal, ValType},
    engine::{
        bytecode::{Const16, Const32, Instruction, Reg, RegSpan, Sign},
        config::FuelCosts,
        BlockType,
        EngineFunc,
    },
    ir::{AnyConst16, IntoShiftAmount, ShiftAmount},
    module::{FuncIdx, FuncTypeIdx, ModuleHeader},
    Engine,
    Error,
    ExternRef,
    FuncRef,
    FuncType,
};
use core::fmt;
use stack::RegisterSpace;
use std::vec::Vec;
use utils::Wrap;
use wasmparser::{
    BinaryReaderError,
    FuncToValidate,
    FuncValidatorAllocations,
    MemArg,
    ValidatorResources,
    VisitOperator,
};

macro_rules! impl_typed_for {
    ( $( $ty:ident ),* $(,)? ) => {
        $(
            impl Typed for $ty {
                const TY: ValType = crate::core::ValType::$ty;
            }

            impl From<TypedVal> for $ty {
                fn from(typed_value: TypedVal) -> Self {
                    // # Note
                    //
                    // We only use a `debug_assert` here instead of a proper `assert`
                    // since the whole translation process assumes that Wasm validation
                    // was already performed and thus type checking does not necessarily
                    // need to happen redundantly outside of debug builds.
                    debug_assert!(matches!(typed_value.ty(), <$ty as Typed>::TY));
                    Self::from(typed_value.untyped())
                }
            }
        )*
    };
}
impl_typed_for! {
    FuncRef,
    ExternRef,
}

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
    /// - Initialized the [`EngineFunc`] in the [`Engine`].
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
    engine_func: EngineFunc,
    /// The Wasm module header information used for translation.
    module: ModuleHeader,
    /// Optional information about lazy Wasm validation.
    func_to_validate: Option<FuncToValidate<ValidatorResources>>,
}

impl fmt::Debug for LazyFuncTranslator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LazyFuncTranslator")
            .field("func_idx", &self.func_idx)
            .field("engine_func", &self.engine_func)
            .field("module", &self.module)
            .field("validate", &self.func_to_validate.is_some())
            .finish()
    }
}

impl LazyFuncTranslator {
    /// Create a new [`LazyFuncTranslator`].
    pub fn new(
        func_idx: FuncIdx,
        engine_func: EngineFunc,
        module: ModuleHeader,
        func_to_validate: Option<FuncToValidate<ValidatorResources>>,
    ) -> Self {
        Self {
            func_idx,
            engine_func,
            module,
            func_to_validate,
        }
    }
}

impl WasmTranslator<'_> for LazyFuncTranslator {
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
                self.engine_func,
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

impl WasmTranslator<'_> for FuncTranslator {
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
        // Note: we use a dummy `RegSpan` as placeholder.
        //
        // We can do this since the branch parameters of the function enclosing block
        // are never used due to optimizations to directly return to the caller instead.
        let branch_params = RegSpan::new(Reg::from(0));
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
        let base = u32::try_from(fuel_costs.base())
            .expect("base fuel must be valid for creating `Instruction::ConsumeFuel`");
        let fuel_instr = Instruction::consume_fuel(base);
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
    fn translate_copy_branch_params(&mut self, branch_params: BoundedRegSpan) -> Result<(), Error> {
        if branch_params.is_empty() {
            // If the block does not have branch parameters there is no need to copy anything.
            return Ok(());
        }
        let fuel_info = self.fuel_info();
        let params = &mut self.alloc.buffer.providers;
        self.alloc
            .stack
            .pop_n(usize::from(branch_params.len()), params);
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
        let params = &mut self.alloc.buffer.providers;
        // Pop the block parameters off the stack.
        self.alloc
            .stack
            .pop_n(usize::from(len_block_params), params);
        // Peek the branch parameter registers which are going to be returned.
        let branch_params = self
            .alloc
            .stack
            .peek_dynamic_n(usize::from(len_branch_params))?;
        // Push the block parameters onto the stack again as if nothing happened.
        self.alloc.stack.push_n(params)?;
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
        lhs: Reg,
        rhs: T,
        make_instr_imm16: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
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
        rhs: Reg,
        make_instr_imm16: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
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
        lhs: TypedVal,
        rhs: TypedVal,
        consteval: fn(TypedVal, TypedVal) -> TypedVal,
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
        lhs: Reg,
        rhs: T,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
    ) -> Result<(), Error>
    where
        T: Into<UntypedVal>,
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
        rhs: Reg,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
    ) -> Result<(), Error>
    where
        T: Into<UntypedVal>,
    {
        let result = self.alloc.stack.push_dynamic()?;
        let lhs = self.alloc.stack.alloc_const(lhs)?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCosts::base)?;
        Ok(())
    }

    /// Translates a [`TrapCode`] as [`Instruction`].
    fn translate_trap(&mut self, trap_code: TrapCode) -> Result<(), Error> {
        bail_unreachable!(self);
        self.push_fueled_instr(Instruction::trap(trap_code), FuelCosts::base)?;
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
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm16: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
        make_instr_imm16_rev: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
        consteval: fn(TypedVal, TypedVal) -> TypedVal,
        make_instr_opt: fn(&mut Self, lhs: Reg, rhs: Reg) -> Result<bool, Error>,
        make_instr_reg_imm_opt: fn(&mut Self, lhs: Reg, rhs: T) -> Result<bool, Error>,
        make_instr_imm_reg_opt: fn(&mut Self, lhs: T, rhs: Reg) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: Copy + From<TypedVal> + Into<TypedVal> + TryInto<Const16<T>>,
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
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        consteval: fn(TypedVal, TypedVal) -> TypedVal,
        make_instr_opt: fn(&mut Self, lhs: Reg, rhs: Reg) -> Result<bool, Error>,
        make_instr_reg_imm_opt: fn(&mut Self, lhs: Reg, rhs: T) -> Result<bool, Error>,
        make_instr_imm_reg_opt: fn(&mut Self, lhs: T, rhs: Reg) -> Result<bool, Error>,
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
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm: fn(result: Reg, lhs: Reg, rhs: Sign<T>) -> Instruction,
        consteval: fn(TypedVal, TypedVal) -> TypedVal,
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
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm16: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
        consteval: fn(TypedVal, TypedVal) -> TypedVal,
        make_instr_opt: fn(&mut Self, lhs: Reg, rhs: Reg) -> Result<bool, Error>,
        make_instr_imm_opt: fn(&mut Self, lhs: Reg, rhs: T) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: Copy + From<TypedVal> + TryInto<Const16<T>>,
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
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        consteval: fn(TypedVal, TypedVal) -> TypedVal,
        make_instr_opt: fn(&mut Self, lhs: Reg, rhs: Reg) -> Result<bool, Error>,
        make_instr_imm_opt: fn(&mut Self, lhs: Reg, rhs: T) -> Result<bool, Error>,
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
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_by: fn(result: Reg, lhs: Reg, rhs: ShiftAmount<T>) -> Instruction,
        make_instr_imm16: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
        consteval: fn(TypedVal, TypedVal) -> TypedVal,
        make_instr_imm_reg_opt: fn(&mut Self, lhs: T, rhs: Reg) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: WasmInteger + IntoShiftAmount,
        Const16<T>: From<i16>,
    {
        bail_unreachable!(self);
        match self.alloc.stack.pop2() {
            (TypedProvider::Register(lhs), TypedProvider::Register(rhs)) => {
                self.push_binary_instr(lhs, rhs, make_instr)
            }
            (TypedProvider::Register(lhs), TypedProvider::Const(rhs)) => {
                let Some(rhs) = T::from(rhs).into_shift_amount() else {
                    // Optimization: Shifting or rotating by zero bits is a no-op.
                    self.alloc.stack.push_register(lhs)?;
                    return Ok(());
                };
                let result = self.alloc.stack.push_dynamic()?;
                self.push_fueled_instr(make_instr_by(result, lhs, rhs), FuelCosts::base)?;
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
    fn translate_divrem<T, NonZeroT>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm16: fn(result: Reg, lhs: Reg, rhs: Const16<NonZeroT>) -> Instruction,
        make_instr_imm16_rev: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
        consteval: fn(TypedVal, TypedVal) -> Result<TypedVal, TrapCode>,
        make_instr_opt: fn(&mut Self, lhs: Reg, rhs: Reg) -> Result<bool, Error>,
        make_instr_reg_imm_opt: fn(&mut Self, lhs: Reg, rhs: T) -> Result<bool, Error>,
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
        make_instr: fn(result: Reg, input: Reg) -> Instruction,
        consteval: fn(input: TypedVal) -> TypedVal,
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
        make_instr: fn(result: Reg, input: Reg) -> Instruction,
        consteval: fn(input: TypedVal) -> Result<TypedVal, TrapCode>,
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

    /// Returns the [`MemArg`] linear `memory` index and load/store `offset`.
    ///
    /// # Panics
    ///
    /// If the [`MemArg`] offset is not 32-bit.
    fn decode_memarg(memarg: MemArg) -> (index::Memory, u32) {
        let memory = index::Memory::from(memarg.memory);
        let offset = u32::try_from(memarg.offset).unwrap_or_else(|_| {
            panic!(
                "encountered 64-bit memory load/store offset: {}",
                memarg.offset
            )
        });
        (memory, offset)
    }

    /// Returns the effective address `ptr+offset` if it is valid.
    fn effective_address(ptr: u32, offset: u32) -> Option<u32> {
        ptr.checked_add(offset)
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
        make_instr: fn(result: Reg, memory: index::Memory) -> Instruction,
        make_instr_offset16: fn(result: Reg, ptr: Reg, offset: Const16<u32>) -> Instruction,
        make_instr_at: fn(result: Reg, address: u32) -> Instruction,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg);
        let ptr = self.alloc.stack.pop();
        let ptr = match ptr {
            Provider::Register(ptr) => ptr,
            Provider::Const(ptr) => {
                let Some(address) = Self::effective_address(u32::from(ptr), offset) else {
                    return self.translate_trap(TrapCode::MemoryOutOfBounds);
                };
                let result = self.alloc.stack.push_dynamic()?;
                self.push_fueled_instr(make_instr_at(result, address), FuelCosts::load)?;
                if !memory.is_default() {
                    self.alloc
                        .instr_encoder
                        .append_instr(Instruction::memory_index(memory))?;
                }
                return Ok(());
            }
        };
        let result = self.alloc.stack.push_dynamic()?;
        if memory.is_default() {
            if let Ok(offset) = <Const16<u32>>::try_from(offset) {
                self.push_fueled_instr(make_instr_offset16(result, ptr, offset), FuelCosts::load)?;
                return Ok(());
            }
        }
        self.push_fueled_instr(make_instr(result, memory), FuelCosts::load)?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::register_and_imm32(ptr, offset))?;
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
        make_instr: fn(ptr: Reg, memory: index::Memory) -> Instruction,
        make_instr_imm: fn(ptr: Reg, memory: index::Memory) -> Instruction,
        make_instr_offset16: fn(ptr: Reg, offset: u16, value: Reg) -> Instruction,
        make_instr_offset16_imm: fn(ptr: Reg, offset: u16, value: Field) -> Instruction,
        make_instr_at: fn(value: Reg, address: u32) -> Instruction,
        make_instr_at_imm: fn(value: Field, address: u32) -> Instruction,
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
        make_instr: fn(ptr: Reg, memory: index::Memory) -> Instruction,
        make_instr_imm: fn(ptr: Reg, memory: index::Memory) -> Instruction,
        make_instr_offset16: fn(ptr: Reg, offset: u16, value: Reg) -> Instruction,
        make_instr_offset16_imm: fn(ptr: Reg, offset: u16, value: Field) -> Instruction,
        make_instr_at: fn(value: Reg, address: u32) -> Instruction,
        make_instr_at_imm: fn(value: Field, address: u32) -> Instruction,
    ) -> Result<(), Error>
    where
        Src: Copy + Wrap<Wrapped> + From<TypedVal>,
        Field: TryFrom<Wrapped> + Into<AnyConst16>,
    {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg);
        let (ptr, value) = self.alloc.stack.pop2();
        let ptr = match ptr {
            Provider::Register(ptr) => ptr,
            Provider::Const(ptr) => {
                return self.translate_istore_wrap_at::<Src, Wrapped, Field>(
                    memory,
                    u32::from(ptr),
                    offset,
                    value,
                    make_instr_at,
                    make_instr_at_imm,
                )
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
        let (instr, param) = match value {
            TypedProvider::Register(value) => (
                make_instr(ptr, memory),
                Instruction::register_and_imm32(value, offset),
            ),
            TypedProvider::Const(value) => match Field::try_from(Src::from(value).wrap()).ok() {
                Some(value) => (
                    make_instr_imm(ptr, memory),
                    Instruction::imm16_and_imm32(value, offset),
                ),
                None => (
                    make_instr(ptr, memory),
                    Instruction::register_and_imm32(self.alloc.stack.alloc_const(value)?, offset),
                ),
            },
        };
        self.push_fueled_instr(instr, FuelCosts::store)?;
        self.alloc.instr_encoder.append_instr(param)?;
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
        ptr: u32,
        offset: u32,
        value: TypedProvider,
        make_instr_at: fn(value: Reg, address: u32) -> Instruction,
        make_instr_at_imm: fn(value: Field, address: u32) -> Instruction,
    ) -> Result<(), Error>
    where
        Src: Copy + From<TypedVal> + Wrap<Wrapped>,
        Field: TryFrom<Wrapped>,
    {
        let Some(address) = Self::effective_address(ptr, offset) else {
            return self.translate_trap(TrapCode::MemoryOutOfBounds);
        };
        match value {
            Provider::Register(value) => {
                self.push_fueled_instr(make_instr_at(value, address), FuelCosts::store)?;
            }
            Provider::Const(value) => {
                if let Ok(value) = Field::try_from(Src::from(value).wrap()) {
                    self.push_fueled_instr(make_instr_at_imm(value, address), FuelCosts::store)?;
                } else {
                    let value = self.alloc.stack.alloc_const(value)?;
                    self.push_fueled_instr(make_instr_at(value, address), FuelCosts::store)?;
                }
            }
        }
        if !memory.is_default() {
            self.alloc
                .instr_encoder
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
        offset: u32,
        value: TypedProvider,
        make_instr_offset16: fn(Reg, u16, Reg) -> Instruction,
        make_instr_offset16_imm: fn(Reg, u16, Field) -> Instruction,
    ) -> Result<Option<Instr>, Error>
    where
        Src: Copy + From<TypedVal> + Wrap<Wrapped>,
        Field: TryFrom<Wrapped>,
    {
        let Ok(offset16) = u16::try_from(offset) else {
            return Ok(None);
        };
        let instr = match value {
            Provider::Register(value) => {
                self.push_fueled_instr(make_instr_offset16(ptr, offset16, value), FuelCosts::store)?
            }
            Provider::Const(value) => match Field::try_from(Src::from(value).wrap()) {
                Ok(value) => self.push_fueled_instr(
                    make_instr_offset16_imm(ptr, offset16, value),
                    FuelCosts::store,
                )?,
                Err(_) => {
                    let value = self.alloc.stack.alloc_const(value)?;
                    self.push_fueled_instr(
                        make_instr_offset16(ptr, offset16, value),
                        FuelCosts::store,
                    )?
                }
            },
        };
        Ok(Some(instr))
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
        make_instr: fn(ptr: Reg, memory: index::Memory) -> Instruction,
        make_instr_offset16: fn(ptr: Reg, offset: u16, value: Reg) -> Instruction,
        make_instr_at: fn(value: Reg, address: u32) -> Instruction,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg);
        let (ptr, value) = self.alloc.stack.pop2();
        let ptr = match ptr {
            Provider::Register(ptr) => ptr,
            Provider::Const(ptr) => {
                return self.translate_fstore_at(
                    memory,
                    u32::from(ptr),
                    offset,
                    value,
                    make_instr_at,
                )
            }
        };
        let value = self.alloc.stack.provider2reg(&value)?;
        if memory.is_default() {
            if let Ok(offset) = u16::try_from(offset) {
                self.push_fueled_instr(make_instr_offset16(ptr, offset, value), FuelCosts::store)?;
                return Ok(());
            }
        }
        self.push_fueled_instr(make_instr(ptr, memory), FuelCosts::store)?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::register_and_imm32(value, offset))?;
        Ok(())
    }

    /// Translates Wasm float `store` instructions to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This is used in cases where the `ptr` is a known constant value.
    fn translate_fstore_at(
        &mut self,
        memory: index::Memory,
        ptr: u32,
        offset: u32,
        value: TypedProvider,
        make_instr_at: fn(value: Reg, address: u32) -> Instruction,
    ) -> Result<(), Error> {
        let Some(address) = Self::effective_address(ptr, offset) else {
            return self.translate_trap(TrapCode::MemoryOutOfBounds);
        };
        let value = self.alloc.stack.provider2reg(&value)?;
        self.push_fueled_instr(make_instr_at(value, address), FuelCosts::store)?;
        if !memory.is_default() {
            self.alloc
                .instr_encoder
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
    /// - Properly chooses the correct `select` instruction encoding and optimizes for
    ///   cases with 32-bit constant values.
    fn translate_select(&mut self, type_hint: Option<ValType>) -> Result<(), Error> {
        bail_unreachable!(self);
        let (lhs, rhs, condition) = self.alloc.stack.pop3();
        let condition = match condition {
            Provider::Register(condition) => {
                // TODO: technically we could look through function local constant values here.
                condition
            }
            Provider::Const(condition) => {
                // Optimization: since condition is a constant value we can const-fold the `select`
                //               instruction and simply push the selected value back to the provider stack.
                let selected = match i32::from(condition) != 0 {
                    true => lhs,
                    false => rhs,
                };
                if let Provider::Register(reg) = selected {
                    if matches!(
                        self.alloc.stack.get_register_space(reg),
                        RegisterSpace::Dynamic | RegisterSpace::Preserve
                    ) {
                        // Case: constant propagating a dynamic or preserved register might overwrite it in
                        //       future instruction translation steps and thus we may require a copy instruction
                        //       to prevent this from happening.
                        let result = self.alloc.stack.push_dynamic()?;
                        let fuel_info = self.fuel_info();
                        self.alloc.instr_encoder.encode_copy(
                            &mut self.alloc.stack,
                            result,
                            selected,
                            fuel_info,
                        )?;
                        return Ok(());
                    }
                }
                self.alloc.stack.push_provider(selected)?;
                return Ok(());
            }
        };
        if lhs == rhs {
            // Optimization: both `lhs` and `rhs` either are the same register or constant values and
            //               thus `select` will always yield this same value irrespective of the condition.
            //
            // TODO: we could technically look through registers representing function local constants and
            //       check whether they are equal to a given constant in cases where `lhs` and `rhs` are referring
            //       to a function local register and a constant value or vice versa.
            self.alloc.stack.push_provider(lhs)?;
            return Ok(());
        }
        let type_infer = match (lhs, rhs) {
            (Provider::Register(lhs), Provider::Register(rhs)) => {
                let result = self.alloc.stack.push_dynamic()?;
                return self.translate_select_regs(result, condition, lhs, rhs);
            }
            (Provider::Register(_), Provider::Const(rhs)) => rhs.ty(),
            (Provider::Const(lhs), Provider::Register(_)) => lhs.ty(),
            (Provider::Const(lhs), Provider::Const(rhs)) => {
                debug_assert_eq!(lhs.ty(), rhs.ty());
                lhs.ty()
            }
        };
        if let Some(type_hint) = type_hint {
            assert_eq!(type_hint, type_infer);
        }
        let result = self.alloc.stack.push_dynamic()?;
        match type_infer {
            ValType::I32 | ValType::F32 => self.translate_select_32(result, condition, lhs, rhs),
            ValType::I64 => self.translate_select_i64(result, condition, lhs, rhs),
            ValType::F64 => self.translate_select_f64(result, condition, lhs, rhs),
            ValType::FuncRef | ValType::ExternRef => {
                self.translate_select_reftype(result, condition, lhs, rhs)
            }
        }
    }

    fn translate_select_regs(
        &mut self,
        result: Reg,
        condition: Reg,
        lhs: Reg,
        rhs: Reg,
    ) -> Result<(), Error> {
        debug_assert_ne!(lhs, rhs);
        self.push_fueled_instr(Instruction::select(result, lhs), FuelCosts::base)?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::register2_ext(condition, rhs))?;
        Ok(())
    }

    fn translate_select_32(
        &mut self,
        result: Reg,
        condition: Reg,
        lhs: Provider<TypedVal>,
        rhs: Provider<TypedVal>,
    ) -> Result<(), Error> {
        debug_assert_ne!(lhs, rhs);
        let (instr, param) = match (lhs, rhs) {
            (Provider::Register(_), Provider::Register(_)) => unreachable!(),
            (Provider::Register(lhs), Provider::Const(rhs)) => {
                debug_assert!(matches!(rhs.ty(), ValType::I32 | ValType::F32));
                (
                    Instruction::select_imm32_rhs(result, lhs),
                    Instruction::register_and_imm32(condition, u32::from(rhs.untyped())),
                )
            }
            (Provider::Const(lhs), Provider::Register(rhs)) => {
                debug_assert!(matches!(lhs.ty(), ValType::I32 | ValType::F32));
                (
                    Instruction::select_imm32_lhs(result, u32::from(lhs.untyped())),
                    Instruction::register2_ext(condition, rhs),
                )
            }
            (Provider::Const(lhs), Provider::Const(rhs)) => {
                debug_assert!(matches!(lhs.ty(), ValType::I32 | ValType::F32));
                debug_assert!(matches!(rhs.ty(), ValType::I32 | ValType::F32));
                (
                    Instruction::select_imm32(result, u32::from(lhs.untyped())),
                    Instruction::register_and_imm32(condition, u32::from(rhs.untyped())),
                )
            }
        };
        self.push_fueled_instr(instr, FuelCosts::base)?;
        self.alloc.instr_encoder.append_instr(param)?;
        Ok(())
    }

    fn translate_select_i64(
        &mut self,
        result: Reg,
        condition: Reg,
        lhs: Provider<TypedVal>,
        rhs: Provider<TypedVal>,
    ) -> Result<(), Error> {
        debug_assert_ne!(lhs, rhs);
        let lhs = match lhs {
            Provider::Register(lhs) => Provider::Register(lhs),
            Provider::Const(lhs) => match <Const32<i64>>::try_from(i64::from(lhs)) {
                Ok(lhs) => Provider::Const(lhs),
                Err(_) => Provider::Register(self.alloc.stack.alloc_const(lhs)?),
            },
        };
        let rhs = match rhs {
            Provider::Register(rhs) => Provider::Register(rhs),
            Provider::Const(rhs) => match <Const32<i64>>::try_from(i64::from(rhs)) {
                Ok(rhs) => Provider::Const(rhs),
                Err(_) => Provider::Register(self.alloc.stack.alloc_const(rhs)?),
            },
        };
        let (instr, param) = match (lhs, rhs) {
            (Provider::Register(lhs), Provider::Register(rhs)) => {
                return self.translate_select_regs(result, condition, lhs, rhs)
            }
            (Provider::Register(lhs), Provider::Const(rhs)) => (
                Instruction::select_i64imm32_rhs(result, lhs),
                Instruction::register_and_imm32(condition, rhs),
            ),
            (Provider::Const(lhs), Provider::Register(rhs)) => (
                Instruction::select_i64imm32_lhs(result, lhs),
                Instruction::register2_ext(condition, rhs),
            ),
            (Provider::Const(lhs), Provider::Const(rhs)) => (
                Instruction::select_i64imm32(result, lhs),
                Instruction::register_and_imm32(condition, rhs),
            ),
        };
        self.push_fueled_instr(instr, FuelCosts::base)?;
        self.alloc.instr_encoder.append_instr(param)?;
        Ok(())
    }

    fn translate_select_f64(
        &mut self,
        result: Reg,
        condition: Reg,
        lhs: Provider<TypedVal>,
        rhs: Provider<TypedVal>,
    ) -> Result<(), Error> {
        debug_assert_ne!(lhs, rhs);
        let lhs = match lhs {
            Provider::Register(lhs) => Provider::Register(lhs),
            Provider::Const(lhs) => match <Const32<f64>>::try_from(f64::from(lhs)) {
                Ok(lhs) => Provider::Const(lhs),
                Err(_) => Provider::Register(self.alloc.stack.alloc_const(lhs)?),
            },
        };
        let rhs = match rhs {
            Provider::Register(rhs) => Provider::Register(rhs),
            Provider::Const(rhs) => match <Const32<f64>>::try_from(f64::from(rhs)) {
                Ok(rhs) => Provider::Const(rhs),
                Err(_) => Provider::Register(self.alloc.stack.alloc_const(rhs)?),
            },
        };
        let (instr, param) = match (lhs, rhs) {
            (Provider::Register(lhs), Provider::Register(rhs)) => {
                return self.translate_select_regs(result, condition, lhs, rhs)
            }
            (Provider::Register(lhs), Provider::Const(rhs)) => (
                Instruction::select_f64imm32_rhs(result, lhs),
                Instruction::register_and_imm32(condition, rhs),
            ),
            (Provider::Const(lhs), Provider::Register(rhs)) => (
                Instruction::select_f64imm32_lhs(result, lhs),
                Instruction::register2_ext(condition, rhs),
            ),
            (Provider::Const(lhs), Provider::Const(rhs)) => (
                Instruction::select_f64imm32(result, lhs),
                Instruction::register_and_imm32(condition, rhs),
            ),
        };
        self.push_fueled_instr(instr, FuelCosts::base)?;
        self.alloc.instr_encoder.append_instr(param)?;
        Ok(())
    }

    fn translate_select_reftype(
        &mut self,
        result: Reg,
        condition: Reg,
        lhs: Provider<TypedVal>,
        rhs: Provider<TypedVal>,
    ) -> Result<(), Error> {
        debug_assert_ne!(lhs, rhs);
        let lhs = match lhs {
            Provider::Register(lhs) => lhs,
            Provider::Const(lhs) => self.alloc.stack.alloc_const(lhs)?,
        };
        let rhs: Reg = match rhs {
            Provider::Register(rhs) => rhs,
            Provider::Const(rhs) => self.alloc.stack.alloc_const(rhs)?,
        };
        self.translate_select_regs(result, condition, lhs, rhs)
    }

    /// Translates a Wasm `reinterpret` instruction.
    fn translate_reinterpret(&mut self, ty: ValType) -> Result<(), Error> {
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

    /// Translates a Wasm `i64.extend_i32_u` instruction.
    fn translate_i64_extend_i32_u(&mut self) -> Result<(), Error> {
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
        debug_assert_eq!(value.ty(), ValType::I32);
        self.alloc.stack.push_const(u64::from(u32::from(value)));
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
    fn translate_return_if(&mut self, condition: Reg) -> Result<(), Error> {
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

    /// Create either [`Instruction::CallIndirectParams`] or [`Instruction::CallIndirectParamsImm16`] depending on the inputs.
    fn call_indirect_params(
        &mut self,
        index: Provider<TypedVal>,
        table_index: u32,
    ) -> Result<Instruction, Error> {
        let instr = match index {
            TypedProvider::Const(index) => match <Const16<u32>>::try_from(u32::from(index)).ok() {
                Some(index) => {
                    // Case: the index is encodable as 16-bit constant value
                    //       which allows us to use an optimized instruction.
                    Instruction::call_indirect_params_imm16(index, table_index)
                }
                None => {
                    // Case: the index is not encodable as 16-bit constant value
                    //       and we need to allocate it as function local constant.
                    let index = self.alloc.stack.alloc_const(index)?;
                    Instruction::call_indirect_params(index, table_index)
                }
            },
            TypedProvider::Register(index) => Instruction::call_indirect_params(index, table_index),
        };
        Ok(instr)
    }

    /// Translates a Wasm `br` instruction with its `relative_depth`.
    fn translate_br(&mut self, relative_depth: u32) -> Result<(), Error> {
        let engine = self.engine().clone();
        match self.alloc.control_stack.acquire_target(relative_depth) {
            AcquiredTarget::Return(_frame) => self.translate_return(),
            AcquiredTarget::Branch(frame) => {
                frame.bump_branches();
                let branch_dst = frame.branch_destination();
                let branch_params = frame.branch_params(&engine);
                self.translate_copy_branch_params(branch_params)?;
                let branch_offset = self.alloc.instr_encoder.try_resolve_label(branch_dst)?;
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
        let values = core::mem::take(&mut self.alloc.buffer.providers);
        let result = f(self, &values[..]);
        let _ = core::mem::replace(&mut self.alloc.buffer.providers, values);
        result
    }

    /// Translates a Wasm `br_table` instruction with its branching targets.
    fn translate_br_table(&mut self, table: wasmparser::BrTable) -> Result<(), Error> {
        let engine = self.engine().clone();
        let index = self.alloc.stack.pop();
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
        let targets = &mut self.alloc.buffer.br_table_targets;
        Self::populate_br_table_buffer(targets, &table)?;
        if targets.iter().all(|&target| target == default_target) {
            // Case: all targets are the same and thus the `br_table` is equal to a `br`.
            return self.translate_br(default_target);
        }
        // Note: The Wasm spec mandates that all `br_table` targets manipulate the
        //       Wasm value stack the same. This implies for Wasmi that all `br_table`
        //       targets have the same branch parameter arity.
        let branch_params = self
            .alloc
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
        let targets = &self.alloc.buffer.br_table_targets;
        for &target in targets {
            match self.alloc.control_stack.acquire_target(target) {
                AcquiredTarget::Return(_) => {
                    self.alloc.instr_encoder.encode_return(
                        &mut self.alloc.stack,
                        values,
                        fuel_info,
                    )?;
                }
                AcquiredTarget::Branch(frame) => {
                    frame.bump_branches();
                    let branch_params = frame.branch_params(&engine);
                    let branch_dst = frame.branch_destination();
                    let branch_offset = self.alloc.instr_encoder.try_resolve_label(branch_dst)?;
                    let instr = match branch_params.len() {
                        0 => Instruction::branch(branch_offset),
                        1..=3 => {
                            Instruction::branch_table_target(branch_params.span(), branch_offset)
                        }
                        _ => make_target(branch_params, branch_offset),
                    };
                    self.alloc.instr_encoder.append_instr(instr)?;
                }
            }
        }
        Ok(())
    }

    /// Translates a Wasm `br_table` instruction without inputs.
    fn translate_br_table_0(&mut self, index: Reg) -> Result<(), Error> {
        let targets = &self.alloc.buffer.br_table_targets;
        let len_targets = targets.len() as u32;
        self.alloc.instr_encoder.push_fueled_instr(
            Instruction::branch_table_0(index, len_targets),
            self.fuel_info(),
            FuelCosts::base,
        )?;
        self.translate_br_table_targets_simple(&[])?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `br_table` instruction with a single input.
    fn translate_br_table_1(&mut self, index: Reg) -> Result<(), Error> {
        let targets = &self.alloc.buffer.br_table_targets;
        let len_targets = targets.len() as u32;
        let fuel_info = self.fuel_info();
        self.alloc.instr_encoder.push_fueled_instr(
            Instruction::branch_table_1(index, len_targets),
            fuel_info,
            FuelCosts::base,
        )?;
        let stack = &mut self.alloc.stack;
        let value = stack.pop();
        let param_instr = match value {
            TypedProvider::Register(register) => Instruction::register(register),
            TypedProvider::Const(immediate) => match immediate.ty() {
                ValType::I32 | ValType::F32 => Instruction::const32(u32::from(immediate.untyped())),
                ValType::I64 => match <Const32<i64>>::try_from(i64::from(immediate)) {
                    Ok(value) => Instruction::i64const32(value),
                    Err(_) => {
                        let register = self.alloc.stack.provider2reg(&value)?;
                        Instruction::register(register)
                    }
                },
                ValType::F64 => match <Const32<f64>>::try_from(f64::from(immediate)) {
                    Ok(value) => Instruction::f64const32(value),
                    Err(_) => {
                        let register = self.alloc.stack.provider2reg(&value)?;
                        Instruction::register(register)
                    }
                },
                ValType::ExternRef | ValType::FuncRef => {
                    let register = self.alloc.stack.provider2reg(&value)?;
                    Instruction::register(register)
                }
            },
        };
        self.alloc.instr_encoder.append_instr(param_instr)?;
        self.translate_br_table_targets_simple(&[value])?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `br_table` instruction with exactly two inputs.
    fn translate_br_table_2(&mut self, index: Reg) -> Result<(), Error> {
        let targets = &self.alloc.buffer.br_table_targets;
        let len_targets = targets.len() as u32;
        let fuel_info = self.fuel_info();
        self.alloc.instr_encoder.push_fueled_instr(
            Instruction::branch_table_2(index, len_targets),
            fuel_info,
            FuelCosts::base,
        )?;
        let stack = &mut self.alloc.stack;
        let (v0, v1) = stack.pop2();
        self.alloc
            .instr_encoder
            .append_instr(Instruction::register2_ext(
                stack.provider2reg(&v0)?,
                stack.provider2reg(&v1)?,
            ))?;
        self.translate_br_table_targets_simple(&[v0, v1])?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `br_table` instruction with exactly three inputs.
    fn translate_br_table_3(&mut self, index: Reg) -> Result<(), Error> {
        let targets = &self.alloc.buffer.br_table_targets;
        let len_targets = targets.len() as u32;
        let fuel_info = self.fuel_info();
        self.alloc.instr_encoder.push_fueled_instr(
            Instruction::branch_table_3(index, len_targets),
            fuel_info,
            FuelCosts::base,
        )?;
        let stack = &mut self.alloc.stack;
        let (v0, v1, v2) = stack.pop3();
        self.alloc
            .instr_encoder
            .append_instr(Instruction::register3_ext(
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
        let values = &mut self.alloc.buffer.providers;
        self.alloc.stack.pop_n(usize::from(len_values), values);
        match BoundedRegSpan::from_providers(values) {
            Some(span) => self.translate_br_table_span(index, span),
            None => self.translate_br_table_many(index),
        }
    }

    /// Translates a Wasm `br_table` instruction with 4 or more inputs that form a [`RegSpan`].
    fn translate_br_table_span(&mut self, index: Reg, values: BoundedRegSpan) -> Result<(), Error> {
        debug_assert!(values.len() > 3);
        let fuel_info = self.fuel_info();
        let targets = &mut self.alloc.buffer.br_table_targets;
        let len_targets = targets.len() as u32;
        self.alloc.instr_encoder.push_fueled_instr(
            Instruction::branch_table_span(index, len_targets),
            fuel_info,
            FuelCosts::base,
        )?;
        self.alloc
            .instr_encoder
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
        let targets = &mut self.alloc.buffer.br_table_targets;
        let len_targets = targets.len() as u32;
        let fuel_info = self.fuel_info();
        self.alloc.instr_encoder.push_fueled_instr(
            Instruction::branch_table_many(index, len_targets),
            fuel_info,
            FuelCosts::base,
        )?;
        let stack = &mut self.alloc.stack;
        let values = &self.alloc.buffer.providers[..];
        debug_assert!(values.len() > 3);
        self.alloc
            .instr_encoder
            .encode_register_list(stack, values)?;
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
}

trait BumpFuelConsumption {
    /// Increases the fuel consumption of the [`Instruction::ConsumeFuel`] instruction by `delta`.
    ///
    /// # Error
    ///
    /// - If `self` is not a [`Instruction::ConsumeFuel`] instruction.
    /// - If the new fuel consumption overflows the internal `u64` value.
    fn bump_fuel_consumption(&mut self, delta: u64) -> Result<(), Error>;
}

impl BumpFuelConsumption for Instruction {
    fn bump_fuel_consumption(&mut self, delta: u64) -> Result<(), Error> {
        match self {
            Self::ConsumeFuel { block_fuel } => block_fuel.bump_by(delta).map_err(Into::into),
            instr => panic!("expected `Instruction::ConsumeFuel` but found: {instr:?}"),
        }
    }
}
