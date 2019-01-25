#[allow(unused_imports)]
use alloc::prelude::*;
use common::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX};
use core::u32;
use parity_wasm::elements::{BlockType, Func, FuncBody, Instruction, TableElementType, ValueType};
use validation::context::ModuleContext;

use validation::util::Locals;
use validation::Error;

use common::stack::StackWithLimit;
use isa;

/// Maximum number of entries in value stack per function.
const DEFAULT_VALUE_STACK_LIMIT: usize = 16384;
/// Maximum number of entries in frame stack per function.
const DEFAULT_FRAME_STACK_LIMIT: usize = 16384;

/// Control stack frame.
#[derive(Debug, Clone)]
struct BlockFrame {
    /// Frame type.
    frame_type: BlockFrameType,
    /// A signature, which is a block signature type indicating the number and types of result values of the region.
    block_type: BlockType,
    /// A label for reference to block instruction.
    begin_position: usize,
    /// A limit integer value, which is an index into the value stack indicating where to reset it to on a branch to that label.
    value_stack_len: usize,
    /// Boolean which signals whether value stack became polymorphic. Value stack starts in non-polymorphic state and
    /// becomes polymorphic only after an instruction that never passes control further is executed,
    /// i.e. `unreachable`, `br` (but not `br_if`!), etc.
    polymorphic_stack: bool,
}

/// Type of block frame.
#[derive(Debug, Clone, Copy, PartialEq)]
enum BlockFrameType {
    /// Usual block frame.
    ///
    /// Can be used for an implicit function block.
    Block { end_label: LabelId },
    /// Loop frame (branching to the beginning of block).
    Loop { header: LabelId },
    /// True-subblock of if expression.
    IfTrue {
        /// If jump happens inside the if-true block then control will
        /// land on this label.
        end_label: LabelId,

        /// If the condition of the `if` statement is unsatisfied, control
        /// will land on this label. This label might point to `else` block if it
        /// exists. Otherwise it equal to `end_label`.
        if_not: LabelId,
    },
    /// False-subblock of if expression.
    IfFalse { end_label: LabelId },
}

impl BlockFrameType {
    /// Returns a label which should be used as a branch destination.
    fn br_destination(&self) -> LabelId {
        match *self {
            BlockFrameType::Block { end_label } => end_label,
            BlockFrameType::Loop { header } => header,
            BlockFrameType::IfTrue { end_label, .. } => end_label,
            BlockFrameType::IfFalse { end_label } => end_label,
        }
    }

    /// Returns a label which should be resolved at the `End` opcode.
    ///
    /// All block types have it except loops. Loops doesn't use end as a branch
    /// destination.
    fn end_label(&self) -> LabelId {
        match *self {
            BlockFrameType::Block { end_label } => end_label,
            BlockFrameType::IfTrue { end_label, .. } => end_label,
            BlockFrameType::IfFalse { end_label } => end_label,
            BlockFrameType::Loop { .. } => panic!("loop doesn't use end label"),
        }
    }

    fn is_loop(&self) -> bool {
        match *self {
            BlockFrameType::Loop { .. } => true,
            _ => false,
        }
    }
}

/// Value type on the stack.
#[derive(Debug, Clone, Copy)]
enum StackValueType {
    /// Any value type.
    Any,
    /// Concrete value type.
    Specific(ValueType),
}

impl StackValueType {
    fn is_any(&self) -> bool {
        match self {
            &StackValueType::Any => true,
            _ => false,
        }
    }

    fn value_type(&self) -> ValueType {
        match self {
            &StackValueType::Any => unreachable!("must be checked by caller"),
            &StackValueType::Specific(value_type) => value_type,
        }
    }
}

impl From<ValueType> for StackValueType {
    fn from(value_type: ValueType) -> Self {
        StackValueType::Specific(value_type)
    }
}

impl PartialEq<StackValueType> for StackValueType {
    fn eq(&self, other: &StackValueType) -> bool {
        if self.is_any() || other.is_any() {
            true
        } else {
            self.value_type() == other.value_type()
        }
    }
}

impl PartialEq<ValueType> for StackValueType {
    fn eq(&self, other: &ValueType) -> bool {
        if self.is_any() {
            true
        } else {
            self.value_type() == *other
        }
    }
}

impl PartialEq<StackValueType> for ValueType {
    fn eq(&self, other: &StackValueType) -> bool {
        other == self
    }
}

/// Instruction outcome.
#[derive(Debug, Clone)]
enum Outcome {
    /// Continue with next instruction.
    NextInstruction,
    /// Unreachable instruction reached.
    Unreachable,
}

pub struct FunctionReader;

impl FunctionReader {
    pub fn read_function(
        module: &ModuleContext,
        func: &Func,
        body: &FuncBody,
    ) -> Result<isa::Instructions, Error> {
        let (params, result_ty) = module.require_function_type(func.type_ref())?;

        let ins_size_estimate = body.code().elements().len();
        let mut context = FunctionValidationContext::new(
            &module,
            Locals::new(params, body.locals())?,
            DEFAULT_VALUE_STACK_LIMIT,
            DEFAULT_FRAME_STACK_LIMIT,
            result_ty,
            ins_size_estimate,
        );

        let end_label = context.sink.new_label();
        push_label(
            BlockFrameType::Block { end_label },
            result_ty,
            context.position,
            &context.value_stack,
            &mut context.frame_stack,
        )?;
        FunctionReader::read_function_body(&mut context, body.code().elements())?;

        assert!(context.frame_stack.is_empty());

        Ok(context.into_code())
    }

