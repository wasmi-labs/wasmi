//! Definitions to compile a Wasm module into `wasmi` bytecode.
//!
//! The implementation is specific to the underlying Wasm parser
//! framework used by `wasmi` which currently is `parity_wasm`.

mod control_frame;
mod error;
mod utils;

use self::{control_frame::ControlFrame, error::TranslationError};
use super::{
    super::{DropKeep, FuncBody, InstructionIdx, InstructionsBuilder, LabelIdx, Target},
    Engine,
};
use crate::{
    v2::{
        interpreter::bytecode::{FuncIdx, GlobalIdx, LocalIdx, SignatureIdx},
        interpreter::{
            bytecode::Offset,
            inst_builder::{Reloc, Signedness, WasmFloatType, WasmIntType},
        },
    },
    RuntimeValue, ValueType,
};
use alloc::vec::Vec;
use parity_wasm::elements::{self as pwasm, Instruction};
use validation::func::{top_label, FunctionValidationContext, StartedWith};

/// Allows to translate a Wasm functions into `wasmi` bytecode.
#[derive(Debug)]
pub struct FuncBodyTranslator {
    /// The underlying engine which the translator feeds.
    engine: Engine,
    /// The underlying instruction builder to incrementally build up `wasmi` bytecode.
    inst_builder: InstructionsBuilder,
    /// The underlying control flow frames representing Wasm control flow.
    control_frames: Vec<ControlFrame>,
}

impl FuncBodyTranslator {
    /// Creates a new Wasm function body translator for the given [`Engine`].
    pub fn new(engine: &Engine) -> Self {
        let mut inst_builder = InstructionsBuilder::default();
        // Push implicit frame for the whole function block.
        let end_label = inst_builder.new_label();
        let control_frames = vec![ControlFrame::Block { end_label }];
        Self {
            engine: engine.clone(),
            inst_builder,
            control_frames,
        }
    }

