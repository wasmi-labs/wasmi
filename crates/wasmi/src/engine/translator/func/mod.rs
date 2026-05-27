#[macro_use]
mod utils;
mod encoder;
mod labels;
mod layout;
mod locals;
mod op;
#[cfg(feature = "simd")]
mod simd;
mod stack;
mod visit;

#[cfg(doc)]
use self::stack::ImmediateOperand;

use self::{
    encoder::{OpEncoder, OpEncoderAllocations, Pos},
    labels::{LabelRef, LabelRegistry},
    layout::{StackLayout, StackSpace},
    locals::{LocalIdx, LocalsRegistry},
    op::{BinaryOp, CommutativeBinaryOp, UnaryOp},
    stack::{
        BlockControlFrame,
        ControlFrame,
        ControlFrameBase,
        ControlFrameKind,
        ElseControlFrame,
        ElseReachability,
        IfControlFrame,
        IfReachability,
        LocalOperand,
        Location,
        LoopControlFrame,
        Operand,
        ResolvedOperand,
        Stack,
        StackAllocations,
    },
    utils::{Reset, ReusableAllocations, UpdateResultSlot},
};
#[cfg(feature = "simd")]
use crate::V128;
use crate::{
    Engine,
    Error,
    FuncType,
    TrapCode,
    ValType,
    core::{FuelCostsProvider, IndexType, RawRef, RawVal, Typed, TypedRawVal},
    engine::{
        BlockType,
        Cell,
        CompiledFuncEntity,
        TranslationError,
        translator::{
            WasmTranslator,
            comparator::{
                LogicalizeCmpInstr,
                NegateCmpInstr,
                TryIntoCmpBranchInstr as _,
                UpdateBranchOffset as _,
            },
            func::{
                op::BinaryOpRhs,
                stack::{RegOperand, TempOperand},
            },
            utils::{ToBits, WasmInteger, required_cells_for_ty},
        },
    },
    ir::{
        self,
        Address,
        BoundedSlotSpan,
        BranchOffset,
        FixedSlotSpan,
        Offset16,
        Op,
        Slot,
        SlotSpan,
        index,
        index::Memory,
    },
    module::{FuncIdx, FuncTypeIdx, MemoryIdx, ModuleHeader, WasmiValueType},
};
use alloc::vec::Vec;
use core::{convert::identity, mem};
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
    /// Wasm value and control stack.
    stack: Stack,
    /// Types of local variables and function parameters.
    locals: LocalsRegistry,
    /// Wasm layout to map stack slots to Wasmi registers.
    layout: StackLayout,
    /// Constructs and encodes function instructions.
    instrs: OpEncoder,
    /// Temporary buffer for operands.
    operands: Vec<Operand>,
    /// Temporary buffer for immediate values.
    immediates: Vec<TypedRawVal>,
}

/// Heap allocated data structured used by the [`FuncTranslator`].
#[derive(Debug, Default)]
pub struct FuncTranslatorAllocations {
    /// Wasm value and control stack.
    stack: StackAllocations,
    /// Types of local variables and function parameters.
    locals: LocalsRegistry,
    /// Wasm layout to map stack slots to Wasmi registers.
    layout: StackLayout,
    /// Constructs and encodes function instructions.
    instrs: OpEncoderAllocations,
    /// Temporary buffer for operands.
    operands: Vec<Operand>,
    /// Temporary buffer for immediate values.
    immediates: Vec<TypedRawVal>,
}

impl Reset for FuncTranslatorAllocations {
    fn reset(&mut self) {
        self.stack.reset();
        self.locals.reset();
        self.layout.reset();
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

    fn features(&self) -> WasmFeatures {
        self.engine.config().wasm_features()
    }

    fn translate_locals(
        &mut self,
        amount: u32,
        value_type: wasmparser::ValType,
    ) -> Result<(), Error> {
        let ty = WasmiValueType::from(value_type).into_inner();
        self.register_locals(amount, ty)?;
        Ok(())
    }

    fn finish_translate_locals(&mut self) -> Result<(), Error> {
        // Note: must initialize function body `block` after registering all
        //       function parameters and locals so that the function `block`
        //       has proper knowledge of its position within the operands stack.
        self.init_func_body_block()?;
        Ok(())
    }

    fn update_pos(&mut self, _pos: usize) {}

    fn finish(
        mut self,
        finalize: impl FnOnce(CompiledFuncEntity),
    ) -> Result<Self::Allocations, Error> {
        // Note: `update_branch_offsets` might change `frame_size` so we need to compute it prior.
        //
        // Context:
        // This only happens if the function has so many instructions that some conditional branch
        // operators need to be encoded as their fallbacks which requires to allocate more function
        // local constant values, thus increasing the size of the function frame.
        self.instrs.update_branch_offsets()?;
        let Some(frame_size) = self.frame_size() else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        finalize(CompiledFuncEntity::new(
            frame_size,
            self.instrs.encoded_ops(),
        ));
        Ok(self.into_allocations())
    }
}

impl ReusableAllocations for FuncTranslator {
    type Allocations = FuncTranslatorAllocations;

