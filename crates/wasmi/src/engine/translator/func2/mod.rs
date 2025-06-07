#![expect(dead_code, unused_imports)]

#[macro_use]
mod utils;
mod instrs;
mod layout;
#[cfg(feature = "simd")]
mod simd;
mod stack;
mod visit;

use self::{
    instrs::InstrEncoder,
    layout::{StackLayout, StackSpace},
    stack::{
        BlockControlFrame,
        ControlFrame,
        ElseControlFrame,
        IfControlFrame,
        LocalIdx,
        LoopControlFrame,
        Operand,
        OperandIdx,
        Stack,
        StackAllocations,
        UnreachableControlFrame,
    },
    utils::Reset,
};
use crate::{
    core::{FuelCostsProvider, Typed, TypedVal, ValType},
    engine::{
        translator::{utils::FuelInfo, Instr, LabelRef, LabelRegistry, WasmTranslator},
        BlockType,
        CompiledFuncEntity,
        TranslationError,
    },
    ir::{BoundedRegSpan, Const16, Const32, Instruction, Reg, RegSpan},
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
    instrs: InstrEncoder,
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
        let consume_fuel = self
            .fuel_costs
            .as_ref()
            .map(|_| self.instrs.push_instr(Instruction::consume_fuel(1)));
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
        let dedup_func_type = self.module.get_type_of_func(self.func);
        self.engine()
            .resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Consumes `self` and returns the underlying reusable [`FuncTranslatorAllocations`].
    fn into_allocations(self) -> FuncTranslatorAllocations {
        FuncTranslatorAllocations {
            stack: self.stack.into_allocations(),
            layout: self.layout,
            labels: self.labels,
            instrs: self.instrs,
        }
    }

    /// Returns the [`Engine`] for which the function is compiled.
    fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Returns `true` if fuel metering is enabled.
    fn is_fuel_metering_enabled(&self) -> bool {
        self.engine.config().get_consume_fuel()
    }

    /// Translates the end of a Wasm `block` control frame.
    fn translate_end_block(&mut self, frame: BlockControlFrame) -> Result<(), Error> {
        if self.stack.is_control_empty() {
            return self.translate_end_func(frame);
        }
        todo!()
    }

    /// Translates the end of the Wasm function enclosing Wasm `block`.
    fn translate_end_func(&mut self, frame: BlockControlFrame) -> Result<(), Error> {
        let _fuel_info = match (&self.fuel_costs, frame.consume_fuel_instr()) {
            (Some(fuel_costs), Some(consume_fuel)) => {
                FuelInfo::some(fuel_costs.clone(), consume_fuel)
            }
            (None, None) => FuelInfo::None,
            _ => unreachable!(),
        };
        let len_results = frame.ty().len_results(&self.engine);
        if self.reachable && frame.is_branched_to() && len_results > 1 {
            let height = frame.height();
            let len_results = usize::from(len_results);
            for depth in 0..len_results {
                let result = self
                    .layout
                    .temp_to_reg(OperandIdx::from(height + len_results - depth - 1))?;
                match self.stack.operand_to_temp(depth) {
                    Some(Operand::Local(operand)) => {
                        let value = self.layout.local_to_reg(operand.local_index())?;
                        self.instrs.push_instr(Instruction::copy(result, value));
                    }
                    Some(Operand::Immediate(operand)) => {
                        let val = operand.val();
                        let instr = match operand.ty() {
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
                        };
                        self.instrs.push_instr(instr);
                    }
                    _ => {}
                }
            }
        }
        self.labels
            .pin_label(frame.label(), self.instrs.next_instr())
            .unwrap_or_else(|err| panic!("failed to pin label: {err}"));
        match len_results {
            0 => {
                self.instrs.push_instr(Instruction::Return);
            }
            1 => {
                let instr = match self.stack.peek(0) {
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
                };
                self.instrs.push_instr(instr);
            }
            _ => {
                let height = frame.height();
                let result = self.layout.temp_to_reg(OperandIdx::from(height - 1))?;
                let values = BoundedRegSpan::new(RegSpan::new(result), len_results);
                self.instrs.push_instr(Instruction::return_span(values));
            }
        }
        Ok(())
    }

    /// Translates the end of a Wasm `loop` control frame.
    fn translate_end_loop(&mut self, _frame: LoopControlFrame) -> Result<(), Error> {
        todo!()
    }

    /// Translates the end of a Wasm `if` control frame.
    fn translate_end_if(&mut self, _frame: IfControlFrame) -> Result<(), Error> {
        todo!()
    }

    /// Translates the end of a Wasm `else` control frame.
    fn translate_end_else(&mut self, _frame: ElseControlFrame) -> Result<(), Error> {
        todo!()
    }

    /// Translates the end of an unreachable Wasm control frame.
    fn translate_end_unreachable(&mut self, _frame: UnreachableControlFrame) -> Result<(), Error> {
        todo!()
    }

    /// Translates a commutative binary Wasm operator to Wasmi bytecode.
    fn translate_binary_commutative<T, R>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm16: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
        consteval: fn(T, T) -> R,
    ) -> Result<(), Error>
    where
        T: Copy + From<TypedVal> + TryInto<Const16<T>>,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                let value = consteval(lhs.val().into(), rhs.val().into());
                self.stack.push_immediate(value)?;
                Ok(())
            }
            (val, Operand::Immediate(imm)) | (Operand::Immediate(imm), val) => {
                let lhs = self.layout.operand_to_reg(val)?;
                let iidx = self.instrs.next_instr();
                let result = self
                    .layout
                    .temp_to_reg(self.stack.push_temp(<R as Typed>::TY, Some(iidx))?)?;
                let instr = match T::from(imm.val()).try_into() {
                    Ok(rhs) => make_instr_imm16(result, lhs, rhs),
                    Err(_) => {
                        let rhs = self.layout.const_to_reg(imm.val())?;
                        make_instr(result, lhs, rhs)
                    }
                };
                assert_eq!(self.instrs.push_instr(instr), iidx);
                Ok(())
            }
            (lhs, rhs) => {
                let lhs = self.layout.operand_to_reg(lhs)?;
                let rhs = self.layout.operand_to_reg(rhs)?;
                let iidx = self.instrs.next_instr();
                let result = self
                    .layout
                    .temp_to_reg(self.stack.push_temp(<R as Typed>::TY, Some(iidx))?)?;
                assert_eq!(self.instrs.push_instr(make_instr(result, lhs, rhs)), iidx);
                Ok(())
            }
        }
    }
}
