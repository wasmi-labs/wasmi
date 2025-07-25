#![expect(dead_code)]

#[macro_use]
mod utils;
mod instrs;
mod layout;
mod op;
#[cfg(feature = "simd")]
mod simd;
mod stack;
mod visit;

use self::{
    instrs::{InstrEncoder, InstrEncoderAllocations},
    layout::{StackLayout, StackSpace},
    stack::{
        BlockControlFrame,
        ControlFrame,
        ControlFrameBase,
        ControlFrameKind,
        ElseControlFrame,
        ElseReachability,
        IfControlFrame,
        IfReachability,
        ImmediateOperand,
        LocalIdx,
        LoopControlFrame,
        Operand,
        OperandIdx,
        Stack,
        StackAllocations,
        TempOperand,
    },
    utils::{Input, Input16, Input32, Reset, ReusableAllocations},
};
use crate::{
    core::{FuelCostsProvider, IndexType, TrapCode, Typed, TypedVal, UntypedVal, ValType},
    engine::{
        translator::{
            comparator::{
                CompareResult as _,
                LogicalizeCmpInstr,
                NegateCmpInstr,
                ReplaceCmpResult,
                TryIntoCmpBranchInstr as _,
            },
            labels::{LabelRef, LabelRegistry},
            utils::{Instr, WasmFloat, WasmInteger, Wrap},
            WasmTranslator,
        },
        BlockType,
        CompiledFuncEntity,
        TranslationError,
    },
    ir::{
        index,
        Address,
        Address32,
        AnyConst16,
        BoundedRegSpan,
        BranchOffset,
        BranchOffset16,
        Comparator,
        ComparatorAndOffset,
        Const16,
        Const32,
        FixedRegSpan,
        Instruction,
        IntoShiftAmount,
        Offset16,
        Offset64,
        Offset64Lo,
        Reg,
        RegSpan,
        Sign,
    },
    module::{FuncIdx, FuncTypeIdx, MemoryIdx, ModuleHeader, TableIdx, WasmiValueType},
    Engine,
    Error,
    FuncType,
};
use alloc::vec::Vec;
use core::mem;
use wasmparser::{MemArg, WasmFeatures};

/// Type concerned with translating from Wasm bytecode to Wasmi bytecode.
#[derive(Debug)]
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
    fuel_costs: Option<FuelCostsProvider>,
    /// Wasm value and control stack.
    stack: Stack,
    /// Wasm layout to map stack slots to Wasmi registers.
    layout: StackLayout,
    /// Registers and pins labels and tracks their users.
    labels: LabelRegistry,
    /// Constructs and encodes function instructions.
    instrs: InstrEncoder,
    /// Temporary buffer for operands.
    operands: Vec<Operand>,
    /// Temporary buffer for immediate values.
    immediates: Vec<TypedVal>,
}

/// Heap allocated data structured used by the [`FuncTranslator`].
#[derive(Debug, Default)]
pub struct FuncTranslatorAllocations {
    /// Wasm value and control stack.
    stack: StackAllocations,
    /// Wasm layout to map stack slots to Wasmi registers.
    layout: StackLayout,
    /// Registers and pins labels and tracks their users.
    labels: LabelRegistry,
    /// Constructs and encodes function instructions.
    instrs: InstrEncoderAllocations,
    /// Temporary buffer for operands.
    operands: Vec<Operand>,
    /// Temporary buffer for immediate values.
    immediates: Vec<TypedVal>,
}

impl Reset for FuncTranslatorAllocations {
    fn reset(&mut self) {
        self.stack.reset();
        self.layout.reset();
        self.labels.reset();
        self.instrs.reset();
        self.operands.clear();
        self.immediates.clear();
    }
}

impl WasmTranslator<'_> for FuncTranslator {
    type Allocations = FuncTranslatorAllocations;

    fn setup(&mut self, _bytes: &[u8]) -> Result<bool, Error> {
        Ok(false)
    }

    #[inline]
    fn features(&self) -> WasmFeatures {
        self.engine.config().wasm_features()
    }

    fn translate_locals(
        &mut self,
        amount: u32,
        value_type: wasmparser::ValType,
    ) -> Result<(), Error> {
        let ty = WasmiValueType::from(value_type).into_inner();
        self.stack.register_locals(amount, ty)?;
        self.layout.register_locals(amount, ty)?;
        Ok(())
    }

    fn finish_translate_locals(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn update_pos(&mut self, _pos: usize) {}

    fn finish(
        mut self,
        finalize: impl FnOnce(CompiledFuncEntity),
    ) -> Result<Self::Allocations, Error> {
        let Some(frame_size) = self.frame_size() else {
            return Err(Error::from(TranslationError::AllocatedTooManyRegisters));
        };
        self.update_branch_offsets()?;
        finalize(CompiledFuncEntity::new(
            frame_size,
            self.instrs.drain(),
            self.layout.consts(),
        ));
        Ok(self.into_allocations())
    }
}

impl ReusableAllocations for FuncTranslator {
    type Allocations = FuncTranslatorAllocations;

    fn into_allocations(self) -> Self::Allocations {
        Self::Allocations {
            stack: self.stack.into_allocations(),
            layout: self.layout,
            labels: self.labels,
            instrs: self.instrs.into_allocations(),
            operands: self.operands,
            immediates: self.immediates,
        }
    }
}

impl FuncTranslator {
    /// Creates a new [`FuncTranslator`].
    pub fn new(
        func: FuncIdx,
        module: ModuleHeader,
        alloc: FuncTranslatorAllocations,
    ) -> Result<Self, Error> {
        let Some(engine) = module.engine().upgrade() else {
            panic!(
                "cannot compile function since engine does no longer exist: {:?}",
                module.engine()
            )
        };
        let config = engine.config();
        let fuel_costs = config
            .get_consume_fuel()
            .then(|| config.fuel_costs())
            .cloned();
        let FuncTranslatorAllocations {
            stack,
            layout,
            labels,
            instrs,
            operands,
            immediates,
        } = alloc.into_reset();
        let stack = Stack::new(&engine, stack);
        let instrs = InstrEncoder::new(&engine, instrs);
        let mut translator = Self {
            func,
            engine,
            module,
            reachable: true,
            fuel_costs,
            stack,
            layout,
            labels,
            instrs,
            operands,
            immediates,
        };
        translator.init_func_body_block()?;
        translator.init_func_params()?;
        Ok(translator)
    }

    /// Initializes the function body enclosing control block.
    fn init_func_body_block(&mut self) -> Result<(), Error> {
        let func_ty = self.module.get_type_of_func(self.func);
        let block_ty = BlockType::func_type(func_ty);
        let end_label = self.labels.new_label();
        let consume_fuel = self.instrs.push_consume_fuel_instr()?;
        self.stack
            .push_func_block(block_ty, end_label, consume_fuel)?;
        Ok(())
    }

    /// Initializes the function's parameters.
    fn init_func_params(&mut self) -> Result<(), Error> {
        for ty in self.func_type().params() {
            self.stack.register_locals(1, *ty)?;
            self.layout.register_locals(1, *ty)?;
        }
        Ok(())
    }

    /// Returns the frame size of the to-be-compiled function.
    ///
    /// Returns `None` if the frame size is out of bounds.
    fn frame_size(&self) -> Option<u16> {
        let frame_size = self
            .stack
            .max_height()
            .checked_add(self.layout.len_locals())?
            .checked_add(self.layout.consts().len())?;
        u16::try_from(frame_size).ok()
    }

    /// Updates the branch offsets of all branch instructions inplace.
    ///
    /// # Panics
    ///
    /// If this is used before all branching labels have been pinned.
    fn update_branch_offsets(&mut self) -> Result<(), Error> {
        for (user, offset) in self.labels.resolved_users() {
            self.instrs
                .update_branch_offset(user, offset?, &mut self.layout)?;
        }
        Ok(())
    }

    /// Returns the [`FuncType`] of the function that is currently translated.
    fn func_type(&self) -> FuncType {
        self.func_type_with(FuncType::clone)
    }

    /// Applies `f` to the [`FuncType`] of the function that is currently translated.
    fn func_type_with<R>(&self, f: impl FnOnce(&FuncType) -> R) -> R {
        self.resolve_func_type_with(self.func, f)
    }

    /// Returns the [`FuncType`] of the function at `func_index`.
    fn resolve_func_type(&self, func_index: FuncIdx) -> FuncType {
        self.resolve_func_type_with(func_index, FuncType::clone)
    }

    /// Applies `f` to the [`FuncType`] of the function at `func_index`.
    fn resolve_func_type_with<R>(&self, func_index: FuncIdx, f: impl FnOnce(&FuncType) -> R) -> R {
        let dedup_func_type = self.module.get_type_of_func(func_index);
        self.engine().resolve_func_type(dedup_func_type, f)
    }