    fn into_allocations(self) -> Self::Allocations {
        Self::Allocations {
            stack: self.stack.into_allocations(),
            locals: self.locals,
            layout: self.layout,
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
        let FuncTranslatorAllocations {
            stack,
            locals,
            layout,
            instrs,
            operands,
            immediates,
        } = alloc.into_reset();
        let stack = Stack::new(&engine, stack);
        let instrs = OpEncoder::new(&engine, instrs);
        let mut translator = Self {
            func,
            engine,
            module,
            reachable: true,
            stack,
            locals,
            layout,
            instrs,
            operands,
            immediates,
        };
        translator.init_func_params()?;
        Ok(translator)
    }

    /// Initializes the function's parameters.
    fn init_func_params(&mut self) -> Result<(), Error> {
        for ty in self.func_type().params() {
            self.register_locals(1, *ty)?;
        }
        Ok(())
    }

    /// Slots an `amount` of local variables of type `ty`.
    fn register_locals(&mut self, amount: u32, ty: ValType) -> Result<(), Error> {
        let Ok(amount) = usize::try_from(amount) else {
            panic!(
                "failed to register {amount} local variables of type {ty:?}: out of bounds `usize`"
            )
        };
        self.locals.register(amount, ty)?;
        self.stack.register_locals(amount, ty)?;
        self.layout.register_locals(amount, ty)?;
        Ok(())
    }

    /// Initializes the function body enclosing control block.
    fn init_func_body_block(&mut self) -> Result<(), Error> {
        let func_ty = self.module.get_type_of_func(self.func);
        let block_ty = BlockType::func_type(func_ty);
        let end_label = self.instrs.new_label();
        let consume_fuel = self.instrs.encode_consume_fuel()?;
        self.stack
            .push_func_block(block_ty, end_label, consume_fuel)?;
        Ok(())
    }

    /// Returns the frame size of the to-be-compiled function.
    ///
    /// Returns `None` if the frame size is out of bounds.
    fn frame_size(&self) -> Option<u16> {
        let frame_size = self
            .stack
            .max_stack_offset()
            .checked_add(self.locals.len())?;
        u16::try_from(frame_size).ok()
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

    /// Returns the [`Engine`] for which the function is compiled.
    fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Copy the top-most `len` operands to [`Operand::Temp`] values.
    ///
    /// Returns a [`SlotSpan`] to the copied operands.
    ///
    /// # Note
    ///
    /// - The top-most `len` operands on the [`Stack`] will be [`Operand::Temp`] upon completion.
    /// - Does nothing if an [`Operand`] is already an [`Operand::Temp`].
    fn move_operands_to_temp(
        &mut self,
        len: usize,
        consume_fuel: Option<Pos<ir::BlockFuel>>,
    ) -> Result<BoundedSlotSpan, Error> {
        debug_assert!(len > 0);
        let mut copied_cells: u16 = 0;
        for n in 0..len {
            let operand = self.stack.operand_to_temp(n);
            copied_cells = copied_cells
                .checked_add(operand.temp_slots().len())
                .ok_or(TranslationError::SlotAccessOutOfBounds)?;
            self.copy_operand_to_temp(operand, consume_fuel)?;
        }
        let first = self.stack.peek(len - 1).temp_slots().head();
        Ok(BoundedSlotSpan::new(SlotSpan::new(first), copied_cells))
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
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        let len_branch_params = target.len_branch_params(&self.engine);
        let Some(branch_slots) = target.branch_slots() else {
            return Ok(());
        };
        self.encode_copies(branch_slots.span(), len_branch_params, consume_fuel_instr)?;
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
                    self.stack.push_temp(*result)?;
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
        results: SlotSpan,
        len_values: u16,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        match len_values {
            0 => Ok(()),
            1 => {
                let result = results.head();
                let value = self.stack.peek(0);
                self.encode_copy(result, value, consume_fuel_instr)?;
                Ok(())
            }
            _ => self.encode_copy_many(results, len_values, consume_fuel_instr),
        }
    }

    /// Convenience wrapper for [`Self::encode_copy_impl`].
    fn encode_copy(
        &mut self,
        result: Slot,
        value: Operand,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<Option<Pos<Op>>, Error> {
        Self::encode_copy_impl(
            result,
            value,
            consume_fuel_instr,
            &mut self.layout,
            &mut self.instrs,
        )
    }

    /// Encodes a single copy instruction.
    ///
    /// # Note
    ///
    /// This won't encode a copy if `result` and `value` yields a no-op copy.
    fn encode_copy_impl(
        result: Slot,
        value: Operand,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
        layout: &mut StackLayout,
        encoder: &mut OpEncoder,
    ) -> Result<Option<Pos<Op>>, Error> {
        let Some(copy_instr) = Self::make_copy_instr(result, value, layout)? else {
            // Case: no-op copy instruction
            return Ok(None);
        };
        let pos = encoder.encode(copy_instr, consume_fuel_instr, FuelCostsProvider::base)?;
        Ok(Some(pos))
    }

    /// Returns the copy instruction to copy the given `operand` to `result`.
    ///
    /// Returns `None` if the resulting copy instruction is a no-op.
    fn make_copy_instr(
        result: Slot,
        value: Operand,
        layout: &mut StackLayout,
    ) -> Result<Option<Op>, Error> {
        let op = match value {
            Operand::Reg(value) => Self::make_copy_reg_instr(result, value),
            Operand::Temp(value) => return Self::make_copy_temp_instr(result, value),
            Operand::Local(value) => return Self::make_copy_local_instr(result, value, layout),
            Operand::Immediate(value) => Self::make_copy_imm_instr(result, value.val())?,
        };
        Ok(Some(op))
    }

    /// Returns the copy instruction to copy the given register `value` to `result`.
    fn make_copy_reg_instr(result: Slot, value: RegOperand) -> Op {
        let ty = value.ty();
        match ty {
            ValType::I32 | ValType::I64 | ValType::ExternRef | ValType::FuncRef => {
                Op::u64_copy_sr(result)
            }
            ValType::F32 => Op::f32_copy_sr(result),
            ValType::F64 => Op::f64_copy_sr(result),
            ValType::V128 => {
                // Note: `v128` typed values may not occupy register operands for now.
                unreachable!()
            }
        }
    }

    /// Returns the copy instruction to copy the given temporary `value` to `result`.
    fn make_copy_temp_instr(result: Slot, value: TempOperand) -> Result<Option<Op>, Error> {
        let ty = value.ty();
        let value = value.temp_slots().head();
        if result == value {
            // Case: no-op copy
            return Ok(None);
        }
        let copy_op = match ty {
            ValType::V128 => {
                let results = SlotSpan::new(result);
                let values = SlotSpan::new(value);
                let Some(op) = Self::make_copy_span(results, values, 2) else {
                    return Ok(None);
                };
                op
            }
            _ => Op::u64_copy_ss(result, value),
        };
        Ok(Some(copy_op))
    }

    /// Returns the copy instruction to copy the given local `value` to `result`.
    fn make_copy_local_instr(
        result: Slot,
        value: LocalOperand,
        layout: &mut StackLayout,
    ) -> Result<Option<Op>, Error> {
        let ty = value.ty();
        let value = layout.local_to_slot(value)?;
        if result == value {
            // Case: no-op copy
            return Ok(None);
        }
        let copy_op = match ty {
            ValType::V128 => {
                let results = SlotSpan::new(result);
                let values = SlotSpan::new(value);
                let Some(op) = Self::make_copy_span(results, values, 2) else {
                    return Ok(None);
                };
                op
            }
            _ => Op::u64_copy_ss(result, value),
        };
        Ok(Some(copy_op))
    }

    /// Returns the copy instruction to copy the given immediate `value` to `result`.
    fn make_copy_imm_instr(result: Slot, value: TypedRawVal) -> Result<Op, Error> {
        let instr = match value.ty() {
            ValType::I32 => Op::u32_copy_si(result, i32::from(value).to_bits()),
            ValType::I64 => Op::u64_copy_si(result, i64::from(value).to_bits()),
            ValType::F32 => Op::u32_copy_si(result, f32::from(value).to_bits()),
            ValType::F64 => Op::u64_copy_si(result, f64::from(value).to_bits()),
            ValType::ExternRef | ValType::FuncRef => {
                Op::u32_copy_si(result, u32::from(RawRef::from(value.raw())))
            }
            #[cfg(feature = "simd")]
            ValType::V128 => {
                let value = V128::from(value).as_u128();
                let value_lo = (value & 0xFFFF_FFFF_FFFF_FFFF) as u64;
                let value_hi = (value >> 64) as u64;
                Op::copy_imm128(result, value_lo, value_hi)
            }
            #[cfg(not(feature = "simd"))]
            ValType::V128 => panic!("unexpected `v128` operand: {value:?}"),
        };
        Ok(instr)
    }

    /// Encode a copy instruction that copies a contiguous span of values.
    ///
    /// # Note
    ///
    /// This won't encode a copy if the resulting copy instruction is a no-op.
    fn encode_copy_span(
        &mut self,
        results: SlotSpan,
        values: SlotSpan,
        len: u16,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        let Some(op) = Self::make_copy_span(results, values, len) else {
            // Case: results and values are equal and therefore the copy is a no-op
            return Ok(());
        };
        self.instrs
            .encode(op, consume_fuel_instr, |costs: &FuelCostsProvider| {
                costs.fuel_for_copying_values::<Cell>(u64::from(len))
            })?;
        Ok(())
    }

    /// Returns an [`Op::copy_span_asc`] or [`Op::copy_span_des`] depending on inputs.
    ///
    /// Returns `None` if the `copy_span` operation is a no-op.
    fn make_copy_span(results: SlotSpan, values: SlotSpan, len: u16) -> Option<Op> {
        if results == values {
            // Case: results and values are equal and therefore the copy is a no-op
            return None;
        }
        let copy_span = match results.head() > values.head() {
            true => Op::copy_span_des,
            false => Op::copy_span_asc,
        };
        Some(copy_span(results, values, len))
    }

    /// Encode a copy instruction that copies many values.
    ///
    /// # Note
    ///
    /// - This won't encode a copy if the resulting copy instruction is a no-op.
    /// - Encodes either `copy`, `copy2`, `copy_span` or `copy_many` depending on the amount
    ///   of noop copies between `results` and `values`.
    fn encode_copy_many(
        &mut self,
        results: SlotSpan,
        len: u16,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        self.peek_operands_into_buffer(usize::from(len));
        let values = &self.operands[..];
        let (results, values) = Self::copy_many_strip_noop_start(results, values, &self.layout)?;
        let values = Self::copy_many_strip_noop_end(results, values, &self.layout)?;
        debug_assert!(!Self::has_overlapping_copies(
            results,
            values,
            &self.layout
        )?);
        match values {
            [] => return Ok(()),
            [val0] => {
                let result = results.head();
                let value = *val0;
                self.encode_copy(result, value, consume_fuel_instr)?;
                return Ok(());
            }
            _values => {}
        }
        debug_assert!(!values.is_empty());
        if let Some(values) = Self::try_form_slot_span_of(values, &self.layout)? {
            return self.encode_copy_span(results, values.span(), values.len(), consume_fuel_instr);
        }
        let values = Self::copy_operands_to_temp(
            values,
            consume_fuel_instr,
            &mut self.layout,
            &mut self.instrs,
        )?;
        self.encode_copy_span(results, values.span(), values.len(), consume_fuel_instr)
    }

    /// Copy `values` to temporary stack [`Slot`]s without changing the translation stack.
    ///
    /// Returns a [`BoundedSlotSpan`] to the cells where `values` have been copied to.
    fn copy_operands_to_temp(
        values: &[Operand],
        pos_fuel: Option<Pos<ir::BlockFuel>>,
        layout: &mut StackLayout,
        instrs: &mut OpEncoder,
    ) -> Result<BoundedSlotSpan, Error> {
        debug_assert!(!values.is_empty());
        let mut copied_slots: u16 = 0;
        for value in values {
            let results = value.temp_slots();
            let result = results.head();
            let value = *value;
            Self::encode_copy_impl(result, value, pos_fuel, layout, instrs)?;
            copied_slots = copied_slots
                .checked_add(results.len())
                .ok_or(TranslationError::SlotOutOfBounds)?;
        }
        let head = values[0].temp_slots().head();
        let span = SlotSpan::new(head);
        Ok(BoundedSlotSpan::new(span, copied_slots))
    }

    /// Tries to strip noop copies from the start of the `copy_many`.
    ///
    /// Returns the stripped `results` [`SlotSpan`] and `values` slice of [`Operand`]s.
    fn copy_many_strip_noop_start<'a>(
        results: SlotSpan,
        values: &'a [Operand],
        layout: &StackLayout,
    ) -> Result<(SlotSpan, &'a [Operand]), Error> {
        let mut result = results.head();
        let mut values = values;
        while let Some((value, rest)) = values.split_first() {
            let ty = value.ty();
            let value = match value {
                Operand::Local(value) => layout.local_to_slot(value)?,
                Operand::Temp(value) => value.temp_slots().head(),
                Operand::Reg(_) | Operand::Immediate(_) => {
                    // Immediate and register values will never yield no-op copies.
                    break;
                }
            };
            if result != value {
                // Can no longer strip no-op copies from the start.
                break;
            }
            result = result.next_n(required_cells_for_ty(ty));
            values = rest;
        }
        Ok((SlotSpan::new(result), values))
    }

    /// Tries to strip noop copies from the end of the `copy_many`.
    ///
    /// Returns the stripped `values` slice of [`Operand`]s.
    fn copy_many_strip_noop_end<'a>(
        results: SlotSpan,
        values: &'a [Operand],
        layout: &StackLayout,
    ) -> Result<&'a [Operand], Error> {
        let Ok(len) = u16::try_from(values.len()) else {
            panic!("out of bounds `copy_many` values length: {}", values.len())
        };
        let mut result = results.head().next_n(len);
        let mut values = values;
        while let Some((value, rest)) = values.split_last() {
            let ty = value.ty();
            let value = match value {
                Operand::Local(value) => layout.local_to_slot(value)?,
                Operand::Temp(value) => value.temp_slots().head(),
                Operand::Reg(_) | Operand::Immediate(_) => {
                    // Immediate and register values will never yield no-op copies.
                    break;
                }
            };
            result = result.prev_n(required_cells_for_ty(ty));
            if result != value {
                // Can no longer strip no-op copies from the end.
                break;
            }
            values = rest;
        }
        Ok(values)
    }

