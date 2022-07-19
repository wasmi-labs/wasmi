use crate::{
    context::ModuleContext,
    stack::StackWithLimit,
    util::Locals,
    Error,
    FuncValidator,
    DEFAULT_MEMORY_INDEX,
    DEFAULT_TABLE_INDEX,
};

use core::u32;
use parity_wasm::elements::{BlockType, Func, FuncBody, Instruction, TableElementType, ValueType};

/// Maximum number of entries in value stack per function.
const DEFAULT_VALUE_STACK_LIMIT: usize = 16384;
/// Maximum number of entries in frame stack per function.
const DEFAULT_FRAME_STACK_LIMIT: usize = 16384;

/// Control stack frame.
#[derive(Debug, Clone)]
pub struct BlockFrame {
    /// The opcode that started this block frame.
    pub started_with: StartedWith,
    /// A signature, which is a block signature type indicating the number and types of result
    /// values of the region.
    pub block_type: BlockType,
    /// A limit integer value, which is an index into the value stack indicating where to reset it
    /// to on a branch to that label.
    pub value_stack_len: usize,
    /// Boolean which signals whether value stack became polymorphic. Value stack starts in
    /// a non-polymorphic state and becomes polymorphic only after an instruction that never passes
    /// control further is executed, i.e. `unreachable`, `br` (but not `br_if`!), etc.
    pub polymorphic_stack: bool,
}

/// An opcode that opened the particular frame.
///
/// We need that to ensure proper combinations with `End` instruction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StartedWith {
    Block,
    If,
    Else,
    Loop,
}

/// Value type on the stack.
#[derive(Debug, Clone, Copy)]
pub enum StackValueType {
    /// Any value type.
    Any,
    /// Concrete value type.
    Specific(ValueType),
}

impl From<ValueType> for StackValueType {
    fn from(value_type: ValueType) -> Self {
        StackValueType::Specific(value_type)
    }
}

impl PartialEq<StackValueType> for StackValueType {
    fn eq(&self, other: &StackValueType) -> bool {
        match (*self, *other) {
            // Any type is equal with any other type.
            (StackValueType::Any, _) => true,
            (_, StackValueType::Any) => true,
            (StackValueType::Specific(self_ty), StackValueType::Specific(other_ty)) => {
                self_ty == other_ty
            }
        }
    }
}

impl PartialEq<ValueType> for StackValueType {
    fn eq(&self, other: &ValueType) -> bool {
        match *self {
            StackValueType::Any => true,
            StackValueType::Specific(value_ty) => value_ty == *other,
        }
    }
}

impl PartialEq<StackValueType> for ValueType {
    fn eq(&self, other: &StackValueType) -> bool {
        other == self
    }
}

pub fn drive<T: FuncValidator>(
    module: &ModuleContext,
    func: &Func,
    body: &FuncBody,
    input: <T as FuncValidator>::Input,
) -> Result<T::Output, Error> {
    let (params, result_ty) = module.require_function_type(func.type_ref())?;

    let code = body.code().elements();
    let code_len = code.len();
    if code_len == 0 {
        return Err(Error("Non-empty function body expected".into()));
    }

    let mut context = FunctionValidationContext::new(
        module,
        Locals::new(params, body.locals())?,
        DEFAULT_VALUE_STACK_LIMIT,
        DEFAULT_FRAME_STACK_LIMIT,
        result_ty,
    )?;

    let mut validator = T::new(&context, body, input);

    for (position, instruction) in code.iter().enumerate() {
        validator
            .next_instruction(&mut context, instruction)
            .map_err(|err| {
                Error(format!(
                    "At instruction {:?}(@{}): {}",
                    instruction, position, err
                ))
            })?;
    }

    // The last `end` opcode should pop last instruction.
    // parity-wasm ensures that there is always `End` opcode at
    // the end of the function body.
    assert!(context.frame_stack.is_empty());

    Ok(validator.finish(&context))
}

/// Function validation context.
pub struct FunctionValidationContext<'a> {
    /// Wasm module
    pub module: &'a ModuleContext,
    /// Local variables.
    pub locals: Locals<'a>,
    /// Value stack.
    pub value_stack: StackWithLimit<StackValueType>,
    /// Frame stack.
    pub frame_stack: StackWithLimit<BlockFrame>,
    /// Function return type.
    pub return_type: BlockType,
}