    /// Resolves the [`FuncType`] at the given Wasm module `type_index`.
    fn resolve_type(&self, type_index: u32) -> FuncType {
        let func_type_idx = FuncTypeIdx::from(type_index);
        let dedup_func_type = self.module.get_func_type(func_type_idx);
        self.engine()
            .resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Returns the [`RegSpan`] of a call instruction before manipulating the operand stack.
    fn call_regspan(&self, len_params: usize) -> Result<RegSpan, Error> {
        let height = self.stack.height();
        let Some(start) = height.checked_sub(len_params) else {
            panic!("operand stack underflow while evaluating call `RegSpan`");
        };
        let start = self.layout.temp_to_reg(OperandIdx::from(start))?;
        Ok(RegSpan::new(start))
    }

    /// Push `results` as [`TempOperand`] onto the [`Stack`] tagged to `instr`.
    ///
    /// Returns the [`RegSpan`] identifying the pushed operands if any.
    fn push_results(
        &mut self,
        instr: Instr,
        results: &[ValType],
    ) -> Result<Option<RegSpan>, Error> {
        let (first, rest) = match results.split_first() {
            Some((first, rest)) => (first, rest),
            None => return Ok(None),
        };
        let first = self.stack.push_temp(*first, Some(instr))?;
        for result in rest {
            self.stack.push_temp(*result, Some(instr))?;
        }
        let start = self.layout.temp_to_reg(first)?;
        Ok(Some(RegSpan::new(start)))
    }

    /// Returns the [`Engine`] for which the function is compiled.
    fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Returns `true` if fuel metering is enabled.
    fn is_fuel_metering_enabled(&self) -> bool {
        self.fuel_costs.is_some()
    }

    /// Copy the top-most `len` operands to [`Operand::Temp`] values.
    ///
    /// # Note
    ///
    /// - The top-most `len` operands on the [`Stack`] will be [`Operand::Temp`] upon completion.
    /// - Does nothing if an [`Operand`] is already an [`Operand::Temp`].
    fn move_operands_to_temp(
        &mut self,
        len: usize,
        consume_fuel: Option<Instr>,
    ) -> Result<(), Error> {
        for n in 0..len {
            let operand = self.stack.operand_to_temp(n);
            self.copy_operand_to_temp(operand, consume_fuel)?;
        }
        Ok(())
    }

    /// Convert all branch params up to `depth` to [`Operand::Temp`].
    ///
    /// # Note
    ///
    /// - The top-most `depth` operands on the [`Stack`] will be [`Operand::Temp`] upon completion.
    /// - Does nothing if an [`Operand`] is already an [`Operand::Temp`].
    fn copy_branch_params(
        &mut self,
        target: &impl ControlFrameBase,
        consume_fuel_instr: Option<Instr>,
    ) -> Result<(), Error> {
        let len_branch_params = target.len_branch_params(&self.engine);
        let Some(branch_results) = self.frame_results(target)? else {
            return Ok(());
        };
        self.encode_copies(branch_results, len_branch_params, consume_fuel_instr)?;
        Ok(())
    }

    /// Pushes the temporary results of the control `frame` onto the [`Stack`].
    ///
    /// # Note
    ///
    /// - Before pushing the results, the [`Stack`] is truncated to the `frame`'s height.
    /// - Not all control frames have temporary results, e.g. Wasm `loop`s, Wasm `if`s with
    ///   a compile-time known branch or Wasm `block`s that are never branched to, do not
    ///   require to call this function.
    fn push_frame_results(&mut self, frame: &impl ControlFrameBase) -> Result<(), Error> {
        let height = frame.height();
        self.stack.trunc(height);
        frame
            .ty()
            .func_type_with(&self.engine, |func_ty| -> Result<(), Error> {
                for result in func_ty.results() {
                    self.stack.push_temp(*result, None)?;
                }
                Ok(())
            })?;
        Ok(())
    }

    /// Encodes a copy instruction for the top-most `len_values` on the stack to `results`.
    ///
    /// # Note
    ///
    /// - This does _not_ pop values from the stack or manipulate the stack otherwise.
    /// - This might allocate new function local constant values if necessary.
    /// - This does _not_ encode a copy if the copy is a no-op.
    fn encode_copies(
        &mut self,
        results: RegSpan,
        len_values: u16,
        consume_fuel_instr: Option<Instr>,
    ) -> Result<(), Error> {
        match len_values {
            0 => Ok(()),
            1 => {
                let result = results.head();
                let value = self.stack.peek(0);
                let Some(copy_instr) = Self::make_copy_instr(result, value, &mut self.layout)?
                else {
                    // Case: no-op copy instruction
                    return Ok(());
                };
                self.instrs
                    .push_instr(copy_instr, consume_fuel_instr, FuelCostsProvider::base)?;
                Ok(())
            }
            2 => {
                let (val0, val1) = self.stack.peek2();
                let val0 = self.layout.operand_to_reg(val0)?;
                let val1 = self.layout.operand_to_reg(val1)?;
                let result0 = results.head();
                let result1 = result0.next();
                if result0 == val0 && result1 == val1 {
                    // Case: no-op copy instruction
                    return Ok(());
                }
                self.instrs.push_instr(
                    Instruction::copy2_ext(results, val0, val1),
                    consume_fuel_instr,
                    FuelCostsProvider::base,
                )?;
                Ok(())
            }
            _ => {
                self.instrs
                    .bump_fuel_consumption(consume_fuel_instr, |costs| {
                        costs.fuel_for_copying_values(u64::from(len_values))
                    })?;
                if let Some(values) = self.try_form_regspan(usize::from(len_values))? {
                    // Case: can encode the copies as a more efficient `copy_span`
                    if results == values {
                        // Case: results and values are equal and therefore the copy is a no-op
                        return Ok(());
                    }
                    debug_assert!(results.head() < values.head());
                    self.instrs.push_instr(
                        Instruction::copy_span(results, values, len_values),
                        consume_fuel_instr,
                        FuelCostsProvider::base,
                    )?;
                    return Ok(());
                }
                self.stack
                    .peek_n(usize::from(len_values), &mut self.operands);
                let [fst, snd, rest @ ..] = &self.operands[..] else {
                    unreachable!("asserted that operands.len() >= 3")
                };
                let fst = self.layout.operand_to_reg(*fst)?;
                let snd = self.layout.operand_to_reg(*snd)?;
                self.instrs.push_instr(
                    Instruction::copy_many_ext(results, fst, snd),
                    consume_fuel_instr,
                    FuelCostsProvider::base,
                )?;
                self.instrs.encode_register_list(rest, &mut self.layout)?;
                Ok(())
            }
        }
    }

    /// Returns the results [`RegSpan`] of the `frame` if any.
    fn frame_results(&self, frame: &impl ControlFrameBase) -> Result<Option<RegSpan>, Error> {
        Self::frame_results_impl(frame, &self.engine, &self.layout)
    }

    /// Returns the results [`RegSpan`] of the `frame` if any.
    fn frame_results_impl(
        frame: &impl ControlFrameBase,
        engine: &Engine,
        layout: &StackLayout,
    ) -> Result<Option<RegSpan>, Error> {
        if frame.len_branch_params(engine) == 0 {
            return Ok(None);
        }
        let height = frame.height();
        let start = layout.temp_to_reg(OperandIdx::from(height))?;
        let span = RegSpan::new(start);
        Ok(Some(span))
    }

    /// Returns `true` if the [`ControlFrame`] at `depth` requires copying for its branch parameters.
    ///
    /// # Note
    ///
    /// Some instructions can be encoded in a more efficient way if no branch parameter copies are required.
    fn requires_branch_param_copies(&self, depth: usize) -> bool {
        let frame = self.stack.peek_control(depth);
        let len_branch_params = usize::from(frame.len_branch_params(&self.engine));
        let frame_height = frame.height();
        let height_matches = frame_height == (self.stack.height() - len_branch_params);
        let only_temps = (0..len_branch_params)
            .map(|depth| self.stack.peek(depth))
            .all(|o| o.is_temp());
        let can_avoid_copies = height_matches && only_temps;
        !can_avoid_copies
    }

    /// Pins the `label` to the next [`Instr`].
    fn pin_label(&mut self, label: LabelRef) {
        self.labels
            .pin_label(label, self.instrs.next_instr())
            .unwrap_or_else(|err| panic!("failed to pin label to next instruction: {err}"));
    }

    /// Convert the [`Operand`] at `depth` into an [`Operand::Temp`] by copying if necessary.
    ///
    /// # Note
    ///
    /// Does nothing if the [`Operand`] is already an [`Operand::Temp`].
    fn copy_operand_to_temp(
        &mut self,
        operand: Operand,
        consume_fuel: Option<Instr>,
    ) -> Result<(), Error> {
        if matches!(operand, Operand::Temp(_)) {
            return Ok(());
        }
        let result = self.layout.temp_to_reg(operand.index())?;
        let Some(copy_instr) = Self::make_copy_instr(result, operand, &mut self.layout)? else {
            unreachable!("filtered out temporary operands already");
        };
        self.instrs
            .push_instr(copy_instr, consume_fuel, FuelCostsProvider::base)?;
        Ok(())
    }

    /// Preserves all local operands on the stack.
    ///
    /// # Note
    ///
    /// This works by encoding copy instructions to `temp` register space.
    fn preserve_all_locals(&mut self) -> Result<(), Error> {
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        for local in self.stack.preserve_all_locals() {
            debug_assert!(matches!(local, Operand::Local(_)));
            let result = self.layout.temp_to_reg(local.index())?;
            let Some(copy_instr) = Self::make_copy_instr(result, local, &mut self.layout)? else {
                unreachable!("`result` and `local` refer to different stack spaces");
            };
            self.instrs
                .push_instr(copy_instr, consume_fuel_instr, FuelCostsProvider::base)?;
        }
        Ok(())
    }

    /// Returns the copy instruction to copy the given `operand` to `result`.
    ///
    /// Returns `None` if the resulting copy instruction is a no-op.
    fn make_copy_instr(
        result: Reg,
        value: Operand,
        layout: &mut StackLayout,
    ) -> Result<Option<Instruction>, Error> {
        let instr = match value {
            Operand::Temp(value) => {
                let value = layout.temp_to_reg(value.operand_index())?;
                if result == value {
                    // Case: no-op copy
                    return Ok(None);
                }
                Instruction::copy(result, value)
            }
            Operand::Local(value) => {
                let value = layout.local_to_reg(value.local_index())?;
                if result == value {
                    // Case: no-op copy
                    return Ok(None);
                }
                Instruction::copy(result, value)
            }
            Operand::Immediate(value) => Self::make_copy_imm_instr(result, value.val(), layout)?,
        };
        Ok(Some(instr))
    }

    /// Returns the copy instruction to copy the given immediate `value` to `result`.
    fn make_copy_imm_instr(
        result: Reg,
        value: TypedVal,
        layout: &mut StackLayout,
    ) -> Result<Instruction, Error> {
        let instr = match value.ty() {
            ValType::I32 => Instruction::copy_imm32(result, i32::from(value)),
            ValType::I64 => {
                let value = i64::from(value);
                match <Const32<i64>>::try_from(value) {
                    Ok(value) => Instruction::copy_i64imm32(result, value),
                    Err(_) => {
                        let value = layout.const_to_reg(value)?;
                        Instruction::copy(result, value)
                    }
                }
            }
            ValType::F32 => Instruction::copy_imm32(result, f32::from(value)),
            ValType::F64 => {
                let value = f64::from(value);
                match <Const32<f64>>::try_from(value) {
                    Ok(value) => Instruction::copy_f64imm32(result, value),
                    Err(_) => {
                        let value = layout.const_to_reg(value)?;
                        Instruction::copy(result, value)
                    }
                }
            }
            ValType::V128 | ValType::FuncRef | ValType::ExternRef => {
                let value = layout.const_to_reg(value)?;
                Instruction::copy(result, value)
            }
        };
        Ok(instr)
    }

    /// Pushes the `instr` to the function with the associated `fuel_costs`.
    fn push_instr(
        &mut self,
        instr: Instruction,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<Instr, Error> {
        let consume_fuel = self.stack.consume_fuel_instr();
        let instr = self.instrs.push_instr(instr, consume_fuel, fuel_costs)?;
        Ok(instr)
    }

    /// Pushes the `instr` to the function with the associated `fuel_costs`.
    fn push_instr_with_result(
        &mut self,
        result_ty: ValType,
        make_instr: impl FnOnce(Reg) -> Instruction,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), Error> {
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        let expected_iidx = self.instrs.next_instr();
        let result = self
            .layout
            .temp_to_reg(self.stack.push_temp(result_ty, Some(expected_iidx))?)?;
        let actual_iidx =
            self.instrs
                .push_instr(make_instr(result), consume_fuel_instr, fuel_costs)?;
        assert_eq!(expected_iidx, actual_iidx);
        Ok(())
    }

    /// Pushes a binary instruction with a result and associated fuel costs.
    fn push_binary_instr_with_result(
        &mut self,
        result_ty: ValType,
        lhs: Operand,
        rhs: Operand,
        make_instr: impl FnOnce(Reg, Reg, Reg) -> Instruction,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), Error> {
        debug_assert_eq!(lhs.ty(), rhs.ty());
        let lhs = self.layout.operand_to_reg(lhs)?;
        let rhs = self.layout.operand_to_reg(rhs)?;
        self.push_instr_with_result(result_ty, |result| make_instr(result, lhs, rhs), fuel_costs)
    }

    /// Pushes an instruction parameter `param` to the list of instructions.
    fn push_param(&mut self, param: Instruction) -> Result<(), Error> {
        self.instrs.push_param(param);
        Ok(())
    }

    /// Populate the `buffer` with the `table` targets including the `table` default target.
    ///
    /// Returns a shared slice to the `buffer` after it has been filled.
    ///
    /// # Note
    ///
    /// The `table` default target is pushed last to the `buffer`.
    fn copy_targets_from_br_table(
        table: &wasmparser::BrTable,
        buffer: &mut Vec<TypedVal>,
    ) -> Result<(), Error> {
        let default_target = table.default();
        buffer.clear();
        for target in table.targets() {
            buffer.push(TypedVal::from(target?));
        }
        buffer.push(TypedVal::from(default_target));
        Ok(())
    }

    /// Encodes a Wasm `br_table` that does not copy branching values.
    ///
    /// # Note
    ///
    /// Upon call the `immediates` buffer contains all `br_table` target values.
    fn encode_br_table_0(&mut self, table: wasmparser::BrTable, index: Reg) -> Result<(), Error> {
        debug_assert_eq!(self.immediates.len(), (table.len() + 1) as usize);
        self.push_instr(
            Instruction::branch_table_0(index, table.len() + 1),
            FuelCostsProvider::base,
        )?;
        // Encode the `br_table` targets:
        let targets = &self.immediates[..];
        for target in targets {
            let Ok(depth) = usize::try_from(u32::from(*target)) else {
                panic!("out of bounds `br_table` target does not fit `usize`: {target:?}");
            };
            let mut frame = self.stack.peek_control_mut(depth).control_frame();
            let offset = self
                .labels
                .try_resolve_label(frame.label(), self.instrs.next_instr())?;
            self.instrs.push_param(Instruction::branch(offset));
            frame.branch_to();
        }
        Ok(())
    }

    /// Encodes a Wasm `br_table` that has to copy `len_values` branching values.
    ///
    /// # Note
    ///
    /// Upon call the `immediates` buffer contains all `br_table` target values.
    fn encode_br_table_n(
        &mut self,
        table: wasmparser::BrTable,
        index: Reg,
        len_values: u16,
    ) -> Result<(), Error> {
        debug_assert_eq!(self.immediates.len(), (table.len() + 1) as usize);
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        let values = self.try_form_regspan_or_move(usize::from(len_values), consume_fuel_instr)?;
        self.push_instr(
            Instruction::branch_table_span(index, table.len() + 1),
            FuelCostsProvider::base,
        )?;
        self.push_param(Instruction::register_span(BoundedRegSpan::new(
            values, len_values,
        )))?;
        // Encode the `br_table` targets:
        let targets = &self.immediates[..];
        for target in targets {
            let Ok(depth) = usize::try_from(u32::from(*target)) else {
                panic!("out of bounds `br_table` target does not fit `usize`: {target:?}");
            };
            let mut frame = self.stack.peek_control_mut(depth).control_frame();
            let Some(results) = Self::frame_results_impl(&frame, &self.engine, &self.layout)?
            else {
                panic!("must have frame results since `br_table` requires to copy values");
            };
            let offset = self
                .labels
                .try_resolve_label(frame.label(), self.instrs.next_instr())?;
            self.instrs
                .push_param(Instruction::branch_table_target(results, offset));
            frame.branch_to();
        }
        Ok(())
    }

    /// Encodes a generic return instruction.
    fn encode_return(&mut self, consume_fuel: Option<Instr>) -> Result<Instr, Error> {
        let len_results = self.func_type_with(FuncType::len_results);
        let instr = match len_results {
            0 => Instruction::Return,
            1 => match self.stack.peek(0) {
                Operand::Local(operand) => {
                    let value = self.layout.local_to_reg(operand.local_index())?;
                    Instruction::return_reg(value)
                }
                Operand::Temp(operand) => {
                    let value = self.layout.temp_to_reg(operand.operand_index())?;
                    Instruction::return_reg(value)
                }
                Operand::Immediate(operand) => {
                    let val = operand.val();
                    match operand.ty() {
                        ValType::I32 => Instruction::return_imm32(i32::from(val)),
                        ValType::I64 => match <Const32<i64>>::try_from(i64::from(val)) {
                            Ok(value) => Instruction::return_i64imm32(value),
                            Err(_) => {
                                let value = self.layout.const_to_reg(val)?;
                                Instruction::return_reg(value)
                            }
                        },
                        ValType::F32 => Instruction::return_imm32(f32::from(val)),
                        ValType::F64 => match <Const32<f64>>::try_from(f64::from(val)) {
                            Ok(value) => Instruction::return_f64imm32(value),
                            Err(_) => {
                                let value = self.layout.const_to_reg(val)?;
                                Instruction::return_reg(value)
                            }
                        },
                        ValType::V128 | ValType::FuncRef | ValType::ExternRef => {
                            let value = self.layout.const_to_reg(val)?;
                            Instruction::return_reg(value)
                        }
                    }
                }
            },
            2 => {
                let (v0, v1) = self.stack.peek2();
                let v0 = self.layout.operand_to_reg(v0)?;
                let v1 = self.layout.operand_to_reg(v1)?;
                Instruction::return_reg2_ext(v0, v1)
            }
            3 => {
                let (v0, v1, v2) = self.stack.peek3();
                let v0 = self.layout.operand_to_reg(v0)?;
                let v1 = self.layout.operand_to_reg(v1)?;
                let v2 = self.layout.operand_to_reg(v2)?;
                Instruction::return_reg3_ext(v0, v1, v2)
            }
            _ => {
                let len_values = usize::from(len_results);
                match self.try_form_regspan(len_values)? {
                    Some(span) => {
                        let values = BoundedRegSpan::new(span, len_results);
                        Instruction::return_span(values)
                    }
                    None => return self.encode_return_many(len_values, consume_fuel),
                }
            }
        };
        let instr = self
            .instrs
            .push_instr(instr, consume_fuel, FuelCostsProvider::base)?;
        Ok(instr)
    }

    /// Encodes an [`Instruction::ReturnMany`] for `len` values.
    ///
    /// # Panics
    ///
    /// If `len` is not greater than or equal to 4.
    fn encode_return_many(
        &mut self,
        len: usize,
        consume_fuel_instr: Option<Instr>,
    ) -> Result<Instr, Error> {
        self.stack.peek_n(len, &mut self.operands);
        let [v0, v1, v2, rest @ ..] = &self.operands[..] else {
            unreachable!("encode_return_many (pre-condition): len >= 4")
        };
        let v0 = self.layout.operand_to_reg(*v0)?;
        let v1 = self.layout.operand_to_reg(*v1)?;
        let v2 = self.layout.operand_to_reg(*v2)?;
        let return_instr = self.instrs.push_instr(
            Instruction::return_many_ext(v0, v1, v2),
            consume_fuel_instr,
            FuelCostsProvider::base,
        )?;
        self.instrs.encode_register_list(rest, &mut self.layout)?;
        Ok(return_instr)
    }

    /// Tries to form a [`RegSpan`] from the top-most `len` operands on the [`Stack`].
    ///
    /// Returns `None` if forming a [`RegSpan`] was not possible.
    fn try_form_regspan(&self, len: usize) -> Result<Option<RegSpan>, Error> {
        if len == 0 {
            return Ok(None);
        }
        let mut start = match self.stack.peek(0) {
            Operand::Immediate(_) => return Ok(None),
            Operand::Local(operand) => self.layout.local_to_reg(operand.local_index())?,
            Operand::Temp(operand) => self.layout.temp_to_reg(operand.operand_index())?,
        };
        for depth in 1..len {
            let cur = match self.stack.peek(depth) {
                Operand::Immediate(_) => return Ok(None),
                Operand::Local(operand) => self.layout.local_to_reg(operand.local_index())?,
                Operand::Temp(operand) => self.layout.temp_to_reg(operand.operand_index())?,
            };
            if start != cur.next() {
                return Ok(None);
            }
            start = cur;
        }
        Ok(Some(RegSpan::new(start)))
    }

    /// Tries to form a [`RegSpan`] from the top-most `len` operands on the [`Stack`] or copy to temporaries.
    ///
    /// Returns `None` if forming a [`RegSpan`] was not possible.
    fn try_form_regspan_or_move(
        &mut self,
        len: usize,
        consume_fuel_instr: Option<Instr>,
    ) -> Result<RegSpan, Error> {
        if let Some(span) = self.try_form_regspan(len)? {
            return Ok(span);
        }
        self.move_operands_to_temp(len, consume_fuel_instr)?;
        let Some(span) = self.try_form_regspan(len)? else {
            unreachable!("the top-most `len` operands are now temporaries thus `RegSpan` forming should succeed")
        };
        Ok(span)
    }

    /// Translates the end of a Wasm `block` control frame.
    fn translate_end_block(&mut self, frame: BlockControlFrame) -> Result<(), Error> {
        let consume_fuel_instr = frame.consume_fuel_instr();
        if frame.is_branched_to() {
            if self.reachable {
                self.copy_branch_params(&frame, consume_fuel_instr)?;
            }
            self.push_frame_results(&frame)?;
        }
        if let Err(err) = self
            .labels
            .pin_label(frame.label(), self.instrs.next_instr())
        {
            panic!("failed to pin label: {err}")
        }
        self.reachable |= frame.is_branched_to();
        if self.reachable && self.stack.is_control_empty() {
            self.encode_return(consume_fuel_instr)?;
        }
        if frame.is_branched_to() {
            // No need to reset `last_instr` if there was no branch to the end of a Wasm `block`.
            self.instrs.reset_last_instr();
        }
        Ok(())
    }

    /// Translates the end of a Wasm `loop` control frame.
    fn translate_end_loop(&mut self, _frame: LoopControlFrame) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        // Nothing needs to be done since Wasm `loop` control frames always only have a single exit.
        //
        // Note: no need to reset `last_instr` since end of `loop` is not a control flow boundary.
        Ok(())
    }

    /// Translates the end of a Wasm `if` control frame.
    fn translate_end_if(&mut self, frame: IfControlFrame) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        let is_end_of_then_reachable = self.reachable;
        let IfReachability::Both { else_label } = frame.reachability() else {
            let is_end_reachable = match frame.reachability() {
                IfReachability::OnlyThen => self.reachable,
                IfReachability::OnlyElse => true,
                IfReachability::Both { .. } => unreachable!(),
            };
            return self.translate_end_if_or_else_only(frame, is_end_reachable);
        };
        let len_results = frame.ty().len_results(self.engine());
        let has_results = len_results >= 1;
        if is_end_of_then_reachable && has_results {
            let consume_fuel_instr = frame.consume_fuel_instr();
            self.copy_branch_params(&frame, consume_fuel_instr)?;
            let end_offset = self
                .labels
                .try_resolve_label(frame.label(), self.instrs.next_instr())
                .unwrap();
            self.instrs.push_instr(
                Instruction::branch(end_offset),
                consume_fuel_instr,
                FuelCostsProvider::base,
            )?;
        }
        self.labels
            .try_pin_label(else_label, self.instrs.next_instr());
        self.stack.push_else_operands(&frame)?;
        if has_results {
            // We haven't visited the `else` block and thus the `else`
            // providers are still on the auxiliary stack and need to
            // be popped. We use them to restore the stack to the state
            // when entering the `if` block so that we can properly copy
            // the `else` results to were they are expected.
            let consume_fuel_instr = self.instrs.push_consume_fuel_instr()?;
            self.copy_branch_params(&frame, consume_fuel_instr)?;
        }
        self.push_frame_results(&frame)?;
        self.labels
            .pin_label(frame.label(), self.instrs.next_instr())
            .unwrap();
        self.reachable = true;
        // Need to reset `last_instr` since end of `if` is a control flow boundary.
        self.instrs.reset_last_instr();
        Ok(())
    }

