//! Function translation for the register-machine bytecode based Wasmi engine.

mod comparator;
mod driver;
mod error;
mod func;
mod labels;
mod relink_result;
mod utils;
mod visit_register;

#[cfg(test)]
mod tests;

#[cfg(doc)]
use crate::Engine;

pub use self::{
    driver::FuncTranslationDriver,
    error::TranslationError,
    func::{FuncTranslator, FuncTranslatorAllocations},
};
use super::code_map::CompiledFuncEntity;
use crate::{
    engine::EngineFunc,
    module::{FuncIdx, ModuleHeader},
    Error,
};
use alloc::vec::Vec;
use core::{fmt, mem};
use wasmparser::{
    BinaryReaderError,
    FuncToValidate,
    FuncValidatorAllocations,
    ValidatorResources,
    VisitOperator,
    WasmFeatures,
};

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

/// Convenience trait used to circumvent the need for `#[cfg]` where bounds.
///
/// Wasm `simd` is disabled, thus this trait is empty.
#[cfg(not(feature = "simd"))]
pub trait VisitSimdOperator<'a> {}
#[cfg(not(feature = "simd"))]
impl<'a, T> VisitSimdOperator<'a> for T where T: WasmTranslator<'a> {}

/// Convenience trait used to circumvent the need for `#[cfg]` where bounds.
///
/// Wasm `simd` is enabled, thus this trait forwards to [`wasmparser::VisitSimdOperator`].
#[cfg(feature = "simd")]
pub trait VisitSimdOperator<'a>:
    wasmparser::VisitSimdOperator<'a, Output = Result<(), Error>>
{
}
#[cfg(feature = "simd")]
impl<'a, T> VisitSimdOperator<'a> for T where
    T: WasmTranslator<'a> + wasmparser::VisitSimdOperator<'a, Output = Result<(), Error>>
{
}