    fn read_function_body(
        context: &mut FunctionValidationContext,
        body: &[Instruction],
    ) -> Result<(), Error> {
        let body_len = body.len();
        if body_len == 0 {
            return Err(Error("Non-empty function body expected".into()));
        }

        loop {
            let instruction = &body[context.position];

            let outcome =
                FunctionReader::read_instruction(context, instruction).map_err(|err| {
                    Error(format!(
                        "At instruction {:?}(@{}): {}",
                        instruction, context.position, err
                    ))
                })?;

            match outcome {
                Outcome::NextInstruction => (),
                Outcome::Unreachable => {
                    make_top_frame_polymorphic(&mut context.value_stack, &mut context.frame_stack)
                }
            }

            context.position += 1;
            if context.position == body_len {
                return Ok(());
            }
        }
    }

    fn read_instruction(
        context: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<Outcome, Error> {
        use self::Instruction::*;

        match *instruction {
            // Nop instruction doesn't do anything. It is safe to just skip it.
            Nop => {}

            Unreachable => {
                context.sink.emit(isa::InstructionInternal::Unreachable);
                return Ok(Outcome::Unreachable);
            }

            Block(block_type) => {
                let end_label = context.sink.new_label();
                push_label(
                    BlockFrameType::Block { end_label },
                    block_type,
                    context.position,
                    &context.value_stack,
                    &mut context.frame_stack,
                )?;
            }
            Loop(block_type) => {
                // Resolve loop header right away.
                let header = context.sink.new_label();
                context.sink.resolve_label(header);

                push_label(
                    BlockFrameType::Loop { header },
                    block_type,
                    context.position,
                    &context.value_stack,
                    &mut context.frame_stack,
                )?;
            }
            If(block_type) => {
                // `if_not` will be resolved whenever `End` or `Else` operator will be met.
                // `end_label` will always be resolved at `End`.
                let if_not = context.sink.new_label();
                let end_label = context.sink.new_label();

                pop_value(
                    &mut context.value_stack,
                    &context.frame_stack,
                    ValueType::I32.into(),
                )?;
                push_label(
                    BlockFrameType::IfTrue { if_not, end_label },
                    block_type,
                    context.position,
                    &context.value_stack,
                    &mut context.frame_stack,
                )?;

                context.sink.emit_br_eqz(Target {
                    label: if_not,
                    drop_keep: isa::DropKeep {
                        drop: 0,
                        keep: isa::Keep::None,
                    },
                });
            }
            Else => {
                let (block_type, if_not, end_label) = {
                    let top_frame = top_label(&context.frame_stack);

                    let (if_not, end_label) = match top_frame.frame_type {
                        BlockFrameType::IfTrue { if_not, end_label } => (if_not, end_label),
                        _ => return Err(Error("Misplaced else instruction".into())),
                    };
                    (top_frame.block_type, if_not, end_label)
                };

                // First, we need to finish if-true block: add a jump from the end of the if-true block
                // to the "end_label" (it will be resolved at End).
                context.sink.emit_br(Target {
                    label: end_label,
                    drop_keep: isa::DropKeep {
                        drop: 0,
                        keep: isa::Keep::None,
                    },
                });

                // Resolve `if_not` to here so when if condition is unsatisfied control flow
                // will jump to this label.
                context.sink.resolve_label(if_not);

                // Then, we pop the current label. It discards all values that pushed in the current
                // frame.
                pop_label(&mut context.value_stack, &mut context.frame_stack)?;
                push_label(
                    BlockFrameType::IfFalse { end_label },
                    block_type,
                    context.position,
                    &context.value_stack,
                    &mut context.frame_stack,
                )?;
            }
            End => {
                let (frame_type, block_type) = {
                    let top = top_label(&context.frame_stack);
                    (top.frame_type, top.block_type)
                };

                if let BlockFrameType::IfTrue { if_not, .. } = frame_type {
                    // A `if` without an `else` can't return a result.
                    if block_type != BlockType::NoResult {
                        return Err(Error(format!(
									"If block without else required to have NoResult block type. But it has {:?} type",
									block_type
								)));
                    }

                    // Resolve `if_not` label. If the `if's` condition doesn't hold the control will jump
                    // to here.
                    context.sink.resolve_label(if_not);
                }

                // Unless it's a loop, resolve the `end_label` position here.
                if !frame_type.is_loop() {
                    let end_label = frame_type.end_label();
                    context.sink.resolve_label(end_label);
                }

                if context.frame_stack.len() == 1 {
                    // We are about to close the last frame. Insert
                    // an explicit return.

                    // Check the return type.
                    if let BlockType::Value(value_type) = context.return_type()? {
                        tee_value(
                            &mut context.value_stack,
                            &context.frame_stack,
                            value_type.into(),
                        )?;
                    }

                    // Emit the return instruction.
                    let drop_keep = drop_keep_return(
                        &context.locals,
                        &context.value_stack,
                        &context.frame_stack,
                    );
                    context
                        .sink
                        .emit(isa::InstructionInternal::Return(drop_keep));
                }

                pop_label(&mut context.value_stack, &mut context.frame_stack)?;

                // Push the result value.
                if let BlockType::Value(value_type) = block_type {
                    push_value(&mut context.value_stack, value_type.into())?;
                }
            }
            Br(depth) => {
                Validator::validate_br(context, depth)?;

                let target = require_target(depth, &context.value_stack, &context.frame_stack);
                context.sink.emit_br(target);

                return Ok(Outcome::Unreachable);
            }
            BrIf(depth) => {
                Validator::validate_br_if(context, depth)?;

                let target = require_target(depth, &context.value_stack, &context.frame_stack);
                context.sink.emit_br_nez(target);
            }
            BrTable(ref table, default) => {
                Validator::validate_br_table(context, table, default)?;

                let mut targets = Vec::new();
                for depth in table.iter() {
                    let target = require_target(*depth, &context.value_stack, &context.frame_stack);
                    targets.push(target);
                }
                let default_target =
                    require_target(default, &context.value_stack, &context.frame_stack);
                context.sink.emit_br_table(&targets, default_target);

                return Ok(Outcome::Unreachable);
            }
            Return => {
                if let BlockType::Value(value_type) = context.return_type()? {
                    tee_value(
                        &mut context.value_stack,
                        &context.frame_stack,
                        value_type.into(),
                    )?;
                }

                let drop_keep =
                    drop_keep_return(&context.locals, &context.value_stack, &context.frame_stack);
                context
                    .sink
                    .emit(isa::InstructionInternal::Return(drop_keep));

                return Ok(Outcome::Unreachable);
            }

            Call(index) => {
                Validator::validate_call(context, index)?;
                context.sink.emit(isa::InstructionInternal::Call(index));
            }
            CallIndirect(index, _reserved) => {
                Validator::validate_call_indirect(context, index)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::CallIndirect(index));
            }

            Drop => {
                Validator::validate_drop(context)?;
                context.sink.emit(isa::InstructionInternal::Drop);
            }
            Select => {
                Validator::validate_select(context)?;
                context.sink.emit(isa::InstructionInternal::Select);
            }

            GetLocal(index) => {
                // We need to calculate relative depth before validation since
                // it will change the value stack size.
                let depth = relative_local_depth(index, &context.locals, &context.value_stack)?;
                Validator::validate_get_local(context, index)?;
                context.sink.emit(isa::InstructionInternal::GetLocal(depth));
            }
            SetLocal(index) => {
                Validator::validate_set_local(context, index)?;
                let depth = relative_local_depth(index, &context.locals, &context.value_stack)?;
                context.sink.emit(isa::InstructionInternal::SetLocal(depth));
            }
            TeeLocal(index) => {
                Validator::validate_tee_local(context, index)?;
                let depth = relative_local_depth(index, &context.locals, &context.value_stack)?;
                context.sink.emit(isa::InstructionInternal::TeeLocal(depth));
            }
            GetGlobal(index) => {
                Validator::validate_get_global(context, index)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::GetGlobal(index));
            }
            SetGlobal(index) => {
                Validator::validate_set_global(context, index)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::SetGlobal(index));
            }

