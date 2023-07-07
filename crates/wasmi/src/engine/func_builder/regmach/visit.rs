use super::{
    bail_unreachable,
    control_frame::{BlockControlFrame, BlockHeight, ControlFrame, UnreachableControlFrame},
    stack::TypedProvider,
    ControlFrameKind,
    FuncTranslator,
    Typed,
    TypedValue,
};
use crate::{
    engine::{
        bytecode,
        bytecode2,
        bytecode2::{BinInstr, BinInstrImm16, Const16, Const32, Instruction, Register, UnaryInstr},
        TranslationError,
    },
    module::{self, BlockType, WasmiValueType},
    Mutability,
};
use wasmi_core::{TrapCode, UntypedValue, ValueType, F32, F64};
use wasmparser::VisitOperator;

/// Used to swap operands of a `rev` variant [`Instruction`] constructor.
macro_rules! swap_ops {
    ($fn_name:path) => {
        |result: Register, lhs: Const16, rhs: Register| -> Instruction {
            $fn_name(result, rhs, lhs)
        }
    };
}

macro_rules! impl_visit_operator {
    ( @mvp $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @sign_extension $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @saturating_float_to_int $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @bulk_memory $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @reference_types $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @tail_call $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @@skipped $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // We skip Wasm operators that we already implement manually.
        impl_visit_operator!($($rest)*);
    };
    ( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // Wildcard match arm for all the other (yet) unsupported Wasm proposals.
        fn $visit(&mut self $($(, $arg: $argty)*)?) -> Self::Output {
            self.unsupported_operator(stringify!($op))
        }
        impl_visit_operator!($($rest)*);
    };
    () => {};
}

impl FuncTranslator<'_> {
    /// Called when translating an unsupported Wasm operator.
    ///
    /// # Note
    ///
    /// We panic instead of returning an error because unsupported Wasm
    /// errors should have been filtered out by the validation procedure
    /// already, therefore encountering an unsupported Wasm operator
    /// in the function translation procedure can be considered a bug.
    fn unsupported_operator(&self, name: &str) -> Result<(), TranslationError> {
        panic!("tried to translate an unsupported Wasm operator: {name}")
    }
}

