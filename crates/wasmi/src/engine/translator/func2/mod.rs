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
    core::{FuelCostsProvider, TrapCode, Typed, TypedVal, UntypedVal, ValType},
    engine::{
        translator::{
            comparator::{CompareResult as _, NegateCmpInstr as _, TryIntoCmpBranchInstr as _},
            labels::{LabelRef, LabelRegistry},
            utils::{Instr, WasmInteger},
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
    },
    module::{FuncIdx, ModuleHeader, WasmiValueType},
    Engine,
    Error,
    FuncType,
};
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
}

impl Reset for FuncTranslatorAllocations {
    fn reset(&mut self) {
        self.stack.reset();
        self.layout.reset();
        self.labels.reset();
        self.instrs.reset();
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

    /// Returns the [`FuncType`] of the function that is currently translated.
    fn func_type(&self) -> FuncType {
        self.func_type_with(FuncType::clone)
    }

    /// Applies `f` to the [`FuncType`] of the function that is currently translated.
    fn func_type_with<R>(&self, f: impl FnOnce(&FuncType) -> R) -> R {
        let dedup_func_type = self.module.get_type_of_func(self.func);
        self.engine().resolve_func_type(dedup_func_type, f)
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

    /// Copy all [`Operand`]s up to `depth` into [`Operand::Temp`]s by copying if necessary.
    ///
    ///
    /// # Note
    ///
    /// - This does _not_ manipulate the [`Stack`].
    /// - Does nothing if an [`Operand`] is already an [`Operand::Temp`].
    fn copy_operands_to_temp(&mut self, depth: usize) -> Result<(), Error> {
        let consume_fuel = self.stack.consume_fuel_instr();
        for n in 0..depth {
            let operand = self.stack.peek(n);
            self.copy_operand_to_temp(operand, consume_fuel)?;
        }
        Ok(())
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
                let val = operand.val();
                match operand.ty() {
                    ValType::I32 => Instruction::copy_imm32(result, i32::from(val)),
                    ValType::I64 => {
                        let val = i64::from(val);
                        match <Const32<i64>>::try_from(val) {
                            Ok(value) => Instruction::copy_i64imm32(result, value),
                            Err(_) => {
                                let value = self.layout.const_to_reg(val)?;
                                Instruction::copy(result, value)
                            }
                        }
                    }
                    ValType::F32 => Instruction::copy_imm32(result, f32::from(val)),
                    ValType::F64 => {
                        let val = f64::from(val);
                        match <Const32<f64>>::try_from(val) {
                            Ok(value) => Instruction::copy_f64imm32(result, value),
                            Err(_) => {
                                let value = self.layout.const_to_reg(val)?;
                                Instruction::copy(result, value)
                            }
                        }
                    }
                    ValType::V128 | ValType::FuncRef | ValType::ExternRef => {
                        let value = self.layout.const_to_reg(val)?;
                        Instruction::copy(result, value)
                    }
                }
            }
        };
        self.instrs
            .push_instr(instr, consume_fuel, FuelCostsProvider::base)?;
        Ok(())
    }

    /// Pushes the `instr` to the function with the associated `fuel_costs`.
    fn push_instr(
        &mut self,
        instr: Instruction,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), Error> {
        let consume_fuel = self.stack.consume_fuel_instr();
        self.instrs.push_instr(instr, consume_fuel, fuel_costs)?;
        Ok(())
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
            _ => {
                let depth = usize::from(len_results);
                self.copy_operands_to_temp(depth)?;
                let first_idx = self.stack.peek(depth).index();
                let result = self.layout.temp_to_reg(first_idx)?;
                let values = BoundedRegSpan::new(RegSpan::new(result), len_results);
                Instruction::return_span(values)
            }
        };
        let instr = self
            .instrs
            .push_instr(instr, consume_fuel, FuelCostsProvider::base)?;
        Ok(instr)
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
        self.push_instr(instr, FuelCostsProvider::base)
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
        debug_assert_eq!(condition.ty(), ValType::I32);
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
        T: From<TypedVal>,
        R: Into<TypedVal>,
    {
        bail_unreachable!(self);
        match self.stack.pop() {
            Operand::Local(input) => {
                debug_assert!(matches!(input.ty(), ValType::I32));
                todo!() // do we need a copy or should we allow to manipulate a local's type?
            }
            Operand::Temp(input) => {
                debug_assert!(matches!(input.ty(), ValType::I32));
                self.stack.push_temp(ValType::I64, None)?;
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

    /// Translates a commutative binary Wasm operator to Wasmi bytecode.
    fn translate_binary_commutative<T, R>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm16: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
        consteval: fn(T, T) -> R,
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
                let lhs = self.layout.operand_to_reg(val)?;
                let rhs = imm.val().into();
                let rhs16 = self.make_imm16(rhs)?;
                self.push_instr_with_result(
                    <R as Typed>::TY,
                    |result| match rhs16 {
                        Operand16::Immediate(rhs) => make_instr_imm16(result, lhs, rhs),
                        Operand16::Reg(rhs) => make_instr(result, lhs, rhs),
                    },
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => {
                let lhs = self.layout.operand_to_reg(lhs)?;
                let rhs = self.layout.operand_to_reg(rhs)?;
                self.push_instr_with_result(
                    <R as Typed>::TY,
                    |result| make_instr(result, lhs, rhs),
                    FuelCostsProvider::base,
                )
            }
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
            (lhs, rhs) => {
                let lhs = self.layout.operand_to_reg(lhs)?;
                let rhs = self.layout.operand_to_reg(rhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| make_instr(result, lhs, rhs),
                    FuelCostsProvider::base,
                )
            }
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
        R: Into<TypedVal>,
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
            (lhs, rhs) => {
                let lhs = self.layout.operand_to_reg(lhs)?;
                let rhs = self.layout.operand_to_reg(rhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| make_instr(result, lhs, rhs),
                    FuelCostsProvider::base,
                )
            }
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
            (lhs, rhs) => {
                let lhs = self.layout.operand_to_reg(lhs)?;
                let rhs = self.layout.operand_to_reg(rhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| make_instr(result, lhs, rhs),
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
}