            I32Load(align, offset) => {
                Validator::validate_load(context, align, 4, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Load(offset));
            }
            I64Load(align, offset) => {
                Validator::validate_load(context, align, 8, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Load(offset));
            }
            F32Load(align, offset) => {
                Validator::validate_load(context, align, 4, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Load(offset));
            }
            F64Load(align, offset) => {
                Validator::validate_load(context, align, 8, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Load(offset));
            }
            I32Load8S(align, offset) => {
                Validator::validate_load(context, align, 1, ValueType::I32)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I32Load8S(offset));
            }
            I32Load8U(align, offset) => {
                Validator::validate_load(context, align, 1, ValueType::I32)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I32Load8U(offset));
            }
            I32Load16S(align, offset) => {
                Validator::validate_load(context, align, 2, ValueType::I32)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I32Load16S(offset));
            }
            I32Load16U(align, offset) => {
                Validator::validate_load(context, align, 2, ValueType::I32)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I32Load16U(offset));
            }
            I64Load8S(align, offset) => {
                Validator::validate_load(context, align, 1, ValueType::I64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I64Load8S(offset));
            }
            I64Load8U(align, offset) => {
                Validator::validate_load(context, align, 1, ValueType::I64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I64Load8U(offset));
            }
            I64Load16S(align, offset) => {
                Validator::validate_load(context, align, 2, ValueType::I64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I64Load16S(offset));
            }
            I64Load16U(align, offset) => {
                Validator::validate_load(context, align, 2, ValueType::I64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I64Load16U(offset));
            }
            I64Load32S(align, offset) => {
                Validator::validate_load(context, align, 4, ValueType::I64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I64Load32S(offset));
            }
            I64Load32U(align, offset) => {
                Validator::validate_load(context, align, 4, ValueType::I64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I64Load32U(offset));
            }

            I32Store(align, offset) => {
                Validator::validate_store(context, align, 4, ValueType::I32)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I32Store(offset));
            }
            I64Store(align, offset) => {
                Validator::validate_store(context, align, 8, ValueType::I64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I64Store(offset));
            }
            F32Store(align, offset) => {
                Validator::validate_store(context, align, 4, ValueType::F32)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::F32Store(offset));
            }
            F64Store(align, offset) => {
                Validator::validate_store(context, align, 8, ValueType::F64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::F64Store(offset));
            }
            I32Store8(align, offset) => {
                Validator::validate_store(context, align, 1, ValueType::I32)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I32Store8(offset));
            }
            I32Store16(align, offset) => {
                Validator::validate_store(context, align, 2, ValueType::I32)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I32Store16(offset));
            }
            I64Store8(align, offset) => {
                Validator::validate_store(context, align, 1, ValueType::I64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I64Store8(offset));
            }
            I64Store16(align, offset) => {
                Validator::validate_store(context, align, 2, ValueType::I64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I64Store16(offset));
            }
            I64Store32(align, offset) => {
                Validator::validate_store(context, align, 4, ValueType::I64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I64Store32(offset));
            }

            CurrentMemory(_) => {
                Validator::validate_current_memory(context)?;
                context.sink.emit(isa::InstructionInternal::CurrentMemory);
            }
            GrowMemory(_) => {
                Validator::validate_grow_memory(context)?;
                context.sink.emit(isa::InstructionInternal::GrowMemory);
            }

            I32Const(v) => {
                Validator::validate_const(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Const(v));
            }
            I64Const(v) => {
                Validator::validate_const(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Const(v));
            }
            F32Const(v) => {
                Validator::validate_const(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Const(v));
            }
            F64Const(v) => {
                Validator::validate_const(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Const(v));
            }

            I32Eqz => {
                Validator::validate_testop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Eqz);
            }
            I32Eq => {
                Validator::validate_relop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Eq);
            }
            I32Ne => {
                Validator::validate_relop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Ne);
            }
            I32LtS => {
                Validator::validate_relop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32LtS);
            }
            I32LtU => {
                Validator::validate_relop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32LtU);
            }
            I32GtS => {
                Validator::validate_relop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32GtS);
            }
            I32GtU => {
                Validator::validate_relop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32GtU);
            }
            I32LeS => {
                Validator::validate_relop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32LeS);
            }
            I32LeU => {
                Validator::validate_relop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32LeU);
            }
            I32GeS => {
                Validator::validate_relop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32GeS);
            }
            I32GeU => {
                Validator::validate_relop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32GeU);
            }

            I64Eqz => {
                Validator::validate_testop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Eqz);
            }
            I64Eq => {
                Validator::validate_relop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Eq);
            }
            I64Ne => {
                Validator::validate_relop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Ne);
            }
            I64LtS => {
                Validator::validate_relop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64LtS);
            }
            I64LtU => {
                Validator::validate_relop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64LtU);
            }
            I64GtS => {
                Validator::validate_relop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64GtS);
            }
            I64GtU => {
                Validator::validate_relop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64GtU);
            }
            I64LeS => {
                Validator::validate_relop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64LeS);
            }
            I64LeU => {
                Validator::validate_relop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64LeU);
            }
            I64GeS => {
                Validator::validate_relop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64GeS);
            }
            I64GeU => {
                Validator::validate_relop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64GeU);
            }

            F32Eq => {
                Validator::validate_relop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Eq);
            }
            F32Ne => {
                Validator::validate_relop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Ne);
            }
            F32Lt => {
                Validator::validate_relop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Lt);
            }
            F32Gt => {
                Validator::validate_relop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Gt);
            }
            F32Le => {
                Validator::validate_relop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Le);
            }
            F32Ge => {
                Validator::validate_relop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Ge);
            }

            F64Eq => {
                Validator::validate_relop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Eq);
            }
            F64Ne => {
                Validator::validate_relop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Ne);
            }
            F64Lt => {
                Validator::validate_relop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Lt);
            }
            F64Gt => {
                Validator::validate_relop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Gt);
            }
            F64Le => {
                Validator::validate_relop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Le);
            }
            F64Ge => {
                Validator::validate_relop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Ge);
            }

            I32Clz => {
                Validator::validate_unop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Clz);
            }
            I32Ctz => {
                Validator::validate_unop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Ctz);
            }
            I32Popcnt => {
                Validator::validate_unop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Popcnt);
            }
            I32Add => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Add);
            }
            I32Sub => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Sub);
            }
            I32Mul => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Mul);
            }
            I32DivS => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32DivS);
            }
            I32DivU => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32DivU);
            }
            I32RemS => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32RemS);
            }
            I32RemU => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32RemU);
            }
            I32And => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32And);
            }
            I32Or => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Or);
            }
            I32Xor => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Xor);
            }
            I32Shl => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Shl);
            }
            I32ShrS => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32ShrS);
            }
            I32ShrU => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32ShrU);
            }
            I32Rotl => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Rotl);
            }
            I32Rotr => {
                Validator::validate_binop(context, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32Rotr);
            }

            I64Clz => {
                Validator::validate_unop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Clz);
            }
            I64Ctz => {
                Validator::validate_unop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Ctz);
            }
            I64Popcnt => {
                Validator::validate_unop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Popcnt);
            }
            I64Add => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Add);
            }
            I64Sub => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Sub);
            }
            I64Mul => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Mul);
            }
            I64DivS => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64DivS);
            }
            I64DivU => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64DivU);
            }
            I64RemS => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64RemS);
            }
            I64RemU => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64RemU);
            }
            I64And => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64And);
            }
            I64Or => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Or);
            }
            I64Xor => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Xor);
            }
            I64Shl => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Shl);
            }
            I64ShrS => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64ShrS);
            }
            I64ShrU => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64ShrU);
            }
            I64Rotl => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Rotl);
            }
            I64Rotr => {
                Validator::validate_binop(context, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64Rotr);
            }

            F32Abs => {
                Validator::validate_unop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Abs);
            }
            F32Neg => {
                Validator::validate_unop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Neg);
            }
            F32Ceil => {
                Validator::validate_unop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Ceil);
            }
            F32Floor => {
                Validator::validate_unop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Floor);
            }
            F32Trunc => {
                Validator::validate_unop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Trunc);
            }
            F32Nearest => {
                Validator::validate_unop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Nearest);
            }
            F32Sqrt => {
                Validator::validate_unop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Sqrt);
            }
            F32Add => {
                Validator::validate_binop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Add);
            }
            F32Sub => {
                Validator::validate_binop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Sub);
            }
            F32Mul => {
                Validator::validate_binop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Mul);
            }
            F32Div => {
                Validator::validate_binop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Div);
            }
            F32Min => {
                Validator::validate_binop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Min);
            }
            F32Max => {
                Validator::validate_binop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Max);
            }
            F32Copysign => {
                Validator::validate_binop(context, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32Copysign);
            }

            F64Abs => {
                Validator::validate_unop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Abs);
            }
            F64Neg => {
                Validator::validate_unop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Neg);
            }
            F64Ceil => {
                Validator::validate_unop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Ceil);
            }
            F64Floor => {
                Validator::validate_unop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Floor);
            }
            F64Trunc => {
                Validator::validate_unop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Trunc);
            }
            F64Nearest => {
                Validator::validate_unop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Nearest);
            }
            F64Sqrt => {
                Validator::validate_unop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Sqrt);
            }
            F64Add => {
                Validator::validate_binop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Add);
            }
            F64Sub => {
                Validator::validate_binop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Sub);
            }
            F64Mul => {
                Validator::validate_binop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Mul);
            }
            F64Div => {
                Validator::validate_binop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Div);
            }
            F64Min => {
                Validator::validate_binop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Min);
            }
            F64Max => {
                Validator::validate_binop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Max);
            }
            F64Copysign => {
                Validator::validate_binop(context, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64Copysign);
            }

            I32WrapI64 => {
                Validator::validate_cvtop(context, ValueType::I64, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32WrapI64);
            }
            I32TruncSF32 => {
                Validator::validate_cvtop(context, ValueType::F32, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32TruncSF32);
            }
            I32TruncUF32 => {
                Validator::validate_cvtop(context, ValueType::F32, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32TruncUF32);
            }
            I32TruncSF64 => {
                Validator::validate_cvtop(context, ValueType::F64, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32TruncSF64);
            }
            I32TruncUF64 => {
                Validator::validate_cvtop(context, ValueType::F64, ValueType::I32)?;
                context.sink.emit(isa::InstructionInternal::I32TruncUF64);
            }
            I64ExtendSI32 => {
                Validator::validate_cvtop(context, ValueType::I32, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64ExtendSI32);
            }
            I64ExtendUI32 => {
                Validator::validate_cvtop(context, ValueType::I32, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64ExtendUI32);
            }
            I64TruncSF32 => {
                Validator::validate_cvtop(context, ValueType::F32, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64TruncSF32);
            }
            I64TruncUF32 => {
                Validator::validate_cvtop(context, ValueType::F32, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64TruncUF32);
            }
            I64TruncSF64 => {
                Validator::validate_cvtop(context, ValueType::F64, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64TruncSF64);
            }
            I64TruncUF64 => {
                Validator::validate_cvtop(context, ValueType::F64, ValueType::I64)?;
                context.sink.emit(isa::InstructionInternal::I64TruncUF64);
            }
            F32ConvertSI32 => {
                Validator::validate_cvtop(context, ValueType::I32, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32ConvertSI32);
            }
            F32ConvertUI32 => {
                Validator::validate_cvtop(context, ValueType::I32, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32ConvertUI32);
            }
            F32ConvertSI64 => {
                Validator::validate_cvtop(context, ValueType::I64, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32ConvertSI64);
            }
            F32ConvertUI64 => {
                Validator::validate_cvtop(context, ValueType::I64, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32ConvertUI64);
            }
            F32DemoteF64 => {
                Validator::validate_cvtop(context, ValueType::F64, ValueType::F32)?;
                context.sink.emit(isa::InstructionInternal::F32DemoteF64);
            }
            F64ConvertSI32 => {
                Validator::validate_cvtop(context, ValueType::I32, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64ConvertSI32);
            }
            F64ConvertUI32 => {
                Validator::validate_cvtop(context, ValueType::I32, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64ConvertUI32);
            }
            F64ConvertSI64 => {
                Validator::validate_cvtop(context, ValueType::I64, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64ConvertSI64);
            }
            F64ConvertUI64 => {
                Validator::validate_cvtop(context, ValueType::I64, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64ConvertUI64);
            }
            F64PromoteF32 => {
                Validator::validate_cvtop(context, ValueType::F32, ValueType::F64)?;
                context.sink.emit(isa::InstructionInternal::F64PromoteF32);
            }

            I32ReinterpretF32 => {
                Validator::validate_cvtop(context, ValueType::F32, ValueType::I32)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I32ReinterpretF32);
            }
            I64ReinterpretF64 => {
                Validator::validate_cvtop(context, ValueType::F64, ValueType::I64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::I64ReinterpretF64);
            }
            F32ReinterpretI32 => {
                Validator::validate_cvtop(context, ValueType::I32, ValueType::F32)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::F32ReinterpretI32);
            }
            F64ReinterpretI64 => {
                Validator::validate_cvtop(context, ValueType::I64, ValueType::F64)?;
                context
                    .sink
                    .emit(isa::InstructionInternal::F64ReinterpretI64);
            }
        }

        Ok(Outcome::NextInstruction)
    }
}