    /// Returns `true` if there are overlapping copies with `results` and `values`.
    ///
    /// # Examples
    ///
    /// - `[ 0 <- 1, 1 <- 1, 2 <- 4 ]` has no overlapping copies.
    /// - `[ 0 <- 1, 1 <- 0 ]` has overlapping copies since register `0`
    ///   is written to in the first copy but read from in the next.
    /// - `[ 3 <- 1, 4 <- 2, 5 <- 3 ]` has overlapping copies since register `3`
    ///   is written to in the first copy but read from in the third.
    fn has_overlapping_copies(
        results: SlotSpan,
        values: &[Operand],
        layout: &StackLayout,
    ) -> Result<bool, Error> {
        if values.is_empty() {
            // An empty set of copies can never have overlapping copies.
            return Ok(false);
        }
        let Ok(len) = u16::try_from(values.len()) else {
            panic!("operand span too large: len={}", values.len());
        };
        let result0 = results.head();
        for (result, value) in results.iter(len).zip(values) {
            // Note: We only have to check the register case since constant value
            //       copies can never overlap.
            let value = match value {
                Operand::Local(value) => layout.local_to_slot(value)?,
                Operand::Temp(value) => value.temp_slots().head(),
                Operand::Reg(_) | Operand::Immediate(_) => {
                    // Immediates and registers can not collide with the result registers.
                    continue;
                }
            };
            if result0 <= value && value < result {
                // Case: `value` is in the range of `result0..result` which
                //       means it has been overwritten by previous copies,
                //       thus we detected a collission.
                return Ok(true);
            }
        }
        // No copy collissions have been found.
        Ok(false)
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

    fn copy_operand_to_reg(&mut self, operand: Operand) -> Result<(), Error> {
        #[cold]
        #[inline(never)]
        fn unsupported_v128(operand: Operand) -> ! {
            unimplemented!(
                "operands of type `v128` cannot be put in registers but found: {operand:?}"
            )
        }
        let ty = operand.ty();
        let operator = match self.resolve_operand_as::<RawVal>(operand)? {
            ResolvedOperand::Reg => {
                // Case: No copy needed as operand already resides in register.
                self.push_result_reg(ty)?;
                return Ok(());
            }
            ResolvedOperand::Slot(value) => match ty {
                ValType::I32 | ValType::I64 | ValType::FuncRef | ValType::ExternRef => {
                    Op::u64_copy_rs(value)
                }
                ValType::F32 => Op::f32_copy_rs(value),
                ValType::F64 => Op::f64_copy_rs(value),
                ValType::V128 => unsupported_v128(operand),
            },
            ResolvedOperand::Immediate(value) => match ty {
                ValType::FuncRef | ValType::ExternRef | ValType::I32 => {
                    Op::u32_copy_ri(u32::from(value))
                }
                ValType::I64 => Op::u64_copy_ri(u64::from(value)),
                ValType::F32 => Op::f32_copy_ri(f32::from(value)),
                ValType::F64 => Op::f64_copy_ri(f64::from(value)),
                ValType::V128 => unsupported_v128(operand),
            },
        };
        self.push_op_with_result_reg(ty, operator, FuelCostsProvider::base)?;
        Ok(())
    }

    /// Convert the [`Operand`] at `depth` into an [`Operand::Temp`] by copying if necessary.
    ///
    /// # Note
    ///
    /// Does nothing if the [`Operand`] is already an [`Operand::Temp`].
    fn copy_operand_to_temp(
        &mut self,
        operand: Operand,
        consume_fuel: Option<Pos<ir::BlockFuel>>,
    ) -> Result<Slot, Error> {
        let result = operand.temp_slots().head();
        self.encode_copy(result, operand, consume_fuel)?;
        Ok(result)
    }

    /// Copies the `operand` to its temporary [`Slot`] if it is an immediate.
    ///
    /// Returns the temporary [`Slot`] of the `operand`.
    ///
    /// # Note
    ///
    /// - Returns the associated [`Slot`] if `operand` is an [`Operand::Temp`] or [`Operand::Local`].
    // TODO: return `BoundedSlotSpan` instead of just `Slot`
    fn copy_immediate_to_slot(&mut self, operand: Operand) -> Result<Location, Error> {
        let location = match operand {
            Operand::Reg(_value) => Location::Reg,
            Operand::Local(operand) => {
                let slot = self.layout.local_to_slot(operand)?;
                Location::Slot(slot)
            }
            Operand::Temp(operand) => {
                let slot = operand.temp_slots().head();
                Location::Slot(slot)
            }
            Operand::Immediate(operand) => {
                let value = operand.val();
                let result = operand.temp_slots().head();
                let copy_instr = Self::make_copy_imm_instr(result, value)?;
                let consume_fuel = self.stack.consume_fuel_instr();
                self.instrs
                    .encode(copy_instr, consume_fuel, FuelCostsProvider::base)?;
                Location::Slot(result)
            }
        };
        Ok(location)
    }

    /// Copies the `operand` to its associated [`Slot`].
    ///
    /// Does nothing if the operand is an [`Operand::Local`] or [`Operand::Temp`].
    // TODO: return `BoundedSlotSpan` instead of just `Slot`
    fn copy_operand_to_slot(&mut self, operand: Operand) -> Result<Slot, Error> {
        let result = operand.temp_slots().head();
        let ty = operand.ty();
        let copy_op = match self.resolve_operand_as::<RawVal>(operand)? {
            ResolvedOperand::Slot(slot) => return Ok(slot),
            ResolvedOperand::Reg => match ty {
                | ValType::I32 | ValType::FuncRef | ValType::ExternRef | ValType::I64 => {
                    Op::u64_copy_sr(result)
                }
                | ValType::F32 => Op::f32_copy_rs(result),
                | ValType::F64 => Op::f64_copy_rs(result),
                | ValType::V128 => unreachable!(),
            },
            ResolvedOperand::Immediate(value) => {
                Self::make_copy_imm_instr(result, TypedRawVal::new(ty, value))?
            }
        };
        let fuel_op = self.stack.consume_fuel_instr();
        self.instrs
            .encode(copy_op, fuel_op, FuelCostsProvider::base)?;
        Ok(result)
    }

    /// Preserves all local operands on the stack.
    ///
    /// # Note
    ///
    /// This works by encoding copy instructions to `temp` register space.
    fn preserve_all_locals(&mut self) -> Result<(), Error> {
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        for local in self.stack.preserve_all_locals() {
            let result = local.temp_slots().head();
            let Some(copy_instr) = Self::make_copy_instr(result, local.into(), &mut self.layout)?
            else {
                unreachable!("`result` and `local` refer to different stack spaces");
            };
            self.instrs
                .encode(copy_instr, consume_fuel_instr, FuelCostsProvider::base)?;
        }
        Ok(())
    }

    /// Pushes the `instr` to the function with the associated `fuel_costs`.
    fn push_instr(
        &mut self,
        instr: Op,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<Pos<Op>, Error> {
        debug_assert!(instr.result_ref().is_none());
        let consume_fuel = self.stack.consume_fuel_instr();
        let instr = self.instrs.encode(instr, consume_fuel, fuel_costs)?;
        Ok(instr)
    }

    /// Pushes a result register operand onto the stack.
    ///
    /// If a register operand of an equivalent type is already on the stack
    /// a copy operator is encoded to turn the existing register operand into
    /// a temporary operand.
    fn push_result_reg(&mut self, ty: ValType) -> Result<(), Error> {
        let fuel_pos = self.stack.consume_fuel_instr();
        if let Some(operand) = self.stack.reg_to_temp(ty) {
            if let Operand::Reg(operand) = operand {
                let result = operand.temp_slots().head();
                let copy_op = Self::make_copy_reg_instr(result, operand);
                self.instrs
                    .encode(copy_op, fuel_pos, FuelCostsProvider::base)?;
            }
        };
        self.stack.push_reg(ty)?;
        Ok(())
    }

    /// Pushes the `instr` to the function with the associated `fuel_costs`.
    fn stage_op_with_result_reg(
        &mut self,
        result_ty: ValType,
        op: Op,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), Error> {
        debug_assert_eq!(op.result_loc().map(|loc| loc.is_reg()), Some(true));
        self.push_result_reg(result_ty)?;
        let fuel_pos = self.stack.consume_fuel_instr();
        self.instrs.stage(op, fuel_pos, fuel_costs)?;
        Ok(())
    }

    /// Pushes the `instr` to the function with the associated `fuel_costs`.
    fn push_op_with_result_reg(
        &mut self,
        result_ty: ValType,
        op: Op,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), Error> {
        debug_assert_eq!(op.result_loc().map(|loc| loc.is_reg()), Some(true));
        self.push_result_reg(result_ty)?;
        let fuel_pos = self.stack.consume_fuel_instr();
        self.instrs.encode(op, fuel_pos, fuel_costs)?;
        Ok(())
    }

    /// Pushes the `instr` to the function with the associated `fuel_costs`.
    fn push_instr_with_result_slot(
        &mut self,
        result_ty: ValType,
        make_instr: impl FnOnce(Slot) -> Op,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), Error> {
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        let result = self.stack.push_temp(result_ty)?.temp_slots().head();
        let op = make_instr(result);
        debug_assert!(op.result_ref().is_some());
        self.instrs.stage(op, consume_fuel_instr, fuel_costs)?;
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
        buffer: &mut Vec<TypedRawVal>,
    ) -> Result<(), Error> {
        let default_target = table.default();
        buffer.clear();
        for target in table.targets() {
            buffer.push(TypedRawVal::from(target?));
        }
        buffer.push(TypedRawVal::from(default_target));
        Ok(())
    }

    /// Encodes a Wasm `br_table` that does not copy branching values.
    ///
    /// # Note
    ///
    /// Upon call the `immediates` buffer contains all `br_table` target values.
    fn encode_br_table_0(
        &mut self,
        table: wasmparser::BrTable,
        index: Location,
    ) -> Result<(), Error> {
        // We add +1 because we include the default target here.
        let len_targets = table.len() + 1;
        debug_assert_eq!(self.immediates.len(), len_targets as usize);
        let op = match index {
            Location::Reg => Op::branch_table_r(len_targets),
            Location::Slot(index) => Op::branch_table_s(len_targets, index),
        };
        self.push_instr(op, FuelCostsProvider::base)?;
        // Encode the `br_table` targets:
        let fuel_pos = self.stack.consume_fuel_instr();
        let targets = &self.immediates[..];
        for target in targets {
            let Ok(depth) = usize::try_from(u32::from(*target)) else {
                panic!("out of bounds `br_table` target does not fit `usize`: {target:?}");
            };
            let mut frame = self.stack.peek_control_mut(depth).control_frame();
            self.instrs
                .encode_branch(frame.label(), identity, fuel_pos, 0)?;
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
        index: Location,
        len_values: u16,
    ) -> Result<(), Error> {
        let len_targets = table.len() + 1;
        debug_assert_eq!(self.immediates.len(), len_targets as usize);
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        let values =
            self.try_form_slot_span_or_move(usize::from(len_values), consume_fuel_instr)?;
        let op = match index {
            Location::Reg => Op::branch_table_span_r(len_targets, values),
            Location::Slot(index) => Op::branch_table_span_s(len_targets, index, values),
        };
        self.push_instr(op, FuelCostsProvider::base)?;
        // Encode the `br_table` targets:
        let fuel_pos = self.stack.consume_fuel_instr();
        let targets = &self.immediates[..];
        for target in targets {
            let Ok(depth) = usize::try_from(u32::from(*target)) else {
                panic!("out of bounds `br_table` target does not fit `usize`: {target:?}");
            };
            let mut frame = self.stack.peek_control_mut(depth).control_frame();
            let Some(results) = frame.branch_slots() else {
                panic!("must have frame results since `br_table` requires to copy values");
            };
            self.instrs.encode_branch(
                frame.label(),
                |offset| ir::BranchTableTarget::new(results.span(), offset),
                fuel_pos,
                0,
            )?;
            frame.branch_to();
        }
        Ok(())
    }

    /// Encodes a generic return instruction.
    fn encode_return(
        &mut self,
        consume_fuel: Option<Pos<ir::BlockFuel>>,
    ) -> Result<Pos<Op>, Error> {
        let len_results = self.func_type_with(FuncType::len_results);
        let return_slot_for_ty = |ty: ValType, slot: Slot| match ty {
            ValType::V128 => Op::return_span(BoundedSlotSpan::new(SlotSpan::new(slot), 2)),
            _ => Op::return_u64_s(slot),
        };
        let instr = match len_results {
            0 => Op::Return {},
            1 => match self.stack.peek(0) {
                Operand::Reg(operand) => match operand.ty() {
                    | ValType::I32 | ValType::I64 | ValType::FuncRef | ValType::ExternRef => {
                        Op::return_u64_r()
                    }
                    | ValType::F32 => Op::return_f32_r(),
                    | ValType::F64 => Op::return_f64_r(),
                    | ValType::V128 => {
                        // Note: `v128` values may not occupy register operands for now.
                        unreachable!()
                    }
                },
                Operand::Local(operand) => {
                    let value = self.layout.local_to_slot(operand)?;
                    return_slot_for_ty(operand.ty(), value)
                }
                Operand::Temp(operand) => {
                    return_slot_for_ty(operand.ty(), operand.temp_slots().head())
                }
                Operand::Immediate(operand) => {
                    let val = operand.val();
                    match operand.ty() {
                        ValType::I32 => Op::return_u32_i(i32::from(val).to_bits()),
                        ValType::I64 => Op::return_u64_i(i64::from(val).to_bits()),
                        ValType::F32 => Op::return_u32_i(f32::from(val).to_bits()),
                        ValType::F64 => Op::return_u64_i(f64::from(val).to_bits()),
                        ValType::FuncRef | ValType::ExternRef => {
                            Op::return_u32_i(u32::from(RawRef::from(val.raw())))
                        }
                        ValType::V128 => {
                            let value = self.stack.peek(0);
                            let temp_slot = self.copy_operand_to_temp(value, consume_fuel)?;
                            Op::return_span(BoundedSlotSpan::new(SlotSpan::new(temp_slot), 2))
                        }
                    }
                }
            },
            _ => {
                let len_results = usize::from(len_results);
                let results = self.move_operands_to_temp(len_results, consume_fuel)?;
                Op::return_span(results)
            }
        };
        let instr = self
            .instrs
            .encode(instr, consume_fuel, FuelCostsProvider::base)?;
        Ok(instr)
    }

    /// Store the top-most [`Operand`]s on the [`Stack`] into the operands buffer.
    fn peek_operands_into_buffer(&mut self, len: usize) {
        self.operands.clear();
        self.operands.extend(self.stack.peek_n(len));
    }

    /// Tries to form a [`BoundedSlotSpan`] from the top-most `n` operands on the [`Stack`].
    ///
    /// Returns `None` if forming a [`BoundedSlotSpan`] was not possible.
    fn try_form_slot_span(&self, len: usize) -> Result<Option<BoundedSlotSpan>, Error> {
        Self::try_form_slot_span_of(self.stack.peek_n(len), &self.layout)
    }

    /// Tries to form a [`BoundedSlotSpan`] from the `values` [`Operand`]s.
    ///
    /// Returns `None` if forming a [`BoundedSlotSpan`] was not possible.
    fn try_form_slot_span_of<T>(
        values: impl IntoIterator<Item = T>,
        layout: &StackLayout,
    ) -> Result<Option<BoundedSlotSpan>, Error>
    where
        T: AsRef<Operand>,
    {
        let mut values = values.into_iter();
        let Some(head) = values.next() else {
            return Ok(None);
        };
        match head.as_ref() {
            Operand::Local(operand) => Self::try_form_span_of_locals(operand, values, layout),
            Operand::Temp(operand) => Self::try_form_span_of_temps(operand, values),
            Operand::Reg(_) | Operand::Immediate(_) => Ok(None),
        }
    }

    /// Tries to form a [`BoundedSlotSpan`] from the local [`Operand`]s in `head` and `values`.
    ///
    /// Returns `None` if forming a [`BoundedSlotSpan`] was not possible.
    fn try_form_span_of_locals<T>(
        head: &LocalOperand,
        values: impl IntoIterator<Item = T>,
        layout: &StackLayout,
    ) -> Result<Option<BoundedSlotSpan>, Error>
    where
        T: AsRef<Operand>,
    {
        let head_slots = layout.local_to_slots(head)?;
        let start = head_slots.span().head();
        let mut len = head_slots.len();
        let mut next = start.next_n(len);
        for value in values {
            match value.as_ref() {
                Operand::Local(operand) => {
                    let slots = layout.local_to_slots(operand)?;
                    if slots.head() != next {
                        // Note: the operands do not form a contiguous span of slots.
                        return Ok(None);
                    }
                    len = len
                        .checked_add(slots.len())
                        .ok_or(TranslationError::SlotAccessOutOfBounds)?;
                    next = next.next_n(slots.len());
                }
                _ => return Ok(None),
            }
        }
        Ok(Some(BoundedSlotSpan::new(SlotSpan::new(start), len)))
    }

    /// Tries to form a [`BoundedSlotSpan`] from the temporary [`Operand`]s in `head` and `values`.
    ///
    /// Returns `None` if forming a [`BoundedSlotSpan`] was not possible.
    fn try_form_span_of_temps<T>(
        head: &TempOperand,
        values: impl IntoIterator<Item = T>,
    ) -> Result<Option<BoundedSlotSpan>, Error>
    where
        T: AsRef<Operand>,
    {
        let head_slots = head.temp_slots();
        let start = head_slots.span().head();
        let mut len = head_slots.len();
        let mut next = start.next_n(len);
        for value in values {
            match value.as_ref() {
                Operand::Temp(operand) => {
                    let slots = operand.temp_slots();
                    if slots.head() != next {
                        // Note: the operands do not form a contiguous span of slots.
                        return Ok(None);
                    }
                    len = len
                        .checked_add(slots.len())
                        .ok_or(TranslationError::SlotAccessOutOfBounds)?;
                    next = next.next_n(slots.len());
                }
                _ => return Ok(None),
            }
        }
        Ok(Some(BoundedSlotSpan::new(SlotSpan::new(start), len)))
    }

    /// Tries to form a [`BoundedSlotSpan`] from the top-most `len` operands on the [`Stack`] or copy to temporaries.
    ///
    /// Returns `None` if forming a [`BoundedSlotSpan`] was not possible.
    fn try_form_slot_span_or_move(
        &mut self,
        len: usize,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<BoundedSlotSpan, Error> {
        if let Some(span) = self.try_form_slot_span(len)? {
            return Ok(span);
        }
        self.move_operands_to_temp(len, consume_fuel_instr)
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
        self.instrs.pin_label(frame.label())?;
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
            self.instrs.encode_branch(
                frame.label(),
                Op::branch,
                consume_fuel_instr,
                FuelCostsProvider::base,
            )?;
        }
        self.instrs.pin_label_if_unpinned(else_label)?;
        self.stack.push_else_operands(&frame)?;
        if has_results {
            // We haven't visited the `else` block and thus the `else`
            // providers are still on the auxiliary stack and need to
            // be popped. We use them to restore the stack to the state
            // when entering the `if` block so that we can properly copy
            // the `else` results to were they are expected.
            let consume_fuel_instr = self.instrs.encode_consume_fuel()?;
            self.copy_branch_params(&frame, consume_fuel_instr)?;
        }
        self.push_frame_results(&frame)?;
        self.instrs.pin_label(frame.label())?;
        self.reachable = true;
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
            let consume_fuel_instr: Option<Pos<ir::BlockFuel>> = frame.consume_fuel_instr();
            self.copy_branch_params(&frame, consume_fuel_instr)?;
        }
        self.push_frame_results(&frame)?;
        self.instrs.pin_label(frame.label())?;
        self.reachable = reachable;
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
        self.instrs.pin_label(frame.label())?;
        self.reachable = end_is_reachable || frame.is_branched_to();
        Ok(())
    }

    /// Translates the end of an unreachable Wasm control frame.
    fn translate_end_unreachable(&mut self, _frame: ControlFrameKind) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        // We reset `last_instr` out of caution in case there is a control flow boundary.
        self.instrs.try_encode_staged()?;
        Ok(())
    }

    /// Translate the Wasm `local.set` and `local.tee` operations.
    ///
    /// # Note
    ///
    /// This applies op-code fusion that replaces the result of the previous instruction
    /// instead of encoding a copy instruction for the `local.set` or `local.tee` if possible.
    fn translate_local_set(&mut self, local_index: u32, input: Operand) -> Result<(), Error> {
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
        let fuel_pos = self.stack.consume_fuel_instr();
        for preserved in self.stack.preserve_locals(local_idx) {
            let result = preserved.temp_slots().head();
            let op_or_noop = Self::make_copy_local_instr(result, preserved, &mut self.layout)?;
            let Some(copy_op) = op_or_noop else {
                // Note: local preservation must not yield no-op copies.
                unreachable!()
            };
            self.instrs
                .encode(copy_op, fuel_pos, FuelCostsProvider::base)?;
        }
        if self.try_replace_result(local_idx, input)? {
            // Case: it was possible to replace the result of the previous
            //       instructions so no copy instruction is required.
            return Ok(());
        }
        // At this point we need to encode a copy instruction.
        let result = self.layout.local_to_slot(local_idx)?;
        let outcome = self.encode_copy(result, input, fuel_pos)?;
        debug_assert!(
            outcome.is_some(),
            "no-op copy cases have been filtered out already"
        );
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
        let Some(mut staged) = self.instrs.peek_staged() else {
            // Case: cannot replace result without staged operator.
            return Ok(false);
        };
        let new_result = self.layout.local_to_slot(new_result)?;
        debug_assert!(matches!(
            self.layout.stack_space(new_result),
            StackSpace::Local
        ));
        let old_result = match old_result {
            Operand::Temp(old_result) => old_result.temp_slots().head(),
            Operand::Reg(_) | Operand::Local(_) | Operand::Immediate(_) => {
                // Case  register: cannot replace register operand result for now.
                //                 (in the future new operators might allow for this)
                // Case immediate: cannot replace immediate value result since they are immutable.
                // Case     local: cannot replace local with another local due to observable behavior.
                return Ok(false);
            }
        };
        let Some(staged_result) = staged.result_mut() else {
            // Case: staged has no result and thus cannot have its result changed.
            return Ok(false);
        };
        if *staged_result != old_result {
            // Case: staged result does not match `old_result` and thus is not available for mutation.
            return Ok(false);
        }
        *staged_result = new_result;
        let (fuel_pos, fuel_used) = self.instrs.drop_staged();
        self.instrs.encode(staged, fuel_pos, fuel_used)?;
        Ok(true)
    }

    /// Encodes an unconditional Wasm `branch` instruction.
    fn encode_br(&mut self, label: LabelRef) -> Result<Pos<Op>, Error> {
        let fuel_pos = self.stack.consume_fuel_instr();
        let (br_op, _) =
            self.instrs
                .encode_branch(label, Op::branch, fuel_pos, FuelCostsProvider::base)?;
        Ok(br_op)
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
            Operand::Reg(_) => Location::Reg,
            Operand::Local(condition) => {
                let slot = self.layout.local_to_slot(condition)?;
                Location::Slot(slot)
            }
            Operand::Temp(condition) => {
                let slot = condition.temp_slots().head();
                Location::Slot(slot)
            }
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
        let fuel_pos = self.stack.consume_fuel_instr();
        self.instrs.encode_branch(
            label,
            |offset| match branch_eqz {
                true => match condition {
                    Location::Slot(condition) => Op::branch_i32_eq_si(offset, condition, 0),
                    Location::Reg => Op::branch_i32_eq_ri(offset, 0),
                },
                false => match condition {
                    Location::Slot(condition) => Op::branch_i32_not_eq_si(offset, condition, 0),
                    Location::Reg => Op::branch_i32_not_eq_ri(offset, 0),
                },
            },
            fuel_pos,
            FuelCostsProvider::base,
        )?;
        Ok(())
    }

    /// Try to fuse a cmp+branch [`Op`] with optional negation.
    fn try_fuse_branch_cmp(
        &mut self,
        condition: Operand,
        label: LabelRef,
        negate: bool,
    ) -> Result<bool, Error> {
        let Some(staged_op) = self.instrs.peek_staged() else {
            // Case: cannot fuse without a known last instruction
            return Ok(false);
        };
        let Some(ir::Location::Reg(result_ty)) = staged_op.result_loc() else {
            // Case: cannot fuse without register result.
            return Ok(false);
        };
        let Operand::Reg(_condition) = condition else {
            // Case: cannot fuse non-register operands
            //  - locals have observable behavior.
            //  - immediates cannot be the result of a previous instruction.
            return Ok(false);
        };
        debug_assert!(matches!(result_ty, ValType::I32 | ValType::I64));
        let cmp_op = match negate {
            false => staged_op,
            true => match staged_op.negate_cmp_instr() {
                Some(negated) => negated,
                None => {
                    // Note: cannot negate staged [`Op`], thus it is not a `cmp` operator and thus not fusable.
                    return Ok(false);
                }
            },
        };
        let Some(fused_cmp_branch) = cmp_op.try_into_cmp_branch_instr(BranchOffset::uninit())
        else {
            return Ok(false);
        };
        let (fuel_pos, _) = self.instrs.drop_staged();
        self.instrs.encode_branch(
            label,
            |offset| fused_cmp_branch.with_branch_offset(offset),
            fuel_pos,
            FuelCostsProvider::base,
        )?;
        Ok(true)
    }

    /// Generically translates a `call` or `return_call` Wasm operator.
    fn translate_call(
        &mut self,
        function_index: u32,
        call_internal: fn(params: BoundedSlotSpan, func: index::InternalFunc) -> Op,
        call_imported: fn(params: BoundedSlotSpan, func: index::Func) -> Op,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let consume_fuel = self.stack.consume_fuel_instr();
        let func_idx = FuncIdx::from(function_index);
        let callee_ty = self.resolve_func_type(func_idx);
        let params = self.adjust_stack_for_call(&callee_ty, consume_fuel)?;
        let instr = match self.module.get_engine_func(func_idx) {
            Some(engine_func) => {
                // Case: We are calling an internal function and can optimize
                //       this case by using the special instruction for it.
                call_internal(params, index::InternalFunc::from(engine_func))
            }
            None => {
                // Case: We are calling an imported function and must use the
                //       general calling operator for it.
                call_imported(params, index::Func::from(function_index))
            }
        };
        self.push_instr(instr, FuelCostsProvider::call)?;
        Ok(())
    }

    /// Generically translates a `call_indirect` or `return_call_indirect` Wasm operator.
    fn translate_call_indirect(
        &mut self,
        type_index: u32,
        table_index: u32,
        op_s: fn(
            table: index::Table,
            func_type: index::FuncType,
            params: BoundedSlotSpan,
            index: Slot,
        ) -> Op,
        op_r: fn(table: index::Table, func_type: index::FuncType, params: BoundedSlotSpan) -> Op,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let index = self.stack.pop();
        let consume_fuel = self.stack.consume_fuel_instr();
        let table = index::Table::from(table_index);
        let callee_ty = self.resolve_type(type_index);
        let index = self.copy_immediate_to_slot(index)?;
        let params = self.adjust_stack_for_call(&callee_ty, consume_fuel)?;
        let func_type = index::FuncType::from(type_index);
        let op = match index {
            Location::Slot(index) => op_s(table, func_type, params, index),
            Location::Reg => op_r(table, func_type, params),
        };
        self.push_instr(op, FuelCostsProvider::call)?;
        Ok(())
    }

    /// Adjusts the stack for a call to a function with type `ty`.
    ///
    /// Returns a bounded [`SlotSpan`] to the start of the call parameters and results
    /// with the length equal to the number of cells storing the call parameters.
    fn adjust_stack_for_call(
        &mut self,
        ty: &FuncType,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<BoundedSlotSpan, Error> {
        let mut params_start = self.stack.next_temp_slots();
        let mut params_len: u16 = 0;
        for _ in 0..ty.len_params() {
            let operand = self.stack.pop();
            let slots = operand.temp_slots();
            params_start = slots.span();
            params_len = params_len
                .checked_add(slots.len())
                .ok_or(TranslationError::AllocatedTooManySlots)?;
            self.copy_operand_to_temp(operand, fuel_pos)?;
        }
        let params = BoundedSlotSpan::new(params_start, params_len);
        for result in ty.results() {
            self.stack.push_temp(*result)?;
        }
        Ok(params)
    }

    /// Translates a unary Wasm instruction to Wasmi bytecode.
    fn translate_unary<Op: UnaryOp>(&mut self) -> Result<(), Error>
    where
        Op::Value: From<RawVal>,
        Op::Result: Into<TypedRawVal> + Typed,
    {
        bail_unreachable!(self);
        let input = self.stack.pop();
        let op = match self.resolve_operand_as::<Op::Value>(input)? {
            ResolvedOperand::Reg => Op::op_rr(),
            ResolvedOperand::Slot(input) => Op::op_rs(input),
            ResolvedOperand::Immediate(input) => {
                match Op::consteval(input) {
                    Ok(result) => {
                        self.stack.push_immediate(result)?;
                    }
                    Err(trap_code) => {
                        self.translate_trap(trap_code)?;
                    }
                }
                return Ok(());
            }
        };
        self.stage_op_with_result_reg(<Op::Result>::TY, op, FuelCostsProvider::base)?;
        Ok(())
    }

    /// Translate a generic Wasm reinterpret-like operation.
    ///
    /// # Note
    ///
    /// This Wasm operation is a no-op. Ideally we only have to change the types on the stack.
    fn translate_reinterpret<T, R>(
        &mut self,
        op_rr: fn() -> Op,
        consteval: fn(T) -> R,
    ) -> Result<(), Error>
    where
        T: From<TypedRawVal> + Typed,
        R: Into<TypedRawVal> + Typed,
    {
        bail_unreachable!(self);
        let input = self.stack.pop();
        debug_assert_eq!(input.ty(), <T as Typed>::TY);
        match input {
            Operand::Reg(_input) => {
                return self.push_op_with_result_reg(
                    <R as Typed>::TY,
                    op_rr(),
                    FuelCostsProvider::base,
                );
            }
            Operand::Local(input) => {
                self.stack
                    .push_local(input.local_index(), <R as Typed>::TY)?;
            }
            Operand::Temp(_input) => {
                self.stack.push_temp(<R as Typed>::TY)?;
            }
            Operand::Immediate(input) => {
                let input: T = input.val().into();
                self.stack.push_immediate(consteval(input))?;
            }
        }
        Ok(())
    }

    /// Copies `operand` to a temporary stack slot if it is an immediate that cannot be encoded using 32-bits.
    ///
    /// - Returns [`ResolvedOperand::Reg`] if `operand` is a register operand.
    /// - Returns [`ResolvedOperand::Slot`] if `operand` is a local or a temporary operand.
    /// - Returns [`ResolvedOperand::Immediate`] if `operand` is an immediate that is representable as 32-bit value.
    /// - Returns [`ResolvedOperand::Slot`] otherwise and encodes a copy storing the immediate into its temporary stack slot.
    fn resolve_operand_as_index32_or_copy(
        &mut self,
        operand: Operand,
        index_ty: IndexType,
    ) -> Result<ResolvedOperand<u32>, Error> {
        let value =
            self.resolve_operand_as::<RawVal>(operand)?
                .filter_map(|value| match index_ty {
                    IndexType::I32 => Some(u32::from(value)),
                    IndexType::I64 => u32::try_from(u64::from(value)).ok(),
                });
        let Some(value) = value else {
            return self
                .copy_immediate_to_slot(operand)
                .map(ResolvedOperand::from);
        };
        Ok(value)
    }

    /// Convenience method to tell that there is no custom optimization.
    fn no_opt_ri<T>(&mut self, _lhs: Operand, _rhs: T) -> Result<bool, Error> {
        Ok(false)
    }

    // TODO: docs
    fn resolve_operand_as<T>(&mut self, operand: Operand) -> Result<ResolvedOperand<T>, Error>
    where
        T: From<RawVal>,
    {
        let resolved = match operand {
            Operand::Reg(_operand) => ResolvedOperand::Reg,
            Operand::Local(operand) => {
                let slot = self.layout.local_to_slot(operand)?;
                ResolvedOperand::Slot(slot)
            }
            Operand::Temp(operand) => {
                let slot = operand.temp_slots().head();
                ResolvedOperand::Slot(slot)
            }
            Operand::Immediate(operand) => {
                let value = T::from(operand.val().raw());
                ResolvedOperand::Immediate(value)
            }
        };
        Ok(resolved)
    }

    fn resolve_operand_as_index(
        &mut self,
        operand: Operand,
        memory: Memory,
    ) -> Result<ResolvedOperand<u64>, Error> {
        let memidx: MemoryIdx = u32::from(memory).into();
        let operand = match self.module.get_type_of_memory(memidx).index_ty() {
            IndexType::I32 => self.resolve_operand_as::<u32>(operand)?.map(u64::from),
            IndexType::I64 => self.resolve_operand_as::<u64>(operand)?,
        };
        Ok(operand)
    }

    // TODO: docs
    #[cold]
    #[inline]
    fn unsupported_operand_pair(lhs: impl AsRef<Operand>, rhs: impl AsRef<Operand>) -> ! {
        #[inline(never)]
        fn impl_(lhs: &Operand, rhs: &Operand) -> ! {
            unreachable!("unsupported operator pair: lhs = {lhs:?}, rhs = {rhs:?}")
        }
        let lhs = lhs.as_ref();
        let rhs = rhs.as_ref();
        impl_(lhs, rhs)
    }

    /// Translates a commutative binary Wasm operator to Wasmi bytecode.
    fn translate_binary_commutative<T: CommutativeBinaryOp>(
        &mut self,
        opt_rhs_imm: fn(this: &mut Self, lhs: Operand, rhs: T::Input) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T::Input: From<RawVal> + Copy,
        T::Result: Into<RawVal>,
    {
        bail_unreachable!(self);
        let (lhs, rhs) = self.stack.pop2();
        let l = self.resolve_operand_as::<T::Input>(lhs)?;
        let r = self.resolve_operand_as::<T::Input>(rhs)?;
        if let (ResolvedOperand::Immediate(lhs), ResolvedOperand::Immediate(rhs)) = (l, r) {
            return self.translate_binary_consteval(lhs, rhs, T::consteval);
        }
        let (l, r) = ResolvedOperand::sort(l, r);
        if let (_, ResolvedOperand::Immediate(rhs)) = (l, r) {
            if opt_rhs_imm(self, lhs, rhs)? {
                return Ok(());
            }
        }
        let operator = match (l, r) {
            (ResolvedOperand::Reg, ResolvedOperand::Slot(rhs)) => T::op_rrs(rhs),
            (ResolvedOperand::Reg, ResolvedOperand::Immediate(rhs)) => T::op_rri(rhs),
            (ResolvedOperand::Slot(lhs), ResolvedOperand::Slot(rhs)) => T::op_rss(lhs, rhs),
            (ResolvedOperand::Slot(lhs), ResolvedOperand::Immediate(rhs)) => T::op_rsi(lhs, rhs),
            _ => Self::unsupported_operand_pair(lhs, rhs),
        };
        self.stage_op_with_result_reg(<T::Result as Typed>::TY, operator, FuelCostsProvider::base)?;
        Ok(())
    }

    /// Translates a non-commutative binary Wasm operator to Wasmi bytecode.
    fn translate_binary<T: BinaryOp>(&mut self) -> Result<(), Error>
    where
        T::Lhs: From<RawVal> + Copy,
        T::Rhs: Copy,
        T::Result: Into<RawVal>,
    {
        bail_unreachable!(self);
        let (lhs, rhs) = self.stack.pop2();
        let l = self.resolve_operand_as::<T::Lhs>(lhs)?;
        let r = self.resolve_operand_as::<RawVal>(rhs)?;
        let r = match r {
            ResolvedOperand::Reg => ResolvedOperand::Reg,
            ResolvedOperand::Slot(rhs) => ResolvedOperand::Slot(rhs),
            ResolvedOperand::Immediate(rhs) => match T::decode_rhs(rhs) {
                BinaryOpRhs::Value(rhs) => ResolvedOperand::Immediate(rhs),
                BinaryOpRhs::Trap(trap_code) => return self.translate_trap(trap_code),
                BinaryOpRhs::ReturnLhs => {
                    self.stack.push_operand(lhs)?;
                    return Ok(());
                }
            },
        };
        if let (ResolvedOperand::Immediate(lhs), ResolvedOperand::Immediate(rhs)) = (l, r) {
            return self.translate_binary_consteval(lhs, rhs, T::consteval);
        }
        let operator = match (l, r) {
            (ResolvedOperand::Reg, ResolvedOperand::Slot(rhs)) => T::op_rrs(rhs),
            (ResolvedOperand::Reg, ResolvedOperand::Immediate(rhs)) => T::op_rri(rhs),
            (ResolvedOperand::Slot(lhs), ResolvedOperand::Reg) => T::op_rsr(lhs),
            (ResolvedOperand::Slot(lhs), ResolvedOperand::Slot(rhs)) => T::op_rss(lhs, rhs),
            (ResolvedOperand::Slot(lhs), ResolvedOperand::Immediate(rhs)) => T::op_rsi(lhs, rhs),
            (ResolvedOperand::Immediate(lhs), ResolvedOperand::Reg) => T::op_rir(lhs),
            (ResolvedOperand::Immediate(lhs), ResolvedOperand::Slot(rhs)) => T::op_ris(lhs, rhs),
            _ => Self::unsupported_operand_pair(lhs, rhs),
        };
        self.stage_op_with_result_reg(<T::Result as Typed>::TY, operator, FuelCostsProvider::base)?;
        Ok(())
    }

    /// Evaluates `consteval(lhs, rhs)` and pushed either its result or tranlates a `trap`.
    fn translate_binary_consteval<Lhs, Rhs, Res>(
        &mut self,
        lhs: Lhs,
        rhs: Rhs,
        consteval: impl FnOnce(Lhs, Rhs) -> Result<Res, TrapCode>,
    ) -> Result<(), Error>
    where
        Res: Into<TypedRawVal>,
    {
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

    /// Translates a generic trap instruction.
    fn translate_trap(&mut self, trap: TrapCode) -> Result<(), Error> {
        self.push_instr(Op::trap(trap), FuelCostsProvider::base)?;
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
        let (mut true_val, mut false_val, condition) = self.stack.pop3();
        if let Some(type_hint) = type_hint {
            debug_assert_eq!(true_val.ty(), type_hint);
            debug_assert_eq!(false_val.ty(), type_hint);
        }
        if true_val.is_same(&false_val) {
            // Optimization: both `lhs` and `rhs` either are the same register or constant values and
            //               thus `select` will always yield this same value irrespective of the condition.
            self.stack.push_operand(true_val)?;
            return Ok(());
        }
        let ty = true_val.ty();
        let condition = self.resolve_operand_as::<i32>(condition)?;
        if let ResolvedOperand::Immediate(condition) = condition {
            let selected = match condition != 0 {
                true => true_val,
                false => false_val,
            };
            self.copy_operand_to_reg(selected)?;
            return Ok(());
        }
        let fusion = self.try_fuse_select(condition)?;
        if fusion.is_fused() {
            self.instrs.drop_staged();
            if matches!(fusion, SelectFusion::FusedSwap) {
                mem::swap(&mut true_val, &mut false_val);
            }
        }
        let operator = match ty {
            ValType::I32 | ValType::FuncRef | ValType::ExternRef => {
                self.i32_select_operator(condition, true_val, false_val)?
            }
            ValType::I64 => self.i64_select_operator(condition, true_val, false_val)?,
            ValType::F32 => self.f32_select_operator(condition, true_val, false_val)?,
            ValType::F64 => self.f64_select_operator(condition, true_val, false_val)?,
            #[cfg(feature = "simd")]
            ValType::V128 => return self.encode_v128_select(condition, true_val, false_val),
            #[cfg(not(feature = "simd"))]
            ValType::V128 => unreachable!("v128 simd is not enabled"),
        };
        self.stage_op_with_result_reg(ty, operator, FuelCostsProvider::base)?;
        Ok(())
    }

    fn i32_select_operator(
        &mut self,
        condition: ResolvedOperand<i32>,
        true_val: Operand,
        false_val: Operand,
    ) -> Result<Op, Error> {
        use ResolvedOperand as Opd;
        let true_val = self.resolve_operand_as::<u32>(true_val)?;
        let false_val = self.resolve_operand_as::<u32>(false_val)?;
        let operator = match (condition, true_val, false_val) {
            (Opd::Reg, Opd::Slot(t), Opd::Slot(f)) => Op::u64_select_rrss(t, f),
            (Opd::Reg, Opd::Slot(t), Opd::Immediate(f)) => Op::u32_select_rrsi(t, f),
            (Opd::Reg, Opd::Immediate(t), Opd::Slot(f)) => Op::u32_select_rris(t, f),
            (Opd::Reg, Opd::Immediate(t), Opd::Immediate(f)) => Op::u32_select_rrii(t, f),
            (Opd::Slot(c), Opd::Reg, Opd::Slot(f)) => Op::u64_select_rsrs(c, f),
            (Opd::Slot(c), Opd::Reg, Opd::Immediate(f)) => Op::u32_select_rsri(c, f),
            (Opd::Slot(c), Opd::Slot(t), Opd::Reg) => Op::u64_select_rssr(c, t),
            (Opd::Slot(c), Opd::Slot(t), Opd::Slot(f)) => Op::u64_select_rsss(c, t, f),
            (Opd::Slot(c), Opd::Slot(t), Opd::Immediate(f)) => Op::u32_select_rssi(c, t, f),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Reg) => Op::u32_select_rsir(c, t),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Slot(f)) => Op::u32_select_rsis(c, t, f),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Immediate(f)) => Op::u32_select_rsii(c, t, f),
            _ => unreachable!(),
        };
        Ok(operator)
    }

    fn i64_select_operator(
        &mut self,
        condition: ResolvedOperand<i32>,
        true_val: Operand,
        false_val: Operand,
    ) -> Result<Op, Error> {
        use ResolvedOperand as Opd;
        let true_val = self.resolve_operand_as::<u64>(true_val)?;
        let false_val = self.resolve_operand_as::<u64>(false_val)?;
        let operator = match (condition, true_val, false_val) {
            (Opd::Reg, Opd::Slot(t), Opd::Slot(f)) => Op::u64_select_rrss(t, f),
            (Opd::Reg, Opd::Slot(t), Opd::Immediate(f)) => Op::u64_select_rrsi(t, f),
            (Opd::Reg, Opd::Immediate(t), Opd::Slot(f)) => Op::u64_select_rris(t, f),
            (Opd::Reg, Opd::Immediate(t), Opd::Immediate(f)) => Op::u64_select_rrii(t, f),
            (Opd::Slot(c), Opd::Reg, Opd::Slot(f)) => Op::u64_select_rsrs(c, f),
            (Opd::Slot(c), Opd::Reg, Opd::Immediate(f)) => Op::u64_select_rsri(c, f),
            (Opd::Slot(c), Opd::Slot(t), Opd::Reg) => Op::u64_select_rssr(c, t),
            (Opd::Slot(c), Opd::Slot(t), Opd::Slot(f)) => Op::u64_select_rsss(c, t, f),
            (Opd::Slot(c), Opd::Slot(t), Opd::Immediate(f)) => Op::u64_select_rssi(c, t, f),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Reg) => Op::u64_select_rsir(c, t),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Slot(f)) => Op::u64_select_rsis(c, t, f),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Immediate(f)) => Op::u64_select_rsii(c, t, f),
            _ => unreachable!(),
        };
        Ok(operator)
    }

    fn f32_select_operator(
        &mut self,
        condition: ResolvedOperand<i32>,
        true_val: Operand,
        false_val: Operand,
    ) -> Result<Op, Error> {
        use ResolvedOperand as Opd;
        let true_val = self.resolve_operand_as::<f32>(true_val)?;
        let false_val = self.resolve_operand_as::<f32>(false_val)?;
        let operator = match (condition, true_val, false_val) {
            (Opd::Reg, Opd::Reg, Opd::Slot(f)) => Op::f32_select_rrrs(f),
            (Opd::Reg, Opd::Reg, Opd::Immediate(f)) => Op::f32_select_rrri(f),
            (Opd::Reg, Opd::Slot(t), Opd::Reg) => Op::f32_select_rrsr(t),
            (Opd::Reg, Opd::Immediate(t), Opd::Reg) => Op::f32_select_rrir(t),
            (Opd::Reg, Opd::Slot(t), Opd::Slot(f)) => Op::f32_select_rrss(t, f),
            (Opd::Reg, Opd::Slot(t), Opd::Immediate(f)) => Op::f32_select_rrsi(t, f),
            (Opd::Reg, Opd::Immediate(t), Opd::Slot(f)) => Op::f32_select_rris(t, f),
            (Opd::Reg, Opd::Immediate(t), Opd::Immediate(f)) => Op::f32_select_rrii(t, f),
            (Opd::Slot(c), Opd::Reg, Opd::Slot(f)) => Op::f32_select_rsrs(c, f),
            (Opd::Slot(c), Opd::Reg, Opd::Immediate(f)) => Op::f32_select_rsri(c, f),
            (Opd::Slot(c), Opd::Slot(t), Opd::Reg) => Op::f32_select_rssr(c, t),
            (Opd::Slot(c), Opd::Slot(t), Opd::Slot(f)) => Op::f32_select_rsss(c, t, f),
            (Opd::Slot(c), Opd::Slot(t), Opd::Immediate(f)) => Op::f32_select_rssi(c, t, f),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Reg) => Op::f32_select_rsir(c, t),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Slot(f)) => Op::f32_select_rsis(c, t, f),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Immediate(f)) => Op::f32_select_rsii(c, t, f),
            _ => unreachable!(),
        };
        Ok(operator)
    }

    fn f64_select_operator(
        &mut self,
        condition: ResolvedOperand<i32>,
        true_val: Operand,
        false_val: Operand,
    ) -> Result<Op, Error> {
        use ResolvedOperand as Opd;
        let true_val = self.resolve_operand_as::<f64>(true_val)?;
        let false_val = self.resolve_operand_as::<f64>(false_val)?;
        let operator = match (condition, true_val, false_val) {
            (Opd::Reg, Opd::Reg, Opd::Slot(f)) => Op::f64_select_rrrs(f),
            (Opd::Reg, Opd::Reg, Opd::Immediate(f)) => Op::f64_select_rrri(f),
            (Opd::Reg, Opd::Slot(t), Opd::Reg) => Op::f64_select_rrsr(t),
            (Opd::Reg, Opd::Immediate(t), Opd::Reg) => Op::f64_select_rrir(t),
            (Opd::Reg, Opd::Slot(t), Opd::Slot(f)) => Op::f64_select_rrss(t, f),
            (Opd::Reg, Opd::Slot(t), Opd::Immediate(f)) => Op::f64_select_rrsi(t, f),
            (Opd::Reg, Opd::Immediate(t), Opd::Slot(f)) => Op::f64_select_rris(t, f),
            (Opd::Reg, Opd::Immediate(t), Opd::Immediate(f)) => Op::f64_select_rrii(t, f),
            (Opd::Slot(c), Opd::Reg, Opd::Slot(f)) => Op::f64_select_rsrs(c, f),
            (Opd::Slot(c), Opd::Reg, Opd::Immediate(f)) => Op::f64_select_rsri(c, f),
            (Opd::Slot(c), Opd::Slot(t), Opd::Reg) => Op::f64_select_rssr(c, t),
            (Opd::Slot(c), Opd::Slot(t), Opd::Slot(f)) => Op::f64_select_rsss(c, t, f),
            (Opd::Slot(c), Opd::Slot(t), Opd::Immediate(f)) => Op::f64_select_rssi(c, t, f),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Reg) => Op::f64_select_rsir(c, t),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Slot(f)) => Op::f64_select_rsis(c, t, f),
            (Opd::Slot(c), Opd::Immediate(t), Opd::Immediate(f)) => Op::f64_select_rsii(c, t, f),
            _ => unreachable!(),
        };
        Ok(operator)
    }

    #[cfg(feature = "simd")]
    fn encode_v128_select(
        &mut self,
        condition: ResolvedOperand<i32>,
        true_val: Operand,
        false_val: Operand,
    ) -> Result<(), Error> {
        let true_val = self.copy_operand_to_slot(true_val)?;
        let false_val = self.copy_operand_to_slot(false_val)?;
        self.push_instr_with_result_slot(
            ValType::V128,
            |result| match condition {
                ResolvedOperand::Reg => Op::v128_select_srss(result, true_val, false_val),
                ResolvedOperand::Slot(condition) => {
                    Op::v128_select_ssss(result, condition, true_val, false_val)
                }
                ResolvedOperand::Immediate(_) => unreachable!(),
            },
            FuelCostsProvider::base,
        )?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
enum SelectFusion {
    None,
    Fused,
    FusedSwap,
}

impl SelectFusion {
    pub fn is_fused(self) -> bool {
        matches!(self, Self::Fused | Self::FusedSwap)
    }
}

impl FuncTranslator {
    /// Tries to fuse a compare instruction with a Wasm `select` instruction.
    ///
    /// # Returns
    ///
    /// - Returns [`SelectFusion::Fused`] or [`SelectFusion::FusedSwap`] if fusion was successful.
    ///     - If [`SelectFusion::FusedSwap`] was returned, true and false operands need to be swapped.
    /// - Returns [`SelectFusion::None`] if fusion could not be applied.
    fn try_fuse_select(&self, condition: ResolvedOperand<i32>) -> Result<SelectFusion, Error> {
        let Some(staged) = self.instrs.peek_staged() else {
            // If there is no last instruction there is no comparison instruction to negate.
            return Ok(SelectFusion::None);
        };
        let Some(staged_result) = staged.result_loc() else {
            // All negatable instructions have a single result register.
            return Ok(SelectFusion::None);
        };
        if let ir::Location::Slot(result_slot) = staged_result {
            if matches!(self.layout.stack_space(result_slot), StackSpace::Local) {
                // The staged operator stores its result into a local variable which
                // is an observable side effect that must not be fused.
                return Ok(SelectFusion::None);
            }
        }
        match (staged_result, condition) {
            (ir::Location::Reg(ValType::I64), ResolvedOperand::Reg) => {}
            (ir::Location::Slot(staged), ResolvedOperand::Slot(condition))
                if staged == condition => {}
            _ => return Ok(SelectFusion::None),
        }
        #[rustfmt::skip]
        let fusion = match staged {
            | Op::I32Eq_Rri { rhs: 0, .. }
            | Op::I32Eq_Rsi { rhs: 0, .. } => SelectFusion::FusedSwap,
            | Op::I32NotEq_Rri { rhs: 0, .. }
            | Op::I32NotEq_Rsi { rhs: 0, .. } => SelectFusion::Fused,
            | _ => SelectFusion::None,
        };
        Ok(fusion)
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
    fn fuse_commutative_cmp_with<T: WasmInteger>(
        &mut self,
        lhs: Operand,
        rhs: T,
        try_fuse: fn(cmp: &Op) -> Option<Op>,
    ) -> Result<bool, Error> {
        if !rhs.is_zero() {
            // Case: cannot fuse with non-zero `rhs`
            return Ok(false);
        }
        let Some(staged) = self.instrs.peek_staged() else {
            // Case: cannot fuse without registered last instruction
            return Ok(false);
        };
        let Operand::Temp(lhs) = lhs else {
            // Case: cannot fuse non-temporary operands
            //  - locals have observable behavior.
            //  - immediates cannot be the result of a previous instruction.
            return Ok(false);
        };
        let lhs_reg = lhs.temp_slots().head();
        let Some(result) = staged.result_ref().copied() else {
            // Case: cannot fuse non-cmp instructions
            return Ok(false);
        };
        if result != lhs_reg {
            // Case: the `cmp` instruction does not feed into the `eqz` and cannot be fused
            return Ok(false);
        }
        let Some(negated) = try_fuse(&staged) else {
            // Case: the `cmp` instruction cannot be negated
            return Ok(false);
        };
        // Need to push back `lhs` but with its type adjusted to be `i32`
        // since that's the return type of `i{32,64}.{eqz,eq,ne}`.
        let new_result = self.stack.push_temp(ValType::I32)?.temp_slots().head();
        // Need to replace `cmp` instruction result register since it might
        // have been misaligned if `lhs` originally referred to the zero operand.
        let Some(negated) = negated.update_result_slot(new_result) else {
            unreachable!("`negated` has been asserted as `cmp` instruction");
        };
        self.instrs.replace_staged(negated)?;
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
    fn translate_load<T: op::LoadOp>(&mut self, memarg: MemArg) -> Result<(), Error> {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg)?;
        let ptr = self.stack.pop();
        let ptr = self.resolve_operand_as_index(ptr, memory)?;
        'opt: {
            // Try to encode an optimized load operator if possible, otherwise fallback.
            if !memory.is_default() {
                break 'opt;
            }
            let offset = match Offset16::try_from(offset) {
                Ok(offset) => offset,
                Err(_) => break 'opt,
            };
            let op = match ptr {
                ResolvedOperand::Reg => T::op_rr_mem0_offset16(offset),
                ResolvedOperand::Slot(ptr) => T::op_rs_mem0_offset16(ptr, offset),
                ResolvedOperand::Immediate(_) => break 'opt,
            };
            self.stage_op_with_result_reg(<T::Result as Typed>::TY, op, FuelCostsProvider::load)?;
            return Ok(());
        }
        // We need to encode a non-optimized fallback load operator.
        let Some(ptr) = ptr.filter_map(|ptr| self.effective_address(memory, ptr, offset)) else {
            return self.translate_trap(TrapCode::MemoryOutOfBounds);
        };
        let op = match ptr {
            ResolvedOperand::Reg => T::op_rr(offset, memory),
            ResolvedOperand::Slot(ptr) => T::op_rs(ptr, offset, memory),
            ResolvedOperand::Immediate(address) => T::op_ri(address, memory),
        };
        self.stage_op_with_result_reg(<T::Result as Typed>::TY, op, FuelCostsProvider::load)?;
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
    fn translate_store<T: op::StoreOp>(&mut self, memarg: MemArg) -> Result<(), Error>
    where
        T::Value: Copy + From<RawVal>,
        T::Immediate: Copy,
    {
        bail_unreachable!(self);
        let (ptr, value) = self.stack.pop2();
        self.encode_store::<T>(memarg, ptr, value)
    }

    fn encode_store<T: op::StoreOp>(
        &mut self,
        memarg: MemArg,
        ptr: Operand,
        value: Operand,
    ) -> Result<(), Error>
    where
        T::Value: Copy + From<RawVal>,
        T::Immediate: Copy,
    {
        let (memory, offset) = Self::decode_memarg(memarg)?;
        let Some(ptr) = self
            .resolve_operand_as_index(ptr, memory)?
            .map(|ptr| self.effective_address(memory, ptr, offset))
            .transpose()
        else {
            return self.translate_trap(TrapCode::MemoryOutOfBounds);
        };
        let value = self
            .resolve_operand_as::<T::Value>(value)?
            .map(T::into_immediate);
        let op = self.choose_store_op::<T>(memarg, ptr, value)?;
        self.push_instr(op, FuelCostsProvider::store)?;
        Ok(())
    }

    /// Selects which store operator to encode based on type `T`.
    fn choose_store_op<T: op::StoreOp>(
        &mut self,
        memarg: MemArg,
        ptr: ResolvedOperand<Address>,
        value: ResolvedOperand<T::Immediate>,
    ) -> Result<Op, Error>
    where
        T::Value: Copy + From<RawVal>,
        T::Immediate: Copy,
    {
        use ResolvedOperand as Opd;
        let (memory, offset) = Self::decode_memarg(memarg)?;
        if let Some(op) = self.choose_store_mem0_offset16_op::<T>(ptr, offset, memory, value)? {
            return Ok(op);
        }
        let op = match (ptr, value) {
            (Opd::Reg, Opd::Reg) => match T::store_rr(offset, memory) {
                Some(op) => op,
                None => unreachable!(),
            },
            (Opd::Reg, Opd::Slot(value)) => T::store_rs(offset, value, memory),
            (Opd::Reg, Opd::Immediate(value)) => T::store_ri(offset, value, memory),
            (Opd::Slot(ptr), Opd::Reg) => T::store_sr(ptr, offset, memory),
            (Opd::Slot(ptr), Opd::Slot(value)) => T::store_ss(ptr, offset, value, memory),
            (Opd::Slot(ptr), Opd::Immediate(value)) => T::store_si(ptr, offset, value, memory),
            (Opd::Immediate(address), Opd::Reg) => T::store_ir(address, memory),
            (Opd::Immediate(address), Opd::Slot(value)) => T::store_is(address, value, memory),
            (Opd::Immediate(address), Opd::Immediate(value)) => T::store_ii(address, value, memory),
        };
        Ok(op)
    }

    /// Selects a Wasm store operator with `(mem 0)` and 16-bit encodable `offset` to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// - Returns `Ok(true)` if encoding is available.
    /// - Returns `Ok(false)` if encoding is not available.
    /// - Returns `Err(_)` if an error occurred.
    fn choose_store_mem0_offset16_op<T: op::StoreOp>(
        &mut self,
        ptr: ResolvedOperand<Address>,
        offset: u64,
        memory: index::Memory,
        value: ResolvedOperand<T::Immediate>,
    ) -> Result<Option<Op>, Error>
    where
        T::Value: Copy + From<RawVal>,
        T::Immediate: Copy,
    {
        use Location as Loc;
        use ResolvedOperand as Opd;
        if !memory.is_default() {
            return Ok(None);
        }
        let Ok(offset) = Offset16::try_from(offset) else {
            return Ok(None);
        };
        let ptr = match ptr {
            Opd::Reg => Loc::Reg,
            Opd::Slot(ptr) => Loc::Slot(ptr),
            Opd::Immediate(_) => return Ok(None),
        };
        let op = match (ptr, value) {
            (Loc::Reg, Opd::Reg) => match T::store_mem0_offset16_rr(offset) {
                Some(op) => op,
                None => unreachable!(),
            },
            (Loc::Reg, Opd::Slot(value)) => T::store_mem0_offset16_rs(offset, value),
            (Loc::Reg, Opd::Immediate(value)) => T::store_mem0_offset16_ri(offset, value),
            (Loc::Slot(ptr), Opd::Reg) => T::store_mem0_offset16_sr(ptr, offset),
            (Loc::Slot(ptr), Opd::Slot(value)) => T::store_mem0_offset16_ss(ptr, offset, value),
            (Loc::Slot(ptr), Opd::Immediate(value)) => {
                T::store_mem0_offset16_si(ptr, offset, value)
            }
        };
        Ok(Some(op))
    }

    /// Returns the [`MemArg`] linear `memory` index and load/store `offset`.
    ///
    /// # Panics
    ///
    /// If the [`MemArg`] offset is not 32-bit.
    fn decode_memarg(memarg: MemArg) -> Result<(index::Memory, u64), Error> {
        let memory = index::Memory::try_from(memarg.memory)?;
        Ok((memory, memarg.offset))
    }

    /// Returns the effective address `ptr+offset` if it is valid.
    // TODO: rename to just `effective_address` and remove old `effective_address`
    fn effective_address(&self, mem: index::Memory, ptr: u64, offset: u64) -> Option<Address> {
        let memory_type = *self
            .module
            .get_type_of_memory(MemoryIdx::from(u32::from(u16::from(mem))));
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
        make_instr: fn(
            results: FixedSlotSpan<2>,
            lhs_lo: Slot,
            lhs_hi: Slot,
            rhs_lo: Slot,
            rhs_hi: Slot,
        ) -> Op,
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
        let rhs_lo = self.copy_operand_to_slot(rhs_lo)?;
        let rhs_hi = self.copy_operand_to_slot(rhs_hi)?;
        let lhs_lo = self.copy_operand_to_slot(lhs_lo)?;
        let lhs_hi = self.copy_operand_to_slot(lhs_hi)?;
        let result_lo = self.stack.push_temp(ValType::I64)?.temp_slots().head();
        let result_hi = self.stack.push_temp(ValType::I64)?.temp_slots().head();
        let Ok(results) = <FixedSlotSpan<2>>::new(SlotSpan::new(result_lo)) else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        debug_assert_eq!(results.to_array(), [result_lo, result_hi]);
        self.push_instr(
            make_instr(results, lhs_lo, lhs_hi, rhs_lo, rhs_hi),
            FuelCostsProvider::base,
        )?;
        Ok(())
    }

    /// Translates a Wasm `i64.mul_wide_sx` instruction from the `wide-arithmetic` proposal.
    fn translate_i64_mul_wide_sx(
        &mut self,
        make_instr: fn(results: FixedSlotSpan<2>, lhs: Slot, rhs: Slot) -> Op,
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
            (lhs, Operand::Immediate(rhs_imm)) => {
                let rhs_val = rhs_imm.val();
                if self.try_opt_i64_mul_wide_sx(lhs, rhs_val, signed)? {
                    return Ok(());
                }
                let lhs = self.copy_operand_to_slot(lhs)?;
                let rhs = self.copy_operand_to_slot(rhs)?;
                (lhs, rhs)
            }
            (Operand::Immediate(lhs_imm), rhs) => {
                let lhs_val = lhs_imm.val();
                if self.try_opt_i64_mul_wide_sx(rhs, lhs_val, signed)? {
                    return Ok(());
                }
                let lhs = self.copy_operand_to_slot(lhs)?;
                let rhs = self.copy_operand_to_slot(rhs)?;
                (lhs, rhs)
            }
            (lhs, rhs) => {
                let lhs = self.copy_operand_to_slot(lhs)?;
                let rhs = self.copy_operand_to_slot(rhs)?;
                (lhs, rhs)
            }
        };
        let result0 = self.stack.push_temp(ValType::I64)?.temp_slots().head();
        self.stack.push_temp(ValType::I64)?;
        let Ok(results) = <FixedSlotSpan<2>>::new(SlotSpan::new(result0)) else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        self.push_instr(make_instr(results, lhs, rhs), FuelCostsProvider::base)?;
        Ok(())
    }

    /// Try to optimize a `i64.mul_wide_sx` instruction with one [`Slot`] and one immediate input.
    ///
    /// - Returns `Ok(true)` if the optimiation was applied successfully.
    /// - Returns `Ok(false)` if no optimization was applied.
    fn try_opt_i64_mul_wide_sx(
        &mut self,
        lhs: Operand,
        rhs: TypedRawVal,
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
            let result = self.stack.push_operand(lhs)?; // lo-bits
            if matches!(lhs, Operand::Temp(_)) {
                // Case: `lhs` is temporary and thus might need a copy to its new result.
                let consume_fuel_instr = self.stack.consume_fuel_instr();
                let result = result.temp_slots().head();
                self.encode_copy(result, lhs, consume_fuel_instr)?;
            }
            self.stack.push_immediate(0_i64)?; // hi-bits
            return Ok(true);
        }
        Ok(false)
    }
}
