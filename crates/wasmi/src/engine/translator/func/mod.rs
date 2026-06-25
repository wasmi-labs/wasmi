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
    utils::{Reset, ReusableAllocations},
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
        CompiledFuncEntry,
        TranslationError,
        code_map::FuncEntry,
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
                stack::{Allocation, BranchParams, PreservedRegs, RegKind},
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
        SlotAndReg,
        SlotSpan,
        index::{self, Memory},
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
    /// Temporary buffer for immediate values.
    immediates: Vec<TypedRawVal>,
}

impl Reset for FuncTranslatorAllocations {
    fn reset(&mut self) {
        self.stack.reset();
        self.locals.reset();
        self.layout.reset();
        self.instrs.reset();
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
        finalize: impl FnOnce(CompiledFuncEntry),
    ) -> Result<Self::Allocations, Error> {
        // Note: `update_branch_offsets` might change `frame_size` so we need to compute it prior.
        //
        // Context:
        // This only happens if the function has so many instructions that some conditional branch
        // operators need to be encoded as their fallbacks which requires to allocate more function
        // local constant values, thus increasing the size of the function frame.
        self.instrs.update_branch_offsets()?;
        let len_local_slots = self.stack.get_local_slots();
        let Some(len_stack_slots) = self.len_stack_slots() else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        finalize(CompiledFuncEntry::new(
            len_local_slots,
            len_stack_slots,
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
        let consume_fuel = self.instrs.encode_consume_fuel_op()?;
        self.stack
            .push_func_block(block_ty, end_label, consume_fuel)?;
        Ok(())
    }

    /// Returns the total number of stack slots used by the compiled function.
    ///
    /// Returns `None` if the this number is out of bounds.
    fn len_stack_slots(&self) -> Option<u16> {
        let len_stack_slots = self
            .stack
            .max_stack_offset()
            .checked_add(self.locals.len())?;
        u16::try_from(len_stack_slots).ok()
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
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<BoundedSlotSpan, Error> {
        debug_assert!(len > 0);
        let mut copied_cells: u16 = 0;
        for n in 0..len {
            let operand = self.stack.operand_to_temp(n);
            copied_cells = copied_cells
                .checked_add(operand.temp_slots().len())
                .ok_or(TranslationError::SlotAccessOutOfBounds)?;
            self.copy_operand_to_temp(operand, fuel_pos)?;
        }
        let first = self.stack.peek(len - 1).temp_slots().head();
        Ok(BoundedSlotSpan::new(SlotSpan::new(first), copied_cells))
    }

    /// Emits copy operators to copy all operands to satisfy the [`BranchParams`].
    ///
    /// # Note
    ///
    /// - This does _not_ change or mutate the operand stack.
    /// - Uses `fuel_pos` to bump fuel consumption if enabled.
    fn copy_branch_params(
        &mut self,
        params: BranchParams,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        self.copy_branch_params_temps(params, fuel_pos)?;
        self.copy_branch_params_regs(params, fuel_pos)?;
        Ok(())
    }

    /// Copies the branch params that are expected in temporary stack slots.
    ///
    /// Part of [`Self::copy_branch_params`].
    fn copy_branch_params_temps(
        &mut self,
        params: BranchParams,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        let len_temps = params.len_temps();
        if len_temps == 0 {
            return Ok(());
        }
        let mut result = params.temp_slots().head();
        let start = params.len();
        let end = params.len_regs();
        let mut prev = None;
        for depth in (end..start).rev() {
            let value = self.stack.peek(depth.into());
            let ty = value.ty();
            let copy_if_not_noop = Self::select_copy_sx_op(result, value, &self.layout)?;
            result = result.next_n(required_cells_for_ty(ty));
            let Some(copy_op) = copy_if_not_noop else {
                // Case: no-op copy instruction
                continue;
            };
            let (new_prev, copy_op) = Self::copy_branch_params_temps_fuse(prev.take(), copy_op);
            prev = new_prev;
            self.copy_branch_params_temps_lower(copy_op, fuel_pos)?;
        }
        // The `prev` might still contain `Some` at this point, so we have to "flush" it.
        self.copy_branch_params_temps_lower(prev.take(), fuel_pos)?;
        Ok(())
    }

    /// Tries to fuse `prev` and `copy_op` if possible.
    ///
    /// Returns a pair of [`Option<Op>`]:
    ///
    /// - The first item represents the new `prev` after this operation.
    /// - The second item represents the [`Op`] to be encoded if any.
    fn copy_branch_params_temps_fuse(prev: Option<Op>, copy_op: Op) -> (Option<Op>, Option<Op>) {
        let (new_results, new_values, new_len) = match copy_op {
            Op::U64Copy_Ss { result, value } => (SlotSpan::new(result), SlotSpan::new(value), 1),
            #[cfg(feature = "simd")]
            Op::V128Copy_Ss { result, value } => (SlotSpan::new(result), SlotSpan::new(value), 2),
            Op::CopySpanAsc {
                results,
                values,
                len,
            }
            | Op::CopySpanDes {
                results,
                values,
                len,
            } => (results, values, len),
            _ => return (Some(copy_op), prev),
        };
        let copy_span = Op::copy_span_asc(new_results, new_values, new_len);
        let Some(Op::CopySpanAsc {
            results,
            values,
            len,
        }) = prev
        else {
            // Case: no fusable previous copy for op-code fusion.
            return (Some(copy_span), None);
        };
        {
            // Try to fuse in ascending slot order:
            let can_fuse_asc = new_results.head() == results.head().next_n(len)
                && new_values.head() == values.head().next_n(len);
            if can_fuse_asc {
                // Case: copy fusion in ascending slot order can be applied.
                let prev = Some(Op::CopySpanAsc {
                    results,
                    values,
                    len: len + new_len,
                });
                return (prev, None);
            }
        }
        {
            // Try to fuse in ascending slot order:
            let can_fuse_des = new_results.head() == results.head().prev_n(new_len)
                && new_values.head() == values.head().prev_n(new_len);
            if can_fuse_des {
                // Case: copy fusion in descending slot order can be applied.
                let prev = Some(Op::CopySpanDes {
                    results: new_results,
                    values: new_values,
                    len: len + new_len,
                });
                return (prev, None);
            }
        }
        // Case: copy fusion was not applied.
        (Some(copy_span), prev)
    }

    /// Encodes `copy_op` if any as lowered (more efficient) version if possible.
    fn copy_branch_params_temps_lower(
        &mut self,
        copy_op: Option<Op>,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        let Some(copy_op) = copy_op else {
            return Ok(());
        };
        // Note: we only care about `CopySpanAsc` and not `CopySpanDes` because during fusion
        //       we only ever memorize `CopySpanAsc` since we can easily restore slot order here.
        let copy_op = 'op: {
            let Op::CopySpanAsc {
                results,
                values,
                len,
            } = copy_op
            else {
                break 'op copy_op;
            };
            match len {
                1 => Op::u64_copy_ss(results.head(), values.head()),
                #[cfg(feature = "simd")]
                2 => Op::v128_copy_ss(results.head(), values.head()),
                _ if results < values => Op::copy_span_asc(results, values, len),
                _ => Op::copy_span_des(results, values, len),
            }
        };
        self.instrs
            .encode_op(copy_op, fuel_pos, FuelCostsProvider::base)?;
        Ok(())
    }

    /// Copies the branch params that are expected in their respective registers.
    ///
    /// Part of [`Self::copy_branch_params`].
    fn copy_branch_params_regs(
        &mut self,
        params: BranchParams,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        for (depth, kind) in params.regs().iter().enumerate() {
            let value = self.stack.peek(depth);
            debug_assert!(kind.matches_ty(value.ty()));
            self.encode_copy_rx_op(value, fuel_pos)?;
        }
        Ok(())
    }

    /// Encodes a single `copy_rx` operator.
    ///
    /// # Note
    ///
    /// This won't encode a copy if `value` is already in its register.
    fn encode_copy_rx_op(
        &mut self,
        value: Operand,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        let Some(op) = Self::select_copy_rx_op(value, &self.layout)? else {
            return Ok(());
        };
        self.instrs
            .encode_op(op, fuel_pos, FuelCostsProvider::base)?;
        Ok(())
    }

    /// Returns the [`Op`] to copy `value` into its register.
    ///
    /// Returns `None` if `value` is already in its register.
    fn select_copy_rx_op(value: Operand, layout: &StackLayout) -> Result<Option<Op>, Error> {
        match value.ty() {
            ValType::I32 | ValType::FuncRef | ValType::ExternRef => {
                Self::select_u32_copy_rx_op(value, layout)
            }
            ValType::I64 => Self::select_u64_copy_rx_op(value, layout),
            ValType::F32 => Self::select_f32_copy_rx_op(value, layout),
            ValType::F64 => Self::select_f64_copy_rx_op(value, layout),
            ValType::V128 => unreachable!(),
        }
    }

    /// Returns the [`Op`] to copy `value` of type `u32` into its register.
    ///
    /// Returns `None` if `value` is already in its register.
    fn select_u32_copy_rx_op(value: Operand, layout: &StackLayout) -> Result<Option<Op>, Error> {
        let op = match value.resolve(layout)? {
            ResolvedOperand::Reg(ty) => {
                debug_assert!(matches!(
                    ty,
                    ValType::I32 | ValType::ExternRef | ValType::FuncRef
                ));
                return Ok(None);
            }
            ResolvedOperand::Slot(value) => Op::u64_copy_rs(value),
            ResolvedOperand::Immediate(value) => Op::u32_copy_ri(u32::from(value.raw())),
        };
        Ok(Some(op))
    }

    /// Returns the [`Op`] to copy `value` of type `u64` into its register.
    ///
    /// Returns `None` if `value` is already in its register.
    fn select_u64_copy_rx_op(value: Operand, layout: &StackLayout) -> Result<Option<Op>, Error> {
        let op = match value.resolve_as::<u64>(layout)? {
            ResolvedOperand::Reg(ty) => {
                debug_assert_eq!(ty, ValType::I64);
                return Ok(None);
            }
            ResolvedOperand::Slot(value) => Op::u64_copy_rs(value),
            ResolvedOperand::Immediate(value) => Op::u64_copy_ri(value),
        };
        Ok(Some(op))
    }

    /// Returns the [`Op`] to copy `value` of type `f32` into its register.
    ///
    /// Returns `None` if `value` is already in its register.
    fn select_f32_copy_rx_op(value: Operand, layout: &StackLayout) -> Result<Option<Op>, Error> {
        let op = match value.resolve_as::<f32>(layout)? {
            ResolvedOperand::Reg(ty) => {
                debug_assert_eq!(ty, ValType::F32);
                return Ok(None);
            }
            ResolvedOperand::Slot(value) => Op::f32_copy_rs(value),
            ResolvedOperand::Immediate(value) => Op::f32_copy_ri(value),
        };
        Ok(Some(op))
    }

    /// Returns the [`Op`] to copy `value` of type `f64` into its register.
    ///
    /// Returns `None` if `value` is already in its register.
    fn select_f64_copy_rx_op(value: Operand, layout: &StackLayout) -> Result<Option<Op>, Error> {
        let op = match value.resolve_as::<f64>(layout)? {
            ResolvedOperand::Reg(ty) => {
                debug_assert_eq!(ty, ValType::F64);
                return Ok(None);
            }
            ResolvedOperand::Slot(value) => Op::f64_copy_rs(value),
            ResolvedOperand::Immediate(value) => Op::f64_copy_ri(value),
        };
        Ok(Some(op))
    }

    /// Convenience wrapper for [`Self::encode_copy_sx_op_impl`].
    fn encode_copy_sx_op(
        &mut self,
        result: Slot,
        value: Operand,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<Option<Pos<Op>>, Error> {
        Self::encode_copy_sx_op_impl(result, value, fuel_pos, &self.layout, &mut self.instrs)
    }

    /// Encodes a single `copy_sx` operator.
    ///
    /// # Note
    ///
    /// This won't encode a copy if `result` and `value` yields a no-op copy.
    fn encode_copy_sx_op_impl(
        result: Slot,
        value: Operand,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
        layout: &StackLayout,
        encoder: &mut OpEncoder,
    ) -> Result<Option<Pos<Op>>, Error> {
        let Some(copy_instr) = Self::select_copy_sx_op(result, value, layout)? else {
            // Case: no-op copy instruction
            return Ok(None);
        };
        let pos = encoder.encode_op(copy_instr, fuel_pos, FuelCostsProvider::base)?;
        Ok(Some(pos))
    }

    /// Returns the copy instruction to copy the given `operand` to `result`.
    ///
    /// Returns `None` if the resulting copy instruction is a no-op.
    fn select_copy_sx_op(
        result: Slot,
        value: Operand,
        layout: &StackLayout,
    ) -> Result<Option<Op>, Error> {
        let ty = value.ty();
        let op = match value.resolve(layout)? {
            ResolvedOperand::Reg(ty) => Self::select_copy_sr_op(result, ty)?,
            ResolvedOperand::Slot(value) => return Self::select_copy_ss_op(result, value, ty),
            ResolvedOperand::Immediate(value) => Self::select_copy_si_op(result, value)?,
        };
        Ok(Some(op))
    }

    /// Returns the [`Op`] to copy the register `value` into `result` for `ty`.
    fn select_copy_sr_op(result: Slot, ty: ValType) -> Result<Op, Error> {
        match ty {
            | ValType::I32 | ValType::I64 | ValType::FuncRef | ValType::ExternRef => {
                Self::select_u64_copy_sr_op(result)
            }
            | ValType::F32 => Self::select_f32_copy_sr_op(result),
            | ValType::F64 => Self::select_f64_copy_sr_op(result),
            | ValType::V128 => unreachable!(),
        }
    }

    /// Returns the [`Op`] to copy the register `value` of type `u64` into `result`.
    fn select_u64_copy_sr_op(result: Slot) -> Result<Op, Error> {
        let op = match u16::from(result) {
            0 => Op::u64_copy_s0r(),
            1 => Op::u64_copy_s1r(),
            2 => Op::u64_copy_s2r(),
            3 => Op::u64_copy_s3r(),
            4 => Op::u64_copy_s4r(),
            5 => Op::u64_copy_s5r(),
            6 => Op::u64_copy_s6r(),
            7 => Op::u64_copy_s7r(),
            8 => Op::u64_copy_s8r(),
            9 => Op::u64_copy_s9r(),
            _ => Op::u64_copy_sr(result),
        };
        Ok(op)
    }

    /// Returns the [`Op`] to copy the register `value` of type `f32` into `result`.
    fn select_f32_copy_sr_op(result: Slot) -> Result<Op, Error> {
        let op = match u16::from(result) {
            0 => Op::f32_copy_s0r(),
            1 => Op::f32_copy_s1r(),
            2 => Op::f32_copy_s2r(),
            3 => Op::f32_copy_s3r(),
            4 => Op::f32_copy_s4r(),
            5 => Op::f32_copy_s5r(),
            6 => Op::f32_copy_s6r(),
            7 => Op::f32_copy_s7r(),
            8 => Op::f32_copy_s8r(),
            9 => Op::f32_copy_s9r(),
            _ => Op::f32_copy_sr(result),
        };
        Ok(op)
    }

    /// Returns the [`Op`] to copy the register `value` of type `f64` into `result`.
    fn select_f64_copy_sr_op(result: Slot) -> Result<Op, Error> {
        let op = match u16::from(result) {
            0 => Op::f64_copy_s0r(),
            1 => Op::f64_copy_s1r(),
            2 => Op::f64_copy_s2r(),
            3 => Op::f64_copy_s3r(),
            4 => Op::f64_copy_s4r(),
            5 => Op::f64_copy_s5r(),
            6 => Op::f64_copy_s6r(),
            7 => Op::f64_copy_s7r(),
            8 => Op::f64_copy_s8r(),
            9 => Op::f64_copy_s9r(),
            _ => Op::f64_copy_sr(result),
        };
        Ok(op)
    }

    /// Returns the [`Op`] to copy the [`Slot`] `value` into `result`.
    ///
    /// Returns `None` if the `copy` is a no-op.
    fn select_copy_ss_op(result: Slot, value: Slot, ty: ValType) -> Result<Option<Op>, Error> {
        if result == value {
            return Ok(None);
        }
        let op = match ty {
            #[cfg(feature = "simd")]
            ValType::V128 => Op::v128_copy_ss(result, value),
            _ => Op::u64_copy_ss(result, value),
        };
        Ok(Some(op))
    }

    /// Returns the [`Op`] to copy the immediate `value` into `result`.
    fn select_copy_si_op(result: Slot, value: TypedRawVal) -> Result<Op, Error> {
        let raw = value.raw();
        let op = match value.ty() {
            | ValType::FuncRef | ValType::ExternRef | ValType::I32 | ValType::F32 => {
                Op::u32_copy_si(result, u32::from(raw))
            }
            | ValType::I64 | ValType::F64 => Op::u64_copy_si(result, u64::from(raw)),
            #[cfg(feature = "simd")]
            | ValType::V128 => Op::v128_copy_si(result, V128::from(raw)),
            #[cfg(not(feature = "simd"))]
            | ValType::V128 => unreachable!(),
        };
        Ok(op)
    }

    /// Returns `true` if there is a need to copy branch parameters for the frame at `depth` with the current stack.
    ///
    /// # Dev. Note
    ///
    /// We take `depth` to a frame instead of a reference to the frame directly
    /// to avoid some borrow-checking issues at users.
    ///
    /// # Note
    ///
    /// Conditional branches can be encoded in a more efficient way
    /// if no branch parameter copies are required.
    fn requires_branch_param_copies(&self, depth: usize) -> bool {
        let frame = self.stack.peek_control(depth);
        let branch_params = frame.branch_params();
        let len_params = usize::from(branch_params.len());
        if len_params == 0 {
            // The frame has no branch params and thus no copies need to be performed.
            return false;
        }
        let frame_height = frame.height();
        let height_matches = frame_height == (self.stack.height() - len_params);
        let has_temp_params = branch_params.len_temps() > 0;
        if !height_matches && has_temp_params {
            // If the height does not match we need to copy
            return true;
        }
        let len_regs = usize::from(branch_params.len_regs());
        let temp_params_require_copies = (len_regs..len_params)
            .map(|depth| self.stack.peek(depth))
            .any(|o| !o.is_temp() || o.in_reg());
        if temp_params_require_copies {
            // The branch paramters expected in temporary stack slots require copy operations.
            return true;
        }
        let reg_params_require_copies = (0..len_regs)
            .map(|depth| self.stack.peek(depth))
            .any(|o| !o.in_reg());
        if reg_params_require_copies {
            // The branch paramters expected in registers require copy operations.
            return true;
        }
        false
    }

    fn copy_operand_to_reg(&mut self, operand: Operand) -> Result<(), Error> {
        let ty = operand.ty();
        let Some(op) = Self::select_copy_rx_op(operand, &self.layout)? else {
            // Case: No copy needed as operand already resides in register.
            self.push_result_reg(ty)?;
            return Ok(());
        };
        self.push_op_with_result_reg(ty, op, FuelCostsProvider::base)?;
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
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<Slot, Error> {
        let result = operand.temp_slots().head();
        self.encode_copy_sx_op(result, operand, fuel_pos)?;
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
        let location = match self.resolve_operand(operand)? {
            ResolvedOperand::Reg(ty) => Location::Reg(ty),
            ResolvedOperand::Slot(value) => Location::Slot(value),
            ResolvedOperand::Immediate(value) => {
                let result = operand.temp_slots().head();
                let copy_instr = Self::select_copy_si_op(result, value)?;
                let consume_fuel = self.stack.fuel_pos();
                self.instrs
                    .encode_op(copy_instr, consume_fuel, FuelCostsProvider::base)?;
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
        let copy_op = match self.resolve_operand::<RawVal>(operand)? {
            ResolvedOperand::Slot(slot) => return Ok(slot),
            ResolvedOperand::Reg(ty) => match ty {
                | ValType::I32 | ValType::FuncRef | ValType::ExternRef | ValType::I64 => {
                    Op::u64_copy_sr(result)
                }
                | ValType::F32 => Op::f32_copy_sr(result),
                | ValType::F64 => Op::f64_copy_sr(result),
                | ValType::V128 => unreachable!(),
            },
            ResolvedOperand::Immediate(value) => {
                Self::select_copy_si_op(result, TypedRawVal::new(ty, value))?
            }
        };
        let fuel_op = self.stack.fuel_pos();
        self.instrs
            .encode_op(copy_op, fuel_op, FuelCostsProvider::base)?;
        Ok(result)
    }

    /// Preserves all local operands on the stack.
    ///
    /// # Note
    ///
    /// This works by encoding copy instructions to `temp` register space.
    fn preserve_all_locals(&mut self, skip: usize) -> Result<(), Error> {
        let fuel_pos = self.stack.fuel_pos();
        for local in self.stack.preserve_all_locals(skip) {
            let result = local.temp_slots().head();
            let Some(copy_instr) = Self::select_copy_sx_op(result, local.into(), &self.layout)?
            else {
                unreachable!("`result` and `local` refer to different stack spaces");
            };
            self.instrs
                .encode_op(copy_instr, fuel_pos, FuelCostsProvider::base)?;
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
        let consume_fuel = self.stack.fuel_pos();
        let instr = self.instrs.encode_op(instr, consume_fuel, fuel_costs)?;
        Ok(instr)
    }

    /// Returns `Some` if the staged [`Op`] can be fused with a `copy_sr` with `result`.
    ///
    /// Returns `None` otherwise.
    fn fuse_copy_sr(&self, result: Slot, input_ty: ValType) -> Option<Op> {
        if !RegKind::Ireg.matches_ty(input_ty) {
            // This check is required to filter out incorrect fusions
            // where input is not originating from the staged operator.
            //
            // Today, all fused operators have an `ireg` result.
            // We need to change this logic if we add more fused operators later.
            return None;
        }
        let result = SlotAndReg::from(result);
        let staged_op = self.instrs.peek_staged()?;
        let op = match staged_op {
            // i32
            Op::I32Add_Rrs { rhs, .. } => Op::i32_add_rs_rs(result, rhs),
            Op::I32Add_Rri { rhs, .. } => Op::i32_add_rs_ri(result, rhs),
            Op::I32Add_Rss { lhs, rhs, .. } => Op::i32_add_rs_ss(result, lhs, rhs),
            Op::I32Add_Rsi { lhs, rhs, .. } => Op::i32_add_rs_si(result, lhs, rhs),
            // i64
            Op::I64Add_Rrs { rhs, .. } => Op::i64_add_rs_rs(result, rhs),
            Op::I64Add_Rri { rhs, .. } => Op::i64_add_rs_ri(result, rhs),
            Op::I64Add_Rss { lhs, rhs, .. } => Op::i64_add_rs_ss(result, lhs, rhs),
            Op::I64Add_Rsi { lhs, rhs, .. } => Op::i64_add_rs_si(result, lhs, rhs),
            _ => return None,
        };
        Some(op)
    }

    /// Pushes a result register operand onto the stack.
    ///
    /// If a register operand of an equivalent type is already on the stack
    /// a copy operator is encoded to turn the existing register operand into
    /// a temporary operand.
    fn push_result_reg(&mut self, ty: ValType) -> Result<(), Error> {
        let fuel_pos = self.stack.fuel_pos();
        if let Some(operand) = self.stack.dealloc_reg(ty) {
            let result = operand.temp_slots().head();
            let op = match self.fuse_copy_sr(result, ty) {
                Some(fused_op) => {
                    self.instrs.drop_staged();
                    fused_op
                }
                None => Self::select_copy_sr_op(result, operand.ty())?,
            };
            self.instrs
                .encode_op(op, fuel_pos, FuelCostsProvider::base)?;
        };
        self.stack.push_temp(ty, Allocation::Reg)?;
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
        let fuel_pos = self.stack.fuel_pos();
        self.instrs.stage_op(op, fuel_pos, fuel_costs)?;
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
        let fuel_pos = self.stack.fuel_pos();
        self.instrs.encode_op(op, fuel_pos, fuel_costs)?;
        Ok(())
    }

    /// Pushes the `instr` to the function with the associated `fuel_costs`.
    #[cfg(feature = "simd")]
    fn push_op_with_result_slot(
        &mut self,
        result_ty: ValType,
        make_instr: impl FnOnce(Slot) -> Op,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), Error> {
        let fuel_pos = self.stack.fuel_pos();
        let result = self
            .stack
            .push_temp(result_ty, Allocation::None)?
            .temp_slots()
            .head();
        let op = make_instr(result);
        debug_assert!(op.result_ref().is_some());
        self.instrs.stage_op(op, fuel_pos, fuel_costs)?;
        Ok(())
    }

    /// Pushes an operator to the function with the associated `fuel_costs` if `make_op` yields `Some`.
    ///
    /// Only pushes the operand to the stack without encoding an operator if `make_op` yields `None`.
    #[cfg(feature = "simd")]
    fn try_push_op_with_result_slot(
        &mut self,
        result_ty: ValType,
        make_op: impl FnOnce(Slot) -> Result<Option<Op>, Error>,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), Error> {
        let fuel_pos = self.stack.fuel_pos();
        let result = self
            .stack
            .push_temp(result_ty, Allocation::None)?
            .temp_slots()
            .head();
        if let Some(op) = make_op(result)? {
            self.instrs.stage_op(op, fuel_pos, fuel_costs)?;
        }
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
            Location::Reg(_) => Op::branch_table_r(len_targets),
            Location::Slot(index) => Op::branch_table_s(len_targets, index),
        };
        self.push_instr(op, FuelCostsProvider::base)?;
        // Encode the `br_table` targets:
        let fuel_pos = self.stack.fuel_pos();
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
        default_branch_params: BranchParams,
    ) -> Result<(), Error> {
        let len_targets = table.len() + 1;
        debug_assert_eq!(self.immediates.len(), len_targets as usize);
        let values = default_branch_params.temp_slots();
        let op = match index {
            Location::Reg(_) => Op::branch_table_span_r(len_targets, values),
            Location::Slot(index) => Op::branch_table_span_s(len_targets, index, values),
        };
        self.push_instr(op, FuelCostsProvider::base)?;
        // Encode the `br_table` targets:
        let fuel_pos = self.stack.fuel_pos();
        let targets = &self.immediates[..];
        for target in targets {
            let Ok(depth) = usize::try_from(u32::from(*target)) else {
                panic!("out of bounds `br_table` target does not fit `usize`: {target:?}");
            };
            let mut frame = self.stack.peek_control_mut(depth).control_frame();
            let results = frame.branch_params().temp_slots();
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

    /// Encodes a branching [`Op`].
    fn encode_branch_op(
        &mut self,
        dst: LabelRef,
        op: impl FnOnce(BranchOffset) -> Op,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<Pos<Op>, Error> {
        self.instrs.pad_to_op_alignment()?;
        let (pos, _) = self.instrs.encode_branch(dst, op, fuel_pos, 0)?;
        Ok(pos)
    }

    /// Returns [`Op::return_span`] if `returned` needs to be copied or otherwise [`Op::return`].
    fn make_return_span(returned: BoundedSlotSpan) -> Op {
        match returned.head() != Slot::from(0) {
            true => Op::return_span(returned),
            false => Op::r#return(),
        }
    }

    /// Encodes a generic return operator.
    fn encode_return(&mut self, fuel_pos: Option<Pos<ir::BlockFuel>>) -> Result<Pos<Op>, Error> {
        let len_results = self.func_type_with(FuncType::len_results);
        let instr = match len_results {
            0 => Op::Return {},
            1 => self.encode_return_value(fuel_pos)?,
            _ => {
                let len_results = usize::from(len_results);
                let results = self.move_operands_to_temp(len_results, fuel_pos)?;
                Self::make_return_span(results)
            }
        };
        let instr = self
            .instrs
            .encode_op(instr, fuel_pos, FuelCostsProvider::base)?;
        Ok(instr)
    }

    /// Encodes a return operator that returns a single value.
    fn encode_return_value(&mut self, fuel_pos: Option<Pos<ir::BlockFuel>>) -> Result<Op, Error> {
        let return_slot_for_ty = |ty: ValType, slot: Slot| match ty {
            ValType::V128 => {
                let returned = BoundedSlotSpan::new(SlotSpan::new(slot), 2);
                Self::make_return_span(returned)
            }
            _ => Op::return_u64_s(slot),
        };
        let value = self.stack.peek(0);
        let ty = value.ty();
        let op = match value.resolve(&self.layout)? {
            ResolvedOperand::Reg(ty) => match ty {
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
            ResolvedOperand::Slot(value) => return_slot_for_ty(ty, value),
            ResolvedOperand::Immediate(value) => match value.ty() {
                ValType::I32 => Op::return_u32_i(i32::from(value).to_bits()),
                ValType::I64 => Op::return_u64_i(i64::from(value).to_bits()),
                ValType::F32 => Op::return_u32_i(f32::from(value).to_bits()),
                ValType::F64 => Op::return_u64_i(f64::from(value).to_bits()),
                ValType::FuncRef | ValType::ExternRef => {
                    Op::return_u32_i(u32::from(RawRef::from(value.raw())))
                }
                ValType::V128 => {
                    let value = self.stack.peek(0);
                    let temp_slot = self.copy_operand_to_temp(value, fuel_pos)?;
                    let returned = BoundedSlotSpan::new(SlotSpan::new(temp_slot), 2);
                    Self::make_return_span(returned)
                }
            },
        };
        Ok(op)
    }

    /// Translates the end of a Wasm `block` control frame.
    fn translate_end_block(&mut self, frame: BlockControlFrame) -> Result<(), Error> {
        let fuel_pos = frame.fuel_pos();
        if frame.is_branched_to() {
            if self.reachable {
                self.copy_branch_params(frame.branch_params(), fuel_pos)?;
            }
            self.stack.push_branch_params(&frame)?;
        }
        self.instrs.pin_label(frame.label())?;
        self.reachable |= frame.is_branched_to();
        if self.reachable && self.stack.is_control_empty() {
            self.encode_return(fuel_pos)?;
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
        let fuel_pos = frame.fuel_pos();
        if is_end_of_then_reachable && has_results {
            self.copy_branch_params(frame.branch_params(), fuel_pos)?;
            self.encode_branch_op(frame.label(), Op::branch, fuel_pos)?;
        }
        self.instrs.pin_label_if_unpinned(else_label)?;
        self.stack.push_else_operands(&frame)?;
        if has_results {
            // We haven't visited the `else` block and thus the `else`
            // providers are still on the auxiliary stack and need to
            // be popped. We use them to restore the stack to the state
            // when entering the `if` block so that we can properly copy
            // the `else` results to were they are expected.
            let fuel_pos = self.instrs.encode_consume_fuel_op()?;
            self.copy_branch_params(frame.branch_params(), fuel_pos)?;
        }
        self.stack.push_branch_params(&frame)?;
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
        let fuel_pos = frame.fuel_pos();
        if end_of_else_reachable {
            self.copy_branch_params(frame.branch_params(), fuel_pos)?;
        }
        self.stack.push_branch_params(&frame)?;
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
        let fuel_pos = frame.fuel_pos();
        if frame.is_branched_to() {
            if end_is_reachable {
                self.copy_branch_params(frame.branch_params(), fuel_pos)?;
            }
            self.stack.push_branch_params(&frame)?;
        }
        self.instrs.pin_label(frame.label())?;
        self.reachable = end_is_reachable || frame.is_branched_to();
        Ok(())
    }

    /// Translates the end of an unreachable Wasm control frame.
    fn translate_end_unreachable(&mut self, _frame: ControlFrameKind) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        // We reset `last_instr` out of caution in case there is a control flow boundary.
        self.instrs.commit_staged_if_any()?;
        Ok(())
    }
}

/// What [`FuncTranslator::translate_local_set`] generated to translate the `local.set`.
#[derive(Debug, Copy, Clone)]
pub enum LocalSetCodegen {
    /// The `local.set` was fused with the staged [`Op`].
    Fused,
    /// The `local.set` was found to be a no-op, e.g. `(local.set $n (local.get $n))`.
    NoOp,
    /// The `local.set` generated a copy operator.
    Copy,
}

impl FuncTranslator {
    /// Translate the Wasm `local.set` and `local.tee` operations.
    ///
    /// # Note
    ///
    /// This applies op-code fusion that replaces the result of the previous instruction
    /// instead of encoding a copy instruction for the `local.set` or `local.tee` if possible.
    fn translate_local_set(
        &mut self,
        local_index: u32,
        input: Operand,
    ) -> Result<LocalSetCodegen, Error> {
        let local_idx = LocalIdx::from(local_index);
        let result = self.layout.local_to_slot(local_idx)?;
        if let Operand::Local(input) = input {
            let input = self.layout.local_to_slot(input)?;
            if result == input {
                // Case: `(local.set $n (local.get $n))` is a no-op so we can ignore it.
                return Ok(LocalSetCodegen::NoOp);
            }
        }
        let unfused_op = Self::select_copy_sx_op(result, input, &self.layout)?
            .expect("already filtered out no-op copies above");
        let fused_op = self.fused_local_set(result, input)?;
        let staged_fuel = match fused_op {
            Some(_) => {
                let (_, fuel_used) = self.instrs.drop_staged();
                Some(fuel_used)
            }
            None => None,
        };
        let ty = input.ty();
        let fuel_pos = self.stack.fuel_pos();
        for preserved in self.stack.preserve_locals(local_idx) {
            let ty = preserved.ty();
            let result = preserved.temp_slots().head();
            let op = match preserved.in_reg() {
                true => Self::select_copy_sr_op(result, ty)?,
                false => {
                    let value = self.layout.local_to_slot(preserved)?;
                    Self::select_copy_ss_op(result, value, ty)?
                        .expect("local preservation must not yield no-op copies")
                }
            };
            self.instrs
                .encode_op(op, fuel_pos, FuelCostsProvider::base)?;
        }
        if input.in_reg() {
            self.stack.register_local_for_reg(ty, local_idx)?;
        } else {
            self.stack.dealloc_local_for_reg(ty, local_idx)?;
        }
        let (outcome, op) = match fused_op {
            Some(fused_op) => (LocalSetCodegen::Fused, fused_op),
            None => (LocalSetCodegen::Copy, unfused_op),
        };
        let fuel_costs = |selector: &FuelCostsProvider| -> u64 {
            match staged_fuel {
                None => selector.base(),
                Some(fuel) => fuel,
            }
        };
        self.instrs.encode_op(op, fuel_pos, fuel_costs)?;
        Ok(outcome)
    }

    /// Returns `Some` if the staged [`Op`] can be fused with the `local.set` and `None` otherwise.
    ///
    /// # Note
    ///
    /// - The `local` argument reflects the local index and `input` is the `input` operand of the `local.set`.
    /// - This does not unstage or drop the staged [`Op`], nor does it encode the fused [`Op`].
    /// - This only returns the resulting fused [`Op`] if any for later consideration.
    fn fused_local_set(&self, result: Slot, input: Operand) -> Result<Option<Op>, Error> {
        if let ResolvedOperand::Reg(_) = self.resolve_operand::<TypedRawVal>(input)? {
            if let Some(fused_op) = self.fuse_copy_sr(result, input.ty()) {
                // The staged operator can be fused with a `copy_sr` operator.
                return Ok(Some(fused_op));
            }
        }
        let Some(mut staged) = self.instrs.peek_staged() else {
            // Cannot replace result if no staged operator exists.
            return Ok(None);
        };
        let Some(old_result) = staged.result_mut() else {
            // Cannot replace result of staged `Op` with non-slot result.
            return Ok(None);
        };
        if matches!(self.layout.stack_space(*old_result), StackSpace::Local) {
            // Cannot replace result of staged `Op` with a local result as its observable behavior.
            return Ok(None);
        }
        let ResolvedOperand::Slot(input) = self.resolve_operand::<TypedRawVal>(input)? else {
            // The `local.set` input is not a `Slot` and is not sourced from the staged `Op`.
            return Ok(None);
        };
        if *old_result != input {
            // The `local.set` input is not equal to staged `Op`'s output, thus the fusion is invalid.
            return Ok(None);
        }
        // All checks passed, now replace result and return new fused `Op`.
        *old_result = result;
        Ok(Some(staged))
    }

    /// Encodes an unconditional Wasm `branch` instruction.
    fn encode_br(&mut self, label: LabelRef) -> Result<(), Error> {
        let fuel_pos = self.stack.fuel_pos();
        self.encode_branch_op(label, Op::branch, fuel_pos)?;
        Ok(())
    }

    /// Encodes a `i32.eqz`+`br_if` or `if` conditional branch instruction.
    fn fused_br_eqz(&self, condition: Operand) -> Result<Option<Op>, Error> {
        self.fused_cmp_branch(condition, true)
    }

    /// Try to fuse a cmp+branch [`Op`] with optional negation.
    fn fused_cmp_branch(&self, condition: Operand, negate: bool) -> Result<Option<Op>, Error> {
        let Some(staged_op) = self.instrs.peek_staged() else {
            // Case: cannot fuse without a known last instruction
            return Ok(None);
        };
        let Some(ir::Location::Reg(result_ty)) = staged_op.result_loc() else {
            // Case: cannot fuse without register result.
            return Ok(None);
        };
        let ResolvedOperand::Reg(_ty) = condition.resolve(&self.layout)? else {
            // Case: cannot fuse non-register operands
            //  - locals have observable behavior.
            //  - immediates cannot be the result of a previous instruction.
            return Ok(None);
        };
        debug_assert!(
            matches!(result_ty, ValType::I32 | ValType::I64),
            "unexpected condition type: {result_ty:?}"
        );
        let cmp_op = match negate {
            false => staged_op,
            true => match staged_op.negate_cmp_instr() {
                Some(negated) => negated,
                None => {
                    // Note: cannot negate staged [`Op`], thus it is not a `cmp` operator and thus not fusable.
                    return Ok(None);
                }
            },
        };
        let fused_op_or_none = cmp_op.try_into_cmp_branch_instr(BranchOffset::uninit());
        Ok(fused_op_or_none)
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
        if let Some(fused_op) = self.fused_cmp_branch(condition, branch_eqz)? {
            let (fuel_pos, _) = self.instrs.drop_staged();
            self.encode_branch_op(
                label,
                |offset| fused_op.with_branch_offset(offset),
                fuel_pos,
            )?;
            return Ok(());
        }
        let condition = match self.resolve_operand::<i32>(condition)? {
            ResolvedOperand::Reg(ty) => Location::Reg(ty),
            ResolvedOperand::Slot(condition) => Location::Slot(condition),
            ResolvedOperand::Immediate(condition) => {
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
        let fuel_pos = self.stack.fuel_pos();
        self.encode_branch_op(
            label,
            |offset| match branch_eqz {
                true => match condition {
                    Location::Slot(condition) => Op::branch_i32_eq_si(offset, condition, 0),
                    Location::Reg(_) => Op::branch_i32_eq_ri(offset, 0),
                },
                false => match condition {
                    Location::Slot(condition) => Op::branch_i32_not_eq_si(offset, condition, 0),
                    Location::Reg(_) => Op::branch_i32_not_eq_ri(offset, 0),
                },
            },
            fuel_pos,
        )?;
        Ok(())
    }

    /// Generically translates a `call` or `return_call` Wasm operator.
    fn translate_call(
        &mut self,
        function_index: u32,
        call_internal: fn(params: BoundedSlotSpan, func: index::InternalFunc) -> Op,
        call_imported: fn(params: BoundedSlotSpan, func: index::Func) -> Op,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let func_idx = FuncIdx::from(function_index);
        let callee_ty = self.resolve_func_type(func_idx);
        let params = self.adjust_stack_for_call(&callee_ty)?;
        let instr = match self.module.get_engine_func(func_idx) {
            Some(engine_func) => {
                // Case: We are calling an internal function and can optimize
                //       this case by using the special instruction for it.
                let Some(func_entity) = self.engine().resolve_func(engine_func) else {
                    unreachable!("missing func entry at: {engine_func:?}")
                };
                call_internal(
                    params,
                    index::InternalFunc::from(func_entity as *const FuncEntry as usize),
                )
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
        let table = index::Table::from(table_index);
        let callee_ty = self.resolve_type(type_index);
        let index = self.copy_immediate_to_slot(index)?;
        let params = self.adjust_stack_for_call(&callee_ty)?;
        let func_type = index::FuncType::from(type_index);
        let op = match index {
            Location::Slot(index) => op_s(table, func_type, params, index),
            Location::Reg(_) => op_r(table, func_type, params),
        };
        self.push_instr(op, FuelCostsProvider::call)?;
        Ok(())
    }

    /// Adjusts the stack for a call to a function with type `ty`.
    ///
    /// Returns a bounded [`SlotSpan`] to the start of the call parameters and results
    /// with the length equal to the number of cells storing the call parameters.
    fn adjust_stack_for_call(&mut self, ty: &FuncType) -> Result<BoundedSlotSpan, Error> {
        let fuel_pos = self.stack.fuel_pos();
        self.preserve_regs(fuel_pos)?;
        let fuel_pos = self.stack.fuel_pos();
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
            self.stack.push_temp(*result, Allocation::None)?;
        }
        Ok(params)
    }

    fn copy_preserved_regs_to_slots(
        &mut self,
        regs: PreservedRegs,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        if !self.reachable {
            // No need to encode copies if unreachable.
            return Ok(());
        }
        if let Some(result) = regs.ireg {
            self.instrs
                .encode_op(Op::u64_copy_sr(result), fuel_pos, FuelCostsProvider::base)?;
        }
        if let Some(result) = regs.freg32 {
            self.instrs
                .encode_op(Op::f32_copy_sr(result), fuel_pos, FuelCostsProvider::base)?;
        }
        if let Some(result) = regs.freg64 {
            self.instrs
                .encode_op(Op::f64_copy_sr(result), fuel_pos, FuelCostsProvider::base)?;
        }
        Ok(())
    }

    /// Preserve all register operands on the [`Stack`].
    fn preserve_regs(&mut self, fuel_pos: Option<Pos<ir::BlockFuel>>) -> Result<(), Error> {
        let regs = self.stack.preserve_all_regs();
        self.copy_preserved_regs_to_slots(regs, fuel_pos)
    }

    /// Preserve all temporary register operands on the [`Stack`] but keep `local` register links.
    fn preserve_temp_regs(
        &mut self,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
        skip: usize,
    ) -> Result<(), Error> {
        let regs = self.stack.preserve_all_temp_regs(skip);
        self.copy_preserved_regs_to_slots(regs, fuel_pos)
    }

    /// Translates a unary Wasm instruction to Wasmi bytecode with a custom optimizer.
    fn translate_unary_with_opt<Op: UnaryOp>(
        &mut self,
        try_opt: fn(&mut FuncTranslator, value: Operand) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        Op::Value: From<TypedRawVal>,
        Op::Result: Into<TypedRawVal> + Typed,
    {
        bail_unreachable!(self);
        let input = self.stack.pop();
        if try_opt(self, input)? {
            // Case: custom optimization took effect, return early.
            return Ok(());
        }
        let op = match self.resolve_operand::<Op::Value>(input)? {
            ResolvedOperand::Reg(_) => Op::op_rr(),
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

    /// Translates a unary Wasm instruction to Wasmi bytecode.
    fn translate_unary<Op: UnaryOp>(&mut self) -> Result<(), Error>
    where
        Op::Value: From<TypedRawVal>,
        Op::Result: Into<TypedRawVal> + Typed,
    {
        self.translate_unary_with_opt::<Op>(|_, _| Ok(false))
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
        let result_ty = <R as Typed>::TY;
        match input {
            input if input.in_reg() => {
                self.push_op_with_result_reg(result_ty, op_rr(), FuelCostsProvider::base)?;
            }
            Operand::Local(input) => {
                self.stack.push_local(input.local_index(), result_ty)?;
            }
            Operand::Temp(_input) => {
                self.stack.push_temp(result_ty, Allocation::None)?;
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
        let value = self
            .resolve_operand::<RawVal>(operand)?
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

    /// Convenience method forwarding to [`Operand::resolve`].
    fn resolve_operand<T>(&self, operand: Operand) -> Result<ResolvedOperand<T>, Error>
    where
        T: From<TypedRawVal>,
    {
        operand.resolve_as::<T>(&self.layout)
    }

    /// Resolves the [`Operand`] into a [`ResolvedOperand<u64>`].
    ///
    /// See [`Self::resolve_operand`] for rational.
    fn resolve_operand_as_index(
        &self,
        operand: Operand,
        memory: Memory,
    ) -> Result<ResolvedOperand<u64>, Error> {
        let memidx: MemoryIdx = u32::from(memory).into();
        let operand = match self.module.get_type_of_memory(memidx).index_ty() {
            IndexType::I32 => self.resolve_operand::<u32>(operand)?.map(u64::from),
            IndexType::I64 => self.resolve_operand::<u64>(operand)?,
        };
        Ok(operand)
    }

    /// Issues a panic message for cases where an invalid operand pair was encountered.
    #[cold]
    #[inline]
    #[track_caller]
    fn unsupported_operand_pair(lhs: impl AsRef<Operand>, rhs: impl AsRef<Operand>) -> ! {
        #[inline(never)]
        #[track_caller]
        fn impl_(lhs: &Operand, rhs: &Operand) -> ! {
            unreachable!("unsupported operator pair:\n\t- lhs = {lhs:?}\n\t- rhs = {rhs:?}")
        }
        let lhs = lhs.as_ref();
        let rhs = rhs.as_ref();
        impl_(lhs, rhs)
    }

    /// Converts the `operand` into a [`Slot`] if possible.
    ///
    /// # Note
    ///
    /// This method shall be used if `lhs = rhs = register` is encountered to resolve
    /// `lhs` or `rhs` to `Slot` in order to encode a fallback operator in case the
    /// base operation does not provide one for this particular variant.
    fn reg_operand_to_slot(&self, operand: Operand) -> Result<Slot, Error> {
        let slot = match operand {
            Operand::Local(operand) => {
                debug_assert!(operand.in_reg());
                self.layout.local_to_slot(operand.local_index())?
            }
            _ => unreachable!(),
        };
        Ok(slot)
    }

    /// Translates a non-commutative binary Wasm operator to Wasmi bytecode.
    fn translate_binary_commutative<T: CommutativeBinaryOp>(&mut self) -> Result<(), Error>
    where
        T::Input: From<TypedRawVal> + Copy,
        T::Result: Into<TypedRawVal>,
    {
        self.translate_binary_commutative_with_opt::<T>(Self::no_opt_ri)
    }

    /// Translates a commutative binary Wasm operator to Wasmi bytecode.
    fn translate_binary_commutative_with_opt<T: CommutativeBinaryOp>(
        &mut self,
        opt_rhs_imm: fn(this: &mut Self, lhs: Operand, rhs: T::Input) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T::Input: From<TypedRawVal> + Copy,
        T::Result: Into<TypedRawVal>,
    {
        bail_unreachable!(self);
        let (lhs, rhs) = self.stack.pop2();
        let l = self.resolve_operand::<T::Input>(lhs)?;
        let r = self.resolve_operand::<T::Input>(rhs)?;
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
            (ResolvedOperand::Reg(_), ResolvedOperand::Reg(_)) => match T::op_rrr() {
                Some(op) => op,
                None => {
                    let rhs_slot = self.reg_operand_to_slot(rhs)?;
                    T::op_rrs(rhs_slot)
                }
            },
            (ResolvedOperand::Reg(_), ResolvedOperand::Slot(rhs)) => T::op_rrs(rhs),
            (ResolvedOperand::Reg(_), ResolvedOperand::Immediate(rhs)) => T::op_rri(rhs),
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
        T::Lhs: From<TypedRawVal> + Copy,
        T::Rhs: Copy,
        T::Result: Into<TypedRawVal>,
    {
        self.translate_binary_with_opt::<T>(Self::no_opt_ri)
    }

    /// Translates a non-commutative binary Wasm operator to Wasmi bytecode.
    fn translate_binary_with_opt<T: BinaryOp>(
        &mut self,
        opt_rhs_imm: fn(this: &mut Self, lhs: Operand, rhs: T::Rhs) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T::Lhs: From<TypedRawVal> + Copy,
        T::Rhs: Copy,
        T::Result: Into<TypedRawVal>,
    {
        bail_unreachable!(self);
        let (lhs, rhs) = self.stack.pop2();
        let l = self.resolve_operand::<T::Lhs>(lhs)?;
        let r = self.resolve_operand::<RawVal>(rhs)?;
        let r = match r {
            ResolvedOperand::Reg(ty) => ResolvedOperand::Reg(ty),
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
        if let (_, ResolvedOperand::Immediate(rhs)) = (l, r) {
            if opt_rhs_imm(self, lhs, rhs)? {
                return Ok(());
            }
        }
        let operator = match (l, r) {
            (ResolvedOperand::Reg(_), ResolvedOperand::Reg(_)) => match T::op_rrr() {
                Some(op) => op,
                None => {
                    let rhs_slot = self.reg_operand_to_slot(rhs)?;
                    T::op_rrs(rhs_slot)
                }
            },
            (ResolvedOperand::Reg(_), ResolvedOperand::Slot(rhs)) => T::op_rrs(rhs),
            (ResolvedOperand::Reg(_), ResolvedOperand::Immediate(rhs)) => T::op_rri(rhs),
            (ResolvedOperand::Slot(lhs), ResolvedOperand::Reg(_)) => T::op_rsr(lhs),
            (ResolvedOperand::Slot(lhs), ResolvedOperand::Slot(rhs)) => T::op_rss(lhs, rhs),
            (ResolvedOperand::Slot(lhs), ResolvedOperand::Immediate(rhs)) => T::op_rsi(lhs, rhs),
            (ResolvedOperand::Immediate(lhs), ResolvedOperand::Reg(_)) => T::op_rir(lhs),
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
        debug_assert_eq!(condition.ty(), ValType::I32);
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
        let condition = self.resolve_operand::<i32>(condition)?;
        let condition = match condition {
            ResolvedOperand::Reg(ty) => Location::Reg(ty),
            ResolvedOperand::Slot(condition) => Location::Slot(condition),
            ResolvedOperand::Immediate(condition) => {
                let selected = match condition != 0 {
                    true => true_val,
                    false => false_val,
                };
                match ty {
                    #[cfg(feature = "simd")]
                    ValType::V128 => {
                        // Note: this is a special case where we have to copy the `v128`
                        //       value that spans across 2 slots into the result slots of
                        //       the `select` operator.
                        let selected = self.resolve_operand::<V128>(selected)?;
                        self.try_push_op_with_result_slot(
                            ty,
                            |result| match selected {
                                ResolvedOperand::Reg(_) => unreachable!(),
                                ResolvedOperand::Slot(value) => {
                                    Self::select_copy_ss_op(result, value, ty)
                                }
                                ResolvedOperand::Immediate(value) => {
                                    Self::select_copy_si_op(result, value.into()).map(Some)
                                }
                            },
                            FuelCostsProvider::base,
                        )?;
                        return Ok(());
                    }
                    _ => self.copy_operand_to_reg(selected)?,
                }
                return Ok(());
            }
        };
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
        condition: Location,
        true_val: Operand,
        false_val: Operand,
    ) -> Result<Op, Error> {
        use Location as Loc;
        use ResolvedOperand as Opd;
        debug_assert!(matches!(
            true_val.ty(),
            ValType::I32 | ValType::ExternRef | ValType::FuncRef
        ));
        debug_assert!(matches!(
            false_val.ty(),
            ValType::I32 | ValType::ExternRef | ValType::FuncRef
        ));
        let true_val = self.resolve_operand::<RawVal>(true_val)?.map(u32::from);
        let false_val = self.resolve_operand::<RawVal>(false_val)?.map(u32::from);
        let operator = match (condition, true_val, false_val) {
            (Loc::Reg(_), Opd::Reg(_), Opd::Slot(f)) => Op::u64_select_rrrs(f),
            (Loc::Reg(_), Opd::Reg(_), Opd::Immediate(f)) => Op::u32_select_rrri(f),
            (Loc::Reg(_), Opd::Slot(t), Opd::Reg(_)) => Op::u64_select_rrsr(t),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Reg(_)) => Op::u32_select_rrir(t),
            (Loc::Reg(_), Opd::Slot(t), Opd::Slot(f)) => Op::u64_select_rrss(t, f),
            (Loc::Reg(_), Opd::Slot(t), Opd::Immediate(f)) => Op::u32_select_rrsi(t, f),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Slot(f)) => Op::u32_select_rris(t, f),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Immediate(f)) => Op::u32_select_rrii(t, f),
            (Loc::Slot(c), Opd::Reg(_), Opd::Slot(f)) => Op::u64_select_rsrs(c, f),
            (Loc::Slot(c), Opd::Reg(_), Opd::Immediate(f)) => Op::u32_select_rsri(c, f),
            (Loc::Slot(c), Opd::Slot(t), Opd::Reg(_)) => Op::u64_select_rssr(c, t),
            (Loc::Slot(c), Opd::Slot(t), Opd::Slot(f)) => Op::u64_select_rsss(c, t, f),
            (Loc::Slot(c), Opd::Slot(t), Opd::Immediate(f)) => Op::u32_select_rssi(c, t, f),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Reg(_)) => Op::u32_select_rsir(c, t),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Slot(f)) => Op::u32_select_rsis(c, t, f),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Immediate(f)) => Op::u32_select_rsii(c, t, f),
            _ => unreachable!(),
        };
        Ok(operator)
    }

    fn i64_select_operator(
        &mut self,
        condition: Location,
        true_val: Operand,
        false_val: Operand,
    ) -> Result<Op, Error> {
        use Location as Loc;
        use ResolvedOperand as Opd;
        let true_val = self.resolve_operand::<u64>(true_val)?;
        let false_val = self.resolve_operand::<u64>(false_val)?;
        let operator = match (condition, true_val, false_val) {
            (Loc::Reg(_), Opd::Reg(_), Opd::Slot(f)) => Op::u64_select_rrrs(f),
            (Loc::Reg(_), Opd::Reg(_), Opd::Immediate(f)) => Op::u64_select_rrri(f),
            (Loc::Reg(_), Opd::Slot(t), Opd::Reg(_)) => Op::u64_select_rrsr(t),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Reg(_)) => Op::u64_select_rrir(t),
            (Loc::Reg(_), Opd::Slot(t), Opd::Slot(f)) => Op::u64_select_rrss(t, f),
            (Loc::Reg(_), Opd::Slot(t), Opd::Immediate(f)) => Op::u64_select_rrsi(t, f),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Slot(f)) => Op::u64_select_rris(t, f),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Immediate(f)) => Op::u64_select_rrii(t, f),
            (Loc::Slot(c), Opd::Reg(_), Opd::Slot(f)) => Op::u64_select_rsrs(c, f),
            (Loc::Slot(c), Opd::Reg(_), Opd::Immediate(f)) => Op::u64_select_rsri(c, f),
            (Loc::Slot(c), Opd::Slot(t), Opd::Reg(_)) => Op::u64_select_rssr(c, t),
            (Loc::Slot(c), Opd::Slot(t), Opd::Slot(f)) => Op::u64_select_rsss(c, t, f),
            (Loc::Slot(c), Opd::Slot(t), Opd::Immediate(f)) => Op::u64_select_rssi(c, t, f),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Reg(_)) => Op::u64_select_rsir(c, t),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Slot(f)) => Op::u64_select_rsis(c, t, f),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Immediate(f)) => Op::u64_select_rsii(c, t, f),
            _ => unreachable!(),
        };
        Ok(operator)
    }

    fn f32_select_operator(
        &mut self,
        condition: Location,
        true_val: Operand,
        false_val: Operand,
    ) -> Result<Op, Error> {
        use Location as Loc;
        use ResolvedOperand as Opd;
        let true_val = self.resolve_operand::<f32>(true_val)?;
        let false_val = self.resolve_operand::<f32>(false_val)?;
        let operator = match (condition, true_val, false_val) {
            (Loc::Reg(_), Opd::Reg(_), Opd::Slot(f)) => Op::f32_select_rrrs(f),
            (Loc::Reg(_), Opd::Reg(_), Opd::Immediate(f)) => Op::f32_select_rrri(f),
            (Loc::Reg(_), Opd::Slot(t), Opd::Reg(_)) => Op::f32_select_rrsr(t),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Reg(_)) => Op::f32_select_rrir(t),
            (Loc::Reg(_), Opd::Slot(t), Opd::Slot(f)) => Op::f32_select_rrss(t, f),
            (Loc::Reg(_), Opd::Slot(t), Opd::Immediate(f)) => Op::f32_select_rrsi(t, f),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Slot(f)) => Op::f32_select_rris(t, f),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Immediate(f)) => Op::f32_select_rrii(t, f),
            (Loc::Slot(c), Opd::Reg(_), Opd::Slot(f)) => Op::f32_select_rsrs(c, f),
            (Loc::Slot(c), Opd::Reg(_), Opd::Immediate(f)) => Op::f32_select_rsri(c, f),
            (Loc::Slot(c), Opd::Slot(t), Opd::Reg(_)) => Op::f32_select_rssr(c, t),
            (Loc::Slot(c), Opd::Slot(t), Opd::Slot(f)) => Op::f32_select_rsss(c, t, f),
            (Loc::Slot(c), Opd::Slot(t), Opd::Immediate(f)) => Op::f32_select_rssi(c, t, f),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Reg(_)) => Op::f32_select_rsir(c, t),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Slot(f)) => Op::f32_select_rsis(c, t, f),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Immediate(f)) => Op::f32_select_rsii(c, t, f),
            _ => unreachable!(),
        };
        Ok(operator)
    }

    fn f64_select_operator(
        &mut self,
        condition: Location,
        true_val: Operand,
        false_val: Operand,
    ) -> Result<Op, Error> {
        use Location as Loc;
        use ResolvedOperand as Opd;
        let true_val = self.resolve_operand::<f64>(true_val)?;
        let false_val = self.resolve_operand::<f64>(false_val)?;
        let operator = match (condition, true_val, false_val) {
            (Loc::Reg(_), Opd::Reg(_), Opd::Slot(f)) => Op::f64_select_rrrs(f),
            (Loc::Reg(_), Opd::Reg(_), Opd::Immediate(f)) => Op::f64_select_rrri(f),
            (Loc::Reg(_), Opd::Slot(t), Opd::Reg(_)) => Op::f64_select_rrsr(t),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Reg(_)) => Op::f64_select_rrir(t),
            (Loc::Reg(_), Opd::Slot(t), Opd::Slot(f)) => Op::f64_select_rrss(t, f),
            (Loc::Reg(_), Opd::Slot(t), Opd::Immediate(f)) => Op::f64_select_rrsi(t, f),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Slot(f)) => Op::f64_select_rris(t, f),
            (Loc::Reg(_), Opd::Immediate(t), Opd::Immediate(f)) => Op::f64_select_rrii(t, f),
            (Loc::Slot(c), Opd::Reg(_), Opd::Slot(f)) => Op::f64_select_rsrs(c, f),
            (Loc::Slot(c), Opd::Reg(_), Opd::Immediate(f)) => Op::f64_select_rsri(c, f),
            (Loc::Slot(c), Opd::Slot(t), Opd::Reg(_)) => Op::f64_select_rssr(c, t),
            (Loc::Slot(c), Opd::Slot(t), Opd::Slot(f)) => Op::f64_select_rsss(c, t, f),
            (Loc::Slot(c), Opd::Slot(t), Opd::Immediate(f)) => Op::f64_select_rssi(c, t, f),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Reg(_)) => Op::f64_select_rsir(c, t),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Slot(f)) => Op::f64_select_rsis(c, t, f),
            (Loc::Slot(c), Opd::Immediate(t), Opd::Immediate(f)) => Op::f64_select_rsii(c, t, f),
            _ => unreachable!(),
        };
        Ok(operator)
    }

    #[cfg(feature = "simd")]
    fn encode_v128_select(
        &mut self,
        condition: Location,
        true_val: Operand,
        false_val: Operand,
    ) -> Result<(), Error> {
        let true_val = self.copy_operand_to_slot(true_val)?;
        let false_val = self.copy_operand_to_slot(false_val)?;
        self.push_op_with_result_slot(
            ValType::V128,
            |result| match condition {
                Location::Reg(_) => Op::v128_select_srss(result, true_val, false_val),
                Location::Slot(condition) => {
                    Op::v128_select_ssss(result, condition, true_val, false_val)
                }
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
    /// Tries to lower a staged `f32.abs` and a `f32.neg` operator into a `f32.nabs` operator.
    ///
    /// Returns `true` if lowering was successful.
    fn try_lower_f32_copysign(&mut self, value: Operand) -> Result<bool, Error> {
        let Some(staged_op) = self.instrs.peek_staged() else {
            // Case: no staged `Op` to lower
            return Ok(false);
        };
        let val = self.resolve_operand::<f32>(value)?;
        if matches!(val, ResolvedOperand::Reg(ValType::F32)) {
            // Case: input/output does not match with staged `Op`
            return Ok(false);
        }
        let lowered_op = match staged_op {
            Op::F32Abs_Rr { .. } => Op::f32_nabs_rr(),
            Op::F32Abs_Rs { value, .. } => Op::f32_nabs_rs(value),
            _ => return Ok(false),
        };
        self.instrs.replace_staged(lowered_op)?;
        Ok(true)
    }

    /// Tries to lower a staged `f64.abs` and a `f64.neg` operator into a `f64.nabs` operator.
    ///
    /// Returns `true` if lowering was successful.
    fn try_lower_f64_copysign(&mut self, value: Operand) -> Result<bool, Error> {
        let Some(staged_op) = self.instrs.peek_staged() else {
            // Case: no staged `Op` to lower
            return Ok(false);
        };
        let val = self.resolve_operand::<f64>(value)?;
        if matches!(val, ResolvedOperand::Reg(ValType::F64)) {
            // Case: input/output does not match with staged `Op`
            return Ok(false);
        }
        let lowered_op = match staged_op {
            Op::F64Abs_Rr { .. } => Op::f64_nabs_rr(),
            Op::F64Abs_Rs { value, .. } => Op::f64_nabs_rs(value),
            _ => return Ok(false),
        };
        self.instrs.replace_staged(lowered_op)?;
        Ok(true)
    }

    /// Tries to fuse a compare instruction with a Wasm `select` instruction.
    ///
    /// # Returns
    ///
    /// - Returns [`SelectFusion::Fused`] or [`SelectFusion::FusedSwap`] if fusion was successful.
    ///     - If [`SelectFusion::FusedSwap`] was returned, true and false operands need to be swapped.
    /// - Returns [`SelectFusion::None`] if fusion could not be applied.
    fn try_fuse_select(&self, condition: Location) -> Result<SelectFusion, Error> {
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
            (ir::Location::Reg(ValType::I64), Location::Reg(_)) => {}
            (ir::Location::Slot(staged), Location::Slot(condition)) if staged == condition => {}
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
        let ResolvedOperand::Reg(_ty) = lhs.resolve(&self.layout)? else {
            // Case: cannot fuse non-register operands
            //  - locals have observable behavior.
            //  - immediates cannot be the result of a previous instruction.
            return Ok(false);
        };
        if !matches!(staged.result_loc(), Some(ir::Location::Reg(_))) {
            // Case: staged operator has no result register.
            return Ok(false);
        }
        let Some(negated) = try_fuse(&staged) else {
            // Case: the `cmp` instruction cannot be negated
            return Ok(false);
        };
        // Need to push back `lhs` but with its type adjusted to be `i32`
        // since that's the return type of `i{32,64}.{eqz,eq,ne}`.
        self.stack.push_temp(ValType::I32, Allocation::Reg)?;
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
        let ptr = self.stack.pop();
        match self.select_load_op::<T>(ptr, memarg)? {
            Op::Trap { trap_code } => self.translate_trap(trap_code),
            op => {
                self.stage_op_with_result_reg(<T::Result as Typed>::TY, op, FuelCostsProvider::load)
            }
        }
    }

    /// Returns a Wasmi `load` operator for `memarg` and `ptr` if any.
    ///
    /// Returns [`Op::Trap`] if `ptr` is known to be out of bounds for the linear memory.
    ///
    /// # Note
    ///
    /// This chooses the right encoding for the given `load` instruction.
    /// If `ptr+offset` is a constant value the address is pre-calculated.
    fn select_load_op<T: op::LoadOp>(&mut self, ptr: Operand, memarg: MemArg) -> Result<Op, Error> {
        let (memory, offset) = Self::decode_memarg(memarg)?;
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
                ResolvedOperand::Reg(_) => T::op_rr_mem0_offset16(offset),
                ResolvedOperand::Slot(ptr) => T::op_rs_mem0_offset16(ptr, offset),
                ResolvedOperand::Immediate(_) => break 'opt,
            };
            return Ok(op);
        }
        // We need to encode a non-optimized fallback load operator.
        let Some(ptr) = ptr.filter_map(|ptr| self.effective_address(memory, ptr, offset)) else {
            return Ok(Op::trap(TrapCode::MemoryOutOfBounds));
        };
        let op = match ptr {
            ResolvedOperand::Reg(_) => T::op_rr(offset, memory),
            ResolvedOperand::Slot(ptr) => T::op_rs(ptr, offset, memory),
            ResolvedOperand::Immediate(address) => T::op_ri(address, memory),
        };
        Ok(op)
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
        T::Value: Copy + From<TypedRawVal>,
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
        T::Value: Copy + From<TypedRawVal>,
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
        let op = self.choose_store_op::<T>(memarg, ptr, value)?;
        self.push_instr(op, FuelCostsProvider::store)?;
        Ok(())
    }

    /// Selects which store operator to encode based on type `T`.
    fn choose_store_op<T: op::StoreOp>(
        &mut self,
        memarg: MemArg,
        ptr: ResolvedOperand<Address>,
        value: Operand,
    ) -> Result<Op, Error>
    where
        T::Value: Copy + From<TypedRawVal>,
        T::Immediate: Copy,
    {
        use ResolvedOperand as Opd;
        let (memory, offset) = Self::decode_memarg(memarg)?;
        if let Some(op) = self.choose_store_mem0_offset16_op::<T>(ptr, offset, memory, value)? {
            return Ok(op);
        }
        let value = self
            .resolve_operand::<T::Value>(value)?
            .map(T::into_immediate);
        let op = match (ptr, value) {
            (Opd::Reg(_), Opd::Reg(_)) => match T::store_rr(offset, memory) {
                Some(op) => op,
                None => unreachable!(),
            },
            (Opd::Reg(_), Opd::Slot(value)) => T::store_rs(offset, value, memory),
            (Opd::Reg(_), Opd::Immediate(value)) => T::store_ri(offset, value, memory),
            (Opd::Slot(ptr), Opd::Reg(_)) => T::store_sr(ptr, offset, memory),
            (Opd::Slot(ptr), Opd::Slot(value)) => T::store_ss(ptr, offset, value, memory),
            (Opd::Slot(ptr), Opd::Immediate(value)) => T::store_si(ptr, offset, value, memory),
            (Opd::Immediate(address), Opd::Reg(_)) => T::store_ir(address, memory),
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
        value: Operand,
    ) -> Result<Option<Op>, Error>
    where
        T::Value: Copy + From<TypedRawVal>,
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
            Opd::Reg(ty) => Loc::Reg(ty),
            Opd::Slot(ptr) => Loc::Slot(ptr),
            Opd::Immediate(_) => return Ok(None),
        };
        let resolved_value = self
            .resolve_operand::<T::Value>(value)?
            .map(T::into_immediate);
        let op = match (ptr, resolved_value) {
            (Loc::Reg(_), Opd::Reg(_)) => match T::store_mem0_offset16_rr(offset) {
                Some(op) => op,
                None => {
                    let value = self.reg_operand_to_slot(value)?;
                    T::store_mem0_offset16_rs(offset, value)
                }
            },
            (Loc::Reg(_), Opd::Slot(value)) => T::store_mem0_offset16_rs(offset, value),
            (Loc::Reg(_), Opd::Immediate(value)) => T::store_mem0_offset16_ri(offset, value),
            (Loc::Slot(ptr), Opd::Reg(_)) => T::store_mem0_offset16_sr(ptr, offset),
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
        let result_lo = self
            .stack
            .push_temp(ValType::I64, Allocation::None)?
            .temp_slots()
            .head();
        let result_hi = self
            .stack
            .push_temp(ValType::I64, Allocation::None)?
            .temp_slots()
            .head();
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
        let result0 = self
            .stack
            .push_temp(ValType::I64, Allocation::None)?
            .temp_slots()
            .head();
        self.stack.push_temp(ValType::I64, Allocation::None)?;
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
                let fuel_pos = self.stack.fuel_pos();
                let result = result.temp_slots().head();
                self.encode_copy_sx_op(result, lhs, fuel_pos)?;
            }
            self.stack.push_immediate(0_i64)?; // hi-bits
            return Ok(true);
        }
        Ok(false)
    }
}