/// Function validator.
struct Validator;

impl Validator {
    fn validate_const(
        context: &mut FunctionValidationContext,
        value_type: ValueType,
    ) -> Result<(), Error> {
        push_value(&mut context.value_stack, value_type.into())?;
        Ok(())
    }

    fn validate_unop(
        context: &mut FunctionValidationContext,
        value_type: ValueType,
    ) -> Result<(), Error> {
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            value_type.into(),
        )?;
        push_value(&mut context.value_stack, value_type.into())?;
        Ok(())
    }

    fn validate_binop(
        context: &mut FunctionValidationContext,
        value_type: ValueType,
    ) -> Result<(), Error> {
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            value_type.into(),
        )?;
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            value_type.into(),
        )?;
        push_value(&mut context.value_stack, value_type.into())?;
        Ok(())
    }

    fn validate_testop(
        context: &mut FunctionValidationContext,
        value_type: ValueType,
    ) -> Result<(), Error> {
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            value_type.into(),
        )?;
        push_value(&mut context.value_stack, ValueType::I32.into())?;
        Ok(())
    }

    fn validate_relop(
        context: &mut FunctionValidationContext,
        value_type: ValueType,
    ) -> Result<(), Error> {
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            value_type.into(),
        )?;
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            value_type.into(),
        )?;
        push_value(&mut context.value_stack, ValueType::I32.into())?;
        Ok(())
    }

    fn validate_cvtop(
        context: &mut FunctionValidationContext,
        value_type1: ValueType,
        value_type2: ValueType,
    ) -> Result<(), Error> {
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            value_type1.into(),
        )?;
        push_value(&mut context.value_stack, value_type2.into())?;
        Ok(())
    }

    fn validate_drop(context: &mut FunctionValidationContext) -> Result<(), Error> {
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            StackValueType::Any,
        )?;
        Ok(())
    }

    fn validate_select(context: &mut FunctionValidationContext) -> Result<(), Error> {
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            ValueType::I32.into(),
        )?;
        let select_type = pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            StackValueType::Any,
        )?;
        pop_value(&mut context.value_stack, &context.frame_stack, select_type)?;
        push_value(&mut context.value_stack, select_type)?;
        Ok(())
    }

    fn validate_get_local(
        context: &mut FunctionValidationContext,
        index: u32,
    ) -> Result<(), Error> {
        let local_type = require_local(&context.locals, index)?;
        push_value(&mut context.value_stack, local_type.into())?;
        Ok(())
    }

    fn validate_set_local(
        context: &mut FunctionValidationContext,
        index: u32,
    ) -> Result<(), Error> {
        let local_type = require_local(&context.locals, index)?;
        let value_type = pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            StackValueType::Any,
        )?;
        if StackValueType::from(local_type) != value_type {
            return Err(Error(format!(
                "Trying to update local {} of type {:?} with value of type {:?}",
                index, local_type, value_type
            )));
        }
        Ok(())
    }

    fn validate_tee_local(
        context: &mut FunctionValidationContext,
        index: u32,
    ) -> Result<(), Error> {
        let local_type = require_local(&context.locals, index)?;
        tee_value(
            &mut context.value_stack,
            &context.frame_stack,
            local_type.into(),
        )?;
        Ok(())
    }

    fn validate_get_global(
        context: &mut FunctionValidationContext,
        index: u32,
    ) -> Result<(), Error> {
        let global_type: StackValueType = {
            let global = context.module.require_global(index, None)?;
            global.content_type().into()
        };
        push_value(&mut context.value_stack, global_type)?;
        Ok(())
    }

    fn validate_set_global(
        context: &mut FunctionValidationContext,
        index: u32,
    ) -> Result<(), Error> {
        let global_type: StackValueType = {
            let global = context.module.require_global(index, Some(true))?;
            global.content_type().into()
        };
        let value_type = pop_value(
            &mut context.value_stack,
            &context.frame_stack,
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
        context: &mut FunctionValidationContext,
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
            &mut context.value_stack,
            &context.frame_stack,
            ValueType::I32.into(),
        )?;
        context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
        push_value(&mut context.value_stack, value_type.into())?;
        Ok(())
    }

    fn validate_store(
        context: &mut FunctionValidationContext,
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

        context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            value_type.into(),
        )?;
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            ValueType::I32.into(),
        )?;
        Ok(())
    }

    fn validate_br(context: &mut FunctionValidationContext, depth: u32) -> Result<(), Error> {
        let (frame_type, frame_block_type) = {
            let frame = require_label(depth, &context.frame_stack)?;
            (frame.frame_type, frame.block_type)
        };
        if !frame_type.is_loop() {
            if let BlockType::Value(value_type) = frame_block_type {
                tee_value(
                    &mut context.value_stack,
                    &context.frame_stack,
                    value_type.into(),
                )?;
            }
        }
        Ok(())
    }

    fn validate_br_if(context: &mut FunctionValidationContext, depth: u32) -> Result<(), Error> {
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            ValueType::I32.into(),
        )?;

        let (frame_type, frame_block_type) = {
            let frame = require_label(depth, &context.frame_stack)?;
            (frame.frame_type, frame.block_type)
        };
        if !frame_type.is_loop() {
            if let BlockType::Value(value_type) = frame_block_type {
                tee_value(
                    &mut context.value_stack,
                    &context.frame_stack,
                    value_type.into(),
                )?;
            }
        }
        Ok(())
    }

    fn validate_br_table(
        context: &mut FunctionValidationContext,
        table: &[u32],
        default: u32,
    ) -> Result<(), Error> {
        let required_block_type: BlockType = {
            let default_block = require_label(default, &context.frame_stack)?;
            let required_block_type = if !default_block.frame_type.is_loop() {
                default_block.block_type
            } else {
                BlockType::NoResult
            };

            for label in table {
                let label_block = require_label(*label, &context.frame_stack)?;
                let label_block_type = if !label_block.frame_type.is_loop() {
                    label_block.block_type
                } else {
                    BlockType::NoResult
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
            &mut context.value_stack,
            &context.frame_stack,
            ValueType::I32.into(),
        )?;
        if let BlockType::Value(value_type) = required_block_type {
            tee_value(
                &mut context.value_stack,
                &context.frame_stack,
                value_type.into(),
            )?;
        }

        Ok(())
    }

    fn validate_call(context: &mut FunctionValidationContext, idx: u32) -> Result<(), Error> {
        let (argument_types, return_type) = context.module.require_function(idx)?;
        for argument_type in argument_types.iter().rev() {
            pop_value(
                &mut context.value_stack,
                &context.frame_stack,
                (*argument_type).into(),
            )?;
        }
        if let BlockType::Value(value_type) = return_type {
            push_value(&mut context.value_stack, value_type.into())?;
        }
        Ok(())
    }

    fn validate_call_indirect(
        context: &mut FunctionValidationContext,
        idx: u32,
    ) -> Result<(), Error> {
        {
            let table = context.module.require_table(DEFAULT_TABLE_INDEX)?;
            if table.elem_type() != TableElementType::AnyFunc {
                return Err(Error(format!(
                    "Table {} has element type {:?} while `anyfunc` expected",
                    idx,
                    table.elem_type()
                )));
            }
        }

        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            ValueType::I32.into(),
        )?;
        let (argument_types, return_type) = context.module.require_function_type(idx)?;
        for argument_type in argument_types.iter().rev() {
            pop_value(
                &mut context.value_stack,
                &context.frame_stack,
                (*argument_type).into(),
            )?;
        }
        if let BlockType::Value(value_type) = return_type {
            push_value(&mut context.value_stack, value_type.into())?;
        }
        Ok(())
    }

    fn validate_current_memory(context: &mut FunctionValidationContext) -> Result<(), Error> {
        context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
        push_value(&mut context.value_stack, ValueType::I32.into())?;
        Ok(())
    }

    fn validate_grow_memory(context: &mut FunctionValidationContext) -> Result<(), Error> {
        context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
        pop_value(
            &mut context.value_stack,
            &context.frame_stack,
            ValueType::I32.into(),
        )?;
        push_value(&mut context.value_stack, ValueType::I32.into())?;
        Ok(())
    }
}