impl<'a> FunctionValidationContext<'a> {
    fn new(
        module: &'a ModuleContext,
        locals: Locals<'a>,
        value_stack_limit: usize,
        frame_stack_limit: usize,
        return_type: BlockType,
    ) -> Result<Self, Error> {
        let mut ctx = FunctionValidationContext {
            module,
            locals,
            value_stack: StackWithLimit::with_limit(value_stack_limit),
            frame_stack: StackWithLimit::with_limit(frame_stack_limit),
            return_type,
        };
        push_label(
            StartedWith::Block,
            return_type,
            &ctx.value_stack,
            &mut ctx.frame_stack,
        )?;
        Ok(ctx)
    }

    pub fn step(&mut self, instruction: &Instruction) -> Result<(), Error> {
        use self::Instruction::*;

        match *instruction {
            // Nop instruction doesn't do anything. It is safe to just skip it.
            Nop => {}

            Unreachable => {
                make_top_frame_polymorphic(&mut self.value_stack, &mut self.frame_stack);
            }

            Block(block_type) => {
                push_label(
                    StartedWith::Block,
                    block_type,
                    &self.value_stack,
                    &mut self.frame_stack,
                )?;
            }
            Loop(block_type) => {
                push_label(
                    StartedWith::Loop,
                    block_type,
                    &self.value_stack,
                    &mut self.frame_stack,
                )?;
            }
            If(block_type) => {
                pop_value(
                    &mut self.value_stack,
                    &self.frame_stack,
                    ValueType::I32.into(),
                )?;
                push_label(
                    StartedWith::If,
                    block_type,
                    &self.value_stack,
                    &mut self.frame_stack,
                )?;
            }
            Else => {
                let block_type = {
                    let top = top_label(&self.frame_stack);
                    if top.started_with != StartedWith::If {
                        return Err(Error("Misplaced else instruction".into()));
                    }
                    top.block_type
                };

                // Then, we pop the current label. It discards all values that pushed in the current
                // frame.
                pop_label(&mut self.value_stack, &mut self.frame_stack)?;
                push_label(
                    StartedWith::Else,
                    block_type,
                    &self.value_stack,
                    &mut self.frame_stack,
                )?;
            }
            End => {
                let block_type = {
                    let top = top_label(&self.frame_stack);

                    if top.started_with == StartedWith::If && top.block_type != BlockType::NoResult
                    {
                        // A `if` without an `else` can't return a result.
                        return Err(Error(format!(
                            "If block without else required to have NoResult block type. But it has {:?} type",
                            top.block_type
                        )));
                    }

                    top.block_type
                };

                // Ignore clippy as pop(..) != pop(..) + push_value(..) under some conditions
                if self.frame_stack.len() == 1 {
                    // We are about to close the last frame.

                    // Check the return type.
                    if let BlockType::Value(value_type) = self.return_type {
                        tee_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
                    }

                    pop_label(&mut self.value_stack, &mut self.frame_stack)?;

                // We just popped the last frame. To avoid some difficulties
                // we prefer to keep this branch explicit, bail out here, thus
                // returning `()`.
                } else {
                    pop_label(&mut self.value_stack, &mut self.frame_stack)?;

                    // Push the result value.
                    if let BlockType::Value(value_type) = block_type {
                        push_value(&mut self.value_stack, value_type.into())?;
                    }
                }
            }
            Br(depth) => {
                self.validate_br(depth)?;
                make_top_frame_polymorphic(&mut self.value_stack, &mut self.frame_stack);
            }
            BrIf(depth) => {
                self.validate_br_if(depth)?;
            }
            BrTable(ref br_table_data) => {
                self.validate_br_table(&br_table_data.table, br_table_data.default)?;
                make_top_frame_polymorphic(&mut self.value_stack, &mut self.frame_stack);
            }
            Return => {
                if let BlockType::Value(value_type) = self.return_type {
                    tee_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
                }
                make_top_frame_polymorphic(&mut self.value_stack, &mut self.frame_stack);
            }

            Call(index) => {
                self.validate_call(index)?;
            }
            CallIndirect(index, _reserved) => {
                self.validate_call_indirect(index)?;
            }

            Drop => {
                self.validate_drop()?;
            }
            Select => {
                self.validate_select()?;
            }

            GetLocal(index) => {
                self.validate_get_local(index)?;
            }
            SetLocal(index) => {
                self.validate_set_local(index)?;
            }
            TeeLocal(index) => {
                self.validate_tee_local(index)?;
            }
            GetGlobal(index) => {
                self.validate_get_global(index)?;
            }
            SetGlobal(index) => {
                self.validate_set_global(index)?;
            }

            I32Load(align, _) => {
                self.validate_load(align, 4, ValueType::I32)?;
            }
            I64Load(align, _) => {
                self.validate_load(align, 8, ValueType::I64)?;
            }
            F32Load(align, _) => {
                self.validate_load(align, 4, ValueType::F32)?;
            }
            F64Load(align, _) => {
                self.validate_load(align, 8, ValueType::F64)?;
            }
            I32Load8S(align, _) => {
                self.validate_load(align, 1, ValueType::I32)?;
            }
            I32Load8U(align, _) => {
                self.validate_load(align, 1, ValueType::I32)?;
            }
            I32Load16S(align, _) => {
                self.validate_load(align, 2, ValueType::I32)?;
            }
            I32Load16U(align, _) => {
                self.validate_load(align, 2, ValueType::I32)?;
            }
            I64Load8S(align, _) => {
                self.validate_load(align, 1, ValueType::I64)?;
            }
            I64Load8U(align, _) => {
                self.validate_load(align, 1, ValueType::I64)?;
            }
            I64Load16S(align, _) => {
                self.validate_load(align, 2, ValueType::I64)?;
            }
            I64Load16U(align, _) => {
                self.validate_load(align, 2, ValueType::I64)?;
            }
            I64Load32S(align, _) => {
                self.validate_load(align, 4, ValueType::I64)?;
            }
            I64Load32U(align, _) => {
                self.validate_load(align, 4, ValueType::I64)?;
            }

            I32Store(align, _) => {
                self.validate_store(align, 4, ValueType::I32)?;
            }
            I64Store(align, _) => {
                self.validate_store(align, 8, ValueType::I64)?;
            }
            F32Store(align, _) => {
                self.validate_store(align, 4, ValueType::F32)?;
            }
            F64Store(align, _) => {
                self.validate_store(align, 8, ValueType::F64)?;
            }
            I32Store8(align, _) => {
                self.validate_store(align, 1, ValueType::I32)?;
            }
            I32Store16(align, _) => {
                self.validate_store(align, 2, ValueType::I32)?;
            }
            I64Store8(align, _) => {
                self.validate_store(align, 1, ValueType::I64)?;
            }
            I64Store16(align, _) => {
                self.validate_store(align, 2, ValueType::I64)?;
            }
            I64Store32(align, _) => {
                self.validate_store(align, 4, ValueType::I64)?;
            }

            CurrentMemory(_) => {
                self.validate_current_memory()?;
            }
            GrowMemory(_) => {
                self.validate_grow_memory()?;
            }

            I32Const(_) => {
                self.validate_const(ValueType::I32)?;
            }
            I64Const(_) => {
                self.validate_const(ValueType::I64)?;
            }
            F32Const(_) => {
                self.validate_const(ValueType::F32)?;
            }
            F64Const(_) => {
                self.validate_const(ValueType::F64)?;
            }

            I32Eqz => {
                self.validate_testop(ValueType::I32)?;
            }
            I32Eq => {
                self.validate_relop(ValueType::I32)?;
            }
            I32Ne => {
                self.validate_relop(ValueType::I32)?;
            }
            I32LtS => {
                self.validate_relop(ValueType::I32)?;
            }
            I32LtU => {
                self.validate_relop(ValueType::I32)?;
            }
            I32GtS => {
                self.validate_relop(ValueType::I32)?;
            }
            I32GtU => {
                self.validate_relop(ValueType::I32)?;
            }
            I32LeS => {
                self.validate_relop(ValueType::I32)?;
            }
            I32LeU => {
                self.validate_relop(ValueType::I32)?;
            }
            I32GeS => {
                self.validate_relop(ValueType::I32)?;
            }
            I32GeU => {
                self.validate_relop(ValueType::I32)?;
            }

            I64Eqz => {
                self.validate_testop(ValueType::I64)?;
            }
            I64Eq => {
                self.validate_relop(ValueType::I64)?;
            }
            I64Ne => {
                self.validate_relop(ValueType::I64)?;
            }
            I64LtS => {
                self.validate_relop(ValueType::I64)?;
            }
            I64LtU => {
                self.validate_relop(ValueType::I64)?;
            }
            I64GtS => {
                self.validate_relop(ValueType::I64)?;
            }
            I64GtU => {
                self.validate_relop(ValueType::I64)?;
            }
            I64LeS => {
                self.validate_relop(ValueType::I64)?;
            }
            I64LeU => {
                self.validate_relop(ValueType::I64)?;
            }
            I64GeS => {
                self.validate_relop(ValueType::I64)?;
            }
            I64GeU => {
                self.validate_relop(ValueType::I64)?;
            }

            F32Eq => {
                self.validate_relop(ValueType::F32)?;
            }
            F32Ne => {
                self.validate_relop(ValueType::F32)?;
            }
            F32Lt => {
                self.validate_relop(ValueType::F32)?;
            }
            F32Gt => {
                self.validate_relop(ValueType::F32)?;
            }
            F32Le => {
                self.validate_relop(ValueType::F32)?;
            }
            F32Ge => {
                self.validate_relop(ValueType::F32)?;
            }

            F64Eq => {
                self.validate_relop(ValueType::F64)?;
            }
            F64Ne => {
                self.validate_relop(ValueType::F64)?;
            }
            F64Lt => {
                self.validate_relop(ValueType::F64)?;
            }
            F64Gt => {
                self.validate_relop(ValueType::F64)?;
            }
            F64Le => {
                self.validate_relop(ValueType::F64)?;
            }
            F64Ge => {
                self.validate_relop(ValueType::F64)?;
            }

            I32Clz => {
                self.validate_unop(ValueType::I32)?;
            }
            I32Ctz => {
                self.validate_unop(ValueType::I32)?;
            }
            I32Popcnt => {
                self.validate_unop(ValueType::I32)?;
            }
            I32Add => {
                self.validate_binop(ValueType::I32)?;
            }
            I32Sub => {
                self.validate_binop(ValueType::I32)?;
            }
            I32Mul => {
                self.validate_binop(ValueType::I32)?;
            }
            I32DivS => {
                self.validate_binop(ValueType::I32)?;
            }
            I32DivU => {
                self.validate_binop(ValueType::I32)?;
            }
            I32RemS => {
                self.validate_binop(ValueType::I32)?;
            }
            I32RemU => {
                self.validate_binop(ValueType::I32)?;
            }
            I32And => {
                self.validate_binop(ValueType::I32)?;
            }
            I32Or => {
                self.validate_binop(ValueType::I32)?;
            }
            I32Xor => {
                self.validate_binop(ValueType::I32)?;
            }
            I32Shl => {
                self.validate_binop(ValueType::I32)?;
            }
            I32ShrS => {
                self.validate_binop(ValueType::I32)?;
            }
            I32ShrU => {
                self.validate_binop(ValueType::I32)?;
            }
            I32Rotl => {
                self.validate_binop(ValueType::I32)?;
            }
            I32Rotr => {
                self.validate_binop(ValueType::I32)?;
            }

            I64Clz => {
                self.validate_unop(ValueType::I64)?;
            }
            I64Ctz => {
                self.validate_unop(ValueType::I64)?;
            }
            I64Popcnt => {
                self.validate_unop(ValueType::I64)?;
            }
            I64Add => {
                self.validate_binop(ValueType::I64)?;
            }
            I64Sub => {
                self.validate_binop(ValueType::I64)?;
            }
            I64Mul => {
                self.validate_binop(ValueType::I64)?;
            }
            I64DivS => {
                self.validate_binop(ValueType::I64)?;
            }
            I64DivU => {
                self.validate_binop(ValueType::I64)?;
            }
            I64RemS => {
                self.validate_binop(ValueType::I64)?;
            }
            I64RemU => {
                self.validate_binop(ValueType::I64)?;
            }
            I64And => {
                self.validate_binop(ValueType::I64)?;
            }
            I64Or => {
                self.validate_binop(ValueType::I64)?;
            }
            I64Xor => {
                self.validate_binop(ValueType::I64)?;
            }
            I64Shl => {
                self.validate_binop(ValueType::I64)?;
            }
            I64ShrS => {
                self.validate_binop(ValueType::I64)?;
            }
            I64ShrU => {
                self.validate_binop(ValueType::I64)?;
            }
            I64Rotl => {
                self.validate_binop(ValueType::I64)?;
            }
            I64Rotr => {
                self.validate_binop(ValueType::I64)?;
            }

            F32Abs => {
                self.validate_unop(ValueType::F32)?;
            }
            F32Neg => {
                self.validate_unop(ValueType::F32)?;
            }
            F32Ceil => {
                self.validate_unop(ValueType::F32)?;
            }
            F32Floor => {
                self.validate_unop(ValueType::F32)?;
            }
            F32Trunc => {
                self.validate_unop(ValueType::F32)?;
            }
            F32Nearest => {
                self.validate_unop(ValueType::F32)?;
            }
            F32Sqrt => {
                self.validate_unop(ValueType::F32)?;
            }
            F32Add => {
                self.validate_binop(ValueType::F32)?;
            }
            F32Sub => {
                self.validate_binop(ValueType::F32)?;
            }
            F32Mul => {
                self.validate_binop(ValueType::F32)?;
            }
            F32Div => {
                self.validate_binop(ValueType::F32)?;
            }
            F32Min => {
                self.validate_binop(ValueType::F32)?;
            }
            F32Max => {
                self.validate_binop(ValueType::F32)?;
            }
            F32Copysign => {
                self.validate_binop(ValueType::F32)?;
            }

            F64Abs => {
                self.validate_unop(ValueType::F64)?;
            }
            F64Neg => {
                self.validate_unop(ValueType::F64)?;
            }
            F64Ceil => {
                self.validate_unop(ValueType::F64)?;
            }
            F64Floor => {
                self.validate_unop(ValueType::F64)?;
            }
            F64Trunc => {
                self.validate_unop(ValueType::F64)?;
            }
            F64Nearest => {
                self.validate_unop(ValueType::F64)?;
            }
            F64Sqrt => {
                self.validate_unop(ValueType::F64)?;
            }
            F64Add => {
                self.validate_binop(ValueType::F64)?;
            }
            F64Sub => {
                self.validate_binop(ValueType::F64)?;
            }
            F64Mul => {
                self.validate_binop(ValueType::F64)?;
            }
            F64Div => {
                self.validate_binop(ValueType::F64)?;
            }
            F64Min => {
                self.validate_binop(ValueType::F64)?;
            }
            F64Max => {
                self.validate_binop(ValueType::F64)?;
            }
            F64Copysign => {
                self.validate_binop(ValueType::F64)?;
            }

            I32WrapI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::I32)?;
            }
            I32TruncSF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::I32)?;
            }
            I32TruncUF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::I32)?;
            }
            I32TruncSF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::I32)?;
            }
            I32TruncUF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::I32)?;
            }
            I64ExtendSI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::I64)?;
            }
            I64ExtendUI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::I64)?;
            }
            I64TruncSF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::I64)?;
            }
            I64TruncUF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::I64)?;
            }
            I64TruncSF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::I64)?;
            }
            I64TruncUF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::I64)?;
            }
            F32ConvertSI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::F32)?;
            }
            F32ConvertUI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::F32)?;
            }
            F32ConvertSI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::F32)?;
            }
            F32ConvertUI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::F32)?;
            }
            F32DemoteF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::F32)?;
            }
            F64ConvertSI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::F64)?;
            }
            F64ConvertUI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::F64)?;
            }
            F64ConvertSI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::F64)?;
            }
            F64ConvertUI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::F64)?;
            }
            F64PromoteF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::F64)?;
            }

            I32ReinterpretF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::I32)?;
            }
            I64ReinterpretF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::I64)?;
            }
            F32ReinterpretI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::F32)?;
            }
            F64ReinterpretI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::F64)?;
            }
        }

        Ok(())
    }

    fn validate_const(&mut self, value_type: ValueType) -> Result<(), Error> {
        push_value(&mut self.value_stack, value_type.into())?;
        Ok(())
    }

    fn validate_unop(&mut self, value_type: ValueType) -> Result<(), Error> {
        pop_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
        push_value(&mut self.value_stack, value_type.into())?;
        Ok(())
    }

    fn validate_binop(&mut self, value_type: ValueType) -> Result<(), Error> {
        pop_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
        pop_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
        push_value(&mut self.value_stack, value_type.into())?;
        Ok(())
    }

    fn validate_testop(&mut self, value_type: ValueType) -> Result<(), Error> {
        pop_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
        push_value(&mut self.value_stack, ValueType::I32.into())?;
        Ok(())
    }

    fn validate_relop(&mut self, value_type: ValueType) -> Result<(), Error> {
        pop_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
        pop_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
        push_value(&mut self.value_stack, ValueType::I32.into())?;
        Ok(())
    }

    fn validate_cvtop(
        &mut self,
        value_type1: ValueType,
        value_type2: ValueType,
    ) -> Result<(), Error> {
        pop_value(&mut self.value_stack, &self.frame_stack, value_type1.into())?;
        push_value(&mut self.value_stack, value_type2.into())?;
        Ok(())
    }

    fn validate_drop(&mut self) -> Result<(), Error> {
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            StackValueType::Any,
        )?;
        Ok(())
    }

    fn validate_select(&mut self) -> Result<(), Error> {
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            ValueType::I32.into(),
        )?;
        let select_type = pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            StackValueType::Any,
        )?;
        pop_value(&mut self.value_stack, &self.frame_stack, select_type)?;
        push_value(&mut self.value_stack, select_type)?;
        Ok(())
    }

    fn validate_get_local(&mut self, index: u32) -> Result<(), Error> {
        let local_type = require_local(&self.locals, index)?;
        push_value(&mut self.value_stack, local_type.into())?;
        Ok(())
    }

    fn validate_set_local(&mut self, index: u32) -> Result<(), Error> {
        let local_type = require_local(&self.locals, index)?;
        let value_type = pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            StackValueType::Any,
        )?;
        if local_type != value_type {
            return Err(Error(format!(
                "Trying to update local {} of type {:?} with value of type {:?}",
                index, local_type, value_type
            )));
        }
        Ok(())
    }

    fn validate_tee_local(&mut self, index: u32) -> Result<(), Error> {
        let local_type = require_local(&self.locals, index)?;
        tee_value(&mut self.value_stack, &self.frame_stack, local_type.into())?;
        Ok(())
    }

    fn validate_get_global(&mut self, index: u32) -> Result<(), Error> {
        let global_type: StackValueType = {
            let global = self.module.require_global(index, None)?;
            global.content_type().into()
        };
        push_value(&mut self.value_stack, global_type)?;
        Ok(())
    }

    fn validate_set_global(&mut self, index: u32) -> Result<(), Error> {
        let global_type: StackValueType = {
            let global = self.module.require_global(index, Some(true))?;
            global.content_type().into()
        };
        let value_type = pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            StackValueType::Any,
        )?;
        if global_type != value_type {
            return Err(Error(format!(
                "Trying to update global {} of type {:?} with value of type {:?}",
                index, global_type, value_type
            )));
        }
        Ok(())
    }

    fn validate_load(
        &mut self,
        align: u32,
        max_align: u32,
        value_type: ValueType,
    ) -> Result<(), Error> {
        if 1u32.checked_shl(align).unwrap_or(u32::MAX) > max_align {
            return Err(Error(format!(
                "Too large memory alignment 2^{} (expected at most {})",
                align, max_align
            )));
        }

        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            ValueType::I32.into(),
        )?;
        self.module.require_memory(DEFAULT_MEMORY_INDEX)?;
        push_value(&mut self.value_stack, value_type.into())?;
        Ok(())
    }

    fn validate_store(
        &mut self,
        align: u32,
        max_align: u32,
        value_type: ValueType,
    ) -> Result<(), Error> {
        if 1u32.checked_shl(align).unwrap_or(u32::MAX) > max_align {
            return Err(Error(format!(
                "Too large memory alignment 2^{} (expected at most {})",
                align, max_align
            )));
        }

        self.module.require_memory(DEFAULT_MEMORY_INDEX)?;
        pop_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            ValueType::I32.into(),
        )?;
        Ok(())
    }

    fn validate_br(&mut self, depth: u32) -> Result<(), Error> {
        let (started_with, frame_block_type) = {
            let frame = require_label(depth, &self.frame_stack)?;
            (frame.started_with, frame.block_type)
        };
        if started_with != StartedWith::Loop {
            if let BlockType::Value(value_type) = frame_block_type {
                tee_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
            }
        }
        Ok(())
    }

    fn validate_br_if(&mut self, depth: u32) -> Result<(), Error> {
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            ValueType::I32.into(),
        )?;

        let (started_with, frame_block_type) = {
            let frame = require_label(depth, &self.frame_stack)?;
            (frame.started_with, frame.block_type)
        };
        if started_with != StartedWith::Loop {
            if let BlockType::Value(value_type) = frame_block_type {
                tee_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
            }
        }
        Ok(())
    }

    fn validate_br_table(&mut self, table: &[u32], default: u32) -> Result<(), Error> {
        let required_block_type: BlockType = {
            let default_block = require_label(default, &self.frame_stack)?;
            let required_block_type = if default_block.started_with == StartedWith::Loop {
                BlockType::NoResult
            } else {
                default_block.block_type
            };

            for label in table {
                let label_block = require_label(*label, &self.frame_stack)?;
                let label_block_type = if label_block.started_with == StartedWith::Loop {
                    BlockType::NoResult
                } else {
                    label_block.block_type
                };
                if required_block_type != label_block_type {
                    return Err(Error(format!(
                        "Labels in br_table points to block of different types: {:?} and {:?}",
                        required_block_type, label_block.block_type
                    )));
                }
            }
            required_block_type
        };

        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            ValueType::I32.into(),
        )?;
        if let BlockType::Value(value_type) = required_block_type {
            tee_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
        }

        Ok(())
    }

    fn validate_call(&mut self, idx: u32) -> Result<(), Error> {
        let (argument_types, return_type) = self.module.require_function(idx)?;
        for argument_type in argument_types.iter().rev() {
            pop_value(
                &mut self.value_stack,
                &self.frame_stack,
                (*argument_type).into(),
            )?;
        }
        if let BlockType::Value(value_type) = return_type {
            push_value(&mut self.value_stack, value_type.into())?;
        }
        Ok(())
    }

    fn validate_call_indirect(&mut self, idx: u32) -> Result<(), Error> {
        {
            let table = self.module.require_table(DEFAULT_TABLE_INDEX)?;
            if table.elem_type() != TableElementType::AnyFunc {
                return Err(Error(format!(
                    "Table {} has element type {:?} while `anyfunc` expected",
                    idx,
                    table.elem_type()
                )));
            }
        }

        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            ValueType::I32.into(),
        )?;
        let (argument_types, return_type) = self.module.require_function_type(idx)?;
        for argument_type in argument_types.iter().rev() {
            pop_value(
                &mut self.value_stack,
                &self.frame_stack,
                (*argument_type).into(),
            )?;
        }
        if let BlockType::Value(value_type) = return_type {
            push_value(&mut self.value_stack, value_type.into())?;
        }
        Ok(())
    }

    fn validate_current_memory(&mut self) -> Result<(), Error> {
        self.module.require_memory(DEFAULT_MEMORY_INDEX)?;
        push_value(&mut self.value_stack, ValueType::I32.into())?;
        Ok(())
    }

    fn validate_grow_memory(&mut self) -> Result<(), Error> {
        self.module.require_memory(DEFAULT_MEMORY_INDEX)?;
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            ValueType::I32.into(),
        )?;
        push_value(&mut self.value_stack, ValueType::I32.into())?;
        Ok(())
    }
}

