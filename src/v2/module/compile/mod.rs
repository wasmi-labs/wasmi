//! Definitions to compile a Wasm module into `wasmi` bytecode.
//!
//! The implementation is specific to the underlying Wasm parser
//! framework used by `wasmi` which currently is `parity_wasm`.

mod control_frame;
use super::{
    super::{FuncBody, InstructionsBuilder},
    Engine,
};
use parity_wasm::elements::{self as pwasm, Instruction};

/// An error that may occur upon translating Wasm to `wasmi` bytecode.
#[derive(Debug)]
pub enum TranslationError {
    Validation,
}

/// A unique label identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LabelIdx(usize);


/// Allows to translate a Wasm functions into `wasmi` bytecode.
#[derive(Debug)]
pub struct FuncBodyTranslator {
    /// The underlying engine which the translator feeds.
    engine: Engine,
    inst_builder: InstructionsBuilder,
}

impl FuncBodyTranslator {
    /// Creates a new Wasm function body translator for the given [`Engine`].
    pub fn new(engine: &Engine) -> Self {
        Self {
            engine: engine.clone(),
            inst_builder: InstructionsBuilder::default(),
        }
    }

    /// Translates the instructions forming a Wasm function body into `wasmi` bytecode.
    ///
    /// Returns a [`FuncBody`] reference to the translated `wasmi` bytecode.
    pub fn translate<'a, I: 'a>(&mut self, instructions: I) -> Result<FuncBody, TranslationError>
    where
        I: IntoIterator<Item = &'a pwasm::Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        for instruction in instructions {
            self.translate_instruction(instruction)?;
        }
        let func_body = self.inst_builder.finish(&self.engine);
        Ok(func_body)
    }

    fn translate_instruction(&mut self, instruction: &Instruction) -> Result<(), TranslationError> {
        use Instruction as Inst;
        match instruction {
            Inst::Unreachable => todo!(),
            Inst::Nop => todo!(),
            Inst::Block(_block_type) => todo!(),
            Inst::Loop(_block_type) => todo!(),
            Inst::If(_block_type) => todo!(),
            Inst::Else => todo!(),
            Inst::End => todo!(),
            Inst::Br(_target) => todo!(),
            Inst::BrIf(_target) => todo!(),
            Inst::BrTable(_br_table) => todo!(),
            Inst::Return => todo!(),
            Inst::Call(_func_idx) => todo!(),
            Inst::CallIndirect(_signature_idx, _table_ref) => todo!(),
            Inst::Drop => todo!(),
            Inst::Select => todo!(),
            Inst::GetLocal(_local_idx) => todo!(),
            Inst::SetLocal(_local_idx) => todo!(),
            Inst::TeeLocal(_local_idx) => todo!(),
            Inst::GetGlobal(_global_idx) => todo!(),
            Inst::SetGlobal(_global_idx) => todo!(),
            Inst::I32Load(_, _) => todo!(),
            Inst::I64Load(_, _) => todo!(),
            Inst::F32Load(_, _) => todo!(),
            Inst::F64Load(_, _) => todo!(),
            Inst::I32Load8S(_, _) => todo!(),
            Inst::I32Load8U(_, _) => todo!(),
            Inst::I32Load16S(_, _) => todo!(),
            Inst::I32Load16U(_, _) => todo!(),
            Inst::I64Load8S(_, _) => todo!(),
            Inst::I64Load8U(_, _) => todo!(),
            Inst::I64Load16S(_, _) => todo!(),
            Inst::I64Load16U(_, _) => todo!(),
            Inst::I64Load32S(_, _) => todo!(),
            Inst::I64Load32U(_, _) => todo!(),
            Inst::I32Store(_, _) => todo!(),
            Inst::I64Store(_, _) => todo!(),
            Inst::F32Store(_, _) => todo!(),
            Inst::F64Store(_, _) => todo!(),
            Inst::I32Store8(_, _) => todo!(),
            Inst::I32Store16(_, _) => todo!(),
            Inst::I64Store8(_, _) => todo!(),
            Inst::I64Store16(_, _) => todo!(),
            Inst::I64Store32(_, _) => todo!(),
            Inst::CurrentMemory(_) => todo!(),
            Inst::GrowMemory(_) => todo!(),
            Inst::I32Const(_) => todo!(),
            Inst::I64Const(_) => todo!(),
            Inst::F32Const(_) => todo!(),
            Inst::F64Const(_) => todo!(),
            Inst::I32Eqz => todo!(),
            Inst::I32Eq => todo!(),
            Inst::I32Ne => todo!(),
            Inst::I32LtS => todo!(),
            Inst::I32LtU => todo!(),
            Inst::I32GtS => todo!(),
            Inst::I32GtU => todo!(),
            Inst::I32LeS => todo!(),
            Inst::I32LeU => todo!(),
            Inst::I32GeS => todo!(),
            Inst::I32GeU => todo!(),
            Inst::I64Eqz => todo!(),
            Inst::I64Eq => todo!(),
            Inst::I64Ne => todo!(),
            Inst::I64LtS => todo!(),
            Inst::I64LtU => todo!(),
            Inst::I64GtS => todo!(),
            Inst::I64GtU => todo!(),
            Inst::I64LeS => todo!(),
            Inst::I64LeU => todo!(),
            Inst::I64GeS => todo!(),
            Inst::I64GeU => todo!(),
            Inst::F32Eq => todo!(),
            Inst::F32Ne => todo!(),
            Inst::F32Lt => todo!(),
            Inst::F32Gt => todo!(),
            Inst::F32Le => todo!(),
            Inst::F32Ge => todo!(),
            Inst::F64Eq => todo!(),
            Inst::F64Ne => todo!(),
            Inst::F64Lt => todo!(),
            Inst::F64Gt => todo!(),
            Inst::F64Le => todo!(),
            Inst::F64Ge => todo!(),
            Inst::I32Clz => todo!(),
            Inst::I32Ctz => todo!(),
            Inst::I32Popcnt => todo!(),
            Inst::I32Add => todo!(),
            Inst::I32Sub => todo!(),
            Inst::I32Mul => todo!(),
            Inst::I32DivS => todo!(),
            Inst::I32DivU => todo!(),
            Inst::I32RemS => todo!(),
            Inst::I32RemU => todo!(),
            Inst::I32And => todo!(),
            Inst::I32Or => todo!(),
            Inst::I32Xor => todo!(),
            Inst::I32Shl => todo!(),
            Inst::I32ShrS => todo!(),
            Inst::I32ShrU => todo!(),
            Inst::I32Rotl => todo!(),
            Inst::I32Rotr => todo!(),
            Inst::I64Clz => todo!(),
            Inst::I64Ctz => todo!(),
            Inst::I64Popcnt => todo!(),
            Inst::I64Add => todo!(),
            Inst::I64Sub => todo!(),
            Inst::I64Mul => todo!(),
            Inst::I64DivS => todo!(),
            Inst::I64DivU => todo!(),
            Inst::I64RemS => todo!(),
            Inst::I64RemU => todo!(),
            Inst::I64And => todo!(),
            Inst::I64Or => todo!(),
            Inst::I64Xor => todo!(),
            Inst::I64Shl => todo!(),
            Inst::I64ShrS => todo!(),
            Inst::I64ShrU => todo!(),
            Inst::I64Rotl => todo!(),
            Inst::I64Rotr => todo!(),
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
    }
}