/// Function validation context.
struct FunctionValidationContext<'a> {
    /// Wasm module
    module: &'a ModuleContext,
    /// Current instruction position.
    position: usize,
    /// Local variables.
    locals: Locals<'a>,
    /// Value stack.
    value_stack: StackWithLimit<StackValueType>,
    /// Frame stack.
    frame_stack: StackWithLimit<BlockFrame>,
    /// Function return type.
    return_type: BlockType,
    /// A sink used to emit optimized code.
    sink: Sink,
}

impl<'a> FunctionValidationContext<'a> {
    fn new(
        module: &'a ModuleContext,
        locals: Locals<'a>,
        value_stack_limit: usize,
        frame_stack_limit: usize,
        return_type: BlockType,
        size_estimate: usize,
    ) -> Self {
        FunctionValidationContext {
            module: module,
            position: 0,
            locals: locals,
            value_stack: StackWithLimit::with_limit(value_stack_limit),
            frame_stack: StackWithLimit::with_limit(frame_stack_limit),
            return_type: return_type,
            sink: Sink::with_instruction_capacity(size_estimate),
        }
    }

    fn return_type(&self) -> Result<BlockType, Error> {
        Ok(self.return_type)
    }

    fn into_code(self) -> isa::Instructions {
        self.sink.into_inner()
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
    Ok(value_stack.push(value_type.into())?)
}

// TODO: Rename value_type -> expected_value_ty
fn pop_value(
    value_stack: &mut StackWithLimit<StackValueType>,
    frame_stack: &StackWithLimit<BlockFrame>,
    value_type: StackValueType,
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
        StackValueType::Specific(stack_value_type) if stack_value_type == value_type => {
            Ok(actual_value)
        }
        StackValueType::Any => Ok(actual_value),
        stack_value_type @ _ => Err(Error(format!(
            "Expected value of type {:?} on top of stack. Got {:?}",
            value_type, stack_value_type
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
    frame_type: BlockFrameType,
    block_type: BlockType,
    position: usize,
    value_stack: &StackWithLimit<StackValueType>,
    frame_stack: &mut StackWithLimit<BlockFrame>,
) -> Result<(), Error> {
    Ok(frame_stack.push(BlockFrame {
        frame_type: frame_type,
        block_type: block_type,
        begin_position: position,
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

fn top_label(frame_stack: &StackWithLimit<BlockFrame>) -> &BlockFrame {
    frame_stack
        .top()
        .expect("this function can't be called with empty frame stack")
}

fn require_label(
    depth: u32,
    frame_stack: &StackWithLimit<BlockFrame>,
) -> Result<&BlockFrame, Error> {
    Ok(frame_stack.get(depth as usize)?)
}

fn require_target(
    depth: u32,
    value_stack: &StackWithLimit<StackValueType>,
    frame_stack: &StackWithLimit<BlockFrame>,
) -> Target {
    let is_stack_polymorphic = top_label(frame_stack).polymorphic_stack;
    let frame =
        require_label(depth, frame_stack).expect("require_target called with a bogus depth");

    // Find out how many values we need to keep (copy to the new stack location after the drop).
    let keep: isa::Keep = match (frame.frame_type, frame.block_type) {
        // A loop doesn't take a value upon a branch. It can return value
        // only via reaching it's closing `End` operator.
        (BlockFrameType::Loop { .. }, _) => isa::Keep::None,

        (_, BlockType::Value(_)) => isa::Keep::Single,
        (_, BlockType::NoResult) => isa::Keep::None,
    };

    // Find out how many values we need to discard.
    let drop = if is_stack_polymorphic {
        // Polymorphic stack is a weird state. Fortunately, it always about the code that
        // will not be executed, so we don't bother and return 0 here.
        0
    } else {
        let value_stack_height = value_stack.len();
        assert!(
			value_stack_height >= frame.value_stack_len,
			"Stack underflow detected: value stack height ({}) is lower than minimum stack len ({})",
			value_stack_height,
			frame.value_stack_len,
		);
        assert!(
            (value_stack_height as u32 - frame.value_stack_len as u32) >= keep as u32,
            "Stack underflow detected: asked to keep {:?} values, but there are only {}",
            keep,
            value_stack_height as u32 - frame.value_stack_len as u32,
        );
        (value_stack_height as u32 - frame.value_stack_len as u32) - keep as u32
    };

    Target {
        label: frame.frame_type.br_destination(),
        drop_keep: isa::DropKeep { drop, keep },
    }
}

fn drop_keep_return(
    locals: &Locals,
    value_stack: &StackWithLimit<StackValueType>,
    frame_stack: &StackWithLimit<BlockFrame>,
) -> isa::DropKeep {
    assert!(
        !frame_stack.is_empty(),
        "drop_keep_return can't be called with the frame stack empty"
    );

    let deepest = (frame_stack.len() - 1) as u32;
    let mut drop_keep = require_target(deepest, value_stack, frame_stack).drop_keep;

    // Drop all local variables and parameters upon exit.
    drop_keep.drop += locals.count();

    drop_keep
}

fn require_local(locals: &Locals, idx: u32) -> Result<ValueType, Error> {
    Ok(locals.type_of_local(idx)?)
}

/// See stack layout definition in mod isa.
fn relative_local_depth(
    idx: u32,
    locals: &Locals,
    value_stack: &StackWithLimit<StackValueType>,
) -> Result<u32, Error> {
    let value_stack_height = value_stack.len() as u32;
    let locals_and_params_count = locals.count();

    let depth = value_stack_height
        .checked_add(locals_and_params_count)
        .and_then(|x| x.checked_sub(idx))
        .ok_or_else(|| Error(String::from("Locals range not in 32-bit range")))?;
    Ok(depth)
}

/// The target of a branch instruction.
///
/// It references a `LabelId` instead of exact instruction address. This is handy
/// for emitting code right away with labels resolved later.
#[derive(Clone)]
struct Target {
    label: LabelId,
    drop_keep: isa::DropKeep,
}

/// Identifier of a label.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct LabelId(usize);

#[derive(Debug, PartialEq, Eq)]
enum Label {
    Resolved(u32),
    NotResolved,
}

struct Sink {
    ins: isa::Instructions,
    labels: Vec<(Label, Vec<isa::Reloc>)>,
}

impl Sink {
    fn with_instruction_capacity(capacity: usize) -> Sink {
        Sink {
            ins: isa::Instructions::with_capacity(capacity),
            labels: Vec::new(),
        }
    }

    fn cur_pc(&self) -> u32 {
        self.ins.current_pc()
    }

    fn pc_or_placeholder<F: FnOnce() -> isa::Reloc>(
        &mut self,
        label: LabelId,
        reloc_creator: F,
    ) -> u32 {
        match self.labels[label.0] {
            (Label::Resolved(dst_pc), _) => dst_pc,
            (Label::NotResolved, ref mut unresolved) => {
                unresolved.push(reloc_creator());
                u32::max_value()
            }
        }
    }

    fn emit(&mut self, instruction: isa::InstructionInternal) {
        self.ins.push(instruction);
    }

    fn emit_br(&mut self, target: Target) {
        let Target { label, drop_keep } = target;
        let pc = self.cur_pc();
        let dst_pc = self.pc_or_placeholder(label, || isa::Reloc::Br { pc });
        self.ins.push(isa::InstructionInternal::Br(isa::Target {
            dst_pc,
            drop_keep: drop_keep.into(),
        }));
    }

    fn emit_br_eqz(&mut self, target: Target) {
        let Target { label, drop_keep } = target;
        let pc = self.cur_pc();
        let dst_pc = self.pc_or_placeholder(label, || isa::Reloc::Br { pc });
        self.ins
            .push(isa::InstructionInternal::BrIfEqz(isa::Target {
                dst_pc,
                drop_keep: drop_keep.into(),
            }));
    }

    fn emit_br_nez(&mut self, target: Target) {
        let Target { label, drop_keep } = target;
        let pc = self.cur_pc();
        let dst_pc = self.pc_or_placeholder(label, || isa::Reloc::Br { pc });
        self.ins
            .push(isa::InstructionInternal::BrIfNez(isa::Target {
                dst_pc,
                drop_keep: drop_keep.into(),
            }));
    }

    fn emit_br_table(&mut self, targets: &[Target], default: Target) {
        use core::iter;

        let pc = self.cur_pc();

        self.ins.push(isa::InstructionInternal::BrTable {
            count: targets.len() as u32 + 1,
        });

        for (idx, &Target { label, drop_keep }) in
            targets.iter().chain(iter::once(&default)).enumerate()
        {
            let dst_pc = self.pc_or_placeholder(label, || isa::Reloc::BrTable { pc, idx });
            self.ins
                .push(isa::InstructionInternal::BrTableTarget(isa::Target {
                    dst_pc,
                    drop_keep: drop_keep.into(),
                }));
        }
    }

    /// Create a new unresolved label.
    fn new_label(&mut self) -> LabelId {
        let label_idx = self.labels.len();
        self.labels.push((Label::NotResolved, Vec::new()));
        LabelId(label_idx)
    }

    /// Resolve the label at the current position.
    ///
    /// Panics if the label is already resolved.
    fn resolve_label(&mut self, label: LabelId) {
        use core::mem;

        if let (Label::Resolved(_), _) = self.labels[label.0] {
            panic!("Trying to resolve already resolved label");
        }
        let dst_pc = self.cur_pc();

        // Patch all relocations that was previously recorded for this
        // particular label.
        let unresolved_rels = mem::replace(&mut self.labels[label.0].1, Vec::new());
        for reloc in unresolved_rels {
            self.ins.patch_relocation(reloc, dst_pc);
        }

        // Mark this label as resolved.
        self.labels[label.0] = (Label::Resolved(dst_pc), Vec::new());
    }

    /// Consume this Sink and returns isa::Instructions.
    fn into_inner(self) -> isa::Instructions {
        // At this moment all labels should be resolved.
        assert!(
            {
                self.labels
                    .iter()
                    .all(|(state, unresolved)| match (state, unresolved) {
                        (Label::Resolved(_), unresolved) if unresolved.is_empty() => true,
                        _ => false,
                    })
            },
            "there are unresolved labels left: {:?}",
            self.labels
        );
        self.ins
    }
}