fn make_top_frame_polymorphic(
    value_stack: &mut StackWithLimit<StackValueType>,
    frame_stack: &mut StackWithLimit<BlockFrame>,
) {
    let frame = frame_stack
        .top_mut()
        .expect("make_top_frame_polymorphic is called with empty frame stack");
    value_stack.resize(frame.value_stack_len, StackValueType::Any);
    frame.polymorphic_stack = true;
}

fn push_value(
    value_stack: &mut StackWithLimit<StackValueType>,
    value_type: StackValueType,
) -> Result<(), Error> {
    Ok(value_stack.push(value_type)?)
}

fn pop_value(
    value_stack: &mut StackWithLimit<StackValueType>,
    frame_stack: &StackWithLimit<BlockFrame>,
    expected_value_ty: StackValueType,
) -> Result<StackValueType, Error> {
    let (is_stack_polymorphic, label_value_stack_len) = {
        let frame = top_label(frame_stack);
        (frame.polymorphic_stack, frame.value_stack_len)
    };
    let stack_is_empty = value_stack.len() == label_value_stack_len;
    let actual_value = if stack_is_empty && is_stack_polymorphic {
        StackValueType::Any
    } else {
        let value_stack_min = frame_stack
            .top()
            .expect("at least 1 topmost block")
            .value_stack_len;
        if value_stack.len() <= value_stack_min {
            return Err(Error("Trying to access parent frame stack values.".into()));
        }
        value_stack.pop()?
    };
    match actual_value {
        StackValueType::Specific(stack_value_type) if stack_value_type == expected_value_ty => {
            Ok(actual_value)
        }
        StackValueType::Any => Ok(actual_value),
        stack_value_type => Err(Error(format!(
            "Expected value of type {:?} on top of stack. Got {:?}",
            expected_value_ty, stack_value_type
        ))),
    }
}

