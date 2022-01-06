use alloc::{string::String, vec::Vec};

use parity_wasm::elements::{BlockType, FuncBody, Instruction};

use crate::isa;
use validation::{
    func::{
        require_label,
        top_label,
        BlockFrame,
        FunctionValidationContext,
        StackValueType,
        StartedWith,
    },
    stack::StackWithLimit,
    util::Locals,
    Error,
    FuncValidator,
};

/// Type of block frame.
#[derive(Debug, Clone, Copy)]
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

pub struct Compiler {
    /// A sink used to emit optimized code.
    sink: Sink,
    label_stack: Vec<BlockFrameType>,
}

impl FuncValidator for Compiler {
    type Input = ();
    type Output = isa::Instructions;
    fn new(_ctx: &FunctionValidationContext, body: &FuncBody, _input: Self::Input) -> Self {
        let code_len = body.code().elements().len();
        let mut compiler = Compiler {
            sink: Sink::with_capacity(code_len),
            label_stack: Vec::new(),
        };

        // Push implicit frame for the outer function block.
        let end_label = compiler.sink.new_label();
        compiler
            .label_stack
            .push(BlockFrameType::Block { end_label });

        compiler
    }
    fn next_instruction(
        &mut self,
        ctx: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), Error> {
        self.compile_instruction(ctx, instruction)
    }
    fn finish(self, _ctx: &FunctionValidationContext) -> Self::Output {
        self.sink.into_inner()
    }
}