    /// Translates the end of a Wasm `else` control frame.
    fn translate_end_else(&mut self, frame: ElseControlFrame) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        match frame.reachability() {
            ElseReachability::OnlyThen {
                is_end_of_then_reachable,
            } => {
                return self.translate_end_if_or_else_only(frame, is_end_of_then_reachable);
            }
            ElseReachability::OnlyElse => {
                return self.translate_end_if_or_else_only(frame, self.reachable);
            }
            _ => {}
        };
        let end_of_then_reachable = frame.is_end_of_then_reachable();
        let end_of_else_reachable = self.reachable;
        let reachable = match (end_of_then_reachable, end_of_else_reachable) {
            (false, false) => frame.is_branched_to(),
            _ => true,
        };
        if end_of_else_reachable {
            let consume_fuel_instr: Option<Instr> = frame.consume_fuel_instr();
            self.copy_branch_params(&frame, consume_fuel_instr)?;
        }
        self.push_frame_results(&frame)?;
        self.labels
            .pin_label(frame.label(), self.instrs.next_instr())
            .unwrap();
        self.reachable = reachable;
        // Need to reset `last_instr` since end of `else` is a control flow boundary.
        self.instrs.reset_last_instr();
        Ok(())
    }

    /// Translates the end of a Wasm `else` control frame where only one branch is known to be reachable.
    fn translate_end_if_or_else_only(
        &mut self,
        frame: impl ControlFrameBase,
        end_is_reachable: bool,
    ) -> Result<(), Error> {
        if frame.is_branched_to() {
            if end_is_reachable {
                let consume_fuel_instr = frame.consume_fuel_instr();
                self.copy_branch_params(&frame, consume_fuel_instr)?;
            }
            self.push_frame_results(&frame)?;
        }
        self.labels
            .pin_label(frame.label(), self.instrs.next_instr())
            .unwrap();
        self.reachable = end_is_reachable || frame.is_branched_to();
        if frame.is_branched_to() {
            // No need to reset `last_instr` if there was no branch to the
            // end of a Wasm `if` where only `then` or `else` is reachable.
            self.instrs.reset_last_instr();
        }
        Ok(())
    }

    /// Translates the end of an unreachable Wasm control frame.
    fn translate_end_unreachable(&mut self, _frame: ControlFrameKind) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        // We reset `last_instr` out of caution in case there is a control flow boundary.
        self.instrs.reset_last_instr();
        Ok(())
    }

    /// Translate the Wasm `local.set` and `local.tee` operations.
    ///
    /// # Note
    ///
    /// This applies op-code fusion that replaces the result of the previous instruction
    /// instead of encoding a copy instruction for the `local.set` or `local.tee` if possible.
    fn translate_local_set(&mut self, local_index: u32, push_result: bool) -> Result<(), Error> {
        bail_unreachable!(self);
        let input = self.stack.pop();
        if let Operand::Local(input) = input {
            if u32::from(input.local_index()) == local_index {
                // Case: `(local.set $n (local.get $n))` is a no-op so we can ignore it.
                //
                // Note: This does not require any preservation since it won't change
                //       the value of `local $n`.
                if push_result {
                    // Need to push back input before we exit.
                    self.stack.push_operand(input.into())?;
                }
                return Ok(());
            }
        }
        let local_idx = LocalIdx::from(local_index);
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        for preserved in self.stack.preserve_locals(local_idx) {
            let result = self.layout.temp_to_reg(preserved)?;
            let value = self.layout.local_to_reg(local_idx)?;
            self.instrs.push_instr(
                Instruction::copy(result, value),
                consume_fuel_instr,
                FuelCostsProvider::base,
            )?;
        }
        if push_result {
            match input {
                Operand::Immediate(input) => {
                    self.stack.push_immediate(input.val())?;
                }
                _ => {
                    self.stack.push_local(local_idx)?;
                }
            }
        }
        if self.try_replace_result(local_idx, input)? {
            // Case: it was possible to replace the result of the previous
            //       instructions so no copy instruction is required.
            return Ok(());
        }
        // At this point we need to encode a copy instruction.
        let result = self.layout.local_to_reg(local_idx)?;
        let Some(copy_instr) = Self::make_copy_instr(result, input, &mut self.layout)? else {
            unreachable!("filtered out no-op copies above already");
        };
        self.instrs
            .push_instr(copy_instr, consume_fuel_instr, FuelCostsProvider::base)?;
        Ok(())
    }

    /// Tries to replace the result of the previous instruction with `new_result` if possible.
    ///
    /// Returns `Ok(true)` if replacement was successful and `Ok(false)` otherwise.
    fn try_replace_result(
        &mut self,
        new_result: LocalIdx,
        old_result: Operand,
    ) -> Result<bool, Error> {
        let result = self.layout.local_to_reg(new_result)?;
        let old_result = match old_result {
            Operand::Immediate(_) => {
                // Case: cannot replace immediate value result.
                return Ok(false);
            }
            Operand::Local(_) => {
                // Case: cannot replace local with another local due to observable behavior.
                return Ok(false);
            }
            Operand::Temp(operand) => self.layout.temp_to_reg(operand.operand_index())?,
        };
        self.instrs
            .try_replace_result(result, old_result, &self.layout, &self.module)
    }

    /// Encodes an unconditional Wasm `branch` instruction.
    fn encode_br(&mut self, label: LabelRef) -> Result<Instr, Error> {
        let instr = self.instrs.next_instr();
        let offset = self.labels.try_resolve_label(label, instr)?;
        let br_instr = self.push_instr(Instruction::branch(offset), FuelCostsProvider::base)?;
        Ok(br_instr)
    }

    /// Encodes a `i32.eqz`+`br_if` or `if` conditional branch instruction.
    fn encode_br_eqz(&mut self, condition: Operand, label: LabelRef) -> Result<(), Error> {
        self.encode_br_if(condition, label, true)
    }

    /// Encodes a `br_if` conditional branch instruction.
    fn encode_br_nez(&mut self, condition: Operand, label: LabelRef) -> Result<(), Error> {
        self.encode_br_if(condition, label, false)
    }

    /// Encodes a generic `br_if` fused conditional branch instruction.
    fn encode_br_if(
        &mut self,
        condition: Operand,
        label: LabelRef,
        branch_eqz: bool,
    ) -> Result<(), Error> {
        if self.try_fuse_branch_cmp(condition, label, branch_eqz)? {
            return Ok(());
        }
        let condition = match condition {
            Operand::Local(condition) => self.layout.local_to_reg(condition.local_index())?,
            Operand::Temp(condition) => self.layout.temp_to_reg(condition.operand_index())?,
            Operand::Immediate(condition) => {
                let condition = i32::from(condition.val());
                let take_branch = match branch_eqz {
                    true => condition == 0,
                    false => condition != 0,
                };
                match take_branch {
                    true => {
                        self.encode_br(label)?;
                        self.reachable = false;
                        return Ok(());
                    }
                    false => return Ok(()),
                }
            }
        };
        let instr = self.instrs.next_instr();
        let offset = self.labels.try_resolve_label(label, instr)?;
        let instr = match BranchOffset16::try_from(offset) {
            Ok(offset) => match branch_eqz {
                true => Instruction::branch_i32_eq_imm16(condition, 0, offset),
                false => Instruction::branch_i32_ne_imm16(condition, 0, offset),
            },
            Err(_) => {
                let zero = self.layout.const_to_reg(0_i32)?;
                let comparator = match branch_eqz {
                    true => Comparator::I32Eq,
                    false => Comparator::I32Ne,
                };
                self.make_branch_cmp_fallback(comparator, condition, zero, offset)?
            }
        };
        self.push_instr(instr, FuelCostsProvider::base)?;
        Ok(())
    }

    /// Create an [`Instruction::BranchCmpFallback`].
    fn make_branch_cmp_fallback(
        &mut self,
        cmp: Comparator,
        lhs: Reg,
        rhs: Reg,
        offset: BranchOffset,
    ) -> Result<Instruction, Error> {
        let params = self
            .layout
            .const_to_reg(ComparatorAndOffset::new(cmp, offset))?;
        Ok(Instruction::branch_cmp_fallback(lhs, rhs, params))
    }

    /// Try to fuse a cmp+branch [`Instruction`] with optional negation.
    fn try_fuse_branch_cmp(
        &mut self,
        condition: Operand,
        label: LabelRef,
        negate: bool,
    ) -> Result<bool, Error> {
        let Some(last_instr) = self.instrs.last_instr() else {
            // Case: cannot fuse without a known last instruction
            return Ok(false);
        };
        let Operand::Temp(condition) = condition else {
            // Case: cannot fuse non-temporary operands
            //  - locals have observable behavior.
            //  - immediates cannot be the result of a previous instruction.
            return Ok(false);
        };
        let Some(origin) = condition.instr() else {
            // Case: cannot fuse temporary operands without origin instruction
            return Ok(false);
        };
        if last_instr != origin {
            // Case: cannot fuse if last instruction does not match origin instruction
            return Ok(false);
        }
        debug_assert!(matches!(condition.ty(), ValType::I32 | ValType::I64));
        let fused_instr = self.try_make_fused_branch_cmp_instr(origin, condition, label, negate)?;
        let Some(fused_instr) = fused_instr else {
            // Case: not possible to perform fusion with last instruction
            return Ok(false);
        };
        assert!(
            self.instrs.try_replace_instr(origin, fused_instr)?,
            "op-code fusion must suceed at this point",
        );
        Ok(true)
    }

    /// Try to return a fused cmp+branch [`Instruction`] from the given parameters.
    ///
    ///
    /// # Note
    ///
    /// - The `instr` parameter refers to the to-be-fused cmp instruction.
    /// - Returns `Ok(Some)` if cmp+branch fusion was successful.
    /// - Returns `Ok(None)`, otherwise.
    fn try_make_fused_branch_cmp_instr(
        &mut self,
        instr: Instr,
        condition: TempOperand,
        label: LabelRef,
        negate: bool,
    ) -> Result<Option<Instruction>, Error> {
        let cmp_instr = *self.instrs.get(instr);
        let Some(result) = cmp_instr.compare_result() else {
            // Note: cannot fuse non-cmp instructions or cmp-instructions without result.
            return Ok(None);
        };
        if matches!(self.layout.stack_space(result), StackSpace::Local) {
            // Note: cannot fuse cmp instructions with observable semantics.
            return Ok(None);
        }
        if result != self.layout.temp_to_reg(condition.operand_index())? {
            // Note: cannot fuse cmp instruction with a result that differs
            //       from the condition operand.
            return Ok(None);
        }
        let cmp_instr = match negate {
            false => cmp_instr,
            true => match cmp_instr.negate_cmp_instr() {
                Some(negated) => negated,
                None => {
                    // Note: cannot negate cmp instruction, thus not possible to fuse.
                    return Ok(None);
                }
            },
        };
        let offset = self.labels.try_resolve_label(label, instr)?;
        let fused = cmp_instr
            .try_into_cmp_branch_instr(offset, &mut self.layout)?
            .expect("cmp+branch fusion must succeed");
        Ok(Some(fused))
    }

    /// Translates a unary Wasm instruction to Wasmi bytecode.
    fn translate_unary<T, R>(
        &mut self,
        make_instr: fn(result: Reg, input: Reg) -> Instruction,
        consteval: fn(input: T) -> R,
    ) -> Result<(), Error>
    where
        T: From<TypedVal>,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        let input = self.stack.pop();
        if let Operand::Immediate(input) = input {
            self.stack.push_immediate(consteval(input.val().into()))?;
            return Ok(());
        }
        let input = self.layout.operand_to_reg(input)?;
        self.push_instr_with_result(
            <R as Typed>::TY,
            |result| make_instr(result, input),
            FuelCostsProvider::base,
        )
    }

    /// Translates a unary Wasm instruction to Wasmi bytecode.
    fn translate_unary_fallible<T, R>(
        &mut self,
        make_instr: fn(result: Reg, input: Reg) -> Instruction,
        consteval: fn(input: T) -> Result<R, TrapCode>,
    ) -> Result<(), Error>
    where
        T: From<TypedVal>,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        let input = self.stack.pop();
        if let Operand::Immediate(input) = input {
            let input = T::from(input.val());
            match consteval(input) {
                Ok(result) => {
                    self.stack.push_immediate(result)?;
                }
                Err(trap) => {
                    self.translate_trap(trap)?;
                }
            }
            return Ok(());
        }
        let input = self.layout.operand_to_reg(input)?;
        self.push_instr_with_result(
            <R as Typed>::TY,
            |result| make_instr(result, input),
            FuelCostsProvider::base,
        )
    }

    /// Translate a generic Wasm reinterpret-like operation.
    ///
    /// # Note
    ///
    /// This Wasm operation is a no-op. Ideally we only have to change the types on the stack.
    fn translate_reinterpret<T, R>(&mut self, consteval: fn(T) -> R) -> Result<(), Error>
    where
        T: From<TypedVal> + Typed,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        match self.stack.pop() {
            Operand::Local(input) => {
                debug_assert_eq!(input.ty(), <T as Typed>::TY);
                // TODO: improve performance by allowing type overwrites for local operands
                let input = self.layout.local_to_reg(input.local_index())?;
                self.push_instr_with_result(
                    <R as Typed>::TY,
                    |result| Instruction::copy(result, input),
                    FuelCostsProvider::base,
                )?;
            }
            Operand::Temp(input) => {
                debug_assert_eq!(input.ty(), <T as Typed>::TY);
                self.stack.push_temp(<R as Typed>::TY, None)?;
            }
            Operand::Immediate(input) => {
                let input: T = input.val().into();
                self.stack.push_immediate(consteval(input))?;
            }
        }
        Ok(())
    }

    /// Creates a new 16-bit encoded [`Input16`] from the given `value`.
    pub fn make_imm16<T>(&mut self, value: T) -> Result<Input16<T>, Error>
    where
        T: Into<UntypedVal> + Copy + TryInto<Const16<T>>,
    {
        match value.try_into() {
            Ok(rhs) => Ok(Input::Immediate(rhs)),
            Err(_) => {
                let rhs = self.layout.const_to_reg(value)?;
                Ok(Input::Reg(rhs))
            }
        }
    }

    /// Creates a new 16-bit encoded [`Input16`] from the given `operand`.
    pub fn make_input16<T>(&mut self, operand: Operand) -> Result<Input16<T>, Error>
    where
        T: From<TypedVal> + Into<UntypedVal> + TryInto<Const16<T>> + Copy,
    {
        self.make_input(operand, |this, imm| {
            let opd16 = match T::from(imm).try_into() {
                Ok(rhs) => Input::Immediate(rhs),
                Err(_) => {
                    let rhs = this.layout.const_to_reg(imm)?;
                    Input::Reg(rhs)
                }
            };
            Ok(opd16)
        })
    }

    /// Create a new generic [`Input`] from the given `operand`.
    fn make_input<R>(
        &mut self,
        operand: Operand,
        f: impl FnOnce(&mut Self, TypedVal) -> Result<Input<R>, Error>,
    ) -> Result<Input<R>, Error> {
        let reg = match operand {
            Operand::Local(operand) => self.layout.local_to_reg(operand.local_index())?,
            Operand::Temp(operand) => self.layout.temp_to_reg(operand.operand_index())?,
            Operand::Immediate(operand) => return f(self, operand.val()),
        };
        Ok(Input::Reg(reg))
    }

    /// Converts the `provider` to a 16-bit index-type constant value.
    ///
    /// # Note
    ///
    /// - Turns immediates that cannot be 16-bit encoded into function local constants.
    /// - The behavior is different whether `memory64` is enabled or disabled.
    pub(super) fn make_index16(
        &mut self,
        operand: Operand,
        index_type: IndexType,
    ) -> Result<Input16<u64>, Error> {
        let value = match operand {
            Operand::Immediate(value) => value.val(),
            operand => {
                debug_assert_eq!(operand.ty(), index_type.ty());
                let reg = self.layout.operand_to_reg(operand)?;
                return Ok(Input::Reg(reg));
            }
        };
        match index_type {
            IndexType::I64 => {
                if let Ok(value) = Const16::try_from(u64::from(value)) {
                    return Ok(Input::Immediate(value));
                }
            }
            IndexType::I32 => {
                if let Ok(value) = Const16::try_from(u32::from(value)) {
                    return Ok(Input::Immediate(<Const16<u64>>::cast(value)));
                }
            }
        }
        let reg = self.layout.const_to_reg(value)?;
        Ok(Input::Reg(reg))
    }

    /// Converts the `provider` to a 32-bit index-type constant value.
    ///
    /// # Note
    ///
    /// - Turns immediates that cannot be 32-bit encoded into function local constants.
    /// - The behavior is different whether `memory64` is enabled or disabled.
    pub(super) fn make_index32(
        &mut self,
        operand: Operand,
        index_type: IndexType,
    ) -> Result<Input32<u64>, Error> {
        let value = match operand {
            Operand::Immediate(value) => value.val(),
            operand => {
                debug_assert_eq!(operand.ty(), index_type.ty());
                let reg = self.layout.operand_to_reg(operand)?;
                return Ok(Input::Reg(reg));
            }
        };
        match index_type {
            IndexType::I64 => {
                if let Ok(value) = Const32::try_from(u64::from(value)) {
                    return Ok(Input::Immediate(value));
                }
            }
            IndexType::I32 => {
                let value32 = Const32::from(u32::from(value));
                return Ok(Input::Immediate(<Const32<u64>>::cast(value32)));
            }
        }
        let reg = self.layout.const_to_reg(value)?;
        Ok(Input::Reg(reg))
    }

    /// Evaluates `consteval(lhs, rhs)` and pushed either its result or tranlates a `trap`.
    fn translate_binary_consteval_fallible<T, R>(
        &mut self,
        lhs: ImmediateOperand,
        rhs: ImmediateOperand,
        consteval: impl FnOnce(T, T) -> Result<R, TrapCode>,
    ) -> Result<(), Error>
    where
        T: From<TypedVal>,
        R: Into<TypedVal>,
    {
        let lhs: T = lhs.val().into();
        let rhs: T = rhs.val().into();
        match consteval(lhs, rhs) {
            Ok(value) => {
                self.stack.push_immediate(value)?;
            }
            Err(trap) => {
                self.translate_trap(trap)?;
            }
        }
        Ok(())
    }

    /// Evaluates `consteval(lhs, rhs)` and pushed either its result or tranlates a `trap`.
    fn translate_binary_consteval<T, R>(
        &mut self,
        lhs: ImmediateOperand,
        rhs: ImmediateOperand,
        consteval: fn(T, T) -> R,
    ) -> Result<(), Error>
    where
        T: From<TypedVal>,
        R: Into<TypedVal>,
    {
        self.translate_binary_consteval_fallible::<T, R>(lhs, rhs, |lhs, rhs| {
            Ok(consteval(lhs, rhs))
        })
    }

    /// Convenience method to tell that there is no custom optimization.
    fn no_opt_ri<T>(&mut self, _lhs: Operand, _rhs: T) -> Result<bool, Error> {
        Ok(false)
    }

    /// Translates a commutative binary Wasm operator to Wasmi bytecode.
    fn translate_binary_commutative<T, R>(
        &mut self,
        make_rr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_ri: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
        consteval: fn(T, T) -> R,
        opt_ri: fn(this: &mut Self, lhs: Operand, rhs: T) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: WasmInteger + TryInto<Const16<T>>,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                self.translate_binary_consteval::<T, R>(lhs, rhs, consteval)
            }
            (val, Operand::Immediate(imm)) | (Operand::Immediate(imm), val) => {
                let rhs = imm.val().into();
                if opt_ri(self, val, rhs)? {
                    return Ok(());
                }
                let lhs = self.layout.operand_to_reg(val)?;
                let rhs16 = self.make_imm16(rhs)?;
                self.push_instr_with_result(
                    <R as Typed>::TY,
                    |result| match rhs16 {
                        Input::Immediate(rhs) => make_ri(result, lhs, rhs),
                        Input::Reg(rhs) => make_rr(result, lhs, rhs),
                    },
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => self.push_binary_instr_with_result(
                <R as Typed>::TY,
                lhs,
                rhs,
                make_rr,
                FuelCostsProvider::base,
            ),
        }
    }

    /// Translates integer division and remainder Wasm operators to Wasmi bytecode.
    fn translate_divrem<T>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm16_rhs: fn(
            result: Reg,
            lhs: Reg,
            rhs: Const16<<T as WasmInteger>::NonZero>,
        ) -> Instruction,
        make_instr_imm16_lhs: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
        consteval: fn(T, T) -> Result<T, TrapCode>,
    ) -> Result<(), Error>
    where
        T: WasmInteger,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                self.translate_binary_consteval_fallible::<T, T>(lhs, rhs, consteval)
            }
            (lhs, Operand::Immediate(rhs)) => {
                let lhs = self.layout.operand_to_reg(lhs)?;
                let rhs = T::from(rhs.val());
                let Some(non_zero_rhs) = <T as WasmInteger>::non_zero(rhs) else {
                    // Optimization: division by zero always traps
                    return self.translate_trap(TrapCode::IntegerDivisionByZero);
                };
                let rhs16 = self.make_imm16(non_zero_rhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| match rhs16 {
                        Input::Immediate(rhs) => make_instr_imm16_rhs(result, lhs, rhs),
                        Input::Reg(rhs) => make_instr(result, lhs, rhs),
                    },
                    FuelCostsProvider::base,
                )
            }
            (Operand::Immediate(lhs), rhs) => {
                let lhs = T::from(lhs.val());
                let lhs16 = self.make_imm16(lhs)?;
                let rhs = self.layout.operand_to_reg(rhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| match lhs16 {
                        Input::Immediate(lhs) => make_instr_imm16_lhs(result, lhs, rhs),
                        Input::Reg(lhs) => make_instr(result, lhs, rhs),
                    },
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => self.push_binary_instr_with_result(
                <T as Typed>::TY,
                lhs,
                rhs,
                make_instr,
                FuelCostsProvider::base,
            ),
        }
    }

    /// Translates binary non-commutative Wasm operators to Wasmi bytecode.
    fn translate_binary<T, R>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm16_rhs: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
        make_instr_imm16_lhs: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
        consteval: fn(T, T) -> R,
    ) -> Result<(), Error>
    where
        T: WasmInteger,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                self.translate_binary_consteval::<T, R>(lhs, rhs, consteval)
            }
            (lhs, Operand::Immediate(rhs)) => {
                let lhs = self.layout.operand_to_reg(lhs)?;
                let rhs = T::from(rhs.val());
                let rhs16 = self.make_imm16(rhs)?;
                self.push_instr_with_result(
                    <R as Typed>::TY,
                    |result| match rhs16 {
                        Input::Immediate(rhs) => make_instr_imm16_rhs(result, lhs, rhs),
                        Input::Reg(rhs) => make_instr(result, lhs, rhs),
                    },
                    FuelCostsProvider::base,
                )
            }
            (Operand::Immediate(lhs), rhs) => {
                let lhs = T::from(lhs.val());
                let lhs16 = self.make_imm16(lhs)?;
                let rhs = self.layout.operand_to_reg(rhs)?;
                self.push_instr_with_result(
                    <R as Typed>::TY,
                    |result| match lhs16 {
                        Input::Immediate(lhs) => make_instr_imm16_lhs(result, lhs, rhs),
                        Input::Reg(lhs) => make_instr(result, lhs, rhs),
                    },
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => self.push_binary_instr_with_result(
                <R as Typed>::TY,
                lhs,
                rhs,
                make_instr,
                FuelCostsProvider::base,
            ),
        }
    }

    /// Translates Wasm `i{32,64}.sub` operators to Wasmi bytecode.
    fn translate_isub<T, R>(
        &mut self,
        make_sub_rr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_add_ri: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
        make_sub_ir: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
        consteval: fn(T, T) -> R,
    ) -> Result<(), Error>
    where
        T: WasmInteger,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                self.translate_binary_consteval::<T, R>(lhs, rhs, consteval)
            }
            (lhs, Operand::Immediate(rhs)) => {
                let lhs = self.layout.operand_to_reg(lhs)?;
                let rhs = T::from(rhs.val());
                let rhs16 = match rhs.wrapping_neg().try_into() {
                    Ok(rhs) => Input::Immediate(rhs),
                    Err(_) => {
                        let rhs = self.layout.const_to_reg(rhs)?;
                        Input::Reg(rhs)
                    }
                };
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| match rhs16 {
                        Input::Immediate(rhs) => make_add_ri(result, lhs, rhs),
                        Input::Reg(rhs) => make_sub_rr(result, lhs, rhs),
                    },
                    FuelCostsProvider::base,
                )
            }
            (Operand::Immediate(lhs), rhs) => {
                let lhs = T::from(lhs.val());
                let lhs16 = self.make_imm16(lhs)?;
                let rhs = self.layout.operand_to_reg(rhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| match lhs16 {
                        Input::Immediate(lhs) => make_sub_ir(result, lhs, rhs),
                        Input::Reg(lhs) => make_sub_rr(result, lhs, rhs),
                    },
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => self.push_binary_instr_with_result(
                <R as Typed>::TY,
                lhs,
                rhs,
                make_sub_rr,
                FuelCostsProvider::base,
            ),
        }
    }

    /// Translates Wasm shift and rotate operators to Wasmi bytecode.
    fn translate_shift<T>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm16_rhs: fn(
            result: Reg,
            lhs: Reg,
            rhs: <T as IntoShiftAmount>::Output,
        ) -> Instruction,
        make_instr_imm16_lhs: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
        consteval: fn(T, T) -> T,
    ) -> Result<(), Error>
    where
        T: WasmInteger + IntoShiftAmount<Input: From<TypedVal>>,
        Const16<T>: From<i16>,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                self.translate_binary_consteval::<T, T>(lhs, rhs, consteval)
            }
            (lhs, Operand::Immediate(rhs)) => {
                let Some(rhs) = T::into_shift_amount(rhs.val().into()) else {
                    // Optimization: Shifting or rotating by zero bits is a no-op.
                    self.stack.push_operand(lhs)?;
                    return Ok(());
                };
                let lhs = self.layout.operand_to_reg(lhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| make_instr_imm16_rhs(result, lhs, rhs),
                    FuelCostsProvider::base,
                )
            }
            (Operand::Immediate(lhs), rhs) => {
                let lhs = T::from(lhs.val());
                if lhs.is_zero() {
                    // Optimization: Shifting or rotating a zero value is a no-op.
                    self.stack.push_immediate(lhs)?;
                    return Ok(());
                }
                let lhs16 = self.make_imm16(lhs)?;
                let rhs = self.layout.operand_to_reg(rhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| match lhs16 {
                        Input::Immediate(lhs) => make_instr_imm16_lhs(result, lhs, rhs),
                        Input::Reg(lhs) => make_instr(result, lhs, rhs),
                    },
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => self.push_binary_instr_with_result(
                <T as Typed>::TY,
                lhs,
                rhs,
                make_instr,
                FuelCostsProvider::base,
            ),
        }
    }

    /// Translate a binary float Wasm operation.
    fn translate_fbinary<T, R>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        consteval: fn(T, T) -> R,
    ) -> Result<(), Error>
    where
        T: WasmFloat,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        let (lhs, rhs) = self.stack.pop2();
        if let (Operand::Immediate(lhs), Operand::Immediate(rhs)) = (lhs, rhs) {
            return self.translate_binary_consteval::<T, R>(lhs, rhs, consteval);
        }
        self.push_binary_instr_with_result(
            <R as Typed>::TY,
            lhs,
            rhs,
            make_instr,
            FuelCostsProvider::base,
        )
    }

    /// Translate Wasmi `{f32,f64}.copysign` instructions.
    ///
    /// # Note
    ///
    /// - This applies some optimization that are valid for copysign instructions.
    /// - Applies constant evaluation if both operands are constant values.
    fn translate_fcopysign<T>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm: fn(result: Reg, lhs: Reg, rhs: Sign<T>) -> Instruction,
        consteval: fn(T, T) -> T,
    ) -> Result<(), Error>
    where
        T: WasmFloat,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                self.translate_binary_consteval::<T, T>(lhs, rhs, consteval)
            }
            (lhs, Operand::Immediate(rhs)) => {
                let lhs = self.layout.operand_to_reg(lhs)?;
                let sign = T::from(rhs.val()).sign();
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| make_instr_imm(result, lhs, sign),
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => {
                if lhs.is_same(&rhs) {
                    // Optimization: `copysign x x` is always just `x`
                    self.stack.push_operand(lhs)?;
                    return Ok(());
                }
                self.push_binary_instr_with_result(
                    <T as Typed>::TY,
                    lhs,
                    rhs,
                    make_instr,
                    FuelCostsProvider::base,
                )
            }
        }
    }

    /// Translates a generic trap instruction.
    fn translate_trap(&mut self, trap: TrapCode) -> Result<(), Error> {
        self.push_instr(Instruction::trap(trap), FuelCostsProvider::base)?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `select` or `select <ty>` instruction.
    ///
    /// # Note
    ///
    /// - This applies constant propagation in case `condition` is a constant value.
    /// - If both `lhs` and `rhs` are equal registers or constant values `lhs` is forwarded.
    /// - Fuses compare instructions with the associated select instructions if possible.
    fn translate_select(&mut self, type_hint: Option<ValType>) -> Result<(), Error> {
        bail_unreachable!(self);
        let (true_val, false_val, condition) = self.stack.pop3();
        if let Some(type_hint) = type_hint {
            debug_assert_eq!(true_val.ty(), type_hint);
            debug_assert_eq!(false_val.ty(), type_hint);
        }
        let ty = true_val.ty();
        if true_val.is_same(&false_val) {
            // Optimization: both `lhs` and `rhs` either are the same register or constant values and
            //               thus `select` will always yield this same value irrespective of the condition.
            self.stack.push_operand(true_val)?;
            return Ok(());
        }
        if let Operand::Immediate(condition) = condition {
            // Optimization: since condition is a constant value we can const-fold the `select`
            //               instruction and simply push the selected value back to the provider stack.
            let condition = i32::from(condition.val()) != 0;
            let selected = match condition {
                true => true_val,
                false => false_val,
            };
            if let Operand::Temp(selected) = selected {
                // Case: the selected operand is a temporary which needs to be copied
                //       if it was the `false_val` since it changed its index. This is
                //       not the case for the `true_val` since `true_val` is the first
                //       value popped from the stack.
                if !condition {
                    let selected = self.layout.temp_to_reg(selected.operand_index())?;
                    self.push_instr_with_result(
                        ty,
                        |result| Instruction::copy(result, selected),
                        FuelCostsProvider::base,
                    )?;
                    return Ok(());
                }
            }
            self.stack.push_operand(selected)?;
            return Ok(());
        }
        let condition = self.layout.operand_to_reg(condition)?;
        let mut true_val = self.layout.operand_to_reg(true_val)?;
        let mut false_val = self.layout.operand_to_reg(false_val)?;
        match self
            .instrs
            .try_fuse_select(ty, condition, &self.layout, &mut self.stack)?
        {
            Some(swap_operands) => {
                if swap_operands {
                    mem::swap(&mut true_val, &mut false_val);
                }
            }
            None => {
                self.push_instr_with_result(
                    ty,
                    |result| Instruction::select_i32_eq_imm16(result, condition, 0_i16),
                    FuelCostsProvider::base,
                )?;
                mem::swap(&mut true_val, &mut false_val);
            }
        };
        self.push_param(Instruction::register2_ext(true_val, false_val))?;
        Ok(())
    }

    /// Create either [`Instruction::CallIndirectParams`] or [`Instruction::CallIndirectParamsImm16`] depending on the inputs.
    fn call_indirect_params(
        &mut self,
        index: Operand,
        table_index: u32,
    ) -> Result<Instruction, Error> {
        let table_type = *self.module.get_type_of_table(TableIdx::from(table_index));
        let index = self.make_index16(index, table_type.index_ty())?;
        let instr = match index {
            Input::Reg(index) => Instruction::call_indirect_params(index, table_index),
            Input::Immediate(index) => Instruction::call_indirect_params_imm16(index, table_index),
        };
        Ok(instr)
    }

    /// Tries to fuse a Wasm `i32.eqz` (or `i32.eq` with 0 `rhs` value) instruction.
    ///
    /// Returns
    ///
    /// - `Ok(true)` if the intruction fusion was successful.
    /// - `Ok(false)` if instruction fusion could not be applied.
    /// - `Err(_)` if an error occurred.
    pub fn fuse_eqz<T: WasmInteger>(&mut self, lhs: Operand, rhs: T) -> Result<bool, Error> {
        self.fuse_commutative_cmp_with(lhs, rhs, NegateCmpInstr::negate_cmp_instr)
    }

    /// Tries to fuse a Wasm `i32.ne` instruction with 0 `rhs` value.
    ///
    /// Returns
    ///
    /// - `Ok(true)` if the intruction fusion was successful.
    /// - `Ok(false)` if instruction fusion could not be applied.
    /// - `Err(_)` if an error occurred.
    pub fn fuse_nez<T: WasmInteger>(&mut self, lhs: Operand, rhs: T) -> Result<bool, Error> {
        self.fuse_commutative_cmp_with(lhs, rhs, LogicalizeCmpInstr::logicalize_cmp_instr)
    }

    /// Tries to fuse a `i{32,64}`.{eq,ne}` instruction with `rhs` of zero.
    ///
    /// Generically applies `f` onto the fused last instruction.
    ///
    /// Returns
    ///
    /// - `Ok(true)` if the intruction fusion was successful.
    /// - `Ok(false)` if instruction fusion could not be applied.
    /// - `Err(_)` if an error occurred.
    pub fn fuse_commutative_cmp_with<T: WasmInteger>(
        &mut self,
        lhs: Operand,
        rhs: T,
        try_fuse: fn(cmp: &Instruction) -> Option<Instruction>,
    ) -> Result<bool, Error> {
        if !rhs.is_zero() {
            // Case: cannot fuse with non-zero `rhs`
            return Ok(false);
        }
        let Some(last_instr) = self.instrs.last_instr() else {
            // Case: cannot fuse without registered last instruction
            return Ok(false);
        };
        let Operand::Temp(lhs) = lhs else {
            // Case: cannot fuse non-temporary operands
            //  - locals have observable behavior.
            //  - immediates cannot be the result of a previous instruction.
            return Ok(false);
        };
        let Some(origin) = lhs.instr() else {
            // Case: `lhs` has no origin instruciton, thus not possible to fuse.
            return Ok(false);
        };
        if origin != last_instr {
            // Case: `lhs`'s origin instruction does not match the last instruction
            return Ok(false);
        }
        let lhs_reg = self.layout.temp_to_reg(lhs.operand_index())?;
        let last_instruction = self.instrs.get(last_instr);
        let Some(result) = last_instruction.compare_result() else {
            // Case: cannot fuse non-cmp instructions
            return Ok(false);
        };
        if result != lhs_reg {
            // Case: the `cmp` instruction does not feed into the `eqz` and cannot be fused
            return Ok(false);
        }
        let Some(negated) = try_fuse(last_instruction) else {
            // Case: the `cmp` instruction cannot be negated
            return Ok(false);
        };
        // Need to push back `lhs` but with its type adjusted to be `i32`
        // since that's the return type of `i{32,64}.{eqz,eq,ne}`.
        let result_idx = self.stack.push_temp(ValType::I32, lhs.instr())?;
        // Need to replace `cmp` instruction result register since it might
        // have been misaligned if `lhs` originally referred to the zero operand.
        let new_result = self.layout.temp_to_reg(result_idx)?;
        let Some(negated) = negated.replace_cmp_result(new_result) else {
            unreachable!("`negated` has been asserted as `cmp` instruction");
        };
        if !self.instrs.try_replace_instr(last_instr, negated)? {
            unreachable!("`negated` has been asserted to be `last_instr`");
        }
        Ok(true)
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
        loaded_ty: ValType,
        make_instr: fn(result: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_offset16: fn(result: Reg, ptr: Reg, offset: Offset16) -> Instruction,
        make_instr_at: fn(result: Reg, address: Address32) -> Instruction,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg);
        let ptr = self.stack.pop();
        let (ptr, offset) = match ptr {
            Operand::Immediate(ptr) => {
                let ptr = ptr.val();
                let Some(address) = self.effective_address(memory, ptr, offset) else {
                    return self.translate_trap(TrapCode::MemoryOutOfBounds);
                };
                if let Ok(address) = Address32::try_from(address) {
                    self.push_instr_with_result(
                        loaded_ty,
                        |result| make_instr_at(result, address),
                        FuelCostsProvider::load,
                    )?;
                    if !memory.is_default() {
                        self.push_param(Instruction::memory_index(memory))?;
                    }
                    return Ok(());
                }
                // Case: we cannot use specialized encoding and thus have to fall back
                //       to the general case where `ptr` is zero and `offset` stores the
                //       `ptr+offset` address value.
                let zero_ptr = self.layout.const_to_reg(0_u64)?;
                (zero_ptr, u64::from(address))
            }
            ptr => {
                let ptr = self.layout.operand_to_reg(ptr)?;
                (ptr, offset)
            }
        };
        if memory.is_default() {
            if let Ok(offset) = Offset16::try_from(offset) {
                self.push_instr_with_result(
                    loaded_ty,
                    |result| make_instr_offset16(result, ptr, offset),
                    FuelCostsProvider::load,
                )?;
                return Ok(());
            }
        }
        let (offset_hi, offset_lo) = Offset64::split(offset);
        self.push_instr_with_result(
            loaded_ty,
            |result| make_instr(result, offset_lo),
            FuelCostsProvider::load,
        )?;
        self.push_param(Instruction::register_and_offset_hi(ptr, offset_hi))?;
        if !memory.is_default() {
            self.push_param(Instruction::memory_index(memory))?;
        }
        Ok(())
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
    fn translate_istore_wrap<T: op::StoreWrapOperator>(
        &mut self,
        memarg: MemArg,
    ) -> Result<(), Error>
    where
        T::Value: Copy + Wrap<T::Wrapped> + From<TypedVal>,
        T::Param: TryFrom<T::Wrapped> + Into<AnyConst16>,
    {
        bail_unreachable!(self);
        let (ptr, value) = self.stack.pop2();
        self.encode_istore_wrap::<T>(memarg, ptr, value)
    }

    /// Encodes Wasm integer `store` and `storeN` instructions as Wasmi bytecode.
    fn encode_istore_wrap<T: op::StoreWrapOperator>(
        &mut self,
        memarg: MemArg,
        ptr: Operand,
        value: Operand,
    ) -> Result<(), Error>
    where
        T::Value: Copy + Wrap<T::Wrapped> + From<TypedVal>,
        T::Param: TryFrom<T::Wrapped> + Into<AnyConst16>,
    {
        let (memory, offset) = Self::decode_memarg(memarg);
        let (ptr, offset) = match ptr {
            Operand::Immediate(ptr) => {
                let ptr = ptr.val();
                let Some(address) = self.effective_address(memory, ptr, offset) else {
                    return self.translate_trap(TrapCode::MemoryOutOfBounds);
                };
                if let Ok(address) = Address32::try_from(address) {
                    return self.encode_istore_wrap_at::<T>(memory, address, value);
                }
                // Case: we cannot use specialized encoding and thus have to fall back
                //       to the general case where `ptr` is zero and `offset` stores the
                //       `ptr+offset` address value.
                let zero_ptr = self.layout.const_to_reg(0_u64)?;
                (zero_ptr, u64::from(address))
            }
            ptr => {
                let ptr = self.layout.operand_to_reg(ptr)?;
                (ptr, offset)
            }
        };
        if memory.is_default() {
            if let Some(_instr) = self.encode_istore_wrap_mem0::<T>(ptr, offset, value)? {
                return Ok(());
            }
        }
        let (offset_hi, offset_lo) = Offset64::split(offset);
        let (instr, param) = {
            match value {
                Operand::Immediate(value) => {
                    let value = value.val();
                    match T::Param::try_from(T::Value::from(value).wrap()).ok() {
                        Some(value) => (
                            T::store_imm(ptr, offset_lo),
                            Instruction::imm16_and_offset_hi(value, offset_hi),
                        ),
                        None => (
                            T::store(ptr, offset_lo),
                            Instruction::register_and_offset_hi(
                                self.layout.const_to_reg(value)?,
                                offset_hi,
                            ),
                        ),
                    }
                }
                value => {
                    let value = self.layout.operand_to_reg(value)?;
                    (
                        T::store(ptr, offset_lo),
                        Instruction::register_and_offset_hi(value, offset_hi),
                    )
                }
            }
        };
        self.push_instr(instr, FuelCostsProvider::store)?;
        self.push_param(param)?;
        if !memory.is_default() {
            self.push_param(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    /// Encodes a Wasm integer `store` and `storeN` instructions as Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This is used in cases where the `ptr` is a known constant value.
    fn encode_istore_wrap_at<T: op::StoreWrapOperator>(
        &mut self,
        memory: index::Memory,
        address: Address32,
        value: Operand,
    ) -> Result<(), Error>
    where
        T::Value: Copy + From<TypedVal> + Wrap<T::Wrapped>,
        T::Param: TryFrom<T::Wrapped>,
    {
        match value {
            Operand::Immediate(value) => {
                let value = value.val();
                let wrapped = T::Value::from(value).wrap();
                if let Ok(value) = T::Param::try_from(wrapped) {
                    self.push_instr(T::store_at_imm(value, address), FuelCostsProvider::store)?;
                } else {
                    let value = self.layout.const_to_reg(value)?;
                    self.push_instr(T::store_at(value, address), FuelCostsProvider::store)?;
                }
            }
            value => {
                let value = self.layout.operand_to_reg(value)?;
                self.push_instr(T::store_at(value, address), FuelCostsProvider::store)?;
            }
        }
        if !memory.is_default() {
            self.push_param(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    /// Encodes a Wasm integer `store` and `storeN` instructions as Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This optimizes for cases where the Wasm linear memory that is operated on is known
    /// to be the default memory.
    /// Returns `Some` in case the optimized instructions have been encoded.
    fn encode_istore_wrap_mem0<T: op::StoreWrapOperator>(
        &mut self,
        ptr: Reg,
        offset: u64,
        value: Operand,
    ) -> Result<Option<Instr>, Error>
    where
        T::Value: Copy + From<TypedVal> + Wrap<T::Wrapped>,
        T::Param: TryFrom<T::Wrapped>,
    {
        let Ok(offset16) = Offset16::try_from(offset) else {
            return Ok(None);
        };
        let instr = match value {
            Operand::Immediate(value) => {
                let value = value.val();
                let wrapped = T::Value::from(value).wrap();
                match T::Param::try_from(wrapped) {
                    Ok(value) => self.push_instr(
                        T::store_offset16_imm(ptr, offset16, value),
                        FuelCostsProvider::store,
                    )?,
                    Err(_) => {
                        let value = self.layout.const_to_reg(value)?;
                        self.push_instr(
                            T::store_offset16(ptr, offset16, value),
                            FuelCostsProvider::store,
                        )?
                    }
                }
            }
            value => {
                let value = self.layout.operand_to_reg(value)?;
                self.push_instr(
                    T::store_offset16(ptr, offset16, value),
                    FuelCostsProvider::store,
                )?
            }
        };
        Ok(Some(instr))
    }

    /// Translates a general Wasm `store` instruction to Wasmi bytecode.
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
    /// - `{f32, f64, v128}.store`
    fn translate_store(
        &mut self,
        memarg: MemArg,
        store: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
        store_offset16: fn(ptr: Reg, offset: Offset16, value: Reg) -> Instruction,
        store_at: fn(value: Reg, address: Address32) -> Instruction,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg);
        let (ptr, value) = self.stack.pop2();
        let (ptr, offset) = match ptr {
            Operand::Immediate(ptr) => {
                let Some(address) = self.effective_address(memory, ptr.val(), offset) else {
                    return self.translate_trap(TrapCode::MemoryOutOfBounds);
                };
                if let Ok(address) = Address32::try_from(address) {
                    return self.encode_fstore_at(memory, address, value, store_at);
                }
                let zero_ptr = self.layout.const_to_reg(0_u64)?;
                (zero_ptr, u64::from(address))
            }
            ptr => {
                let ptr = self.layout.operand_to_reg(ptr)?;
                (ptr, offset)
            }
        };
        let (offset_hi, offset_lo) = Offset64::split(offset);
        let value = self.layout.operand_to_reg(value)?;
        if memory.is_default() {
            if let Ok(offset) = Offset16::try_from(offset) {
                self.push_instr(store_offset16(ptr, offset, value), FuelCostsProvider::store)?;
                return Ok(());
            }
        }
        self.push_instr(store(ptr, offset_lo), FuelCostsProvider::store)?;
        self.push_param(Instruction::register_and_offset_hi(value, offset_hi))?;
        if !memory.is_default() {
            self.push_param(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    /// Encodes a Wasm `store` instruction with immediate address as Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This is used in cases where the `ptr` is a known constant value.
    fn encode_fstore_at(
        &mut self,
        memory: index::Memory,
        address: Address32,
        value: Operand,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
    ) -> Result<(), Error> {
        let value = self.layout.operand_to_reg(value)?;
        self.push_instr(make_instr_at(value, address), FuelCostsProvider::store)?;
        if !memory.is_default() {
            self.push_param(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    /// Returns the [`MemArg`] linear `memory` index and load/store `offset`.
    ///
    /// # Panics
    ///
    /// If the [`MemArg`] offset is not 32-bit.
    fn decode_memarg(memarg: MemArg) -> (index::Memory, u64) {
        let memory = index::Memory::from(memarg.memory);
        (memory, memarg.offset)
    }

    /// Returns the effective address `ptr+offset` if it is valid.
    fn effective_address(&self, mem: index::Memory, ptr: TypedVal, offset: u64) -> Option<Address> {
        let memory_type = *self
            .module
            .get_type_of_memory(MemoryIdx::from(u32::from(mem)));
        let ptr = match memory_type.is_64() {
            true => u64::from(ptr),
            false => u64::from(u32::from(ptr)),
        };
        let Some(address) = ptr.checked_add(offset) else {
            // Case: address overflows any legal memory index.
            return None;
        };
        if let Some(max) = memory_type.maximum() {
            // The memory's maximum size in bytes.
            let max_size = max << memory_type.page_size_log2();
            if address > max_size {
                // Case: address overflows the memory's maximum size.
                return None;
            }
        }
        if !memory_type.is_64() && address >= 1 << 32 {
            // Case: address overflows the 32-bit memory index.
            return None;
        }
        let Ok(address) = Address::try_from(address) else {
            // Case: address is too big for the system to handle properly.
            return None;
        };
        Some(address)
    }

    /// Translates a Wasm `i64.binop128` instruction from the `wide-arithmetic` proposal.
    fn translate_i64_binop128(
        &mut self,
        make_instr: fn(results: [Reg; 2], lhs_lo: Reg) -> Instruction,
        const_eval: fn(lhs_lo: i64, lhs_hi: i64, rhs_lo: i64, rhs_hi: i64) -> (i64, i64),
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (rhs_lo, rhs_hi) = self.stack.pop2();
        let (lhs_lo, lhs_hi) = self.stack.pop2();
        if let (
            Operand::Immediate(lhs_lo),
            Operand::Immediate(lhs_hi),
            Operand::Immediate(rhs_lo),
            Operand::Immediate(rhs_hi),
        ) = (lhs_lo, lhs_hi, rhs_lo, rhs_hi)
        {
            let (result_lo, result_hi) = const_eval(
                lhs_lo.val().into(),
                lhs_hi.val().into(),
                rhs_lo.val().into(),
                rhs_hi.val().into(),
            );
            self.stack.push_immediate(result_lo)?;
            self.stack.push_immediate(result_hi)?;
            return Ok(());
        }
        let rhs_lo = self.layout.operand_to_reg(rhs_lo)?;
        let rhs_hi = self.layout.operand_to_reg(rhs_hi)?;
        let lhs_lo = self.layout.operand_to_reg(lhs_lo)?;
        let lhs_hi = self.layout.operand_to_reg(lhs_hi)?;
        let result_lo = self.stack.push_temp(ValType::I64, None)?;
        let result_hi = self.stack.push_temp(ValType::I64, None)?;
        let result_lo = self.layout.temp_to_reg(result_lo)?;
        let result_hi = self.layout.temp_to_reg(result_hi)?;
        self.push_instr(
            make_instr([result_lo, result_hi], lhs_lo),
            FuelCostsProvider::base,
        )?;
        self.push_param(Instruction::register3_ext(lhs_hi, rhs_lo, rhs_hi))?;
        Ok(())
    }

    /// Translates a Wasm `i64.mul_wide_sx` instruction from the `wide-arithmetic` proposal.
    fn translate_i64_mul_wide_sx(
        &mut self,
        make_instr: fn(results: FixedRegSpan<2>, lhs: Reg, rhs: Reg) -> Instruction,
        const_eval: fn(lhs: i64, rhs: i64) -> (i64, i64),
        signed: bool,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (lhs, rhs) = self.stack.pop2();
        let (lhs, rhs) = match (lhs, rhs) {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                let (result_lo, result_hi) = const_eval(lhs.val().into(), rhs.val().into());
                self.stack.push_immediate(result_lo)?;
                self.stack.push_immediate(result_hi)?;
                return Ok(());
            }
            (lhs, Operand::Immediate(rhs)) => {
                let rhs = rhs.val();
                if self.try_opt_i64_mul_wide_sx(lhs, rhs, signed)? {
                    return Ok(());
                }
                let lhs = self.layout.operand_to_reg(lhs)?;
                let rhs = self.layout.const_to_reg(rhs)?;
                (lhs, rhs)
            }
            (Operand::Immediate(lhs), rhs) => {
                let lhs = lhs.val();
                if self.try_opt_i64_mul_wide_sx(rhs, lhs, signed)? {
                    return Ok(());
                }
                let lhs = self.layout.const_to_reg(lhs)?;
                let rhs = self.layout.operand_to_reg(rhs)?;
                (lhs, rhs)
            }
            (lhs, rhs) => {
                let lhs = self.layout.operand_to_reg(lhs)?;
                let rhs = self.layout.operand_to_reg(rhs)?;
                (lhs, rhs)
            }
        };
        let result0 = self.stack.push_temp(ValType::I64, None)?;
        let _result1 = self.stack.push_temp(ValType::I64, None)?;
        let result0 = self.layout.temp_to_reg(result0)?;
        let Ok(results) = <FixedRegSpan<2>>::new(RegSpan::new(result0)) else {
            return Err(Error::from(TranslationError::AllocatedTooManyRegisters));
        };
        self.push_instr(make_instr(results, lhs, rhs), FuelCostsProvider::base)?;
        Ok(())
    }

    /// Try to optimize a `i64.mul_wide_sx` instruction with one [`Reg`] and one immediate input.
    ///
    /// - Returns `Ok(true)` if the optimiation was applied successfully.
    /// - Returns `Ok(false)` if no optimization was applied.
    fn try_opt_i64_mul_wide_sx(
        &mut self,
        lhs: Operand,
        rhs: TypedVal,
        signed: bool,
    ) -> Result<bool, Error> {
        let rhs = i64::from(rhs);
        if rhs == 0 {
            // Case: `mul(x, 0)` or `mul(0, x)` always evaluates to 0.
            self.stack.push_immediate(0_i64)?; // lo-bits
            self.stack.push_immediate(0_i64)?; // hi-bits
            return Ok(true);
        }
        if rhs == 1 && !signed {
            // Case: `mul(x, 1)` or `mul(1, x)` always evaluates to just `x`.
            // This is only valid if `x` is not a singed (negative) value.
            self.stack.push_operand(lhs)?; // lo-bits
            self.stack.push_immediate(0_i64)?; // hi-bits
            return Ok(true);
        }
        Ok(false)
    }
}