    /// Translates the instructions forming a Wasm function body into `wasmi` bytecode.
    ///
    /// Returns a [`FuncBody`] reference to the translated `wasmi` bytecode.
    pub fn translate<'a, I: 'a>(
        &mut self,
        validator: &mut FunctionValidationContext,
        instructions: I,
    ) -> Result<FuncBody, TranslationError>
    where
        I: IntoIterator<Item = &'a pwasm::Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        for instruction in instructions {
            self.translate_instruction(validator, instruction)?;
        }
        let func_body = self.inst_builder.finish(&self.engine);
        Ok(func_body)
    }

    /// Pops the top most control frame.
    ///
    /// # Panics
    ///
    /// If the control flow frame stack is empty.
    fn pop_control_frame(&mut self) -> ControlFrame {
        self.control_frames
            .pop()
            .expect("encountered unexpected empty control frame stack")
    }

    /// Try to resolve the given label.
    ///
    /// In case the label cannot yet be resolved register the [`Reloc`] as its user.
    fn try_resolve_label<F>(&mut self, label: LabelIdx, reloc_provider: F) -> InstructionIdx
    where
        F: FnOnce(InstructionIdx) -> Reloc,
    {
        let pc = self.inst_builder.current_pc();
        self.inst_builder
            .try_resolve_label(label, || reloc_provider(pc))
    }

    /// Validate the Wasm `inst` and translate the respective `wasmi` bytecode.
    fn validate_translate<F, R>(
        &mut self,
        validator: &mut FunctionValidationContext,
        inst: &Instruction,
        f: F,
    ) -> Result<(), TranslationError>
    where
        F: FnOnce(&mut InstructionsBuilder) -> R,
    {
        validator.step(inst)?;
        f(&mut self.inst_builder);
        Ok(())
    }

    /// Translates a single Wasm instruction into `wasmi` bytecode.
    ///
    /// # Errors
    ///
    /// If there are validation or translation problems.
    fn translate_instruction(
        &mut self,
        validator: &mut FunctionValidationContext,
        inst: &Instruction,
    ) -> Result<(), TranslationError> {
        use Instruction as Inst;
        match inst {
            Inst::Unreachable => {
                self.validate_translate(validator, inst, InstructionsBuilder::unreachable)?;
            }
            Inst::Nop => {
                // No need to translate a no-op into `wasmi` bytecode.
                validator.step(inst)?;
            }
            Inst::Block(_block_type) => {
                self.translate_block(validator, inst)?;
            }
            Inst::Loop(_block_type) => {
                self.translate_loop(validator, inst)?;
            }
            Inst::If(_block_type) => {
                self.translate_if(validator, inst)?;
            }
            Inst::Else => {
                self.translate_else(validator, inst)?;
            }
            Inst::End => {
                self.translate_end(validator, inst)?;
            }
            Inst::Br(depth) => {
                self.translate_br(depth, validator, inst)?;
            }
            Inst::BrIf(depth) => {
                self.translate_br_if(validator, inst, depth)?;
            }
            Inst::BrTable(br_table) => {
                self.translate_br_table(validator, br_table, inst)?;
            }
            Inst::Return => {
                self.translate_return(validator, inst)?;
            }
            Inst::Call(func_idx) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.call(FuncIdx::from(*func_idx))
                })?;
            }
            Inst::CallIndirect(signature_idx, _table_ref) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.call_indirect(SignatureIdx::from(*signature_idx))
                })?;
            }
            Inst::Drop => self.validate_translate(validator, inst, InstructionsBuilder::drop)?,
            Inst::Select => {
                self.validate_translate(validator, inst, InstructionsBuilder::select)?
            }
            Inst::GetLocal(index) => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.get_local(LocalIdx::from(*index))
            })?,
            Inst::SetLocal(index) => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.set_local(LocalIdx::from(*index))
            })?,
            Inst::TeeLocal(index) => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.tee_local(LocalIdx::from(*index))
            })?,
            Inst::GetGlobal(index) => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.get_global(GlobalIdx::from(*index))
            })?,
            Inst::SetGlobal(index) => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.set_global(GlobalIdx::from(*index))
            })?,
            Inst::I32Load(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load(ValueType::I32, Offset::from(*offset))
                })?
            }
            Inst::I64Load(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load(ValueType::I64, Offset::from(*offset))
                })?
            }
            Inst::F32Load(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load(ValueType::F32, Offset::from(*offset))
                })?
            }
            Inst::F64Load(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load(ValueType::F64, Offset::from(*offset))
                })?
            }
            Inst::I32Load8S(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load_extend::<i32, i8>(Offset::from(*offset))
                })?
            }
            Inst::I32Load8U(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load_extend::<i32, u8>(Offset::from(*offset))
                })?
            }
            Inst::I32Load16S(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load_extend::<i32, i16>(Offset::from(*offset))
                })?
            }
            Inst::I32Load16U(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load_extend::<i32, u16>(Offset::from(*offset))
                })?
            }
            Inst::I64Load8S(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load_extend::<i64, i8>(Offset::from(*offset))
                })?
            }
            Inst::I64Load8U(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load_extend::<i64, u8>(Offset::from(*offset))
                })?
            }
            Inst::I64Load16S(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load_extend::<i64, i16>(Offset::from(*offset))
                })?
            }
            Inst::I64Load16U(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load_extend::<i64, u16>(Offset::from(*offset))
                })?
            }
            Inst::I64Load32S(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load_extend::<i64, i32>(Offset::from(*offset))
                })?
            }
            Inst::I64Load32U(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.load_extend::<i64, u32>(Offset::from(*offset))
                })?
            }
            Inst::I32Store(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.store(ValueType::I32, Offset::from(*offset))
                })?
            }
            Inst::I64Store(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.store(ValueType::I64, Offset::from(*offset))
                })?
            }
            Inst::F32Store(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.store(ValueType::F32, Offset::from(*offset))
                })?
            }
            Inst::F64Store(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.store(ValueType::F64, Offset::from(*offset))
                })?
            }
            Inst::I32Store8(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.store_truncate::<i32, i8>(Offset::from(*offset))
                })?
            }
            Inst::I32Store16(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.store_truncate::<i32, i16>(Offset::from(*offset))
                })?
            }
            Inst::I64Store8(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.store_truncate::<i64, i8>(Offset::from(*offset))
                })?
            }
            Inst::I64Store16(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.store_truncate::<i64, i16>(Offset::from(*offset))
                })?
            }
            Inst::I64Store32(_memory_idx, offset) => {
                self.validate_translate(validator, inst, |inst_builder| {
                    inst_builder.store_truncate::<i64, i32>(Offset::from(*offset))
                })?
            }
            Inst::CurrentMemory(_) => {
                self.validate_translate(validator, inst, InstructionsBuilder::memory_size)?
            }
            Inst::GrowMemory(_) => {
                self.validate_translate(validator, inst, InstructionsBuilder::memory_grow)?
            }
            Inst::I32Const(value) => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.constant(RuntimeValue::from(*value))
            })?,
            Inst::I64Const(value) => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.constant(RuntimeValue::from(*value))
            })?,
            Inst::F32Const(value) => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.constant(RuntimeValue::from(*value))
            })?,
            Inst::F64Const(value) => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.constant(RuntimeValue::from(*value))
            })?,
            Inst::I32Eqz => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_eqz(WasmIntType::I32)
            })?,
            Inst::I32Eq => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.eq(ValueType::I32)
            })?,
            Inst::I32Ne => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.ne(ValueType::I32)
            })?,
            Inst::I32LtS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_lt(WasmIntType::I32, Signedness::Signed)
            })?,
            Inst::I32LtU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_lt(WasmIntType::I32, Signedness::Unsigned)
            })?,
            Inst::I32GtS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_gt(WasmIntType::I32, Signedness::Signed)
            })?,
            Inst::I32GtU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_gt(WasmIntType::I32, Signedness::Unsigned)
            })?,
            Inst::I32LeS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_le(WasmIntType::I32, Signedness::Signed)
            })?,
            Inst::I32LeU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_le(WasmIntType::I32, Signedness::Unsigned)
            })?,
            Inst::I32GeS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_ge(WasmIntType::I32, Signedness::Signed)
            })?,
            Inst::I32GeU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_ge(WasmIntType::I32, Signedness::Unsigned)
            })?,
            Inst::I64Eqz => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_eqz(WasmIntType::I64)
            })?,
            Inst::I64Eq => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.eq(ValueType::I64)
            })?,
            Inst::I64Ne => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.ne(ValueType::I64)
            })?,
            Inst::I64LtS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_lt(WasmIntType::I64, Signedness::Signed)
            })?,
            Inst::I64LtU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_lt(WasmIntType::I64, Signedness::Unsigned)
            })?,
            Inst::I64GtS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_gt(WasmIntType::I64, Signedness::Signed)
            })?,
            Inst::I64GtU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_gt(WasmIntType::I64, Signedness::Unsigned)
            })?,
            Inst::I64LeS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_le(WasmIntType::I64, Signedness::Signed)
            })?,
            Inst::I64LeU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_le(WasmIntType::I64, Signedness::Unsigned)
            })?,
            Inst::I64GeS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_ge(WasmIntType::I64, Signedness::Signed)
            })?,
            Inst::I64GeU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_ge(WasmIntType::I64, Signedness::Unsigned)
            })?,
            Inst::F32Eq => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.eq(ValueType::F32)
            })?,
            Inst::F32Ne => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.ne(ValueType::F32)
            })?,
            Inst::F32Lt => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.float_lt(WasmFloatType::F32)
            })?,
            Inst::F32Gt => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.float_gt(WasmFloatType::F32)
            })?,
            Inst::F32Le => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.float_le(WasmFloatType::F32)
            })?,
            Inst::F32Ge => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.float_ge(WasmFloatType::F32)
            })?,
            Inst::F64Eq => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.eq(ValueType::F64)
            })?,
            Inst::F64Ne => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.ne(ValueType::F64)
            })?,
            Inst::F64Lt => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.float_lt(WasmFloatType::F64)
            })?,
            Inst::F64Gt => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.float_gt(WasmFloatType::F64)
            })?,
            Inst::F64Le => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.float_le(WasmFloatType::F64)
            })?,
            Inst::F64Ge => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.float_ge(WasmFloatType::F64)
            })?,
            Inst::I32Clz => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_clz(WasmIntType::I32)
            })?,
            Inst::I32Ctz => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_ctz(WasmIntType::I32)
            })?,
            Inst::I32Popcnt => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_popcnt(WasmIntType::I32)
            })?,
            Inst::I32Add => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_add(WasmIntType::I32)
            })?,
            Inst::I32Sub => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_sub(WasmIntType::I32)
            })?,
            Inst::I32Mul => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_mul(WasmIntType::I32)
            })?,
            Inst::I32DivS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_div(WasmIntType::I32, Signedness::Signed)
            })?,
            Inst::I32DivU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_div(WasmIntType::I32, Signedness::Unsigned)
            })?,
            Inst::I32RemS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_rem(WasmIntType::I32, Signedness::Signed)
            })?,
            Inst::I32RemU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_rem(WasmIntType::I32, Signedness::Unsigned)
            })?,
            Inst::I32And => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_and(WasmIntType::I32)
            })?,
            Inst::I32Or => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_or(WasmIntType::I32)
            })?,
            Inst::I32Xor => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_xor(WasmIntType::I32)
            })?,
            Inst::I32Shl => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_shl(WasmIntType::I32)
            })?,
            Inst::I32ShrS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_shr(WasmIntType::I32, Signedness::Signed)
            })?,
            Inst::I32ShrU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_shr(WasmIntType::I32, Signedness::Unsigned)
            })?,
            Inst::I32Rotl => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_rotl(WasmIntType::I32)
            })?,
            Inst::I32Rotr => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_rotr(WasmIntType::I32)
            })?,
            Inst::I64Clz => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_clz(WasmIntType::I64)
            })?,
            Inst::I64Ctz => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_ctz(WasmIntType::I64)
            })?,
            Inst::I64Popcnt => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_popcnt(WasmIntType::I64)
            })?,
            Inst::I64Add => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_add(WasmIntType::I64)
            })?,
            Inst::I64Sub => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_sub(WasmIntType::I64)
            })?,
            Inst::I64Mul => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_mul(WasmIntType::I64)
            })?,
            Inst::I64DivS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_div(WasmIntType::I64, Signedness::Signed)
            })?,
            Inst::I64DivU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_div(WasmIntType::I64, Signedness::Unsigned)
            })?,
            Inst::I64RemS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_rem(WasmIntType::I64, Signedness::Signed)
            })?,
            Inst::I64RemU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_rem(WasmIntType::I64, Signedness::Unsigned)
            })?,
            Inst::I64And => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_and(WasmIntType::I64)
            })?,
            Inst::I64Or => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_or(WasmIntType::I64)
            })?,
            Inst::I64Xor => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_xor(WasmIntType::I64)
            })?,
            Inst::I64Shl => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_shl(WasmIntType::I64)
            })?,
            Inst::I64ShrS => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_shr(WasmIntType::I64, Signedness::Signed)
            })?,
            Inst::I64ShrU => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_shr(WasmIntType::I64, Signedness::Unsigned)
            })?,
            Inst::I64Rotl => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_rotl(WasmIntType::I64)
            })?,
            Inst::I64Rotr => self.validate_translate(validator, inst, |inst_builder| {
                inst_builder.int_rotr(WasmIntType::I64)
            })?,
            Inst::F32Abs => todo!(),
            Inst::F32Neg => todo!(),
            Inst::F32Ceil => todo!(),
            Inst::F32Floor => todo!(),
            Inst::F32Trunc => todo!(),
            Inst::F32Nearest => todo!(),
            Inst::F32Sqrt => todo!(),
            Inst::F32Add => todo!(),
            Inst::F32Sub => todo!(),
            Inst::F32Mul => todo!(),
            Inst::F32Div => todo!(),
            Inst::F32Min => todo!(),
            Inst::F32Max => todo!(),
            Inst::F32Copysign => todo!(),
            Inst::F64Abs => todo!(),
            Inst::F64Neg => todo!(),
            Inst::F64Ceil => todo!(),
            Inst::F64Floor => todo!(),
            Inst::F64Trunc => todo!(),
            Inst::F64Nearest => todo!(),
            Inst::F64Sqrt => todo!(),
            Inst::F64Add => todo!(),
            Inst::F64Sub => todo!(),
            Inst::F64Mul => todo!(),
            Inst::F64Div => todo!(),
            Inst::F64Min => todo!(),
            Inst::F64Max => todo!(),
            Inst::F64Copysign => todo!(),
            Inst::I32WrapI64 => todo!(),
            Inst::I32TruncSF32 => todo!(),
            Inst::I32TruncUF32 => todo!(),
            Inst::I32TruncSF64 => todo!(),
            Inst::I32TruncUF64 => todo!(),
            Inst::I64ExtendSI32 => todo!(),
            Inst::I64ExtendUI32 => todo!(),
            Inst::I64TruncSF32 => todo!(),
            Inst::I64TruncUF32 => todo!(),
            Inst::I64TruncSF64 => todo!(),
            Inst::I64TruncUF64 => todo!(),
            Inst::F32ConvertSI32 => todo!(),
            Inst::F32ConvertUI32 => todo!(),
            Inst::F32ConvertSI64 => todo!(),
            Inst::F32ConvertUI64 => todo!(),
            Inst::F32DemoteF64 => todo!(),
            Inst::F64ConvertSI32 => todo!(),
            Inst::F64ConvertUI32 => todo!(),
            Inst::F64ConvertSI64 => todo!(),
            Inst::F64ConvertUI64 => todo!(),
            Inst::F64PromoteF32 => todo!(),
            Inst::I32ReinterpretF32 => todo!(),
            Inst::I64ReinterpretF64 => todo!(),
            Inst::F32ReinterpretI32 => todo!(),
            Inst::F64ReinterpretI64 => todo!(),
        }
        Ok(())
    }

    /// Translates a Wasm `block` control flow instruction into `wasmi` bytecode.
    fn translate_block(
        &mut self,
        validator: &mut FunctionValidationContext,
        inst: &Instruction,
    ) -> Result<(), TranslationError> {
        validator.step(inst)?;
        let end_label = self.inst_builder.new_label();
        self.control_frames.push(ControlFrame::Block { end_label });
        Ok(())
    }

    /// Translates a Wasm `loop` control flow instruction into `wasmi` bytecode.
    fn translate_loop(
        &mut self,
        validator: &mut FunctionValidationContext,
        inst: &Instruction,
    ) -> Result<(), TranslationError> {
        validator.step(inst)?;
        let header = self.inst_builder.new_label();
        self.inst_builder.resolve_label(header);
        self.control_frames.push(ControlFrame::Loop { header });
        Ok(())
    }

    /// Translates a Wasm `if` control flow instruction into `wasmi` bytecode.
    fn translate_if(
        &mut self,
        validator: &mut FunctionValidationContext,
        inst: &Instruction,
    ) -> Result<(), TranslationError> {
        validator.step(inst)?;
        let else_label = self.inst_builder.new_label();
        let end_label = self.inst_builder.new_label();
        self.control_frames.push(ControlFrame::If {
            else_label,
            end_label,
        });
        let dst_pc = self.try_resolve_label(else_label, |pc| Reloc::Br { inst_idx: pc });
        self.inst_builder
            .branch_eqz(Target::new(dst_pc, DropKeep::new(0, 0)));
        Ok(())
    }

    /// Translates a Wasm `else` control flow instruction into `wasmi` bytecode.
    fn translate_else(
        &mut self,
        validator: &mut FunctionValidationContext,
        inst: &Instruction,
    ) -> Result<(), TranslationError> {
        validator.step(inst)?;
        let top_frame = self.pop_control_frame();
        let (else_label, end_label) = match top_frame {
            ControlFrame::If { else_label, end_label } => (else_label, end_label),
            unexpected => unreachable!(
                "expect Wasm `if` control flow frame at this point due to validation but found: {:?}",
                unexpected,
            ),
        };
        let dst_pc = self.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
        self.inst_builder
            .branch(Target::new(dst_pc, DropKeep::new(0, 0)));
        self.inst_builder.resolve_label(else_label);
        self.control_frames.push(ControlFrame::Else { end_label });
        Ok(())
    }

    /// Translates a Wasm `end` control flow instruction into `wasmi` bytecode.
    fn translate_end(
        &mut self,
        validator: &mut FunctionValidationContext,
        inst: &Instruction,
    ) -> Result<(), TranslationError> {
        let started_with = top_label(&validator.frame_stack).started_with;
        let return_drop_keep = if validator.frame_stack.len() == 1 {
            // We are about to close the last frame.
            Some(utils::drop_keep_return(
                &validator.locals,
                &validator.value_stack,
                &validator.frame_stack,
            ))
        } else {
            None
        };
        validator.step(inst)?;
        let top_frame = self.pop_control_frame();
        if let ControlFrame::If { else_label, .. } = top_frame {
            // At this point we can resolve the `Else` label.
            self.inst_builder.resolve_label(else_label);
        }
        if started_with != StartedWith::Loop {
            let end_label = top_frame.end_label();
            self.inst_builder.resolve_label(end_label);
        }
        if let Some(drop_keep) = return_drop_keep {
            // It was the last instruction therefore we emit the explicit return instruction.
            let drop_keep = drop_keep.unwrap_or_else(|error| {
                panic!(
                    "due to validation the value stack must not have underflowed. \
                            Validation also ensures that the frame stack is not empty: {:?}",
                    error
                )
            });
            self.inst_builder.ret(drop_keep);
        }
        Ok(())
    }

    /// Translates a Wasm `br` control flow instruction into `wasmi` bytecode.
    fn translate_br(
        &mut self,
        depth: &u32,
        validator: &mut FunctionValidationContext,
        inst: &Instruction,
    ) -> Result<(), TranslationError> {
        let target = utils::require_target(
            *depth,
            validator.value_stack.len(),
            &validator.frame_stack,
            &self.control_frames,
        );
        validator.step(inst)?;
        let (end_label, drop_keep) = target.unwrap_or_else(|error| {
            panic!(
                "due to validation the value stack must not underflow \
                        and the branching depth is valid at this point: {:?}",
                error
            )
        });
        let dst_pc = self.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
        self.inst_builder.branch(Target::new(dst_pc, drop_keep));
        Ok(())
    }

    /// Translates a Wasm `br_if` control flow instruction into `wasmi` bytecode.
    fn translate_br_if(
        &mut self,
        validator: &mut FunctionValidationContext,
        inst: &Instruction,
        depth: &u32,
    ) -> Result<(), TranslationError> {
        validator.step(inst)?;
        let (end_label, drop_keep) = utils::require_target(
            *depth,
            validator.value_stack.len(),
            &validator.frame_stack,
            &self.control_frames,
        )
        .unwrap_or_else(|error| {
            panic!(
                "due to validation the value stack must not underflow \
                        and the branching depth is valid at this point: {:?}",
                error
            )
        });
        let dst_pc = self.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
        self.inst_builder.branch_nez(Target::new(dst_pc, drop_keep));
        Ok(())
    }

    /// Translates a Wasm `br_table` control flow instruction into `wasmi` bytecode.
    fn translate_br_table(
        &mut self,
        validator: &mut FunctionValidationContext,
        br_table: &pwasm::BrTableData,
        inst: &Instruction,
    ) -> Result<(), TranslationError> {
        // At this point, the condition value is at the top of the stack.
        // But at the point of actual jump the condition will already be
        // popped off.
        let value_stack_height = validator.value_stack.len().saturating_sub(1);
        let targets = br_table
            .table
            .iter()
            .map(|depth| {
                utils::require_target(
                    *depth,
                    value_stack_height,
                    &validator.frame_stack,
                    &self.control_frames,
                )
            })
            .collect::<Result<Vec<_>, _>>();
        let default_target = utils::require_target(
            br_table.default,
            value_stack_height,
            &validator.frame_stack,
            &self.control_frames,
        );
        validator.step(inst)?;
        const REQUIRE_TARGET_PROOF: &str = "could not resolve targets or default target of the \
                    `br_table` even though it validated properly";
        let targets = targets.unwrap_or_else(|error| panic!("{}: {}", REQUIRE_TARGET_PROOF, error));
        let default_target =
            default_target.unwrap_or_else(|error| panic!("{}: {}", REQUIRE_TARGET_PROOF, error));
        let mut branch_arm_target = |index, label, drop_keep| {
            let dst_pc = self.try_resolve_label(label, |pc| Reloc::BrTable {
                inst_idx: pc,
                target_idx: index,
            });
            Target::new(dst_pc, drop_keep)
        };
        let targets = targets
            .into_iter()
            .enumerate()
            .map(|(target_idx, (label, drop_keep))| branch_arm_target(target_idx, label, drop_keep))
            .collect::<Vec<_>>();
        let default_target = {
            let (label_idx, drop_keep) = default_target;
            branch_arm_target(targets.len(), label_idx, drop_keep)
        };
        self.inst_builder.branch_table(default_target, targets);
        Ok(())
    }

    /// Translates a Wasm `return` control flow instruction into `wasmi` bytecode.
    fn translate_return(
        &mut self,
        validator: &mut FunctionValidationContext,
        inst: &Instruction,
    ) -> Result<(), TranslationError> {
        let drop_keep = utils::drop_keep_return(
            &validator.locals,
            &validator.value_stack,
            &validator.frame_stack,
        );
        validator.step(inst)?;
        let drop_keep = drop_keep.unwrap_or_else(|error| {
            panic!(
                "due to validation the value stack must not have underflowed. \
                         Validation also ensures that the frame stack is not empty: {:?}",
                error
            )
        });
        self.inst_builder.ret(drop_keep);
        Ok(())
    }
}