impl Compiler {
    fn compile_instruction(
        &mut self,
        context: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), Error> {
        use self::Instruction::*;

        match *instruction {
            Unreachable => {
                self.sink.emit(isa::InstructionInternal::Unreachable);
                context.step(instruction)?;
            }
            Block(_) => {
                context.step(instruction)?;

                let end_label = self.sink.new_label();
                self.label_stack.push(BlockFrameType::Block { end_label });
            }
            Loop(_) => {
                context.step(instruction)?;

                // Resolve loop header right away.
                let header = self.sink.new_label();
                self.sink.resolve_label(header);
                self.label_stack.push(BlockFrameType::Loop { header });
            }
            If(_) => {
                context.step(instruction)?;

                // `if_not` will be resolved whenever `End` or `Else` operator will be met.
                // `end_label` will always be resolved at `End`.
                let if_not = self.sink.new_label();
                let end_label = self.sink.new_label();
                self.label_stack
                    .push(BlockFrameType::IfTrue { if_not, end_label });

                self.sink.emit_br_eqz(Target {
                    label: if_not,
                    drop_keep: isa::DropKeep {
                        drop: 0,
                        keep: isa::Keep::None,
                    },
                });
            }
            Else => {
                context.step(instruction)?;

                let top_label = self.label_stack.pop().expect(
                    "label_stack should reflect the frame stack;
                    frame stack is never empty while being processed; qed",
                );
                let (if_not, end_label) = match top_label {
                    BlockFrameType::IfTrue { if_not, end_label } => (if_not, end_label),
                    _ => unreachable!(
                        "validation ensures that the top frame was opened by If block;
                        `top_label` should be `IfTrue` at this point;
                        this statement is unreachable;
                        qed"
                    ),
                };

                // First, we need to finish if-true block: add a jump from the end of the if-true block
                // to the "end_label" (it will be resolved at End).
                self.sink.emit_br(Target {
                    label: end_label,
                    drop_keep: isa::DropKeep {
                        drop: 0,
                        keep: isa::Keep::None,
                    },
                });

                // Resolve `if_not` to here so when if condition is unsatisfied control flow
                // will jump to this label.
                self.sink.resolve_label(if_not);

                self.label_stack.push(BlockFrameType::IfFalse { end_label });
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

                let top_frame_type = self.label_stack.pop().expect(
                    "label_stack should reflect the frame stack;
                    frame stack is never empty while being processed; qed",
                );

                if let BlockFrameType::IfTrue { if_not, .. } = top_frame_type {
                    // Resolve `if_not` label. If the `if's` condition doesn't hold the control will jump
                    // to here.
                    self.sink.resolve_label(if_not);
                }

                // Unless it's a loop, resolve the `end_label` position here.
                if started_with != StartedWith::Loop {
                    let end_label = top_frame_type.end_label();
                    self.sink.resolve_label(end_label);
                }

                if let Some(drop_keep) = return_drop_keep {
                    // It was the last instruction. Emit the explicit return instruction.
                    let drop_keep = drop_keep.expect(
                        "validation step ensures that the value stack underflows;
                        validation also ensures that the frame stack is not empty;
                        `drop_keep_return` can't fail;
                        qed",
                    );
                    self.sink.emit(isa::InstructionInternal::Return(drop_keep));
                }
            }
            Br(depth) => {
                let target = require_target(
                    depth,
                    context.value_stack.len(),
                    &context.frame_stack,
                    &self.label_stack,
                );

                context.step(instruction)?;

                let target = target.expect(
                    "validation step ensures that the value stack underflows;
                    validation also ensures that the depth is correct;
                    require_target doesn't fail;
                    qed",
                );
                self.sink.emit_br(target);
            }
            BrIf(depth) => {
                context.step(instruction)?;

                let target = require_target(
                    depth,
                    context.value_stack.len(),
                    &context.frame_stack,
                    &self.label_stack,
                )
                .expect(
                    "validation step ensures that the value stack underflows;
                    validation also ensures that the depth is correct;
                    require_target doesn't fail;
                    qed",
                );
                self.sink.emit_br_nez(target);
            }
            BrTable(ref br_table_data) => {
                // At this point, the condition value is at the top of the stack.
                // But at the point of actual jump the condition will already be
                // popped off.
                let value_stack_height = context.value_stack.len().saturating_sub(1);

                let targets = br_table_data
                    .table
                    .iter()
                    .map(|depth| {
                        require_target(
                            *depth,
                            value_stack_height,
                            &context.frame_stack,
                            &self.label_stack,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>();
                let default_target = require_target(
                    br_table_data.default,
                    value_stack_height,
                    &context.frame_stack,
                    &self.label_stack,
                );

                context.step(instruction)?;

                // These two unwraps are guaranteed to succeed by validation.
                const REQUIRE_TARGET_PROOF: &str =
                    "validation step ensures that the value stack underflows;
                    validation also ensures that the depth is correct;
                    qed";
                let targets = targets.expect(REQUIRE_TARGET_PROOF);
                let default_target = default_target.expect(REQUIRE_TARGET_PROOF);

                self.sink.emit_br_table(&targets, default_target);
            }
            Return => {
                let drop_keep =
                    drop_keep_return(&context.locals, &context.value_stack, &context.frame_stack);

                context.step(instruction)?;

                let drop_keep = drop_keep.expect(
                    "validation step ensures that the value stack underflows;
                    validation also ensures that the frame stack is not empty;
                    `drop_keep_return` can't fail;
                    qed",
                );
                self.sink.emit(isa::InstructionInternal::Return(drop_keep));
            }
            Call(index) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::Call(index));
            }
            CallIndirect(index, _reserved) => {
                context.step(instruction)?;
                self.sink
                    .emit(isa::InstructionInternal::CallIndirect(index));
            }

            Drop => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::Drop);
            }
            Select => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::Select);
            }

            GetLocal(index) => {
                // We need to calculate relative depth before validation since
                // it will change the value stack size.
                let depth = relative_local_depth(index, &context.locals, &context.value_stack)?;
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::GetLocal(depth));
            }
            SetLocal(index) => {
                context.step(instruction)?;
                let depth = relative_local_depth(index, &context.locals, &context.value_stack)?;
                self.sink.emit(isa::InstructionInternal::SetLocal(depth));
            }
            TeeLocal(index) => {
                context.step(instruction)?;
                let depth = relative_local_depth(index, &context.locals, &context.value_stack)?;
                self.sink.emit(isa::InstructionInternal::TeeLocal(depth));
            }
            GetGlobal(index) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::GetGlobal(index));
            }
            SetGlobal(index) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::SetGlobal(index));
            }

            I32Load(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Load(offset));
            }
            I64Load(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Load(offset));
            }
            F32Load(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Load(offset));
            }
            F64Load(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Load(offset));
            }
            I32Load8S(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Load8S(offset));
            }
            I32Load8U(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Load8U(offset));
            }
            I32Load16S(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Load16S(offset));
            }
            I32Load16U(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Load16U(offset));
            }
            I64Load8S(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Load8S(offset));
            }
            I64Load8U(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Load8U(offset));
            }
            I64Load16S(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Load16S(offset));
            }
            I64Load16U(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Load16U(offset));
            }
            I64Load32S(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Load32S(offset));
            }
            I64Load32U(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Load32U(offset));
            }

            I32Store(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Store(offset));
            }
            I64Store(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Store(offset));
            }
            F32Store(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Store(offset));
            }
            F64Store(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Store(offset));
            }
            I32Store8(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Store8(offset));
            }
            I32Store16(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Store16(offset));
            }
            I64Store8(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Store8(offset));
            }
            I64Store16(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Store16(offset));
            }
            I64Store32(_, offset) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Store32(offset));
            }

            CurrentMemory(_) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::CurrentMemory);
            }
            GrowMemory(_) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::GrowMemory);
            }

            I32Const(v) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Const(v));
            }
            I64Const(v) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Const(v));
            }
            F32Const(v) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Const(v));
            }
            F64Const(v) => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Const(v));
            }

            I32Eqz => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Eqz);
            }
            I32Eq => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Eq);
            }
            I32Ne => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Ne);
            }
            I32LtS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32LtS);
            }
            I32LtU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32LtU);
            }
            I32GtS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32GtS);
            }
            I32GtU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32GtU);
            }
            I32LeS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32LeS);
            }
            I32LeU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32LeU);
            }
            I32GeS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32GeS);
            }
            I32GeU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32GeU);
            }

            I64Eqz => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Eqz);
            }
            I64Eq => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Eq);
            }
            I64Ne => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Ne);
            }
            I64LtS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64LtS);
            }
            I64LtU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64LtU);
            }
            I64GtS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64GtS);
            }
            I64GtU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64GtU);
            }
            I64LeS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64LeS);
            }
            I64LeU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64LeU);
            }
            I64GeS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64GeS);
            }
            I64GeU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64GeU);
            }

            F32Eq => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Eq);
            }
            F32Ne => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Ne);
            }
            F32Lt => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Lt);
            }
            F32Gt => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Gt);
            }
            F32Le => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Le);
            }
            F32Ge => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Ge);
            }

            F64Eq => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Eq);
            }
            F64Ne => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Ne);
            }
            F64Lt => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Lt);
            }
            F64Gt => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Gt);
            }
            F64Le => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Le);
            }
            F64Ge => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Ge);
            }

            I32Clz => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Clz);
            }
            I32Ctz => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Ctz);
            }
            I32Popcnt => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Popcnt);
            }
            I32Add => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Add);
            }
            I32Sub => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Sub);
            }
            I32Mul => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Mul);
            }
            I32DivS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32DivS);
            }
            I32DivU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32DivU);
            }
            I32RemS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32RemS);
            }
            I32RemU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32RemU);
            }
            I32And => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32And);
            }
            I32Or => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Or);
            }
            I32Xor => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Xor);
            }
            I32Shl => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Shl);
            }
            I32ShrS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32ShrS);
            }
            I32ShrU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32ShrU);
            }
            I32Rotl => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Rotl);
            }
            I32Rotr => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32Rotr);
            }

            I64Clz => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Clz);
            }
            I64Ctz => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Ctz);
            }
            I64Popcnt => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Popcnt);
            }
            I64Add => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Add);
            }
            I64Sub => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Sub);
            }
            I64Mul => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Mul);
            }
            I64DivS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64DivS);
            }
            I64DivU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64DivU);
            }
            I64RemS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64RemS);
            }
            I64RemU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64RemU);
            }
            I64And => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64And);
            }
            I64Or => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Or);
            }
            I64Xor => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Xor);
            }
            I64Shl => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Shl);
            }
            I64ShrS => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64ShrS);
            }
            I64ShrU => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64ShrU);
            }
            I64Rotl => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Rotl);
            }
            I64Rotr => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64Rotr);
            }

            F32Abs => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Abs);
            }
            F32Neg => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Neg);
            }
            F32Ceil => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Ceil);
            }
            F32Floor => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Floor);
            }
            F32Trunc => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Trunc);
            }
            F32Nearest => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Nearest);
            }
            F32Sqrt => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Sqrt);
            }
            F32Add => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Add);
            }
            F32Sub => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Sub);
            }
            F32Mul => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Mul);
            }
            F32Div => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Div);
            }
            F32Min => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Min);
            }
            F32Max => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Max);
            }
            F32Copysign => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32Copysign);
            }

            F64Abs => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Abs);
            }
            F64Neg => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Neg);
            }
            F64Ceil => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Ceil);
            }
            F64Floor => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Floor);
            }
            F64Trunc => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Trunc);
            }
            F64Nearest => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Nearest);
            }
            F64Sqrt => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Sqrt);
            }
            F64Add => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Add);
            }
            F64Sub => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Sub);
            }
            F64Mul => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Mul);
            }
            F64Div => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Div);
            }
            F64Min => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Min);
            }
            F64Max => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Max);
            }
            F64Copysign => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64Copysign);
            }

            I32WrapI64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32WrapI64);
            }
            I32TruncSF32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32TruncSF32);
            }
            I32TruncUF32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32TruncUF32);
            }
            I32TruncSF64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32TruncSF64);
            }
            I32TruncUF64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32TruncUF64);
            }
            I64ExtendSI32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64ExtendSI32);
            }
            I64ExtendUI32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64ExtendUI32);
            }
            I64TruncSF32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64TruncSF32);
            }
            I64TruncUF32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64TruncUF32);
            }
            I64TruncSF64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64TruncSF64);
            }
            I64TruncUF64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64TruncUF64);
            }
            F32ConvertSI32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32ConvertSI32);
            }
            F32ConvertUI32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32ConvertUI32);
            }
            F32ConvertSI64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32ConvertSI64);
            }
            F32ConvertUI64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32ConvertUI64);
            }
            F32DemoteF64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32DemoteF64);
            }
            F64ConvertSI32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64ConvertSI32);
            }
            F64ConvertUI32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64ConvertUI32);
            }
            F64ConvertSI64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64ConvertSI64);
            }
            F64ConvertUI64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64ConvertUI64);
            }
            F64PromoteF32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64PromoteF32);
            }

            I32ReinterpretF32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I32ReinterpretF32);
            }
            I64ReinterpretF64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::I64ReinterpretF64);
            }
            F32ReinterpretI32 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F32ReinterpretI32);
            }
            F64ReinterpretI64 => {
                context.step(instruction)?;
                self.sink.emit(isa::InstructionInternal::F64ReinterpretI64);
            }
            _ => {
                context.step(instruction)?;
            }
        };

        assert_eq!(self.label_stack.len(), context.frame_stack.len(),);

        Ok(())
    }
}

