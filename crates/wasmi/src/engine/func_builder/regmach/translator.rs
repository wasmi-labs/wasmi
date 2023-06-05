//! Register-machine bytecode Wasm function body translator.

#![allow(unused_imports)] // TODO: remove

use super::{
    control_frame::BlockControlFrame,
    instr_encoder::InstrEncoder,
    register_alloc::RegisterAlloc,
    ControlStack,
    ProviderStack,
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
    /// The control stack.
    control_stack: ControlStack,
}

impl FuncTranslatorAllocations {
    /// Resets the [`FuncTranslatorAllocations`].
    fn reset(&mut self) {
        self.providers.reset();
        self.instr_encoder.reset();
        self.reg_alloc.reset();
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
            self.alloc.reg_alloc.register_locals(1)?;
        }
        Ok(())
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
        // TODO: not needed at the moment since we are required to
        //       calculate all required registers in order to properly
        //       adjust the consume fuel instruction of the function entry
        //       block. However, this only is determined at the end of the
        //       translation process. Therefore we might not really need
        //       this method at all and maybe can remove this later.
        Ok(())
    }

    /// Finishes constructing the function and returns its [`CompiledFunc`].
    pub fn finish(&mut self) -> Result<(), TranslationError> {
        self.alloc.reg_alloc.defrag(&mut self.alloc.instr_encoder);
        self.alloc.instr_encoder.update_branch_offsets()?;
        let len_registers = self.alloc.reg_alloc.len_registers();
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
}