impl<'a> VisitOperator<'a> for FuncTranslator<'a> {
    type Output = Result<(), TranslationError>;

    wasmparser::for_each_operator!(impl_visit_operator);

    fn visit_unreachable(&mut self) -> Self::Output {
        bail_unreachable!(self);
        self.alloc
            .instr_encoder
            .push_instr(Instruction::Trap(TrapCode::UnreachableCodeReached))?;
        self.reachable = false;
        Ok(())
    }

    fn visit_nop(&mut self) -> Self::Output {
        // Nothing to do for Wasm `nop` instructions.
        Ok(())
    }

    fn visit_block(&mut self, block_type: wasmparser::BlockType) -> Self::Output {
        let block_type = BlockType::new(block_type, self.res);
        if self.is_reachable() {
            // Inherit [`Instruction::ConsumeFuel`] from parent control frame.
            //
            // # Note
            //
            // This is an optimization to reduce the number of [`Instruction::ConsumeFuel`]
            // and is applicable since Wasm `block` are entered unconditionally.
            let consume_fuel = self.alloc.control_stack.last().consume_fuel_instr();
            let stack_height =
                BlockHeight::new(self.engine(), self.alloc.stack.height(), block_type)?;
            let end_label = self.alloc.instr_encoder.new_label();
            let len_block_params = block_type.len_params(self.engine()) as usize;
            let len_branch_params = block_type.len_results(self.engine()) as usize;
            let branch_params = self.alloc_branch_params(len_block_params, len_branch_params)?;
            self.alloc.control_stack.push_frame(BlockControlFrame::new(
                block_type,
                end_label,
                branch_params,
                stack_height,
                consume_fuel,
            ));
        } else {
            self.alloc
                .control_stack
                .push_frame(UnreachableControlFrame::new(
                    ControlFrameKind::Block,
                    block_type,
                ));
        }
        Ok(())
    }

    fn visit_loop(&mut self, _blockty: wasmparser::BlockType) -> Self::Output {
        todo!()
    }

    fn visit_if(&mut self, _blockty: wasmparser::BlockType) -> Self::Output {
        todo!()
    }

    fn visit_else(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_end(&mut self) -> Self::Output {
        match self.alloc.control_stack.pop_frame() {
            ControlFrame::Block(frame) => self.translate_end_block(frame),
            ControlFrame::Loop(frame) => self.translate_end_loop(frame),
            ControlFrame::If(frame) => self.translate_end_if(frame),
            ControlFrame::Unreachable(frame) => self.translate_end_unreachable(frame),
        }
    }

    fn visit_br(&mut self, relative_depth: u32) -> Self::Output {
        todo!()
    }

    fn visit_br_if(&mut self, _relative_depth: u32) -> Self::Output {
        todo!()
    }

    fn visit_br_table(&mut self, _targets: wasmparser::BrTable<'a>) -> Self::Output {
        todo!()
    }

    fn visit_return(&mut self) -> Self::Output {
        bail_unreachable!(self);
        let instr = match self.func_type().results() {
            [] => {
                // Case: Function returns nothing therefore all return statements must return nothing.
                Instruction::Return
            }
            [ValueType::I32] => match self.alloc.stack.pop() {
                // Case: Function returns a single `i32` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_reg(value),
                TypedProvider::Const(value) => Instruction::ReturnImm32 {
                    value: Const32::from_i32(i32::from(value)),
                },
            },
            [ValueType::I64] => match self.alloc.stack.pop() {
                // Case: Function returns a single `i64` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_reg(value),
                TypedProvider::Const(value) => {
                    if let Ok(value) = i32::try_from(i64::from(value)) {
                        Instruction::return_i64imm32(value)
                    } else {
                        Instruction::ReturnImm {
                            value: self.engine().alloc_const(value)?,
                        }
                    }
                }
            },
            [ValueType::F32] => match self.alloc.stack.pop() {
                // Case: Function returns a single `f32` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_reg(value),
                TypedProvider::Const(value) => Instruction::ReturnImm32 {
                    value: Const32::from_f32(F32::from(value)),
                },
            },
            [ValueType::F64 | ValueType::FuncRef | ValueType::ExternRef] => {
                match self.alloc.stack.pop() {
                    // Case: Function returns a single `f64` value which allows for special operator.
                    TypedProvider::Register(value) => Instruction::return_reg(value),
                    TypedProvider::Const(value) => Instruction::ReturnImm {
                        value: self.engine().alloc_const(value)?,
                    },
                }
            }
            _ => todo!(),
        };
        self.alloc.instr_encoder.push_instr(instr)?;
        self.reachable = false;
        Ok(())
    }

    fn visit_call(&mut self, _function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_call_indirect(
        &mut self,
        _type_index: u32,
        _table_index: u32,
        _table_byte: u8,
    ) -> Self::Output {
        todo!()
    }

    fn visit_return_call(&mut self, _function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_return_call_indirect(&mut self, _type_index: u32, _table_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_drop(&mut self) -> Self::Output {
        bail_unreachable!(self);
        self.alloc.stack.pop();
        Ok(())
    }

    fn visit_select(&mut self) -> Self::Output {
        self.translate_select(None)
    }

    fn visit_typed_select(&mut self, ty: wasmparser::ValType) -> Self::Output {
        let type_hint = WasmiValueType::from(ty).into_inner();
        self.translate_select(Some(type_hint))
    }

    fn visit_local_get(&mut self, local_index: u32) -> Self::Output {
        self.alloc.stack.push_local(local_index)?;
        Ok(())
    }

    fn visit_local_set(&mut self, _local_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_local_tee(&mut self, _local_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_global_get(&mut self, global_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let global_idx = module::GlobalIdx::from(global_index);
        let (global_type, init_value) = self.res.get_global(global_idx);
        let content = global_type.content();
        if let (Mutability::Const, Some(init_expr)) = (global_type.mutability(), init_value) {
            if let Some(value) = init_expr.eval_const() {
                // Optmization: Access to immutable internally defined global variables
                //              can be replaced with their constant initialization value.
                self.alloc.stack.push_const(TypedValue::new(content, value));
                return Ok(());
            }
            if let Some(func_index) = init_expr.funcref() {
                // Optimization: Forward to `ref.func x` translation.
                self.visit_ref_func(func_index.into_u32())?;
                return Ok(());
            }
        }
        // Case: The `global.get` instruction accesses a mutable or imported
        //       global variable and thus cannot be optimized away.
        let global_idx = bytecode::GlobalIdx::from(global_index);
        let result = self.alloc.stack.push_dynamic()?;
        self.alloc
            .instr_encoder
            .push_instr(Instruction::global_get(result, global_idx))?;
        Ok(())
    }

    fn visit_global_set(&mut self, global_index: u32) -> Self::Output {
        bail_unreachable!(self);
        let global = bytecode::GlobalIdx::from(global_index);
        match self.alloc.stack.pop() {
            TypedProvider::Register(input) => {
                self.alloc
                    .instr_encoder
                    .push_instr(Instruction::global_set(global, input))?;
                Ok(())
            }
            TypedProvider::Const(input) => {
                let (global_type, _init_value) =
                    self.res.get_global(module::GlobalIdx::from(global_index));
                match global_type.content() {
                    ValueType::I32 => {
                        self.alloc
                            .instr_encoder
                            .push_instr(Instruction::global_set_imm32(global))?;
                        self.alloc
                            .instr_encoder
                            .push_instr(Instruction::const32(i32::from(input)))?;
                        Ok(())
                    }
                    ValueType::F32 => {
                        self.alloc
                            .instr_encoder
                            .push_instr(Instruction::global_set_imm32(global))?;
                        self.alloc
                            .instr_encoder
                            .push_instr(Instruction::const32(f32::from(input)))?;
                        Ok(())
                    }
                    ValueType::I64 | ValueType::F64 | ValueType::FuncRef | ValueType::ExternRef => {
                        let cref = self.engine().alloc_const(input)?;
                        self.alloc
                            .instr_encoder
                            .push_instr(Instruction::global_set_imm(global))?;
                        self.alloc
                            .instr_encoder
                            .push_instr(Instruction::const_ref(cref))?;
                        Ok(())
                    }
                }
            }
        }
    }

    fn visit_i32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i32_load,
            Instruction::i32_load_offset16,
            Instruction::i32_load_at,
        )
    }

    fn visit_i64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load,
            Instruction::i64_load_offset16,
            Instruction::i64_load_at,
        )
    }

    fn visit_f32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::f32_load,
            Instruction::f32_load_offset16,
            Instruction::f32_load_at,
        )
    }

    fn visit_f64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::f64_load,
            Instruction::f64_load_offset16,
            Instruction::f64_load_at,
        )
    }

    fn visit_i32_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i32_load8_s,
            Instruction::i32_load8_s_offset16,
            Instruction::i32_load8_s_at,
        )
    }

    fn visit_i32_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i32_load8_u,
            Instruction::i32_load8_u_offset16,
            Instruction::i32_load8_u_at,
        )
    }

    fn visit_i32_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i32_load16_s,
            Instruction::i32_load16_s_offset16,
            Instruction::i32_load16_s_at,
        )
    }

    fn visit_i32_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i32_load16_u,
            Instruction::i32_load16_u_offset16,
            Instruction::i32_load16_u_at,
        )
    }

    fn visit_i64_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load8_s,
            Instruction::i64_load8_s_offset16,
            Instruction::i64_load8_s_at,
        )
    }

    fn visit_i64_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load8_u,
            Instruction::i64_load8_u_offset16,
            Instruction::i64_load8_u_at,
        )
    }

    fn visit_i64_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load16_s,
            Instruction::i64_load16_s_offset16,
            Instruction::i64_load16_s_at,
        )
    }

    fn visit_i64_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load16_u,
            Instruction::i64_load16_u_offset16,
            Instruction::i64_load16_u_at,
        )
    }

    fn visit_i64_load32_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load32_s,
            Instruction::i64_load32_s_offset16,
            Instruction::i64_load32_s_at,
        )
    }

    fn visit_i64_load32_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::i64_load32_u,
            Instruction::i64_load32_u_offset16,
            Instruction::i64_load32_u_at,
        )
    }

    fn visit_i32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<i32>(
            memarg,
            Instruction::i32_store,
            Instruction::i32_store_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_store_at,
            Instruction::i32_store_imm_at,
        )
    }

    fn visit_i64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<i64>(
            memarg,
            Instruction::i64_store,
            Instruction::i64_store_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_store_at,
            Instruction::i64_store_imm_at,
        )
    }

    fn visit_f32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<f32>(
            memarg,
            Instruction::f32_store,
            Instruction::f32_store_imm,
            Self::make_instr_imm_param_32,
            Instruction::f32_store_at,
            Instruction::f32_store_imm_at,
        )
    }

    fn visit_f64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<f64>(
            memarg,
            Instruction::f64_store,
            Instruction::f64_store_imm,
            Self::make_instr_imm_param_64,
            Instruction::f64_store_at,
            Instruction::f64_store_imm_at,
        )
    }

    fn visit_i32_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store_trunc::<i32>(
            memarg,
            Instruction::i32_store8,
            Instruction::i32_store8_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_store8_at,
            |address, value| Instruction::i32_store8_imm_at(address, value as i8),
        )
    }

    fn visit_i32_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store_trunc::<i32>(
            memarg,
            Instruction::i32_store16,
            Instruction::i32_store16_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_store16_at,
            |address, value| Instruction::i32_store16_imm_at(address, value as i16),
        )
    }

    fn visit_i64_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store_trunc::<i64>(
            memarg,
            Instruction::i64_store8,
            Instruction::i64_store8_imm,
            |_this, value| Ok(Instruction::const32(value as i8)),
            Instruction::i64_store8_at,
            |address, value| Instruction::i64_store8_imm_at(address, value as i8),
        )
    }

    fn visit_i64_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store_trunc::<i64>(
            memarg,
            Instruction::i64_store16,
            Instruction::i64_store16_imm,
            |_this, value| Ok(Instruction::const32(value as i16)),
            Instruction::i64_store16_at,
            |address, value| Instruction::i64_store16_imm_at(address, value as i16),
        )
    }

    fn visit_i64_store32(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        self.translate_store::<i64>(
            memarg,
            Instruction::i64_store32,
            Instruction::i64_store32_imm,
            |_this, value| Ok(Instruction::const32(value as i32)),
            Instruction::i64_store32_at,
            Instruction::i64_store32_imm_at,
        )
    }

    fn visit_memory_size(&mut self, _mem: u32, _mem_byte: u8) -> Self::Output {
        todo!()
    }

    fn visit_memory_grow(&mut self, _mem: u32, _mem_byte: u8) -> Self::Output {
        todo!()
    }

    fn visit_i32_const(&mut self, value: i32) -> Self::Output {
        self.alloc.stack.push_const(value);
        Ok(())
    }

    fn visit_i64_const(&mut self, value: i64) -> Self::Output {
        self.alloc.stack.push_const(value);
        Ok(())
    }

    fn visit_f32_const(&mut self, value: wasmparser::Ieee32) -> Self::Output {
        self.alloc.stack.push_const(F32::from_bits(value.bits()));
        Ok(())
    }

    fn visit_f64_const(&mut self, value: wasmparser::Ieee64) -> Self::Output {
        self.alloc.stack.push_const(F64::from_bits(value.bits()));
        Ok(())
    }

    fn visit_ref_null(&mut self, _ty: wasmparser::ValType) -> Self::Output {
        todo!()
    }

    fn visit_ref_is_null(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_ref_func(&mut self, _function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_eqz(&mut self) -> Self::Output {
        bail_unreachable!(self);
        // Push a zero on the value stack so we can translate `i32.eqz` as `i32.eq(x, 0)`.
        self.alloc.stack.push_const(0_i32);
        self.visit_i32_eq()
    }

    fn visit_i32_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32>(
            Instruction::i32_eq,
            Instruction::i32_eq_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_eq_imm16,
            TypedValue::i32_eq,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x == x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i32_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32>(
            Instruction::i32_ne,
            Instruction::i32_ne_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_ne_imm16,
            TypedValue::i32_ne,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x != x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i32_lt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_lt_s,
            Instruction::i32_lt_s_imm,
            Instruction::i32_gt_s_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_lt_s_imm16,
            swap_ops!(Instruction::i32_gt_s_imm16),
            TypedValue::i32_lt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i32| {
                if rhs == i32::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, _rhs: Register| {
                if lhs == i32::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_lt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_lt_u,
            Instruction::i32_lt_u_imm,
            Instruction::i32_gt_u_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_lt_u_imm16,
            swap_ops!(Instruction::i32_gt_u_imm16),
            TypedValue::i32_lt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u32| {
                if rhs == u32::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, _rhs: Register| {
                if lhs == u32::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_gt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_gt_s,
            Instruction::i32_gt_s_imm,
            Instruction::i32_lt_s_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_gt_s_imm16,
            swap_ops!(Instruction::i32_lt_s_imm16),
            TypedValue::i32_gt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i32| {
                if rhs == i32::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, _rhs: Register| {
                if lhs == i32::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_gt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_gt_u,
            Instruction::i32_gt_u_imm,
            Instruction::i32_lt_u_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_gt_u_imm16,
            swap_ops!(Instruction::i32_lt_u_imm16),
            TypedValue::i32_gt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u32| {
                if rhs == u32::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, _rhs: Register| {
                if lhs == u32::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_le_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_le_s,
            Instruction::i32_le_s_imm,
            Instruction::i32_ge_s_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_le_s_imm16,
            swap_ops!(Instruction::i32_ge_s_imm16),
            TypedValue::i32_le_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i32| {
                if rhs == i32::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, _rhs: Register| {
                if lhs == i32::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_le_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_le_u,
            Instruction::i32_le_u_imm,
            Instruction::i32_ge_u_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_le_u_imm16,
            swap_ops!(Instruction::i32_ge_u_imm16),
            TypedValue::i32_le_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u32| {
                if rhs == u32::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, _rhs: Register| {
                if lhs == u32::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_ge_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_ge_s,
            Instruction::i32_ge_s_imm,
            Instruction::i32_le_s_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_ge_s_imm16,
            swap_ops!(Instruction::i32_le_s_imm16),
            TypedValue::i32_ge_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i32| {
                if rhs == i32::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, _rhs: Register| {
                if lhs == i32::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_ge_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_ge_u,
            Instruction::i32_ge_u_imm,
            Instruction::i32_le_u_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_ge_u_imm16,
            swap_ops!(Instruction::i32_le_u_imm16),
            TypedValue::i32_ge_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u32| {
                if rhs == u32::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, _rhs: Register| {
                if lhs == u32::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_eqz(&mut self) -> Self::Output {
        bail_unreachable!(self);
        // Push a zero on the value stack so we can translate `i64.eqz` as `i64.eq(x, 0)`.
        self.alloc.stack.push_const(0_i64);
        self.visit_i64_eq()
    }

    fn visit_i64_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64>(
            Instruction::i64_eq,
            Instruction::i64_eq_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_eq_imm16,
            TypedValue::i64_eq,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x == x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i64_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64>(
            Instruction::i64_ne,
            Instruction::i64_ne_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_ne_imm16,
            TypedValue::i64_ne,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x != x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i64_lt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_lt_s,
            Instruction::i64_lt_s_imm,
            Instruction::i64_gt_s_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_lt_s_imm16,
            swap_ops!(Instruction::i64_gt_s_imm16),
            TypedValue::i64_lt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i64| {
                if rhs == i64::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, _rhs: Register| {
                if lhs == i64::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_lt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_lt_u,
            Instruction::i64_lt_u_imm,
            Instruction::i64_gt_u_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_lt_u_imm16,
            swap_ops!(Instruction::i64_gt_u_imm16),
            TypedValue::i64_lt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u64| {
                if rhs == u64::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, _rhs: Register| {
                if lhs == u64::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_gt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_gt_s,
            Instruction::i64_gt_s_imm,
            Instruction::i64_lt_s_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_gt_s_imm16,
            swap_ops!(Instruction::i64_lt_s_imm16),
            TypedValue::i64_gt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i64| {
                if rhs == i64::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, _rhs: Register| {
                if lhs == i64::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_gt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_gt_u,
            Instruction::i64_gt_u_imm,
            Instruction::i64_lt_u_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_gt_u_imm16,
            swap_ops!(Instruction::i64_lt_u_imm16),
            TypedValue::i64_gt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u64| {
                if rhs == u64::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, _rhs: Register| {
                if lhs == u64::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_le_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_le_s,
            Instruction::i64_le_s_imm,
            Instruction::i64_ge_s_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_le_s_imm16,
            swap_ops!(Instruction::i64_ge_s_imm16),
            TypedValue::i64_le_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i64| {
                if rhs == i64::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, _rhs: Register| {
                if lhs == i64::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_le_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_le_u,
            Instruction::i64_le_u_imm,
            Instruction::i64_ge_u_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_le_u_imm16,
            swap_ops!(Instruction::i64_ge_u_imm16),
            TypedValue::i64_le_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u64| {
                if rhs == u64::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, _rhs: Register| {
                if lhs == u64::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_ge_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_ge_s,
            Instruction::i64_ge_s_imm,
            Instruction::i64_le_s_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_ge_s_imm16,
            swap_ops!(Instruction::i64_le_s_imm16),
            TypedValue::i64_ge_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i64| {
                if rhs == i64::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, _rhs: Register| {
                if lhs == i64::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_ge_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_ge_u,
            Instruction::i64_ge_u_imm,
            Instruction::i64_le_u_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_ge_u_imm16,
            swap_ops!(Instruction::i64_le_u_imm16),
            TypedValue::i64_ge_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: u64| {
                if rhs == u64::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, _rhs: Register| {
                if lhs == u64::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_eq(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f32>(
            Instruction::f32_eq,
            Instruction::f32_eq_imm,
            Self::make_instr_imm_param_32,
            TypedValue::f32_eq,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x == x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _reg_in: Register, imm_in: f32| {
                if imm_in.is_nan() {
                    // Optimization: `NaN == x` or `x == NaN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_ne(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f32>(
            Instruction::f32_ne,
            Instruction::f32_ne_imm,
            Self::make_instr_imm_param_32,
            TypedValue::f32_ne,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x != x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _reg_in: Register, imm_in: f32| {
                if imm_in.is_nan() {
                    // Optimization: `NaN == x` or `x == NaN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_lt(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f32_lt,
            Instruction::f32_lt_imm,
            Instruction::f32_gt_imm,
            Self::make_instr_imm_param_32,
            TypedValue::f32_lt,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f32| {
                if rhs.is_nan() {
                    // Optimization: `x < NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if rhs.is_infinite() && rhs.is_sign_negative() {
                    // Optimization: `x < -INF` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f32, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if lhs.is_infinite() && lhs.is_sign_positive() {
                    // Optimization: `+INF < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_gt(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f32_gt,
            Instruction::f32_gt_imm,
            Instruction::f32_lt_imm,
            Self::make_instr_imm_param_32,
            TypedValue::f32_gt,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f32| {
                if rhs.is_nan() {
                    // Optimization: `x > NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if rhs.is_infinite() && rhs.is_sign_positive() {
                    // Optimization: `x > INF` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f32, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if lhs.is_infinite() && lhs.is_sign_negative() {
                    // Optimization: `-INF > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_le(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f32_le,
            Instruction::f32_le_imm,
            Instruction::f32_ge_imm,
            Self::make_instr_imm_param_32,
            TypedValue::f32_le,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f32| {
                if rhs.is_nan() {
                    // Optimization: `x <= NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f32, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN <= x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_ge(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f32_ge,
            Instruction::f32_ge_imm,
            Instruction::f32_le_imm,
            Self::make_instr_imm_param_32,
            TypedValue::f32_ge,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f32| {
                if rhs.is_nan() {
                    // Optimization: `x >= NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f32, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN >= x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_eq(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f64>(
            Instruction::f64_eq,
            Instruction::f64_eq_imm,
            Self::make_instr_imm_param_64,
            TypedValue::f64_eq,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x == x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _reg_in: Register, imm_in: f64| {
                if imm_in.is_nan() {
                    // Optimization: `NaN == x` or `x == NaN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_ne(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f64>(
            Instruction::f64_ne,
            Instruction::f64_ne_imm,
            Self::make_instr_imm_param_64,
            TypedValue::f64_ne,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x != x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _reg_in: Register, imm_in: f64| {
                if imm_in.is_nan() {
                    // Optimization: `NaN == x` or `x == NaN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_lt(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f64_lt,
            Instruction::f64_lt_imm,
            Instruction::f64_gt_imm,
            Self::make_instr_imm_param_64,
            TypedValue::f64_lt,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f64| {
                if rhs.is_nan() {
                    // Optimization: `x < NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if rhs.is_infinite() && rhs.is_sign_negative() {
                    // Optimization: `x < -INF` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f64, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if lhs.is_infinite() && lhs.is_sign_positive() {
                    // Optimization: `+INF < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_gt(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f64_gt,
            Instruction::f64_gt_imm,
            Instruction::f64_lt_imm,
            Self::make_instr_imm_param_64,
            TypedValue::f64_gt,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f64| {
                if rhs.is_nan() {
                    // Optimization: `x > NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if rhs.is_infinite() && rhs.is_sign_positive() {
                    // Optimization: `x > INF` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f64, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                if lhs.is_infinite() && lhs.is_sign_negative() {
                    // Optimization: `-INF > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_le(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f64_le,
            Instruction::f64_le_imm,
            Instruction::f64_ge_imm,
            Self::make_instr_imm_param_64,
            TypedValue::f64_le,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f64| {
                if rhs.is_nan() {
                    // Optimization: `x <= NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f64, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN <= x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_ge(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f64_ge,
            Instruction::f64_ge_imm,
            Instruction::f64_le_imm,
            Self::make_instr_imm_param_64,
            TypedValue::f64_ge,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: f64| {
                if rhs.is_nan() {
                    // Optimization: `x >= NAN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: f64, _rhs: Register| {
                if lhs.is_nan() {
                    // Optimization: `NAN >= x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_clz(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_clz, TypedValue::i32_clz)
    }

    fn visit_i32_ctz(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_ctz, TypedValue::i32_ctz)
    }

    fn visit_i32_popcnt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_popcnt, TypedValue::i32_popcnt)
    }

    fn visit_i32_add(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_add,
            Instruction::i32_add_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_add_imm16,
            TypedValue::i32_add,
            Self::no_custom_opt,
            |this, reg: Register, value: i32| {
                if value == 0 {
                    // Optimization: `add x + 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_sub(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_sub,
            Instruction::i32_sub_imm,
            Instruction::i32_sub_imm_rev,
            Self::make_instr_imm_param_32,
            Instruction::i32_sub_imm16,
            Instruction::i32_sub_imm16_rev,
            TypedValue::i32_sub,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `sub x - x` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == 0 {
                    // Optimization: `sub x - 0` is same as `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i32_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_mul,
            Instruction::i32_mul_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_mul_imm16,
            TypedValue::i32_mul,
            Self::no_custom_opt,
            |this, reg: Register, value: i32| {
                if value == 0 {
                    // Optimization: `add x * 0` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                if value == 1 {
                    // Optimization: `add x * 1` is always `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_div_s(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i32_div_s,
            Instruction::i32_div_s_imm,
            Instruction::i32_div_s_imm_rev,
            Self::make_instr_imm_param_32,
            Instruction::i32_div_s_imm16,
            Instruction::i32_div_s_imm16_rev,
            TypedValue::i32_div_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x / x` is always `1`
                    this.alloc.stack.push_const(1_i32);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_div_u(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i32_div_u,
            Instruction::i32_div_u_imm,
            Instruction::i32_div_u_imm_rev,
            Self::make_instr_imm_param_32,
            Instruction::i32_div_u_imm16,
            Instruction::i32_div_u_imm16_rev,
            TypedValue::i32_div_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x / x` is always `1`
                    this.alloc.stack.push_const(1_i32);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_rem_s(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i32_rem_s,
            Instruction::i32_rem_s_imm,
            Instruction::i32_rem_s_imm_rev,
            Self::make_instr_imm_param_32,
            Instruction::i32_rem_s_imm16,
            Instruction::i32_rem_s_imm16_rev,
            TypedValue::i32_rem_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x % x` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i32| {
                if rhs == 1 || rhs == -1 {
                    // Optimization: `x % 1` or `x % -1` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_rem_u(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i32_rem_u,
            Instruction::i32_rem_u_imm,
            Instruction::i32_rem_u_imm_rev,
            Self::make_instr_imm_param_32,
            Instruction::i32_rem_u_imm16,
            Instruction::i32_rem_u_imm16_rev,
            TypedValue::i32_rem_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x % x` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i32| {
                if rhs == 1 {
                    // Optimization: `x % 1` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_and(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_and,
            Instruction::i32_and_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_and_imm16,
            TypedValue::i32_and,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x & x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i32| {
                if value == -1 {
                    // Optimization: `x & -1` is same as `x`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x & 0` is same as `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_or(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_or,
            Instruction::i32_or_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_or_imm16,
            TypedValue::i32_or,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x | x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i32| {
                if value == -1 {
                    // Optimization: `x | -1` is same as `-1`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_const(-1_i32);
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x | 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_xor(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_xor,
            Instruction::i32_xor_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_xor_imm16,
            TypedValue::i32_xor,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x ^ x` is always `0`
                    this.alloc.stack.push_const(0_i32);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i32| {
                if value == 0 {
                    // Optimization: `x ^ 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_shl(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_shl,
            Instruction::i32_shl_imm,
            Self::make_instr_imm_param_32::<i32>,
            Instruction::i32_shl_imm_rev,
            Instruction::i32_shl_imm16_rev,
            TypedValue::i32_shl,
            Self::no_custom_opt,
        )
    }

    fn visit_i32_shr_s(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_shr_s,
            Instruction::i32_shr_s_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_shr_s_imm_rev,
            Instruction::i32_shr_s_imm16_rev,
            TypedValue::i32_shr_s,
            |this, lhs: i32, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_shr_u(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_shr_u,
            Instruction::i32_shr_u_imm,
            Self::make_instr_imm_param_32::<i32>,
            Instruction::i32_shr_u_imm_rev,
            Instruction::i32_shr_u_imm16_rev,
            TypedValue::i32_shr_u,
            Self::no_custom_opt,
        )
    }

    fn visit_i32_rotl(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_rotl,
            Instruction::i32_rotl_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_rotl_imm_rev,
            Instruction::i32_rotl_imm16_rev,
            TypedValue::i32_rotl,
            |this, lhs: i32, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1.rotate_left(x)` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_rotr(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_rotr,
            Instruction::i32_rotr_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_rotr_imm_rev,
            Instruction::i32_rotr_imm16_rev,
            TypedValue::i32_rotr,
            |this, lhs: i32, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1.rotate_right(x)` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_clz(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_clz, TypedValue::i64_clz)
    }

    fn visit_i64_ctz(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_ctz, TypedValue::i64_ctz)
    }

    fn visit_i64_popcnt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_popcnt, TypedValue::i64_popcnt)
    }

    fn visit_i64_add(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_add,
            Instruction::i64_add_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_add_imm16,
            TypedValue::i64_add,
            Self::no_custom_opt,
            |this, reg: Register, value: i64| {
                if value == 0 {
                    // Optimization: `add x + 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_sub(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_sub,
            Instruction::i64_sub_imm,
            Instruction::i64_sub_imm_rev,
            Self::make_instr_imm_param_64,
            Instruction::i64_sub_imm16,
            Instruction::i64_sub_imm16_rev,
            TypedValue::i64_sub,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `sub x - x` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == 0 {
                    // Optimization: `sub x - 0` is same as `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i64_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_mul,
            Instruction::i64_mul_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_mul_imm16,
            TypedValue::i64_mul,
            Self::no_custom_opt,
            |this, reg: Register, value: i64| {
                if value == 0 {
                    // Optimization: `add x * 0` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                if value == 1 {
                    // Optimization: `add x * 1` is always `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_div_s(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i64_div_s,
            Instruction::i64_div_s_imm,
            Instruction::i64_div_s_imm_rev,
            Self::make_instr_imm_param_64,
            Instruction::i64_div_s_imm16,
            Instruction::i64_div_s_imm16_rev,
            TypedValue::i64_div_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x / x` is always `1`
                    this.alloc.stack.push_const(1_i64);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_div_u(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i64_div_u,
            Instruction::i64_div_u_imm,
            Instruction::i64_div_u_imm_rev,
            Self::make_instr_imm_param_64,
            Instruction::i64_div_u_imm16,
            Instruction::i64_div_u_imm16_rev,
            TypedValue::i64_div_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x / x` is always `1`
                    this.alloc.stack.push_const(1_i64);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_rem_s(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i64_rem_s,
            Instruction::i64_rem_s_imm,
            Instruction::i64_rem_s_imm_rev,
            Self::make_instr_imm_param_64,
            Instruction::i64_rem_s_imm16,
            Instruction::i64_rem_s_imm16_rev,
            TypedValue::i64_rem_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x % x` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i64| {
                if rhs == 1 || rhs == -1 {
                    // Optimization: `x % 1` or `x % -1` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_rem_u(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i64_rem_u,
            Instruction::i64_rem_u_imm,
            Instruction::i64_rem_u_imm_rev,
            Self::make_instr_imm_param_64,
            Instruction::i64_rem_u_imm16,
            Instruction::i64_rem_u_imm16_rev,
            TypedValue::i64_rem_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x % x` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, _lhs: Register, rhs: i64| {
                if rhs == 1 {
                    // Optimization: `x % 1` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_and(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_and,
            Instruction::i64_and_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_and_imm16,
            TypedValue::i64_and,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x & x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i64| {
                if value == -1 {
                    // Optimization: `x & -1` is same as `x`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x & 0` is same as `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_or(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_or,
            Instruction::i64_or_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_or_imm16,
            TypedValue::i64_or,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x | x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i64| {
                if value == -1 {
                    // Optimization: `x | -1` is same as `-1`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_const(-1_i64);
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x | 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_xor(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_xor,
            Instruction::i64_xor_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_xor_imm16,
            TypedValue::i64_xor,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x ^ x` is always `0`
                    this.alloc.stack.push_const(0_i64);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i64| {
                if value == 0 {
                    // Optimization: `x ^ 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_shl(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_shl,
            Instruction::i64_shl_imm,
            Self::make_instr_imm_param_64::<i64>,
            Instruction::i64_shl_imm_rev,
            Instruction::i64_shl_imm16_rev,
            TypedValue::i64_shl,
            Self::no_custom_opt,
        )
    }

    fn visit_i64_shr_s(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_shr_s,
            Instruction::i64_shr_s_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_shr_s_imm_rev,
            Instruction::i64_shr_s_imm16_rev,
            TypedValue::i64_shr_s,
            |this, lhs: i64, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_shr_u(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_shr_u,
            Instruction::i64_shr_u_imm,
            Self::make_instr_imm_param_64::<i64>,
            Instruction::i64_shr_u_imm_rev,
            Instruction::i64_shr_u_imm16_rev,
            TypedValue::i64_shr_u,
            Self::no_custom_opt,
        )
    }

    fn visit_i64_rotl(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_rotl,
            Instruction::i64_rotl_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_rotl_imm_rev,
            Instruction::i64_rotl_imm16_rev,
            TypedValue::i64_rotl,
            |this, lhs: i64, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_rotr(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_rotr,
            Instruction::i64_rotr_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_rotr_imm_rev,
            Instruction::i64_rotr_imm16_rev,
            TypedValue::i64_rotr,
            |this, lhs: i64, _rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_abs(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_abs, TypedValue::f32_abs)
    }

    fn visit_f32_neg(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_neg, TypedValue::f32_neg)
    }

    fn visit_f32_ceil(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_ceil, TypedValue::f32_ceil)
    }

    fn visit_f32_floor(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_floor, TypedValue::f32_floor)
    }

    fn visit_f32_trunc(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_trunc, TypedValue::f32_trunc)
    }

    fn visit_f32_nearest(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_nearest, TypedValue::f32_nearest)
    }

    fn visit_f32_sqrt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_sqrt, TypedValue::f32_sqrt)
    }

    fn visit_f32_add(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f32_add,
            Instruction::f32_add_imm,
            Self::make_instr_imm_param_32,
            TypedValue::f32_add,
            Self::no_custom_opt,
            |this, reg: Register, value: f32| {
                if value == 0.0 || value == -0.0 {
                    // Optimization: `add x + 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_sub(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f32_sub,
            Instruction::f32_sub_imm,
            Instruction::f32_sub_imm_rev,
            Self::make_instr_imm_param_32,
            TypedValue::f32_sub,
            Self::no_custom_opt,
            |this, lhs: Register, rhs: f32| {
                if rhs == 0.0 && rhs.is_sign_positive() {
                    // Optimization: `x - 0` is same as `x`
                    //
                    // Note due to behavior dictated by the Wasm specification
                    // we cannot apply this optimization for negative zeros.
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            // Unfortuantely we cannot optimize for the case that `lhs == 0.0`
            // since the Wasm specification mandates different behavior in
            // dependence of `rhs` which we do not know at this point.
            Self::no_custom_opt,
        )
    }

    fn visit_f32_mul(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f32_mul,
            Instruction::f32_mul_imm,
            Self::make_instr_imm_param_32::<f32>,
            TypedValue::f32_mul,
            Self::no_custom_opt,
            // Unfortunately we cannot apply `x * 0` or `0 * x` optimizations
            // since Wasm mandates different behaviors if `x` is infinite or
            // NaN in these cases.
            Self::no_custom_opt,
        )
    }

    fn visit_f32_div(&mut self) -> Self::Output {
        self.translate_fbinary::<f32>(
            Instruction::f32_div,
            Instruction::f32_div_imm,
            Instruction::f32_div_imm_rev,
            Self::make_instr_imm_param_32,
            TypedValue::f32_div,
            Self::no_custom_opt,
            Self::no_custom_opt,
            Self::no_custom_opt,
        )
    }

    fn visit_f32_min(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f32_min,
            Instruction::f32_min_imm,
            Self::make_instr_imm_param_32,
            TypedValue::f32_min,
            Self::no_custom_opt,
            |this, reg: Register, value: f32| {
                if value.is_infinite() && value.is_sign_positive() {
                    // Optimization: `min(x, +inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_max(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f32_max,
            Instruction::f32_max_imm,
            Self::make_instr_imm_param_32,
            TypedValue::f32_max,
            Self::no_custom_opt,
            |this, reg: Register, value: f32| {
                if value.is_infinite() && value.is_sign_negative() {
                    // Optimization: `max(x, -inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_copysign(&mut self) -> Self::Output {
        self.translate_fcopysign(
            Instruction::f32_copysign,
            Instruction::f32_copysign_imm,
            Instruction::f32_copysign_imm_rev,
            Self::make_instr_imm_param_32::<f32>,
            TypedValue::f32_copysign,
        )
    }

    fn visit_f64_abs(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_abs, TypedValue::f64_abs)
    }

    fn visit_f64_neg(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_neg, TypedValue::f64_neg)
    }

    fn visit_f64_ceil(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_ceil, TypedValue::f64_ceil)
    }

    fn visit_f64_floor(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_floor, TypedValue::f64_floor)
    }

    fn visit_f64_trunc(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_trunc, TypedValue::f64_trunc)
    }

    fn visit_f64_nearest(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_nearest, TypedValue::f64_nearest)
    }

    fn visit_f64_sqrt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_sqrt, TypedValue::f64_sqrt)
    }

    fn visit_f64_add(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f64_add,
            Instruction::f64_add_imm,
            Self::make_instr_imm_param_64,
            TypedValue::f64_add,
            Self::no_custom_opt,
            |this, reg: Register, value: f64| {
                if value == 0.0 || value == -0.0 {
                    // Optimization: `add x + 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_sub(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f64_sub,
            Instruction::f64_sub_imm,
            Instruction::f64_sub_imm_rev,
            Self::make_instr_imm_param_64,
            TypedValue::f64_sub,
            Self::no_custom_opt,
            |this, lhs: Register, rhs: f64| {
                if rhs == 0.0 && rhs.is_sign_positive() {
                    // Optimization: `x - 0` is same as `x`
                    //
                    // Note due to behavior dictated by the Wasm specification
                    // we cannot apply this optimization for negative zeros.
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            // Unfortuantely we cannot optimize for the case that `lhs == 0.0`
            // since the Wasm specification mandates different behavior in
            // dependence of `rhs` which we do not know at this point.
            Self::no_custom_opt,
        )
    }

    fn visit_f64_mul(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f64_mul,
            Instruction::f64_mul_imm,
            Self::make_instr_imm_param_64::<f64>,
            TypedValue::f64_mul,
            Self::no_custom_opt,
            // Unfortunately we cannot apply `x * 0` or `0 * x` optimizations
            // since Wasm mandates different behaviors if `x` is infinite or
            // NaN in these cases.
            Self::no_custom_opt,
        )
    }

    fn visit_f64_div(&mut self) -> Self::Output {
        self.translate_fbinary::<f64>(
            Instruction::f64_div,
            Instruction::f64_div_imm,
            Instruction::f64_div_imm_rev,
            Self::make_instr_imm_param_64,
            TypedValue::f64_div,
            Self::no_custom_opt,
            Self::no_custom_opt,
            Self::no_custom_opt,
        )
    }

    fn visit_f64_min(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f64_min,
            Instruction::f64_min_imm,
            Self::make_instr_imm_param_64,
            TypedValue::f64_min,
            Self::no_custom_opt,
            |this, reg: Register, value: f64| {
                if value.is_infinite() && value.is_sign_positive() {
                    // Optimization: `min(x, +inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_max(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f64_max,
            Instruction::f64_max_imm,
            Self::make_instr_imm_param_64,
            TypedValue::f64_max,
            Self::no_custom_opt,
            |this, reg: Register, value: f64| {
                if value.is_infinite() && value.is_sign_negative() {
                    // Optimization: `min(x, +inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_copysign(&mut self) -> Self::Output {
        self.translate_fcopysign(
            Instruction::f64_copysign,
            Instruction::f64_copysign_imm,
            Instruction::f64_copysign_imm_rev,
            Self::make_instr_imm_param_64::<f64>,
            TypedValue::f64_copysign,
        )
    }

    fn visit_i32_wrap_i64(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_wrap_i64, TypedValue::i32_wrap_i64)
    }

    fn visit_i32_trunc_f32_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f32_s, TypedValue::i32_trunc_f32_s)
    }

    fn visit_i32_trunc_f32_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f32_u, TypedValue::i32_trunc_f32_u)
    }

    fn visit_i32_trunc_f64_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f64_s, TypedValue::i32_trunc_f64_s)
    }

    fn visit_i32_trunc_f64_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i32_trunc_f64_u, TypedValue::i32_trunc_f64_u)
    }

    fn visit_i64_extend_i32_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend_i32_s, TypedValue::i64_extend_i32_s)
    }

    fn visit_i64_extend_i32_u(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend_i32_u, TypedValue::i64_extend_i32_u)
    }

    fn visit_i64_trunc_f32_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f32_s, TypedValue::i64_trunc_f32_s)
    }

    fn visit_i64_trunc_f32_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f32_u, TypedValue::i64_trunc_f32_u)
    }

    fn visit_i64_trunc_f64_s(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f64_s, TypedValue::i64_trunc_f64_s)
    }

    fn visit_i64_trunc_f64_u(&mut self) -> Self::Output {
        self.translate_unary_fallible(Instruction::i64_trunc_f64_u, TypedValue::i64_trunc_f64_u)
    }

    fn visit_f32_convert_i32_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::f32_convert_i32_s,
            TypedValue::f32_convert_i32_s,
        )
    }

    fn visit_f32_convert_i32_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::f32_convert_i32_u,
            TypedValue::f32_convert_i32_u,
        )
    }

    fn visit_f32_convert_i64_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::f32_convert_i64_s,
            TypedValue::f32_convert_i64_s,
        )
    }

    fn visit_f32_convert_i64_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::f32_convert_i64_u,
            TypedValue::f32_convert_i64_u,
        )
    }

    fn visit_f32_demote_f64(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_demote_f64, TypedValue::f32_demote_f64)
    }

    fn visit_f64_convert_i32_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::f64_convert_i32_s,
            TypedValue::f64_convert_i32_s,
        )
    }

    fn visit_f64_convert_i32_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::f64_convert_i32_u,
            TypedValue::f64_convert_i32_u,
        )
    }

    fn visit_f64_convert_i64_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::f64_convert_i64_s,
            TypedValue::f64_convert_i64_s,
        )
    }

    fn visit_f64_convert_i64_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::f64_convert_i64_u,
            TypedValue::f64_convert_i64_u,
        )
    }

    fn visit_f64_promote_f32(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_promote_f32, TypedValue::f64_promote_f32)
    }

    fn visit_i32_reinterpret_f32(&mut self) -> Self::Output {
        self.translate_reinterpret(ValueType::I32)
    }

    fn visit_i64_reinterpret_f64(&mut self) -> Self::Output {
        self.translate_reinterpret(ValueType::I64)
    }

    fn visit_f32_reinterpret_i32(&mut self) -> Self::Output {
        self.translate_reinterpret(ValueType::F32)
    }

    fn visit_f64_reinterpret_i64(&mut self) -> Self::Output {
        self.translate_reinterpret(ValueType::F64)
    }

    fn visit_i32_extend8_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_extend8_s, TypedValue::i32_extend8_s)
    }

    fn visit_i32_extend16_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_extend16_s, TypedValue::i32_extend16_s)
    }

    fn visit_i64_extend8_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend8_s, TypedValue::i64_extend8_s)
    }

    fn visit_i64_extend16_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend16_s, TypedValue::i64_extend16_s)
    }

    fn visit_i64_extend32_s(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_extend32_s, TypedValue::i64_extend32_s)
    }

    fn visit_i32_trunc_sat_f32_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i32_trunc_sat_f32_s,
            TypedValue::i32_trunc_sat_f32_s,
        )
    }

    fn visit_i32_trunc_sat_f32_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i32_trunc_sat_f32_u,
            TypedValue::i32_trunc_sat_f32_u,
        )
    }

    fn visit_i32_trunc_sat_f64_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i32_trunc_sat_f64_s,
            TypedValue::i32_trunc_sat_f64_s,
        )
    }

    fn visit_i32_trunc_sat_f64_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i32_trunc_sat_f64_u,
            TypedValue::i32_trunc_sat_f64_u,
        )
    }

    fn visit_i64_trunc_sat_f32_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i64_trunc_sat_f32_s,
            TypedValue::i64_trunc_sat_f32_s,
        )
    }

    fn visit_i64_trunc_sat_f32_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i64_trunc_sat_f32_u,
            TypedValue::i64_trunc_sat_f32_u,
        )
    }

    fn visit_i64_trunc_sat_f64_s(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i64_trunc_sat_f64_s,
            TypedValue::i64_trunc_sat_f64_s,
        )
    }

    fn visit_i64_trunc_sat_f64_u(&mut self) -> Self::Output {
        self.translate_unary(
            Instruction::i64_trunc_sat_f64_u,
            TypedValue::i64_trunc_sat_f64_u,
        )
    }

    fn visit_memory_init(&mut self, _data_index: u32, _mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_data_drop(&mut self, _data_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_copy(&mut self, _dst_mem: u32, _src_mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_fill(&mut self, _mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_init(&mut self, _elem_index: u32, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_elem_drop(&mut self, _elem_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_copy(&mut self, _dst_table: u32, _src_table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_fill(&mut self, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_get(&mut self, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_set(&mut self, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_grow(&mut self, _table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_size(&mut self, _table: u32) -> Self::Output {
        todo!()
    }
}