/// Computes how many values should be dropped and kept for the specific branch.
///
/// Returns `Err` if underflow of the value stack detected.
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
        if (actual_value_stack_height as u32 - start_value_stack_height as u32) < keep.count() {
            return Err(Error(format!(
                "Stack underflow detected: asked to keep {:?} values, but there are only {}",
                keep,
                actual_value_stack_height as u32 - start_value_stack_height as u32,
            )));
        }
        (actual_value_stack_height as u32 - start_value_stack_height as u32) - keep.count()
    };

    Ok(isa::DropKeep { drop, keep })
}

/// Returns the requested target for branch referred by `depth`.
///
/// Returns `Err` if
/// - if the `depth` is greater than the current height of the frame stack
/// - if underflow of the value stack detected.
fn require_target(
    depth: u32,
    value_stack_height: usize,
    frame_stack: &StackWithLimit<BlockFrame>,
    label_stack: &[BlockFrameType],
) -> Result<Target, Error> {
    let is_stack_polymorphic = top_label(frame_stack).polymorphic_stack;
    let frame = require_label(depth, frame_stack)?;

    // Get the label by the given `depth`.
    let idx = label_stack
        .len()
        .checked_sub(1)
        .expect("this is ensured by `require_label` above")
        .checked_sub(depth as usize)
        .expect("this is ensured by `require_label` above");
    let label = label_stack
        .get(idx)
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
/// Returns `Err` if:
/// - frame stack is empty.
/// - underflow of the value stack detected.
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
    let deepest = frame_stack
        .len()
        .checked_sub(1)
        .expect("frame_stack is not empty") as u32;
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
    fn with_capacity(capacity: usize) -> Sink {
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
            drop_keep,
        }));
    }

    fn emit_br_eqz(&mut self, target: Target) {
        let Target { label, drop_keep } = target;
        let pc = self.cur_pc();
        let dst_pc = self.pc_or_placeholder(label, || isa::Reloc::Br { pc });
        self.ins
            .push(isa::InstructionInternal::BrIfEqz(isa::Target {
                dst_pc,
                drop_keep,
            }));
    }

    fn emit_br_nez(&mut self, target: Target) {
        let Target { label, drop_keep } = target;
        let pc = self.cur_pc();
        let dst_pc = self.pc_or_placeholder(label, || isa::Reloc::Br { pc });
        self.ins
            .push(isa::InstructionInternal::BrIfNez(isa::Target {
                dst_pc,
                drop_keep,
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
                    drop_keep,
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
        let unresolved_rels = mem::take(&mut self.labels[label.0].1);
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
                    .all(|(state, unresolved)| {
                        matches!((state, unresolved), (Label::Resolved(_), unresolved) if unresolved.is_empty())
                    })
            },
            "there are unresolved labels left: {:?}",
            self.labels
        );
        self.ins
    }
}
