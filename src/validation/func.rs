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
    /// The opcode that started this block frame.
    started_with: StartedWith,
    /// A signature, which is a block signature type indicating the number and types of result
    /// values of the region.
    block_type: BlockType,
    /// A label for reference to block instruction.
    begin_position: usize,
    /// A limit integer value, which is an index into the value stack indicating where to reset it
    /// to on a branch to that label.
    value_stack_len: usize,
    /// Boolean which signals whether value stack became polymorphic. Value stack starts in
    /// a non-polymorphic state and becomes polymorphic only after an instruction that never passes
    /// control further is executed, i.e. `unreachable`, `br` (but not `br_if`!), etc.
    polymorphic_stack: bool,
}

/// An opcode that opened the particular frame.
///
/// We need that to ensure proper combinations with `End` instruction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum StartedWith {
    Block,
    If,
    Else,
    Loop,
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

// TODO: This is going to be a compiler.

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
            StartedWith::Block,
            result_ty,
            context.position,
            &context.value_stack,
            &mut context.frame_stack,
        )?;
        context
            .label_stack
            .push(BlockFrameType::Block { end_label });
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

            FunctionReader::read_instruction(context, instruction).map_err(|err| {
                Error(format!(
                    "At instruction {:?}(@{}): {}",
                    instruction, context.position, err
                ))
            })?;

            context.position += 1;
            if context.position == body_len {
                return Ok(());
            }
        }
    }

    fn read_instruction(
        context: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), Error> {
        use self::Instruction::*;

        match *instruction {
            Unreachable => {
                context.sink.emit(isa::InstructionInternal::Unreachable);
                context.step(instruction)?;
            }
            Block(_) => {
                context.step(instruction)?;

                let end_label = context.sink.new_label();
                context
                    .label_stack
                    .push(BlockFrameType::Block { end_label });
            }
            Loop(_) => {
                context.step(instruction)?;

                // Resolve loop header right away.
                let header = context.sink.new_label();
                context.sink.resolve_label(header);
                context.label_stack.push(BlockFrameType::Loop { header });
            }
            If(_) => {
                context.step(instruction)?;

                // `if_not` will be resolved whenever `End` or `Else` operator will be met.
                // `end_label` will always be resolved at `End`.
                let if_not = context.sink.new_label();
                let end_label = context.sink.new_label();
                context
                    .label_stack
                    .push(BlockFrameType::IfTrue { if_not, end_label });

                context.sink.emit_br_eqz(Target {
                    label: if_not,
                    drop_keep: isa::DropKeep {
                        drop: 0,
                        keep: isa::Keep::None,
                    },
                });
            }
            Else => {
                context.step(instruction)?;

                let (if_not, end_label) = {
                    // TODO: We will have to place this before validation step to ensure that
                    // the block type is indeed if_true.

                    let top_label = context.label_stack.last().unwrap();
                    let (if_not, end_label) = match *top_label {
                        BlockFrameType::IfTrue { if_not, end_label } => (if_not, end_label),
                        _ => panic!("validation ensures that the top frame is actually if_true"),
                    };
                    (if_not, end_label)
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

                context.label_stack.pop().unwrap();
                context
                    .label_stack
                    .push(BlockFrameType::IfFalse { end_label });
            }
            End => {
                let started_with = top_label(&context.frame_stack).started_with;
                let return_drop_keep = if context.frame_stack.len() == 1 {
                    // We are about to close the last frame.
                    Some(drop_keep_return(
                        &context.locals,
                        &context.value_stack,
                        &context.frame_stack,
                    ))
                } else {
                    None
                };

                context.step(instruction)?;

                // let started_with = started_with.expect("validation ensures that it is ok");

                // TODO: We will have to place this before validation step to ensure that
                // the block type is indeed if_true.
                let frame_type = context.label_stack.last().cloned().unwrap();

                if let BlockFrameType::IfTrue { if_not, .. } = frame_type {
                    // Resolve `if_not` label. If the `if's` condition doesn't hold the control will jump
                    // to here.
                    context.sink.resolve_label(if_not);
                }

                // Unless it's a loop, resolve the `end_label` position here.
                if started_with != StartedWith::Loop {
                    let end_label = frame_type.end_label();
                    context.sink.resolve_label(end_label);
                }

                if let Some(drop_keep) = return_drop_keep {
                    // TODO: The last one.
                    // It was the last instruction. Emit the explicit return instruction.
                    let drop_keep = drop_keep.expect("validation should ensure this doesn't fail");
                    context
                        .sink
                        .emit(isa::InstructionInternal::Return(drop_keep));
                }

                // Finally, pop the label.
                context.label_stack.pop().unwrap();
            }
            Br(depth) => {
                let target = require_target(
                    depth,
                    context.value_stack.len(),
                    &context.frame_stack,
                    &context.label_stack,
                );

                context.step(instruction)?;

                let target = target.expect("validation step should ensure that this doesn't fail");
                context.sink.emit_br(target);
            }
            BrIf(depth) => {
                context.step(instruction)?;

                let target = require_target(
                    depth,
                    context.value_stack.len(),
                    &context.frame_stack,
                    &context.label_stack,
                );

                let target = target.expect("validation step should ensure that this doesn't fail");
                context.sink.emit_br_nez(target);
            }
            BrTable(ref table, default) => {
                // At this point, the condition value is at the top of the stack.
                // But at the point of actual jump the condition will already be
                // popped off.
                let value_stack_height = context.value_stack.len().saturating_sub(1);

                let mut targets = table.iter().map(|depth|
                    require_target(
                        *depth,
                        value_stack_height,
                        &context.frame_stack,
                        &context.label_stack,
                    )
                ).collect::<Result<Vec<_>, _>>();
                let default_target = require_target(
                    default,
                    value_stack_height,
                    &context.frame_stack,
                    &context.label_stack,
                );

                context.step(instruction)?;

                // These two unwraps are guaranteed to succeed by validation.
                let targets = targets.unwrap();
                let default_target = default_target.unwrap();

                context.sink.emit_br_table(&targets, default_target);
            }
            Return => {
                let drop_keep =
                    drop_keep_return(&context.locals, &context.value_stack, &context.frame_stack);

                context.step(instruction)?;

                let drop_keep =
                    drop_keep.expect("validation step should ensure that this doesn't fail");

                context
                    .sink
                    .emit(isa::InstructionInternal::Return(drop_keep));
            }
            _ => {
                context.step(instruction)?;
            }
        };

        assert_eq!(context.label_stack.len(), context.frame_stack.len(),);

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

    // TODO: to be moved to the compiler.
    label_stack: Vec<BlockFrameType>,
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
            label_stack: Vec::new(),
        }
    }

    fn return_type(&self) -> BlockType {
        self.return_type
    }

    fn into_code(self) -> isa::Instructions {
        self.sink.into_inner()
    }

    fn step(&mut self, instruction: &Instruction) -> Result<(), Error> {
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
                    self.position,
                    &self.value_stack,
                    &mut self.frame_stack,
                )?;
            }
            Loop(block_type) => {
                push_label(
                    StartedWith::Loop,
                    block_type,
                    self.position,
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
                    self.position,
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
                    self.position,
                    &self.value_stack,
                    &mut self.frame_stack,
                )?;
            }
            End => {
                let (started_with, block_type) = {
                    let top = top_label(&self.frame_stack);

                    if top.started_with == StartedWith::If && top.block_type != BlockType::NoResult
                    {
                        // A `if` without an `else` can't return a result.
                        return Err(Error(format!(
                            "If block without else required to have NoResult block type. But it has {:?} type",
                            top.block_type
                        )));
                    }

                    (top.started_with, top.block_type)
                };

                if self.frame_stack.len() == 1 {
                    // We are about to close the last frame. Insert
                    // an explicit return.

                    // Check the return type.
                    if let BlockType::Value(value_type) = self.return_type() {
                        tee_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
                    }
                }

                pop_label(&mut self.value_stack, &mut self.frame_stack)?;

                // Push the result value.
                if let BlockType::Value(value_type) = block_type {
                    push_value(&mut self.value_stack, value_type.into())?;
                }
            }
            Br(depth) => {
                self.validate_br(depth)?;
                make_top_frame_polymorphic(&mut self.value_stack, &mut self.frame_stack);
            }
            BrIf(depth) => {
                self.validate_br_if(depth)?;
            }
            BrTable(ref table, default) => {
                self.validate_br_table(table, default)?;
                make_top_frame_polymorphic(&mut self.value_stack, &mut self.frame_stack);
            }
            Return => {
                if let BlockType::Value(value_type) = self.return_type() {
                    tee_value(&mut self.value_stack, &self.frame_stack, value_type.into())?;
                }
                make_top_frame_polymorphic(&mut self.value_stack, &mut self.frame_stack);
            }

            Call(index) => {
                self.validate_call(index)?;
                self.sink.emit(isa::InstructionInternal::Call(index));
            }
            CallIndirect(index, _reserved) => {
                self.validate_call_indirect(index)?;
                self.sink
                    .emit(isa::InstructionInternal::CallIndirect(index));
            }

            Drop => {
                self.validate_drop()?;
                self.sink.emit(isa::InstructionInternal::Drop);
            }
            Select => {
                self.validate_select()?;
                self.sink.emit(isa::InstructionInternal::Select);
            }

            GetLocal(index) => {
                // We need to calculate relative depth before validation since
                // it will change the value stack size.
                let depth = relative_local_depth(index, &self.locals, &self.value_stack)?;
                self.validate_get_local(index)?;
                self.sink.emit(isa::InstructionInternal::GetLocal(depth));
            }
            SetLocal(index) => {
                self.validate_set_local(index)?;
                let depth = relative_local_depth(index, &self.locals, &self.value_stack)?;
                self.sink.emit(isa::InstructionInternal::SetLocal(depth));
            }
            TeeLocal(index) => {
                self.validate_tee_local(index)?;
                let depth = relative_local_depth(index, &self.locals, &self.value_stack)?;
                self.sink.emit(isa::InstructionInternal::TeeLocal(depth));
            }
            GetGlobal(index) => {
                self.validate_get_global(index)?;
                self.sink.emit(isa::InstructionInternal::GetGlobal(index));
            }
            SetGlobal(index) => {
                self.validate_set_global(index)?;
                self.sink.emit(isa::InstructionInternal::SetGlobal(index));
            }

            I32Load(align, offset) => {
                self.validate_load(align, 4, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Load(offset));
            }
            I64Load(align, offset) => {
                self.validate_load(align, 8, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Load(offset));
            }
            F32Load(align, offset) => {
                self.validate_load(align, 4, ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Load(offset));
            }
            F64Load(align, offset) => {
                self.validate_load(align, 8, ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Load(offset));
            }
            I32Load8S(align, offset) => {
                self.validate_load(align, 1, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Load8S(offset));
            }
            I32Load8U(align, offset) => {
                self.validate_load(align, 1, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Load8U(offset));
            }
            I32Load16S(align, offset) => {
                self.validate_load(align, 2, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Load16S(offset));
            }
            I32Load16U(align, offset) => {
                self.validate_load(align, 2, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Load16U(offset));
            }
            I64Load8S(align, offset) => {
                self.validate_load(align, 1, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Load8S(offset));
            }
            I64Load8U(align, offset) => {
                self.validate_load(align, 1, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Load8U(offset));
            }
            I64Load16S(align, offset) => {
                self.validate_load(align, 2, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Load16S(offset));
            }
            I64Load16U(align, offset) => {
                self.validate_load(align, 2, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Load16U(offset));
            }
            I64Load32S(align, offset) => {
                self.validate_load(align, 4, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Load32S(offset));
            }
            I64Load32U(align, offset) => {
                self.validate_load(align, 4, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Load32U(offset));
            }

            I32Store(align, offset) => {
                self.validate_store(align, 4, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Store(offset));
            }
            I64Store(align, offset) => {
                self.validate_store(align, 8, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Store(offset));
            }
            F32Store(align, offset) => {
                self.validate_store(align, 4, ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Store(offset));
            }
            F64Store(align, offset) => {
                self.validate_store(align, 8, ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Store(offset));
            }
            I32Store8(align, offset) => {
                self.validate_store(align, 1, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Store8(offset));
            }
            I32Store16(align, offset) => {
                self.validate_store(align, 2, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Store16(offset));
            }
            I64Store8(align, offset) => {
                self.validate_store(align, 1, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Store8(offset));
            }
            I64Store16(align, offset) => {
                self.validate_store(align, 2, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Store16(offset));
            }
            I64Store32(align, offset) => {
                self.validate_store(align, 4, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Store32(offset));
            }

            CurrentMemory(_) => {
                self.validate_current_memory()?;
                self.sink.emit(isa::InstructionInternal::CurrentMemory);
            }
            GrowMemory(_) => {
                self.validate_grow_memory()?;
                self.sink.emit(isa::InstructionInternal::GrowMemory);
            }

            I32Const(v) => {
                self.validate_const(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Const(v));
            }
            I64Const(v) => {
                self.validate_const(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Const(v));
            }
            F32Const(v) => {
                self.validate_const(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Const(v));
            }
            F64Const(v) => {
                self.validate_const(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Const(v));
            }

            I32Eqz => {
                self.validate_testop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Eqz);
            }
            I32Eq => {
                self.validate_relop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Eq);
            }
            I32Ne => {
                self.validate_relop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Ne);
            }
            I32LtS => {
                self.validate_relop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32LtS);
            }
            I32LtU => {
                self.validate_relop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32LtU);
            }
            I32GtS => {
                self.validate_relop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32GtS);
            }
            I32GtU => {
                self.validate_relop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32GtU);
            }
            I32LeS => {
                self.validate_relop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32LeS);
            }
            I32LeU => {
                self.validate_relop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32LeU);
            }
            I32GeS => {
                self.validate_relop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32GeS);
            }
            I32GeU => {
                self.validate_relop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32GeU);
            }

            I64Eqz => {
                self.validate_testop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Eqz);
            }
            I64Eq => {
                self.validate_relop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Eq);
            }
            I64Ne => {
                self.validate_relop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Ne);
            }
            I64LtS => {
                self.validate_relop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64LtS);
            }
            I64LtU => {
                self.validate_relop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64LtU);
            }
            I64GtS => {
                self.validate_relop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64GtS);
            }
            I64GtU => {
                self.validate_relop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64GtU);
            }
            I64LeS => {
                self.validate_relop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64LeS);
            }
            I64LeU => {
                self.validate_relop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64LeU);
            }
            I64GeS => {
                self.validate_relop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64GeS);
            }
            I64GeU => {
                self.validate_relop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64GeU);
            }

            F32Eq => {
                self.validate_relop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Eq);
            }
            F32Ne => {
                self.validate_relop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Ne);
            }
            F32Lt => {
                self.validate_relop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Lt);
            }
            F32Gt => {
                self.validate_relop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Gt);
            }
            F32Le => {
                self.validate_relop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Le);
            }
            F32Ge => {
                self.validate_relop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Ge);
            }

            F64Eq => {
                self.validate_relop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Eq);
            }
            F64Ne => {
                self.validate_relop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Ne);
            }
            F64Lt => {
                self.validate_relop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Lt);
            }
            F64Gt => {
                self.validate_relop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Gt);
            }
            F64Le => {
                self.validate_relop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Le);
            }
            F64Ge => {
                self.validate_relop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Ge);
            }

            I32Clz => {
                self.validate_unop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Clz);
            }
            I32Ctz => {
                self.validate_unop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Ctz);
            }
            I32Popcnt => {
                self.validate_unop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Popcnt);
            }
            I32Add => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Add);
            }
            I32Sub => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Sub);
            }
            I32Mul => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Mul);
            }
            I32DivS => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32DivS);
            }
            I32DivU => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32DivU);
            }
            I32RemS => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32RemS);
            }
            I32RemU => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32RemU);
            }
            I32And => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32And);
            }
            I32Or => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Or);
            }
            I32Xor => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Xor);
            }
            I32Shl => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Shl);
            }
            I32ShrS => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32ShrS);
            }
            I32ShrU => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32ShrU);
            }
            I32Rotl => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Rotl);
            }
            I32Rotr => {
                self.validate_binop(ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32Rotr);
            }

            I64Clz => {
                self.validate_unop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Clz);
            }
            I64Ctz => {
                self.validate_unop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Ctz);
            }
            I64Popcnt => {
                self.validate_unop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Popcnt);
            }
            I64Add => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Add);
            }
            I64Sub => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Sub);
            }
            I64Mul => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Mul);
            }
            I64DivS => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64DivS);
            }
            I64DivU => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64DivU);
            }
            I64RemS => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64RemS);
            }
            I64RemU => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64RemU);
            }
            I64And => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64And);
            }
            I64Or => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Or);
            }
            I64Xor => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Xor);
            }
            I64Shl => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Shl);
            }
            I64ShrS => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64ShrS);
            }
            I64ShrU => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64ShrU);
            }
            I64Rotl => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Rotl);
            }
            I64Rotr => {
                self.validate_binop(ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64Rotr);
            }

            F32Abs => {
                self.validate_unop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Abs);
            }
            F32Neg => {
                self.validate_unop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Neg);
            }
            F32Ceil => {
                self.validate_unop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Ceil);
            }
            F32Floor => {
                self.validate_unop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Floor);
            }
            F32Trunc => {
                self.validate_unop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Trunc);
            }
            F32Nearest => {
                self.validate_unop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Nearest);
            }
            F32Sqrt => {
                self.validate_unop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Sqrt);
            }
            F32Add => {
                self.validate_binop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Add);
            }
            F32Sub => {
                self.validate_binop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Sub);
            }
            F32Mul => {
                self.validate_binop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Mul);
            }
            F32Div => {
                self.validate_binop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Div);
            }
            F32Min => {
                self.validate_binop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Min);
            }
            F32Max => {
                self.validate_binop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Max);
            }
            F32Copysign => {
                self.validate_binop(ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32Copysign);
            }

            F64Abs => {
                self.validate_unop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Abs);
            }
            F64Neg => {
                self.validate_unop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Neg);
            }
            F64Ceil => {
                self.validate_unop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Ceil);
            }
            F64Floor => {
                self.validate_unop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Floor);
            }
            F64Trunc => {
                self.validate_unop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Trunc);
            }
            F64Nearest => {
                self.validate_unop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Nearest);
            }
            F64Sqrt => {
                self.validate_unop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Sqrt);
            }
            F64Add => {
                self.validate_binop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Add);
            }
            F64Sub => {
                self.validate_binop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Sub);
            }
            F64Mul => {
                self.validate_binop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Mul);
            }
            F64Div => {
                self.validate_binop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Div);
            }
            F64Min => {
                self.validate_binop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Min);
            }
            F64Max => {
                self.validate_binop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Max);
            }
            F64Copysign => {
                self.validate_binop(ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64Copysign);
            }

            I32WrapI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32WrapI64);
            }
            I32TruncSF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32TruncSF32);
            }
            I32TruncUF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32TruncUF32);
            }
            I32TruncSF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32TruncSF64);
            }
            I32TruncUF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32TruncUF64);
            }
            I64ExtendSI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64ExtendSI32);
            }
            I64ExtendUI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64ExtendUI32);
            }
            I64TruncSF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64TruncSF32);
            }
            I64TruncUF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64TruncUF32);
            }
            I64TruncSF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64TruncSF64);
            }
            I64TruncUF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64TruncUF64);
            }
            F32ConvertSI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32ConvertSI32);
            }
            F32ConvertUI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32ConvertUI32);
            }
            F32ConvertSI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32ConvertSI64);
            }
            F32ConvertUI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32ConvertUI64);
            }
            F32DemoteF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32DemoteF64);
            }
            F64ConvertSI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64ConvertSI32);
            }
            F64ConvertUI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64ConvertUI32);
            }
            F64ConvertSI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64ConvertSI64);
            }
            F64ConvertUI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64ConvertUI64);
            }
            F64PromoteF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64PromoteF32);
            }

            I32ReinterpretF32 => {
                self.validate_cvtop(ValueType::F32, ValueType::I32)?;
                self.sink.emit(isa::InstructionInternal::I32ReinterpretF32);
            }
            I64ReinterpretF64 => {
                self.validate_cvtop(ValueType::F64, ValueType::I64)?;
                self.sink.emit(isa::InstructionInternal::I64ReinterpretF64);
            }
            F32ReinterpretI32 => {
                self.validate_cvtop(ValueType::I32, ValueType::F32)?;
                self.sink.emit(isa::InstructionInternal::F32ReinterpretI32);
            }
            F64ReinterpretI64 => {
                self.validate_cvtop(ValueType::I64, ValueType::F64)?;
                self.sink.emit(isa::InstructionInternal::F64ReinterpretI64);
            }
        }

        Ok(())
    }

    fn validate_const(
        &mut self,
        value_type: ValueType,
    ) -> Result<(), Error> {
        push_value(&mut self.value_stack, value_type.into())?;
        Ok(())
    }

    fn validate_unop(
        &mut self,
        value_type: ValueType,
    ) -> Result<(), Error> {
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            value_type.into(),
        )?;
        push_value(&mut self.value_stack, value_type.into())?;
        Ok(())
    }

    fn validate_binop(
        &mut self,
        value_type: ValueType,
    ) -> Result<(), Error> {
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            value_type.into(),
        )?;
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            value_type.into(),
        )?;
        push_value(&mut self.value_stack, value_type.into())?;
        Ok(())
    }

    fn validate_testop(
        &mut self,
        value_type: ValueType,
    ) -> Result<(), Error> {
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            value_type.into(),
        )?;
        push_value(&mut self.value_stack, ValueType::I32.into())?;
        Ok(())
    }

    fn validate_relop(
        &mut self,
        value_type: ValueType,
    ) -> Result<(), Error> {
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            value_type.into(),
        )?;
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            value_type.into(),
        )?;
        push_value(&mut self.value_stack, ValueType::I32.into())?;
        Ok(())
    }

    fn validate_cvtop(
        &mut self,
        value_type1: ValueType,
        value_type2: ValueType,
    ) -> Result<(), Error> {
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            value_type1.into(),
        )?;
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

    fn validate_get_local(
        &mut self,
        index: u32,
    ) -> Result<(), Error> {
        let local_type = require_local(&self.locals, index)?;
        push_value(&mut self.value_stack, local_type.into())?;
        Ok(())
    }

    fn validate_set_local(
        &mut self,
        index: u32,
    ) -> Result<(), Error> {
        let local_type = require_local(&self.locals, index)?;
        let value_type = pop_value(
            &mut self.value_stack,
            &self.frame_stack,
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
        &mut self,
        index: u32,
    ) -> Result<(), Error> {
        let local_type = require_local(&self.locals, index)?;
        tee_value(
            &mut self.value_stack,
            &self.frame_stack,
            local_type.into(),
        )?;
        Ok(())
    }

    fn validate_get_global(
        &mut self,
        index: u32,
    ) -> Result<(), Error> {
        let global_type: StackValueType = {
            let global = self.module.require_global(index, None)?;
            global.content_type().into()
        };
        push_value(&mut self.value_stack, global_type)?;
        Ok(())
    }

    fn validate_set_global(
        &mut self,
        index: u32,
    ) -> Result<(), Error> {
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
        pop_value(
            &mut self.value_stack,
            &self.frame_stack,
            value_type.into(),
        )?;
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
                tee_value(
                    &mut self.value_stack,
                    &self.frame_stack,
                    value_type.into(),
                )?;
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
                tee_value(
                    &mut self.value_stack,
                    &self.frame_stack,
                    value_type.into(),
                )?;
            }
        }
        Ok(())
    }

    fn validate_br_table(
        &mut self,
        table: &[u32],
        default: u32,
    ) -> Result<(), Error> {
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
            tee_value(
                &mut self.value_stack,
                &self.frame_stack,
                value_type.into(),
            )?;
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

    fn validate_call_indirect(
        &mut self,
        idx: u32,
    ) -> Result<(), Error> {
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
    started_with: StartedWith,
    block_type: BlockType,
    position: usize,
    value_stack: &StackWithLimit<StackValueType>,
    frame_stack: &mut StackWithLimit<BlockFrame>,
) -> Result<(), Error> {
    Ok(frame_stack.push(BlockFrame {
        started_with,
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
    // TODO: This actually isn't safe.
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

fn compute_drop_keep(
    in_stack_polymorphic_state: bool,
    started_with: StartedWith,
    block_type: BlockType,
    actual_value_stack_height: usize,
    start_value_stack_height: usize,
) -> Result<isa::DropKeep, Error> {
    // Find out how many values we need to keep (copy to the new stack location after the drop).
    let keep: isa::Keep = match (started_with, block_type) {
        // A loop doesn't take a value upon a branch. It can return value
        // only via reaching it's closing `End` operator.
        (StartedWith::Loop, _) => isa::Keep::None,

        (_, BlockType::Value(_)) => isa::Keep::Single,
        (_, BlockType::NoResult) => isa::Keep::None,
    };

    // Find out how many values we need to discard.
    let drop = if in_stack_polymorphic_state {
        // Polymorphic stack is a weird state. Fortunately, it is always about the code that
        // will not be executed, so we don't bother and return 0 here.
        0
    } else {
        if actual_value_stack_height < start_value_stack_height {
            return Err(Error(format!(
                "Stack underflow detected: value stack height ({}) is lower than minimum stack len ({})",
                actual_value_stack_height,
                start_value_stack_height,
            )));
        }
        if (actual_value_stack_height as u32 - start_value_stack_height as u32) < keep as u32 {
            return Err(Error(format!(
                "Stack underflow detected: asked to keep {:?} values, but there are only {}",
                keep,
                actual_value_stack_height as u32 - start_value_stack_height as u32,
            )));
        }
        (actual_value_stack_height as u32 - start_value_stack_height as u32) - keep as u32
    };

    Ok(isa::DropKeep { drop, keep })
}

fn require_target(
    depth: u32,
    value_stack_height: usize,
    frame_stack: &StackWithLimit<BlockFrame>,
    label_stack: &[BlockFrameType],
) -> Result<Target, Error> {
    let is_stack_polymorphic = top_label(frame_stack).polymorphic_stack;
    let frame = require_label(depth, frame_stack)?;

    // TODO: Clean this mess.
    let label = label_stack
        .get(label_stack.len() - 1 - (depth as usize))
        .expect("this is ensured by `require_label` above");

    let drop_keep = compute_drop_keep(
        is_stack_polymorphic,
        frame.started_with,
        frame.block_type,
        value_stack_height,
        frame.value_stack_len,
    )?;

    Ok(Target {
        label: label.br_destination(),
        drop_keep,
    })
}

/// Compute drop/keep for the return statement.
///
/// This function is a bit of unusual since it is called before validation and thus
/// should deal with invalid code.
fn drop_keep_return(
    locals: &Locals,
    value_stack: &StackWithLimit<StackValueType>,
    frame_stack: &StackWithLimit<BlockFrame>,
) -> Result<isa::DropKeep, Error> {
    if frame_stack.is_empty() {
        return Err(Error(
            "drop_keep_return can't be called with the frame stack empty".into(),
        ));
    }

    let is_stack_polymorphic = top_label(frame_stack).polymorphic_stack;
    let deepest = (frame_stack.len() - 1) as u32;
    let frame = require_label(deepest, frame_stack).expect("frame_stack is not empty");
    let mut drop_keep = compute_drop_keep(
        is_stack_polymorphic,
        frame.started_with,
        frame.block_type,
        value_stack.len(),
        frame.value_stack_len,
    )?;

    // Drop all local variables and parameters upon exit.
    drop_keep.drop += locals.count();

    Ok(drop_keep)
}

fn require_local(locals: &Locals, idx: u32) -> Result<ValueType, Error> {
    Ok(locals.type_of_local(idx)?)
}

/// Returns a relative depth on the stack of a local variable specified
/// by `idx`.
///
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