fn tee_value(
    value_stack: &mut StackWithLimit<StackValueType>,
    frame_stack: &StackWithLimit<BlockFrame>,
    value_type: StackValueType,
) -> Result<(), Error> {
    let _ = pop_value(value_stack, frame_stack, value_type)?;
    push_value(value_stack, value_type)?;
    Ok(())
}

fn push_label(
    started_with: StartedWith,
    block_type: BlockType,
    value_stack: &StackWithLimit<StackValueType>,
    frame_stack: &mut StackWithLimit<BlockFrame>,
) -> Result<(), Error> {
    Ok(frame_stack.push(BlockFrame {
        started_with,
        block_type,
        value_stack_len: value_stack.len(),
        polymorphic_stack: false,
    })?)
}

// TODO: Refactor
fn pop_label(
    value_stack: &mut StackWithLimit<StackValueType>,
    frame_stack: &mut StackWithLimit<BlockFrame>,
) -> Result<(), Error> {
    // Don't pop frame yet. This is essential since we still might pop values from the value stack
    // and this in turn requires current frame to check whether or not we've reached
    // unreachable.
    let block_type = frame_stack.top()?.block_type;
    match block_type {
        BlockType::NoResult => (),
        BlockType::Value(required_value_type) => {
            let _ = pop_value(
                value_stack,
                frame_stack,
                StackValueType::Specific(required_value_type),
            )?;
        }
    }

    let frame = frame_stack.pop()?;
    if value_stack.len() != frame.value_stack_len {
        return Err(Error(format!(
            "Unexpected stack height {}, expected {}",
            value_stack.len(),
            frame.value_stack_len
        )));
    }

    Ok(())
}

/// Returns the top most frame from the frame stack.
///
/// # Panics
///
/// Can be called only when the frame stack is not empty: that is, it is ok to call this function
/// after initialization of the validation and until the validation reached the latest `End`
/// operator.
pub fn top_label(frame_stack: &StackWithLimit<BlockFrame>) -> &BlockFrame {
    frame_stack
        .top()
        .expect("this function can't be called with empty frame stack")
}

pub fn require_label(
    depth: u32,
    frame_stack: &StackWithLimit<BlockFrame>,
) -> Result<&BlockFrame, Error> {
    Ok(frame_stack.get(depth as usize)?)
}

fn require_local(locals: &Locals, idx: u32) -> Result<ValueType, Error> {
    locals.type_of_local(idx)
}
