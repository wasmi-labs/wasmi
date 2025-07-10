#![expect(dead_code)]

#[macro_use]
mod utils;
mod instrs;
mod layout;
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
    utils::{Operand16, Reset, ReusableAllocations},
};
use crate::{
    core::{FuelCostsProvider, IndexType, TrapCode, Typed, TypedVal, UntypedVal, ValType},
    engine::{
        translator::{
            comparator::{
                CompareResult as _,
                LogicalizeCmpInstr as _,
                NegateCmpInstr as _,
                TryIntoCmpBranchInstr as _,
            },
            labels::{LabelRef, LabelRegistry},
            utils::{Instr, WasmFloat, WasmInteger},
            WasmTranslator,
        },
        BlockType,
        CompiledFuncEntity,
        TranslationError,
    },
    ir::{
        BoundedRegSpan,
        BranchOffset,
        BranchOffset16,
        Comparator,
        ComparatorAndOffset,
        Const16,
        Const32,
        Instruction,
        IntoShiftAmount,
        Reg,
        RegSpan,
        Sign,
    },
    module::{FuncIdx, FuncTypeIdx, ModuleHeader, TableIdx, WasmiValueType},
    Engine,
    Error,
    FuncType,
};
use alloc::vec::Vec;
use core::mem;
use wasmparser::WasmFeatures;

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
}

