use super::CompiledFunc;
pub use crate::engine::TranslationError;
use crate::{
    engine::regmach::{FuncTranslator, FuncTranslatorAllocations},
    module::{FuncIdx, ModuleResources, ReusableAllocations},
};
use wasmparser::{BinaryReaderError, VisitOperator};

/// The used function validator type.
type FuncValidator = wasmparser::FuncValidator<wasmparser::ValidatorResources>;

/// The interface to build a `wasmi` bytecode function using Wasm bytecode.
///
/// # Note
///
/// This includes validation of the incoming Wasm bytecode.
pub struct FuncBuilder<'parser> {
    /// The current position in the Wasm binary while parsing operators.
    pos: usize,
    /// The Wasm function validator.
    validator: FuncValidator,
    /// The chosen function translator.
    translator: FuncTranslator<'parser>,
}

impl<'parser> FuncBuilder<'parser> {
    /// Creates a new [`FuncBuilder`].
    pub fn new(
        func: FuncIdx,
        compiled_func_2: CompiledFunc,
        res: ModuleResources<'parser>,
        validator: FuncValidator,
        allocations: FuncTranslatorAllocations,
    ) -> Result<Self, TranslationError> {
        let translator = FuncTranslator::new(func, compiled_func_2, res, allocations)?;
        Ok(Self {
            pos: 0,
            validator,
            translator,
        })
    }

    /// Translates the given local variables for the translated function.
    pub fn translate_locals(
        &mut self,
        offset: usize,
        amount: u32,
        value_type: wasmparser::ValType,
    ) -> Result<(), TranslationError> {
        self.validator.define_locals(offset, amount, value_type)?;
        self.translator.register_locals(amount)?;
        Ok(())
    }

    /// This informs the [`FuncBuilder`] that the function header translation is finished.
    ///
    /// # Note
    ///
    /// This was introduced to properly calculate the fuel costs for all local variables
    /// and function parameters. After this function call no more locals and parameters may
    /// be added to this function translation.
    pub fn finish_translate_locals(&mut self) -> Result<(), TranslationError> {
        self.translator.finish_translate_locals()?;
        Ok(())
    }

    /// Updates the current position within the Wasm binary while parsing operators.
    pub fn update_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    /// Returns the current position within the Wasm binary while parsing operators.
    pub fn current_pos(&self) -> usize {
        self.pos
    }

    /// Finishes constructing the function by initializing its [`CompiledFunc`].
    pub fn finish(mut self, offset: usize) -> Result<ReusableAllocations, TranslationError> {
        self.validator.finish(offset)?;
        self.translator.finish()?;
        let translation = self.translator.into_allocations();
        let validation = self.validator.into_allocations();
        let allocations = ReusableAllocations {
            translation,
            validation,
        };
        Ok(allocations)
    }

    /// Translates into `wasmi` bytecode if the current code path is reachable.
    fn validate_then_translate<V, T>(
        &mut self,
        validate: V,
        translate: T,
    ) -> Result<(), TranslationError>
    where
        V: FnOnce(&mut FuncValidator) -> Result<(), BinaryReaderError>,
        T: FnOnce(&mut FuncTranslator<'parser>) -> Result<(), TranslationError>,
    {
        validate(&mut self.validator)?;
        translate(&mut self.translator)?;
        Ok(())
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
                |translator| translator.$visit($arg.clone()).map_err(Into::into),
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
                move |translator| translator.$visit($($($arg),*)?).map_err(Into::into),
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

impl<'a> VisitOperator<'a> for FuncBuilder<'a> {
    type Output = Result<(), TranslationError>;

    wasmparser::for_each_operator!(impl_visit_operator);
}