/// A WebAssembly (Wasm) function translator.
pub trait WasmTranslator<'parser>:
    VisitOperator<'parser, Output = Result<(), Error>> + VisitSimdOperator<'parser>
{
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

    /// Returns a reference to the [`WasmFeatures`] used by the [`WasmTranslator`].
    fn features(&self) -> WasmFeatures;

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

    fn features(&self) -> WasmFeatures {
        self.translator.features()
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

    fn finish(self, finalize: impl FnOnce(CompiledFuncEntity)) -> Result<Self::Allocations, Error> {
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
    ( @mvp BrTable { $arg:ident: $argty:ty } => $visit:ident $_ann:tt $($rest:tt)* ) => {
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
    ( @reference_types TypedSelectMulti { tys: $tys_type:ty } => $visit:ident $_ann:tt $($rest:tt)* ) => {
        fn $visit(&mut self, tys: $tys_type) -> Self::Output {
            let offset = self.current_pos();
            let tys_cloned = tys.clone();
            self.validate_then_translate(
                move |validator| validator.visitor(offset).$visit(tys_cloned),
                move |translator| translator.$visit(tys),
            )
        }
        impl_visit_operator!($($rest)*);
    };
    ( @reference_types $($rest:tt)* ) => {
        impl_visit_operator!(@@supported $($rest)*);
    };
    ( @tail_call $($rest:tt)* ) => {
        impl_visit_operator!(@@supported $($rest)*);
    };
    ( @wide_arithmetic $($rest:tt)* ) => {
        impl_visit_operator!(@@supported $($rest)*);
    };
    ( @@supported $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $_ann:tt $($rest:tt)* ) => {
        fn $visit(&mut self $($(,$arg: $argty)*)?) -> Self::Output {
            let offset = self.current_pos();
            self.validate_then_translate(
                move |validator| validator.visitor(offset).$visit($($($arg),*)?),
                move |translator| translator.$visit($($($arg),*)?),
            )
        }
        impl_visit_operator!($($rest)*);
    };
    ( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $ann:tt $($rest:tt)* ) => {
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

    #[cfg(feature = "simd")]
    fn simd_visitor(
        &mut self,
    ) -> Option<&mut dyn wasmparser::VisitSimdOperator<'a, Output = Self::Output>> {
        Some(self)
    }

    wasmparser::for_each_visit_operator!(impl_visit_operator);
}

#[cfg(feature = "simd")]
macro_rules! impl_visit_simd_operator {
    ( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $_ann:tt $($rest:tt)* ) => {
        fn $visit(&mut self $($(,$arg: $argty)*)?) -> Self::Output {
            let offset = self.current_pos();
            self.validate_then_translate(
                move |validator| validator.simd_visitor(offset).$visit($($($arg),*)?),
                move |translator| translator.$visit($($($arg),*)?),
            )
        }
        impl_visit_simd_operator!($($rest)*);
    };
    () => {};
}

#[cfg(feature = "simd")]
impl<'a, T> wasmparser::VisitSimdOperator<'a> for ValidatingFuncTranslator<T>
where
    T: WasmTranslator<'a>,
{
    wasmparser::for_each_visit_simd_operator!(impl_visit_simd_operator);
}

/// A lazy Wasm function translator that defers translation when the function is first used.
#[derive(Debug)]
pub struct LazyFuncTranslator {
    /// The index of the lazily compiled function within its module.
    func_idx: FuncIdx,
    /// The identifier of the to be compiled function.
    engine_func: EngineFunc,
    /// The Wasm module header information used for translation.
    module: ModuleHeader,
    /// Information about Wasm validation during lazy translation.
    validation: Validation,
}

/// Information about Wasm validation for lazy translation.
enum Validation {
    /// Wasm validation is performed.
    Checked(FuncToValidate<ValidatorResources>),
    /// Wasm validation is checked.
    ///
    /// # Dev. Note
    ///
    /// We still need Wasm features to properly parse the Wasm.
    Unchecked(WasmFeatures),
}

impl Validation {
    /// Returns `true` if `self` performs validates Wasm upon lazy translation.
    pub fn is_checked(&self) -> bool {
        matches!(self, Self::Checked(_))
    }

    /// Returns the [`WasmFeatures`] used for Wasm parsing and validation.
    pub fn features(&self) -> WasmFeatures {
        match self {
            Validation::Checked(func_to_validate) => func_to_validate.features,
            Validation::Unchecked(wasm_features) => *wasm_features,
        }
    }

    /// Returns the [`FuncToValidate`] if `self` is checked.
    pub fn take_func_to_validate(&mut self) -> Option<FuncToValidate<ValidatorResources>> {
        let features = self.features();
        match mem::replace(self, Self::Unchecked(features)) {
            Self::Checked(func_to_validate) => Some(func_to_validate),
            Self::Unchecked(_) => None,
        }
    }
}

impl fmt::Debug for Validation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LazyFuncTranslator")
            .field("validate", &self.is_checked())
            .field("features", &self.features())
            .finish()
    }
}

impl LazyFuncTranslator {
    /// Create a new [`LazyFuncTranslator`].
    pub fn new(
        func_idx: FuncIdx,
        engine_func: EngineFunc,
        module: ModuleHeader,
        func_to_validate: FuncToValidate<ValidatorResources>,
    ) -> Self {
        Self {
            func_idx,
            engine_func,
            module,
            validation: Validation::Checked(func_to_validate),
        }
    }

    /// Create a new [`LazyFuncTranslator`] that does not validate Wasm upon lazy translation.
    pub fn new_unchecked(
        func_idx: FuncIdx,
        engine_func: EngineFunc,
        module: ModuleHeader,
        features: WasmFeatures,
    ) -> Self {
        Self {
            func_idx,
            engine_func,
            module,
            validation: Validation::Unchecked(features),
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
                self.validation.take_func_to_validate(),
            );
        Ok(true)
    }

    #[inline]
    fn features(&self) -> WasmFeatures {
        self.validation.features()
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
    ( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $ann:tt $($rest:tt)* ) => {
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

    #[cfg(feature = "simd")]
    fn simd_visitor(
        &mut self,
    ) -> Option<&mut dyn wasmparser::VisitSimdOperator<'a, Output = Self::Output>> {
        Some(self)
    }

    wasmparser::for_each_visit_operator!(impl_visit_operator);
}

#[cfg(feature = "simd")]
impl wasmparser::VisitSimdOperator<'_> for LazyFuncTranslator {
    wasmparser::for_each_visit_simd_operator!(impl_visit_operator);
}