impl Reset for FuncTranslatorAllocations {
    fn reset(&mut self) {
        self.stack.reset();
        self.layout.reset();
        self.labels.reset();
        self.instrs.reset();
        self.operands.clear();
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
        let Ok(max_height) = u16::try_from(self.stack.max_height()) else {
            return Err(Error::from(TranslationError::AllocatedTooManyRegisters));
        };
        self.update_branch_offsets()?;
        finalize(CompiledFuncEntity::new(
            max_height,
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

    /// Encode the top-most `len` operands on the stack as register list.
    ///
    /// # Note
    ///
    /// This is used for the following n-ary instructions:
    ///
    /// - [`Instruction::ReturnMany`]
    /// - [`Instruction::CopyMany`]
    /// - [`Instruction::CallInternal`]
    /// - [`Instruction::CallImported`]
    /// - [`Instruction::CallIndirect`]
    /// - [`Instruction::ReturnCallInternal`]
    /// - [`Instruction::ReturnCallImported`]
    /// - [`Instruction::ReturnCallIndirect`]
    pub fn encode_register_list(&mut self, len: usize) -> Result<(), Error> {
        self.stack.pop_n(len, &mut self.operands);
        let mut remaining = &self.operands[..];
        let mut operand_to_reg =
            |operand: &Operand| -> Result<Reg, Error> { self.layout.operand_to_reg(*operand) };
        let instr = loop {
            match remaining {
                [] => return Ok(()),
                [v0] => {
                    let v0 = operand_to_reg(v0)?;
                    break Instruction::register(v0);
                }
                [v0, v1] => {
                    let v0 = operand_to_reg(v0)?;
                    let v1 = operand_to_reg(v1)?;
                    break Instruction::register2_ext(v0, v1);
                }
                [v0, v1, v2] => {
                    let v0 = operand_to_reg(v0)?;
                    let v1 = operand_to_reg(v1)?;
                    let v2 = operand_to_reg(v2)?;
                    break Instruction::register3_ext(v0, v1, v2);
                }
                [v0, v1, v2, rest @ ..] => {
                    let v0 = operand_to_reg(v0)?;
                    let v1 = operand_to_reg(v1)?;
                    let v2 = operand_to_reg(v2)?;
                    let instr = Instruction::register_list_ext(v0, v1, v2);
                    self.instrs.push_param(instr);
                    remaining = rest;
                }
            };
        };
        self.instrs.push_param(instr);
        Ok(())
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

    /// Convert all branch params up to `depth` to [`Operand::Temp`].
    ///
    /// # Note
    ///
    /// - The top-most `depth` operands on the [`Stack`] will be [`Operand::Temp`] upon completion.
    /// - Does nothing if an [`Operand`] is already an [`Operand::Temp`].
    fn copy_branch_params(
        &mut self,
        depth: usize,
        consume_fuel: Option<Instr>,
    ) -> Result<(), Error> {
        for n in 0..depth {
            let operand = self.stack.operand_to_temp(n);
            self.copy_operand_to_temp(operand, consume_fuel)?;
        }
        Ok(())
    }

    /// Copy the top-most `len` [`Operand`]s into [`Operand::Temp`]s by copying if necessary.
    ///
    /// Returns the [`OperandIdx`] of the first [`Operand`].
    ///
    /// # Note
    ///
    /// - This does _not_ manipulate the [`Stack`].
    /// - Does nothing if an [`Operand`] is already an [`Operand::Temp`].
    fn copy_operands_to_temp(
        &mut self,
        len: usize,
        consume_fuel: Option<Instr>,
    ) -> Result<Option<OperandIdx>, Error> {
        let mut idx = None;
        for n in 0..len {
            let operand = self.stack.peek(n);
            self.copy_operand_to_temp(operand, consume_fuel)?;
            idx = Some(operand.index());
        }
        Ok(idx)
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
        frame_height == (self.stack.height() - len_branch_params)
            && (0..len_branch_params)
                .map(|depth| self.stack.peek(depth))
                .all(|o| o.is_temp())
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
        let instr = match operand {
            Operand::Temp(_) => return Ok(()),
            Operand::Local(operand) => {
                let result = self.layout.temp_to_reg(operand.operand_index())?;
                let value = self.layout.local_to_reg(operand.local_index())?;
                Instruction::copy(result, value)
            }
            Operand::Immediate(operand) => {
                let result = self.layout.temp_to_reg(operand.operand_index())?;
                self.make_copy_imm_instr(result, operand.val())?
            }
        };
        self.instrs
            .push_instr(instr, consume_fuel, FuelCostsProvider::base)?;
        Ok(())
    }

    /// Returns the copy instruction to copy the given immediate `value`.
    fn make_copy_imm_instr(&mut self, result: Reg, value: TypedVal) -> Result<Instruction, Error> {
        let instr = match value.ty() {
            ValType::I32 => Instruction::copy_imm32(result, i32::from(value)),
            ValType::I64 => {
                let value = i64::from(value);
                match <Const32<i64>>::try_from(value) {
                    Ok(value) => Instruction::copy_i64imm32(result, value),
                    Err(_) => {
                        let value = self.layout.const_to_reg(value)?;
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
                        let value = self.layout.const_to_reg(value)?;
                        Instruction::copy(result, value)
                    }
                }
            }
            ValType::V128 | ValType::FuncRef | ValType::ExternRef => {
                let value = self.layout.const_to_reg(value)?;
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
                let len_copies = usize::from(len_results);
                match self.try_form_regspan(len_copies)? {
                    Some(span) => {
                        let values = BoundedRegSpan::new(span, len_results);
                        Instruction::return_span(values)
                    }
                    None => {
                        let Some(first_idx) =
                            self.copy_operands_to_temp(len_copies, consume_fuel)?
                        else {
                            unreachable!("`first_idx` must be `Some` since `len_copies` is >0")
                        };
                        let result = self.layout.temp_to_reg(first_idx)?;
                        let values = BoundedRegSpan::new(RegSpan::new(result), len_results);
                        Instruction::return_span(values)
                    }
                }
            }
        };
        let instr = self
            .instrs
            .push_instr(instr, consume_fuel, FuelCostsProvider::base)?;
        Ok(instr)
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

    /// Translates the end of a Wasm `block` control frame.
    fn translate_end_block(&mut self, frame: BlockControlFrame) -> Result<(), Error> {
        let consume_fuel_instr = frame.consume_fuel_instr();
        if self.reachable && frame.is_branched_to() {
            let len_values = frame.len_branch_params(&self.engine);
            self.copy_branch_params(usize::from(len_values), consume_fuel_instr)?;
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
        Ok(())
    }

    /// Translates the end of a Wasm `loop` control frame.
    fn translate_end_loop(&mut self, _frame: LoopControlFrame) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        // Nothing needs to be done since Wasm `loop` control frames always only have a single exit.
        Ok(())
    }

    /// Translates the end of a Wasm `if` control frame.
    fn translate_end_if(&mut self, frame: IfControlFrame) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        let IfReachability::Both { else_label } = frame.reachability() else {
            let reachability = frame.reachability().into();
            return self.translate_end_if_or_else_only(frame, reachability);
        };
        let end_of_then_reachable = self.reachable;
        let len_results = frame.ty().len_results(self.engine());
        let has_results = len_results >= 1;
        if end_of_then_reachable && has_results {
            let consume_fuel_instr = frame.consume_fuel_instr();
            self.copy_branch_params(usize::from(len_results), consume_fuel_instr)?;
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
        let next_instr = self.instrs.next_instr();
        self.labels.try_pin_label(else_label, next_instr);
        self.labels.pin_label(frame.label(), next_instr).unwrap();
        self.reachable = true;
        Ok(())
    }

    /// Translates the end of a Wasm `else` control frame.
    fn translate_end_else(&mut self, frame: ElseControlFrame) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        let reachability = frame.reachability();
        if matches!(
            reachability,
            ElseReachability::OnlyThen | ElseReachability::OnlyElse
        ) {
            return self.translate_end_if_or_else_only(frame, reachability);
        }
        let end_of_then_reachable = frame.is_end_of_then_reachable();
        let end_of_else_reachable = self.reachable;
        let reachable = match (end_of_then_reachable, end_of_else_reachable) {
            (false, false) => frame.is_branched_to(),
            _ => true,
        };
        if end_of_else_reachable {
            let len_values = frame.len_branch_params(&self.engine);
            let consume_fuel_instr: Option<Instr> = frame.consume_fuel_instr();
            self.copy_branch_params(usize::from(len_values), consume_fuel_instr)?;
        }
        self.labels
            .pin_label(frame.label(), self.instrs.next_instr())
            .unwrap();
        self.reachable = reachable;
        Ok(())
    }

    /// Translates the end of a Wasm `else` control frame where only one branch is known to be reachable.
    fn translate_end_if_or_else_only(
        &mut self,
        frame: impl ControlFrameBase,
        reachability: ElseReachability,
    ) -> Result<(), Error> {
        let end_is_reachable = match reachability {
            ElseReachability::OnlyThen => self.reachable,
            ElseReachability::OnlyElse => true,
            ElseReachability::Both => unreachable!(),
        };
        if end_is_reachable && frame.is_branched_to() {
            let len_values = frame.len_branch_params(&self.engine);
            let consume_fuel_instr = frame.consume_fuel_instr();
            self.copy_branch_params(usize::from(len_values), consume_fuel_instr)?;
        }
        self.labels
            .pin_label(frame.label(), self.instrs.next_instr())
            .unwrap();
        self.reachable = end_is_reachable || frame.is_branched_to();
        Ok(())
    }

    /// Translates the end of an unreachable Wasm control frame.
    fn translate_end_unreachable(&mut self, _frame: ControlFrameKind) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
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
        let instr = match input {
            Operand::Immediate(operand) => self.make_copy_imm_instr(result, operand.val())?,
            operand => {
                let input = self.layout.operand_to_reg(operand)?;
                Instruction::copy(result, input)
            }
        };
        self.instrs
            .push_instr(instr, consume_fuel_instr, FuelCostsProvider::base)?;
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
    fn encode_br(&mut self, label: LabelRef) -> Result<(), Error> {
        let instr = self.instrs.next_instr();
        let offset = self.labels.try_resolve_label(label, instr)?;
        self.push_instr(Instruction::branch(offset), FuelCostsProvider::base)?;
        self.reachable = false;
        Ok(())
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
                    true => return self.encode_br(label),
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
        let Operand::Temp(condition) = condition else {
            return Ok(false);
        };
        debug_assert!(matches!(condition.ty(), ValType::I32 | ValType::I64));
        let Some(origin) = condition.instr() else {
            return Ok(false);
        };
        let fused_instr = self.try_make_fused_branch_cmp_instr(origin, condition, label, negate)?;
        let Some(fused_instr) = fused_instr else {
            return Ok(false);
        };
        self.instrs.try_replace_instr(origin, fused_instr)
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
        cmp_instr.try_into_cmp_branch_instr(offset, &mut self.layout)
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
                todo!() // do we need a copy or should we allow to manipulate a local's type?
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

    /// Creates a new 16-bit encoded [`Operand16`] from the given `value`.
    pub fn make_imm16<T>(&mut self, value: T) -> Result<Operand16<T>, Error>
    where
        T: Into<UntypedVal> + Copy + TryInto<Const16<T>>,
    {
        match value.try_into() {
            Ok(rhs) => Ok(Operand16::Immediate(rhs)),
            Err(_) => {
                let rhs = self.layout.const_to_reg(value)?;
                Ok(Operand16::Reg(rhs))
            }
        }
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
    ) -> Result<Operand16<u64>, Error> {
        let value = match operand {
            Operand::Immediate(value) => value.val(),
            operand => {
                debug_assert_eq!(operand.ty(), index_type.ty());
                let reg = self.layout.operand_to_reg(operand)?;
                return Ok(Operand16::Reg(reg));
            }
        };
        match index_type {
            IndexType::I64 => {
                if let Ok(value) = Const16::try_from(u64::from(value)) {
                    return Ok(Operand16::Immediate(value));
                }
            }
            IndexType::I32 => {
                if let Ok(value) = Const16::try_from(u32::from(value)) {
                    return Ok(Operand16::Immediate(<Const16<u64>>::cast(value)));
                }
            }
        }
        let reg = self.layout.const_to_reg(value)?;
        Ok(Operand16::Reg(reg))
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
                        Operand16::Immediate(rhs) => make_ri(result, lhs, rhs),
                        Operand16::Reg(rhs) => make_rr(result, lhs, rhs),
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
                        Operand16::Immediate(rhs) => make_instr_imm16_rhs(result, lhs, rhs),
                        Operand16::Reg(rhs) => make_instr(result, lhs, rhs),
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
                        Operand16::Immediate(lhs) => make_instr_imm16_lhs(result, lhs, rhs),
                        Operand16::Reg(lhs) => make_instr(result, lhs, rhs),
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
                        Operand16::Immediate(rhs) => make_instr_imm16_rhs(result, lhs, rhs),
                        Operand16::Reg(rhs) => make_instr(result, lhs, rhs),
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
                        Operand16::Immediate(lhs) => make_instr_imm16_lhs(result, lhs, rhs),
                        Operand16::Reg(lhs) => make_instr(result, lhs, rhs),
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
                    Ok(rhs) => Operand16::Immediate(rhs),
                    Err(_) => {
                        let rhs = self.layout.const_to_reg(rhs)?;
                        Operand16::Reg(rhs)
                    }
                };
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| match rhs16 {
                        Operand16::Immediate(rhs) => make_add_ri(result, lhs, rhs),
                        Operand16::Reg(rhs) => make_sub_rr(result, lhs, rhs),
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
                        Operand16::Immediate(lhs) => make_sub_ir(result, lhs, rhs),
                        Operand16::Reg(lhs) => make_sub_rr(result, lhs, rhs),
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
                        Operand16::Immediate(lhs) => make_instr_imm16_lhs(result, lhs, rhs),
                        Operand16::Reg(lhs) => make_instr(result, lhs, rhs),
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
        self.instrs
            .push_param(Instruction::register2_ext(true_val, false_val));
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
            Operand16::Reg(index) => Instruction::call_indirect_params(index, table_index),
            Operand16::Immediate(index) => {
                Instruction::call_indirect_params_imm16(index, table_index)
            }
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
        if !rhs.is_zero() {
            // Case: cannot fuse with non-zero `rhs`
            return Ok(false);
        }
        let lhs_reg = match lhs {
            Operand::Immediate(_) => {
                // Case: const-eval opt should take place instead since both operands are const
                return Ok(false);
            }
            operand => self.layout.operand_to_reg(operand)?,
        };
        let Some(last_instr) = self.instrs.last_instr() else {
            // Case: cannot fuse without registered last instruction
            return Ok(false);
        };
        let last_instruction = *self.instrs.get(last_instr);
        let Some(result) = last_instruction.compare_result() else {
            // Case: cannot fuse non-cmp instructions
            return Ok(false);
        };
        if matches!(self.layout.stack_space(result), StackSpace::Local) {
            // Case: cannot fuse cmp instructions with local result
            // Note: local results have observable side effects which must not change
            return Ok(false);
        }
        if result != lhs_reg {
            // Case: the `cmp` instruction does not feed into the `eqz` and cannot be fused
            return Ok(false);
        }
        let Some(negated) = last_instruction.negate_cmp_instr() else {
            // Case: the `cmp` instruction cannot be negated
            return Ok(false);
        };
        if !self.instrs.try_replace_instr(last_instr, negated)? {
            // Case: could not replace the `cmp` instruction with the fused one
            return Ok(false);
        }
        self.stack.push_operand(lhs)?;
        Ok(true)
    }

    /// Tries to fuse a Wasm `i32.ne` instruction with 0 `rhs` value.
    ///
    /// Returns
    ///
    /// - `Ok(true)` if the intruction fusion was successful.
    /// - `Ok(false)` if instruction fusion could not be applied.
    /// - `Err(_)` if an error occurred.
    pub fn fuse_nez<T: WasmInteger>(&mut self, lhs: Operand, rhs: T) -> Result<bool, Error> {
        if !rhs.is_zero() {
            // Case: cannot fuse with non-zero `rhs`
            return Ok(false);
        }
        let lhs_reg = match lhs {
            Operand::Immediate(_) => {
                // Case: const-eval opt should take place instead since both operands are const
                return Ok(false);
            }
            operand => self.layout.operand_to_reg(operand)?,
        };
        let Some(last_instr) = self.instrs.last_instr() else {
            // Case: cannot fuse without registered last instruction
            return Ok(false);
        };
        let last_instruction = *self.instrs.get(last_instr);
        let Some(result) = last_instruction.compare_result() else {
            // Case: cannot fuse non-cmp instructions
            return Ok(false);
        };
        if matches!(self.layout.stack_space(result), StackSpace::Local) {
            // Case: cannot fuse cmp instructions with local result
            // Note: local results have observable side effects which must not change
            return Ok(false);
        }
        if result != lhs_reg {
            // Case: the `cmp` instruction does not feed into the `nez` and cannot be fused
            return Ok(false);
        }
        let Some(logicalized) = last_instruction.logicalize_cmp_instr() else {
            // Case: the `cmp` instruction cannot be logicalized
            return Ok(false);
        };
        if !self.instrs.try_replace_instr(last_instr, logicalized)? {
            // Case: could not replace the `cmp` instruction with the fused one
            return Ok(false);
        }
        self.stack.push_operand(lhs)?;
        Ok(true)
    }
}
