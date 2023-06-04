//! Register-machine bytecode Wasm function body translator.

#![allow(unused_imports)] // TODO: remove

use super::{instr_encoder::InstrEncoder, register_alloc::RegisterAlloc, ProviderStack};
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
        bytecode2::Instruction,
        config::FuelCosts,
        func_builder::control_frame::ControlFrameKind,
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
    /// The stack of input locals or constants during translation.
    providers: ProviderStack,
    /// The instruction sequence encoder.
    instr_encoder: InstrEncoder,
    /// The register allocator.
    reg_alloc: RegisterAlloc,
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
    ) -> Self {
        Self {
            func,
            compiled_func,
            res,
            reachable: true,
            alloc,
        }
    }

    /// Consumes `self` and returns the underlying reusable [`FuncTranslatorAllocations`].
    pub fn into_allocations(self) -> FuncTranslatorAllocations {
        self.alloc
    }

    /// Registers an `amount` of local variables.
    ///
    /// # Panics
    ///
    /// If too many local variables have been registered.
    pub fn register_locals(&mut self, amount: u32) -> Result<(), TranslationError> {
        self.alloc.reg_alloc.register_locals(amount)
    }

    /// This informs the [`FuncTranslator`] that the function header translation is finished.
    ///
    /// # Note
    ///
    /// This was introduced to properly calculate the fuel costs for all local variables
    /// and function parameters. After this function call no more locals and parameters may
    /// be added to this function translation.
    pub fn finish_translate_locals(&mut self) -> Result<(), TranslationError> {
        Ok(()) // TODO: not needed at the moment. We might be able to remove this entirely.
    }

    /// Finishes constructing the function and returns its [`CompiledFunc`].
    pub fn finish(&mut self) -> Result<(), TranslationError> {
        self.alloc.reg_alloc.defrag(&mut self.alloc.instr_encoder);
        todo!()
    }
}
