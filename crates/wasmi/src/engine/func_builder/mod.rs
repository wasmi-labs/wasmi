mod control_frame;
mod control_stack;
mod error;
mod inst_builder;
mod labels;
mod locals_registry;
mod translator;
mod value_stack;

use self::{
    control_frame::{BlockControlFrame, ControlFrame},
    control_stack::ControlFlowStack,
    locals_registry::LocalsRegistry,
    translator::FuncTranslator,
};
pub use self::{
    error::TranslationError,
    inst_builder::{Instr, InstructionsBuilder, RelativeDepth},
};
use super::{FuncBody, Instruction};
use crate::{
    module::{BlockType, FuncIdx, ModuleResources, ReusableAllocations},
    Engine,
};
use alloc::vec::Vec;
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
    /// The underlying Wasm to `wasmi` bytecode translator.
    translator: FuncTranslator<'parser>,
}

/// Reusable allocations of a [`FuncBuilder`].
#[derive(Debug, Default)]
pub struct FunctionBuilderAllocations {
    /// The control flow frame stack that represents the Wasm control flow.
    control_frames: ControlFlowStack,
    /// The instruction builder.
    ///
    /// # Note
    ///
    /// Allows to incrementally construct the instruction of a function.
    inst_builder: InstructionsBuilder,
    /// Buffer for translating `br_table`.
    br_table_branches: Vec<Instruction>,
}

impl FunctionBuilderAllocations {
    /// Resets the data structures of the [`FunctionBuilderAllocations`].
    ///
    /// # Note
    ///
    /// This must be called before reusing this [`FunctionBuilderAllocations`]
    /// by another [`FuncBuilder`].
    fn reset(&mut self) {
        self.control_frames.reset();
        self.inst_builder.reset();
        self.br_table_branches.clear();
    }
}

impl<'parser> FuncBuilder<'parser> {
    /// Creates a new [`FuncBuilder`].
    pub fn new(
        engine: &Engine,
        func: FuncIdx,
        res: ModuleResources<'parser>,
        validator: FuncValidator,
        allocations: FunctionBuilderAllocations,
    ) -> Self {
        let mut allocations = allocations;
        allocations.reset();
        let mut locals = LocalsRegistry::default();
        Self::register_func_body_block(func, res, &mut allocations);
        Self::register_func_params(func, res, &mut locals);
        Self {
            pos: 0,
            validator,
            translator: FuncTranslator::new(engine, func, res, allocations),
        }
    }

    /// Registers the `block` control frame surrounding the entire function body.
    fn register_func_body_block(
        func: FuncIdx,
        res: ModuleResources<'parser>,
        allocations: &mut FunctionBuilderAllocations,
    ) {
        let func_type = res.get_type_of_func(func);
        let block_type = BlockType::func_type(func_type);
        let end_label = allocations.inst_builder.new_label();
        let block_frame = BlockControlFrame::new(block_type, end_label, 0);
        allocations.control_frames.push_frame(block_frame);
    }

    /// Registers the function parameters in the emulated value stack.
    fn register_func_params(
        func: FuncIdx,
        res: ModuleResources<'parser>,
        locals: &mut LocalsRegistry,
    ) -> usize {
        let dedup_func_type = res.get_type_of_func(func);
        let func_type = res
            .engine()
            .resolve_func_type(dedup_func_type, Clone::clone);
        let params = func_type.params();
        for _param_type in params {
            locals.register_locals(1);
        }
        params.len()
    }

    /// Translates the given local variables for the translated function.
    pub fn translate_locals(
        &mut self,
        offset: usize,
        amount: u32,
        value_type: wasmparser::ValType,
    ) -> Result<(), TranslationError> {
        self.validator.define_locals(offset, amount, value_type)?;
        self.translator.register_locals(amount);
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

    /// Finishes constructing the function and returns its [`FuncBody`].
    pub fn finish(
        mut self,
        offset: usize,
    ) -> Result<(FuncBody, ReusableAllocations), TranslationError> {
        self.validator.finish(offset)?;
        let func_body = self.translator.finish()?;
        let allocations = ReusableAllocations {
            translation: self.translator.into_allocations(),
            validation: self.validator.into_allocations(),
        };
        Ok((func_body, allocations))
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
            let arg_cloned = $arg.clone();
            self.validate_then_translate(
                |validator| validator.visitor(offset).$visit(arg_cloned),
                |translator| translator.$visit($arg),
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
    ( @@supported $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        fn $visit(&mut self $($(,$arg: $argty)*)?) -> Self::Output {
            let offset = self.current_pos();
            self.validate_then_translate(
                |v| v.visitor(offset).$visit($($($arg),*)?),
                |t| t.$visit($($($arg),*)?),
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
