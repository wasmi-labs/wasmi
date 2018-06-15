use std::u32;
use std::collections::HashMap;
use parity_wasm::elements::{Opcode, BlockType, ValueType, TableElementType, Func, FuncBody};
use common::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX};
use validation::context::ModuleContext;

use validation::Error;
use validation::util::Locals;

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
	Block {
		end_label: LabelId,
	},
	/// Loop frame (branching to the beginning of block).
	Loop {
		header: LabelId,
	},
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
	IfFalse {
		end_label: LabelId,
	}
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

/// Function validator.
pub struct Validator;

/// Instruction outcome.
#[derive(Debug, Clone)]
enum InstructionOutcome {
	/// Continue with next instruction.
	ValidateNextInstruction,
	/// Unreachable instruction reached.
	Unreachable,
}

impl Validator {
	pub fn validate_function(
		module: &ModuleContext,
		func: &Func,
		body: &FuncBody,
	) -> Result<isa::Instructions, Error> {
		let (params, result_ty) = module.require_function_type(func.type_ref())?;

		let mut context = FunctionValidationContext::new(
			&module,
			Locals::new(params, body.locals()),
			DEFAULT_VALUE_STACK_LIMIT,
			DEFAULT_FRAME_STACK_LIMIT,
			result_ty,
		);

		let end_label = context.sink.new_label();
		context.push_label(
			BlockFrameType::Block {
				end_label,
			},
			result_ty
		)?;
		Validator::validate_function_block(&mut context, body.code().elements())?;

		while !context.frame_stack.is_empty() {
			context.pop_label()?;
		}

		Ok(context.into_code())
	}

	fn validate_function_block(context: &mut FunctionValidationContext, body: &[Opcode]) -> Result<(), Error> {
		let body_len = body.len();
		if body_len == 0 {
			return Err(Error("Non-empty function body expected".into()));
		}

		loop {
			let opcode = &body[context.position];

			let outcome = Validator::validate_instruction(context, opcode)
				.map_err(|err| Error(format!("At instruction {:?}(@{}): {}", opcode, context.position, err)))?;

			println!("opcode: {:?}, outcome={:?}", opcode, outcome);
			match outcome {
				InstructionOutcome::ValidateNextInstruction => (),
				InstructionOutcome::Unreachable => context.unreachable()?,
			}

			context.position += 1;
			if context.position == body_len {
				return Ok(());
			}
		}
	}

	fn validate_instruction(context: &mut FunctionValidationContext, opcode: &Opcode) -> Result<InstructionOutcome, Error> {
		use self::Opcode::*;
		match *opcode {
			// Nop instruction doesn't do anything. It is safe to just skip it.
			Nop => {},

			Unreachable => {
				context.sink.emit(isa::Instruction::Unreachable);
				return Ok(InstructionOutcome::Unreachable);
			}

			Block(block_type) => {
				let end_label = context.sink.new_label();
				context.push_label(
					BlockFrameType::Block {
						end_label
					},
					block_type
				)?;
			}
			Loop(block_type) => {
				// Resolve loop header right away.
				let header = context.sink.new_label();
				context.sink.resolve_label(header);

				context.push_label(
					BlockFrameType::Loop {
						header,
					},
					block_type
				)?;
			}
			If(block_type) => {
				// `if_not` will be resolved whenever `End` or `Else` operator will be met.
				// `end_label` will always be resolved at `End`.
				let if_not = context.sink.new_label();
				let end_label = context.sink.new_label();

				context.pop_value(ValueType::I32.into())?;
				context.push_label(
					BlockFrameType::IfTrue {
						if_not,
						end_label,
					},
					block_type
				)?;

				context.sink.emit_br_eqz(Target {
					label: if_not,
					drop_keep: DropKeep { drop: 0, keep: 0 },
				});
			}
			Else => {
				let (block_type, if_not, end_label) = {
					let top_frame = context.top_label()?;

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
					drop_keep: DropKeep { drop: 0, keep: 0 },
				});

				// Resolve `if_not` to here so when if condition is unsatisfied control flow
				// will jump to this label.
				context.sink.resolve_label(if_not);

				// Then, we validate. Validator will pop the if..else block and the push else..end block.
				context.pop_label()?;

				if let BlockType::Value(value_type) = block_type {
					context.pop_value(value_type.into())?;
				}
				context.push_label(
					BlockFrameType::IfFalse {
						end_label,
					},
					block_type,
				)?;
			}
			End => {
				{
					let frame_type = context.top_label()?.frame_type;

					if let BlockFrameType::IfTrue { if_not, .. } = frame_type {
						// A `if` without an `else` can't return a result.
						if context.top_label()?.block_type != BlockType::NoResult {
							return Err(
								Error(
									format!(
										"If block without else required to have NoResult block type. But it has {:?} type",
										context.top_label()?.block_type
									)
								)
							);
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
				}

				if context.frame_stack.len() == 1 {
					// We are about to close the last frame. Insert
					// an explicit return.
					let DropKeep { drop, keep } = context.drop_keep_return()?;
					context.sink.emit(isa::Instruction::Return {
						drop,
						keep,
					});
				}

				context.pop_label()?;
			}
			Br(depth) => {
				Validator::validate_br(context, depth)?;

				let target = context.require_target(depth)?;
				context.sink.emit_br(target);

				return Ok(InstructionOutcome::Unreachable);
			}
			BrIf(depth) => {
				Validator::validate_br_if(context, depth)?;

				let target = context.require_target(depth)?;
				context.sink.emit_br_nez(target);
			}
			BrTable(ref table, default) => {
				Validator::validate_br_table(context, table, default)?;

				let mut targets = Vec::new();
				for depth in table.iter() {
					let target = context.require_target(*depth)?;
					targets.push(target);
				}
				let default_target = context.require_target(default)?;
				context.sink.emit_br_table(&targets, default_target);

				return Ok(InstructionOutcome::Unreachable);
			}
			Return => {
				if let BlockType::Value(value_type) = context.return_type()? {
					context.tee_value(value_type.into())?;
				}

				let DropKeep { drop, keep } = context.drop_keep_return()?;
				context.sink.emit(isa::Instruction::Return {
					drop,
					keep,
				});

				return Ok(InstructionOutcome::Unreachable);
			}

			Call(index) => {
				Validator::validate_call(context, index)?;
				context.sink.emit(isa::Instruction::Call(index));
			}
			CallIndirect(index, _reserved) => {
				Validator::validate_call_indirect(context, index)?;
				context.sink.emit(isa::Instruction::CallIndirect(index));
			}

			Drop => {
				Validator::validate_drop(context)?;
				context.sink.emit(isa::Instruction::Drop);
			}
			Select => {
				Validator::validate_select(context)?;
				context.sink.emit(isa::Instruction::Select);
			}

			GetLocal(index) => {
				// We need to calculate relative depth before validation since
				// it will change the value stack size.
				let depth = context.relative_local_depth(index)?;
				Validator::validate_get_local(context, index)?;
				context.sink.emit(
					isa::Instruction::GetLocal(depth),
				);
			}
			SetLocal(index) => {
				Validator::validate_set_local(context, index)?;
				let depth = context.relative_local_depth(index)?;
				context.sink.emit(
					isa::Instruction::SetLocal(depth),
				);
			}
			TeeLocal(index) => {
				Validator::validate_tee_local(context, index)?;
				let depth = context.relative_local_depth(index)?;
				context.sink.emit(
					isa::Instruction::TeeLocal(depth),
				);
			}
			GetGlobal(index) => {
				Validator::validate_get_global(context, index)?;
				context.sink.emit(isa::Instruction::GetGlobal(index));
			}
			SetGlobal(index) => {
				Validator::validate_set_global(context, index)?;
				context.sink.emit(isa::Instruction::SetGlobal(index));
			}

			I32Load(align, offset) => {
				Validator::validate_load(context, align, 4, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Load(offset));
			}
			I64Load(align, offset) => {
				Validator::validate_load(context, align, 8, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Load(offset));
			}
			F32Load(align, offset) => {
				Validator::validate_load(context, align, 4, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Load(offset));
			}
			F64Load(align, offset) => {
				Validator::validate_load(context, align, 8, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Load(offset));
			}
			I32Load8S(align, offset) => {
				Validator::validate_load(context, align, 1, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Load8S(offset));
			}
			I32Load8U(align, offset) => {
				Validator::validate_load(context, align, 1, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Load8U(offset));
			}
			I32Load16S(align, offset) => {
				Validator::validate_load(context, align, 2, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Load16S(offset));
			}
			I32Load16U(align, offset) => {
				Validator::validate_load(context, align, 2, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Load16U(offset));
			}
			I64Load8S(align, offset) => {
				Validator::validate_load(context, align, 1, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Load8S(offset));
			}
			I64Load8U(align, offset) => {
				Validator::validate_load(context, align, 1, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Load8U(offset));
			}
			I64Load16S(align, offset) => {
				Validator::validate_load(context, align, 2, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Load16S(offset));
			}
			I64Load16U(align, offset) => {
				Validator::validate_load(context, align, 2, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Load16U(offset));
			}
			I64Load32S(align, offset) => {
				Validator::validate_load(context, align, 4, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Load32S(offset));
			}
			I64Load32U(align, offset) => {
				Validator::validate_load(context, align, 4, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Load32U(offset));
			}

			I32Store(align, offset) => {
				Validator::validate_store(context, align, 4, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Store(offset));
			}
			I64Store(align, offset) => {
				Validator::validate_store(context, align, 8, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Store(offset));
			}
			F32Store(align, offset) => {
				Validator::validate_store(context, align, 4, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Store(offset));
			}
			F64Store(align, offset) => {
				Validator::validate_store(context, align, 8, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Store(offset));
			}
			I32Store8(align, offset) => {
				Validator::validate_store(context, align, 1, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Store8(offset));
			}
			I32Store16(align, offset) => {
				Validator::validate_store(context, align, 2, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Store16(offset));
			}
			I64Store8(align, offset) => {
				Validator::validate_store(context, align, 1, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Store8(offset));
			}
			I64Store16(align, offset) => {
				Validator::validate_store(context, align, 2, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Store16(offset));
			}
			I64Store32(align, offset) => {
				Validator::validate_store(context, align, 4, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Store32(offset));
			}

			CurrentMemory(_) => {
				Validator::validate_current_memory(context)?;
				context.sink.emit(isa::Instruction::CurrentMemory);
			}
			GrowMemory(_) => {
				Validator::validate_grow_memory(context)?;
				context.sink.emit(isa::Instruction::GrowMemory);
			}

			I32Const(v) => {
				Validator::validate_const(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Const(v));
			}
			I64Const(v) => {
				Validator::validate_const(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Const(v));
			}
			F32Const(v) => {
				Validator::validate_const(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Const(v));
			}
			F64Const(v) => {
				Validator::validate_const(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Const(v));
			}

			I32Eqz => {
				Validator::validate_testop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Eqz);
			}
			I32Eq => {
				Validator::validate_relop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Eq);
			}
			I32Ne => {
				Validator::validate_relop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Ne);
			}
			I32LtS => {
				Validator::validate_relop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32LtS);
			}
			I32LtU => {
				Validator::validate_relop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32LtU);
			}
			I32GtS => {
				Validator::validate_relop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32GtS);
			}
			I32GtU => {
				Validator::validate_relop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32GtU);
			}
			I32LeS => {
				Validator::validate_relop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32LeS);
			}
			I32LeU => {
				Validator::validate_relop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32LeU);
			}
			I32GeS => {
				Validator::validate_relop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32GeS);
			}
			I32GeU => {
				Validator::validate_relop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32GeU);
			}

			I64Eqz => {
				Validator::validate_testop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Eqz);
			}
			I64Eq => {
				Validator::validate_relop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Eq);
			}
			I64Ne => {
				Validator::validate_relop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Ne);
			}
			I64LtS => {
				Validator::validate_relop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64LtS);
			}
			I64LtU => {
				Validator::validate_relop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64LtU);
			}
			I64GtS => {
				Validator::validate_relop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64GtS);
			}
			I64GtU => {
				Validator::validate_relop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64GtU);
			}
			I64LeS => {
				Validator::validate_relop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64LeS);
			}
			I64LeU => {
				Validator::validate_relop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64LeU);
			}
			I64GeS => {
				Validator::validate_relop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64GeS);
			}
			I64GeU => {
				Validator::validate_relop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64GeU);
			}

			F32Eq => {
				Validator::validate_relop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Eq);
			}
			F32Ne => {
				Validator::validate_relop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Ne);
			}
			F32Lt => {
				Validator::validate_relop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Lt);
			}
			F32Gt => {
				Validator::validate_relop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Gt);
			}
			F32Le => {
				Validator::validate_relop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Le);
			}
			F32Ge => {
				Validator::validate_relop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Ge);
			}

			F64Eq => {
				Validator::validate_relop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Eq);
			}
			F64Ne => {
				Validator::validate_relop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Ne);
			}
			F64Lt => {
				Validator::validate_relop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Lt);
			}
			F64Gt => {
				Validator::validate_relop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Gt);
			}
			F64Le => {
				Validator::validate_relop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Le);
			}
			F64Ge => {
				Validator::validate_relop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Ge);
			}

			I32Clz => {
				Validator::validate_unop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Clz);
			}
			I32Ctz => {
				Validator::validate_unop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Ctz);
			}
			I32Popcnt => {
				Validator::validate_unop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Popcnt);
			}
			I32Add => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Add);
			}
			I32Sub => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Sub);
			}
			I32Mul => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Mul);
			}
			I32DivS => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32DivS);
			}
			I32DivU => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32DivU);
			}
			I32RemS => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32RemS);
			}
			I32RemU => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32RemU);
			}
			I32And => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32And);
			}
			I32Or => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Or);
			}
			I32Xor => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Xor);
			}
			I32Shl => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Shl);
			}
			I32ShrS => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32ShrS);
			}
			I32ShrU => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32ShrU);
			}
			I32Rotl => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Rotl);
			}
			I32Rotr => {
				Validator::validate_binop(context, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32Rotr);
			}

			I64Clz => {
				Validator::validate_unop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Clz);
			}
			I64Ctz => {
				Validator::validate_unop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Ctz);
			}
			I64Popcnt => {
				Validator::validate_unop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Popcnt);
			}
			I64Add => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Add);
			}
			I64Sub => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Sub);
			}
			I64Mul => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Mul);
			}
			I64DivS => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64DivS);
			}
			I64DivU => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64DivU);
			}
			I64RemS => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64RemS);
			}
			I64RemU => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64RemU);
			}
			I64And => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64And);
			}
			I64Or => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Or);
			}
			I64Xor => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Xor);
			}
			I64Shl => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Shl);
			}
			I64ShrS => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64ShrS);
			}
			I64ShrU => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64ShrU);
			}
			I64Rotl => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Rotl);
			}
			I64Rotr => {
				Validator::validate_binop(context, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64Rotr);
			}

			F32Abs => {
				Validator::validate_unop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Abs);
			}
			F32Neg => {
				Validator::validate_unop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Neg);
			}
			F32Ceil => {
				Validator::validate_unop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Ceil);
			}
			F32Floor => {
				Validator::validate_unop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Floor);
			}
			F32Trunc => {
				Validator::validate_unop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Trunc);
			}
			F32Nearest => {
				Validator::validate_unop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Nearest);
			}
			F32Sqrt => {
				Validator::validate_unop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Sqrt);
			}
			F32Add => {
				Validator::validate_binop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Add);
			}
			F32Sub => {
				Validator::validate_binop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Sub);
			}
			F32Mul => {
				Validator::validate_binop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Mul);
			}
			F32Div => {
				Validator::validate_binop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Div);
			}
			F32Min => {
				Validator::validate_binop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Min);
			}
			F32Max => {
				Validator::validate_binop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Max);
			}
			F32Copysign => {
				Validator::validate_binop(context, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32Copysign);
			}

			F64Abs => {
				Validator::validate_unop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Abs);
			}
			F64Neg => {
				Validator::validate_unop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Neg);
			}
			F64Ceil => {
				Validator::validate_unop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Ceil);
			}
			F64Floor => {
				Validator::validate_unop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Floor);
			}
			F64Trunc => {
				Validator::validate_unop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Trunc);
			}
			F64Nearest => {
				Validator::validate_unop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Nearest);
			}
			F64Sqrt => {
				Validator::validate_unop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Sqrt);
			}
			F64Add => {
				Validator::validate_binop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Add);
			}
			F64Sub => {
				Validator::validate_binop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Sub);
			}
			F64Mul => {
				Validator::validate_binop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Mul);
			}
			F64Div => {
				Validator::validate_binop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Div);
			}
			F64Min => {
				Validator::validate_binop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Min);
			}
			F64Max => {
				Validator::validate_binop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Max);
			}
			F64Copysign => {
				Validator::validate_binop(context, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64Copysign);
			}

			I32WrapI64 => {
				Validator::validate_cvtop(context, ValueType::I64, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32WrapI64);
			}
			I32TruncSF32 => {
				Validator::validate_cvtop(context, ValueType::F32, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32TruncSF32);
			}
			I32TruncUF32 => {
				Validator::validate_cvtop(context, ValueType::F32, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32TruncUF32);
			}
			I32TruncSF64 => {
				Validator::validate_cvtop(context, ValueType::F64, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32TruncSF64);
			}
			I32TruncUF64 => {
				Validator::validate_cvtop(context, ValueType::F64, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32TruncUF64);
			}
			I64ExtendSI32 => {
				Validator::validate_cvtop(context, ValueType::I32, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64ExtendSI32);
			}
			I64ExtendUI32 => {
				Validator::validate_cvtop(context, ValueType::I32, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64ExtendUI32);
			}
			I64TruncSF32 => {
				Validator::validate_cvtop(context, ValueType::F32, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64TruncSF32);
			}
			I64TruncUF32 => {
				Validator::validate_cvtop(context, ValueType::F32, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64TruncUF32);
			}
			I64TruncSF64 => {
				Validator::validate_cvtop(context, ValueType::F64, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64TruncSF64);
			}
			I64TruncUF64 => {
				Validator::validate_cvtop(context, ValueType::F64, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64TruncUF64);
			}
			F32ConvertSI32 => {
				Validator::validate_cvtop(context, ValueType::I32, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32ConvertSI32);
			}
			F32ConvertUI32 => {
				Validator::validate_cvtop(context, ValueType::I32, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32ConvertUI32);
			}
			F32ConvertSI64 => {
				Validator::validate_cvtop(context, ValueType::I64, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32ConvertSI64);
			}
			F32ConvertUI64 => {
				Validator::validate_cvtop(context, ValueType::I64, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32ConvertUI64);
			}
			F32DemoteF64 => {
				Validator::validate_cvtop(context, ValueType::F64, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32DemoteF64);
			}
			F64ConvertSI32 => {
				Validator::validate_cvtop(context, ValueType::I32, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64ConvertSI32);
			}
			F64ConvertUI32 => {
				Validator::validate_cvtop(context, ValueType::I32, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64ConvertUI32);
			}
			F64ConvertSI64 => {
				Validator::validate_cvtop(context, ValueType::I64, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64ConvertSI64);
			}
			F64ConvertUI64 => {
				Validator::validate_cvtop(context, ValueType::I64, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64ConvertUI64);
			}
			F64PromoteF32 => {
				Validator::validate_cvtop(context, ValueType::F32, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64PromoteF32);
			}

			I32ReinterpretF32 => {
				Validator::validate_cvtop(context, ValueType::F32, ValueType::I32)?;
				context.sink.emit(isa::Instruction::I32ReinterpretF32);
			}
			I64ReinterpretF64 => {
				Validator::validate_cvtop(context, ValueType::F64, ValueType::I64)?;
				context.sink.emit(isa::Instruction::I64ReinterpretF64);
			}
			F32ReinterpretI32 => {
				Validator::validate_cvtop(context, ValueType::I32, ValueType::F32)?;
				context.sink.emit(isa::Instruction::F32ReinterpretI32);
			}
			F64ReinterpretI64 => {
				Validator::validate_cvtop(context, ValueType::I64, ValueType::F64)?;
				context.sink.emit(isa::Instruction::F64ReinterpretI64);
			}
		}

		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_const(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<(), Error> {
		context.push_value(value_type.into())?;
		Ok(())
	}

	fn validate_unop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<(), Error> {
		context.pop_value(value_type.into())?;
		context.push_value(value_type.into())?;
		Ok(())
	}

	fn validate_binop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<(), Error> {
		context.pop_value(value_type.into())?;
		context.pop_value(value_type.into())?;
		context.push_value(value_type.into())?;
		Ok(())
	}

	fn validate_testop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<(), Error> {
		context.pop_value(value_type.into())?;
		context.push_value(ValueType::I32.into())?;
		Ok(())
	}

	fn validate_relop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<(), Error> {
		context.pop_value(value_type.into())?;
		context.pop_value(value_type.into())?;
		context.push_value(ValueType::I32.into())?;
		Ok(())
	}

	fn validate_cvtop(context: &mut FunctionValidationContext, value_type1: ValueType, value_type2: ValueType) -> Result<(), Error> {
		context.pop_value(value_type1.into())?;
		context.push_value(value_type2.into())?;
		Ok(())
	}

	fn validate_drop(context: &mut FunctionValidationContext) -> Result<(), Error> {
		context.pop_value(StackValueType::Any).map(|_| ())?;
		Ok(())
	}

	fn validate_select(context: &mut FunctionValidationContext) -> Result<(), Error> {
		context.pop_value(ValueType::I32.into())?;
		let select_type = context.pop_value(StackValueType::Any)?;
		context.pop_value(select_type)?;
		context.push_value(select_type)?;
		Ok(())
	}

	fn validate_get_local(context: &mut FunctionValidationContext, index: u32) -> Result<(), Error> {
		let local_type = context.require_local(index)?;
		context.push_value(local_type)?;
		Ok(())
	}

	fn validate_set_local(context: &mut FunctionValidationContext, index: u32) -> Result<(), Error> {
		let local_type = context.require_local(index)?;
		let value_type = context.pop_value(StackValueType::Any)?;
		if local_type != value_type {
			return Err(Error(format!("Trying to update local {} of type {:?} with value of type {:?}", index, local_type, value_type)));
		}
		Ok(())
	}

	fn validate_tee_local(context: &mut FunctionValidationContext, index: u32) -> Result<(), Error> {
		let local_type = context.require_local(index)?;
		context.tee_value(local_type)?;
		Ok(())
	}

	fn validate_get_global(context: &mut FunctionValidationContext, index: u32) -> Result<(), Error> {
		let global_type: StackValueType = {
			let global = context.module.require_global(index, None)?;
			global.content_type().into()
		};
		context.push_value(global_type)?;
		Ok(())
	}

	fn validate_set_global(context: &mut FunctionValidationContext, index: u32) -> Result<(), Error> {
		let global_type: StackValueType = {
			let global = context.module.require_global(index, Some(true))?;
			global.content_type().into()
		};
		let value_type = context.pop_value(StackValueType::Any)?;
		if global_type != value_type {
			return Err(Error(format!("Trying to update global {} of type {:?} with value of type {:?}", index, global_type, value_type)));
		}
		Ok(())
	}

	fn validate_load(context: &mut FunctionValidationContext, align: u32, max_align: u32, value_type: ValueType) -> Result<(), Error> {
		if 1u32.checked_shl(align).unwrap_or(u32::MAX) > max_align {
			return Err(Error(format!("Too large memory alignment 2^{} (expected at most {})", align, max_align)));
		}

		context.pop_value(ValueType::I32.into())?;
		context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.push_value(value_type.into())?;
		Ok(())
	}

	fn validate_store(context: &mut FunctionValidationContext, align: u32, max_align: u32, value_type: ValueType) -> Result<(), Error> {
		if 1u32.checked_shl(align).unwrap_or(u32::MAX) > max_align {
			return Err(Error(format!("Too large memory alignment 2^{} (expected at most {})", align, max_align)));
		}

		context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.pop_value(value_type.into())?;
		context.pop_value(ValueType::I32.into())?;
		Ok(())
	}

	fn validate_br(context: &mut FunctionValidationContext, idx: u32) -> Result<(), Error> {
		let (frame_type, frame_block_type) = {
			let frame = context.require_label(idx)?;
			(frame.frame_type, frame.block_type)
		};
		if !frame_type.is_loop() {
			if let BlockType::Value(value_type) = frame_block_type {
				context.tee_value(value_type.into())?;
			}
		}
		Ok(())
	}

	fn validate_br_if(context: &mut FunctionValidationContext, idx: u32) -> Result<(), Error> {
		context.pop_value(ValueType::I32.into())?;

		let (frame_type, frame_block_type) = {
			let frame = context.require_label(idx)?;
			(frame.frame_type, frame.block_type)
		};
		if !frame_type.is_loop() {
			if let BlockType::Value(value_type) = frame_block_type {
				context.tee_value(value_type.into())?;
			}
		}
		Ok(())
	}

	fn validate_br_table(context: &mut FunctionValidationContext, table: &[u32], default: u32) -> Result<(), Error> {
		let required_block_type: BlockType = {
			let default_block = context.require_label(default)?;
			let required_block_type = if !default_block.frame_type.is_loop() {
				default_block.block_type
			} else {
				BlockType::NoResult
			};

			for label in table {
				let label_block = context.require_label(*label)?;
				let label_block_type = if !label_block.frame_type.is_loop() {
					label_block.block_type
				} else {
					BlockType::NoResult
				};
				if required_block_type != label_block_type {
					return Err(
						Error(
							format!(
								"Labels in br_table points to block of different types: {:?} and {:?}",
								required_block_type,
								label_block.block_type
							)
						)
					);
				}
			}
			required_block_type
		};

		context.pop_value(ValueType::I32.into())?;
		if let BlockType::Value(value_type) = required_block_type {
			context.tee_value(value_type.into())?;
		}

		Ok(())
	}

	fn validate_call(context: &mut FunctionValidationContext, idx: u32) -> Result<(), Error> {
		let (argument_types, return_type) = context.module.require_function(idx)?;
		for argument_type in argument_types.iter().rev() {
			context.pop_value((*argument_type).into())?;
		}
		if let BlockType::Value(value_type) = return_type {
			context.push_value(value_type.into())?;
		}
		Ok(())
	}

	fn validate_call_indirect(context: &mut FunctionValidationContext, idx: u32) -> Result<(), Error> {
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

		context.pop_value(ValueType::I32.into())?;
		let (argument_types, return_type) = context.module.require_function_type(idx)?;
		for argument_type in argument_types.iter().rev() {
			context.pop_value((*argument_type).into())?;
		}
		if let BlockType::Value(value_type) = return_type {
			context.push_value(value_type.into())?;
		}
		Ok(())
	}

	fn validate_current_memory(context: &mut FunctionValidationContext) -> Result<(), Error> {
		context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.push_value(ValueType::I32.into())?;
		Ok(())
	}

	fn validate_grow_memory(context: &mut FunctionValidationContext) -> Result<(), Error> {
		context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.pop_value(ValueType::I32.into())?;
		context.push_value(ValueType::I32.into())?;
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

	// TODO: comment
	sink: Sink,
}

impl<'a> FunctionValidationContext<'a> {
	fn new(
		module: &'a ModuleContext,
		locals: Locals<'a>,
		value_stack_limit: usize,
		frame_stack_limit: usize,
		return_type: BlockType,
	) -> Self {
		FunctionValidationContext {
			module: module,
			position: 0,
			locals: locals,
			value_stack: StackWithLimit::with_limit(value_stack_limit),
			frame_stack: StackWithLimit::with_limit(frame_stack_limit),
			return_type: return_type,
			sink: Sink::new(),
		}
	}

	fn push_value(&mut self, value_type: StackValueType) -> Result<(), Error> {
		Ok(self.value_stack.push(value_type.into())?)
	}

	fn pop_value(&mut self, value_type: StackValueType) -> Result<StackValueType, Error> {
		let (is_stack_polymorphic, label_value_stack_len) = {
			let frame = self.top_label()?;
			(frame.polymorphic_stack, frame.value_stack_len)
		};
		let stack_is_empty = self.value_stack.len() == label_value_stack_len;
		let actual_value = if stack_is_empty && is_stack_polymorphic {
			StackValueType::Any
		} else {
			self.check_stack_access()?;
			self.value_stack.pop()?
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

	fn check_stack_access(&self) -> Result<(), Error> {
		let value_stack_min = self.frame_stack.top().expect("at least 1 topmost block").value_stack_len;
		if self.value_stack.len() > value_stack_min {
			Ok(())
		} else {
			Err(Error("Trying to access parent frame stack values.".into()))
		}
	}

	fn tee_value(&mut self, value_type: StackValueType) -> Result<(), Error> {
		let _ = self.pop_value(value_type)?;
		self.push_value(value_type)?;
		Ok(())
	}

	fn unreachable(&mut self) -> Result<(), Error> {
		let frame = self.frame_stack.top_mut()?;
		self.value_stack.resize(frame.value_stack_len, StackValueType::Any);
		frame.polymorphic_stack = true;
		Ok(())
	}

	fn top_label(&self) -> Result<&BlockFrame, Error> {
		Ok(self.frame_stack.top()?)
	}

	fn push_label(&mut self, frame_type: BlockFrameType, block_type: BlockType) -> Result<(), Error> {
		Ok(self.frame_stack.push(BlockFrame {
			frame_type: frame_type,
			block_type: block_type,
			begin_position: self.position,
			value_stack_len: self.value_stack.len(),
			polymorphic_stack: false,
		})?)
	}

	fn pop_label(&mut self) -> Result<(), Error> {
		// Don't pop frame yet. This is essential since we still might pop values from the value stack
		// and this in turn requires current frame to check whether or not we've reached
		// unreachable.
		let block_type = self.frame_stack.top()?.block_type;
		match block_type {
			BlockType::NoResult => (),
			BlockType::Value(required_value_type) => {
				self.pop_value(StackValueType::Specific(required_value_type))?;
			}
		}

		let frame = self.frame_stack.pop()?;
		if self.value_stack.len() != frame.value_stack_len {
			return Err(Error(format!(
				"Unexpected stack height {}, expected {}",
				self.value_stack.len(),
				frame.value_stack_len
			)));
		}

		if let BlockType::Value(value_type) = frame.block_type {
			self.push_value(value_type.into())?;
		}

		Ok(())
	}

	fn require_label(&self, idx: u32) -> Result<&BlockFrame, Error> {
		Ok(self.frame_stack.get(idx as usize)?)
	}

	fn return_type(&self) -> Result<BlockType, Error> {
		Ok(self.return_type)
	}

	fn require_local(&self, idx: u32) -> Result<StackValueType, Error> {
		Ok(self.locals.type_of_local(idx).map(StackValueType::from)?)
	}

	fn require_target(&self, depth: u32) -> Result<Target, Error> {
		let is_stack_polymorphic = self.top_label()?.polymorphic_stack;
		let frame = self.require_label(depth)?;

		let keep: u8 = match (frame.frame_type, frame.block_type) {
			(BlockFrameType::Loop { .. }, _) => 0,
			(_, BlockType::NoResult) => 0,
			(_, BlockType::Value(_)) => 1,
		};

		let value_stack_height = self.value_stack.len();
		let drop = if is_stack_polymorphic { 0 } else {
			// TODO: Remove this.
			// println!("value_stack_height = {}", value_stack_height);
			// println!("frame.value_stack_len = {}", frame.value_stack_len);
			// println!("keep = {}", keep);

			if value_stack_height < frame.value_stack_len {
				// TODO: Better error message.
				return Err(
					Error(
						format!(
							"Stack underflow detected: value stack height ({}) is lower than minimum stack len ({})",
							value_stack_height,
							frame.value_stack_len,
						)
					)
				);
			}
			if (value_stack_height as u32 - frame.value_stack_len as u32) < keep as u32 {
				// TODO: Better error message.
				return Err(
					Error(
						format!(
							"Stack underflow detected: asked to keep {} values, but there are only {}",
							keep,
							(value_stack_height as u32 - frame.value_stack_len as u32),
						)
					)
				);
			}

			(value_stack_height as u32 - frame.value_stack_len as u32) - keep as u32
		};

		Ok(Target {
			label: frame.frame_type.br_destination(),
			drop_keep: DropKeep {
				drop,
				keep,
			},
		})
	}

	fn drop_keep_return(&self) -> Result<DropKeep, Error> {
		assert!(
			!self.frame_stack.is_empty(),
			"drop_keep_return can't be called with the frame stack empty"
		);

		let deepest = (self.frame_stack.len() - 1) as u32;
		let mut drop_keep = self.require_target(deepest)?.drop_keep;

		// Drop all local variables and parameters upon exit.
		drop_keep.drop += self.locals.count()?;

		Ok(drop_keep)
	}

	fn relative_local_depth(&mut self, idx: u32) -> Result<u32, Error> {
		// TODO: Comment stack layout
		let value_stack_height = self.value_stack.len() as u32;
		let locals_and_params_count = self.locals.count()?;

		let depth = value_stack_height
			.checked_add(locals_and_params_count)
			.and_then(|x| x.checked_sub(idx))
			.ok_or_else(||
				Error(String::from("Locals range no in 32-bit range"))
			)?;
		Ok(depth)
	}

	fn into_code(self) -> isa::Instructions {
		isa::Instructions {
			code: self.sink.into_inner(),
		}
	}
}

#[derive(Clone)]
struct DropKeep {
	drop: u32,
	keep: u8,
}

#[derive(Clone)]
struct Target {
	label: LabelId,
	drop_keep: DropKeep,
}

enum Reloc {
	Br {
		pc: u32,
	},
	BrTable {
		pc: u32,
		idx: usize,
	},
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct LabelId(usize);
enum Label {
	Resolved(u32),
	NotResolved,
}

struct Sink {
	ins: Vec<isa::Instruction>,
	labels: Vec<Label>,
	unresolved: HashMap<LabelId, Vec<Reloc>>,
}

impl Sink {
	// TODO: Default size estimate?
	fn new() -> Sink {
		Sink {
			ins: Vec::new(),
			labels: Vec::new(),
			unresolved: HashMap::new(),
		}
	}

	fn cur_pc(&self) -> u32 {
		self.ins.len() as u32
	}

	fn pc_or_placeholder<F: FnOnce() -> Reloc>(&mut self, label: LabelId, reloc_creator: F) -> u32 {
		match self.labels[label.0] {
			Label::Resolved(dst_pc) => dst_pc,
			Label::NotResolved => {
				self.unresolved
					.entry(label)
					.or_insert(Vec::new())
					.push(reloc_creator());
				u32::max_value()
			}
		}
	}

	fn emit(&mut self, instruction: isa::Instruction) {
		self.ins.push(instruction);
	}

	fn emit_br(&mut self, target: Target) {
		let Target {
			label,
			drop_keep: DropKeep {
				drop,
				keep,
			},
		} = target;
		let pc = self.cur_pc();
		let dst_pc = self.pc_or_placeholder(label, || Reloc::Br { pc });
		self.ins.push(isa::Instruction::Br(isa::Target {
			dst_pc,
			drop,
			keep,
		}));
	}

	fn emit_br_eqz(&mut self, target: Target) {
		let Target {
			label,
			drop_keep: DropKeep {
				drop,
				keep,
			},
		} = target;
		let pc = self.cur_pc();
		let dst_pc = self.pc_or_placeholder(label, || Reloc::Br { pc });
		self.ins.push(isa::Instruction::BrIfEqz(isa::Target {
			dst_pc,
			drop,
			keep,
		}));
	}

	fn emit_br_nez(&mut self, target: Target) {
		let Target {
			label,
			drop_keep: DropKeep {
				drop,
				keep,
			},
		} = target;
		let pc = self.cur_pc();
		let dst_pc = self.pc_or_placeholder(label, || Reloc::Br { pc });
		self.ins.push(isa::Instruction::BrIfNez(isa::Target {
			dst_pc,
			drop,
			keep,
		}));
	}

	fn emit_br_table(&mut self, targets: &[Target], default: Target) {
		use std::iter;

		let pc = self.cur_pc();
		let mut isa_targets = Vec::new();
		for (idx, &Target { label, drop_keep: DropKeep {
				drop,
				keep,
			}}) in targets.iter().chain(iter::once(&default)).enumerate() {
			let dst_pc = self.pc_or_placeholder(label, || Reloc::BrTable { pc, idx });
			isa_targets.push(
				isa::Target {
					dst_pc,
					keep,
					drop,
				},
			);
		}
		self.ins.push(isa::Instruction::BrTable(
			isa_targets.into_boxed_slice(),
		));
	}

	fn new_label(&mut self) -> LabelId {
		let label_idx = self.labels.len();
		self.labels.push(
			Label::NotResolved,
		);
		LabelId(label_idx)
	}

	/// Resolve the label at the current position.
	///
	/// Panics if the label is already resolved.
	fn resolve_label(&mut self, label: LabelId) {
		if let Label::Resolved(_) = self.labels[label.0] {
			panic!("Trying to resolve already resolved label");
		}
		let dst_pc = self.cur_pc();

		// Patch all relocations that was previously recorded for this
		// particular label.
		let unresolved_rels = self.unresolved.remove(&label).unwrap_or(Vec::new());
		for reloc in unresolved_rels {
			match reloc {
				Reloc::Br { pc } => match self.ins[pc as usize] {
					isa::Instruction::Br(ref mut target)
					| isa::Instruction::BrIfEqz(ref mut target)
					| isa::Instruction::BrIfNez(ref mut target) => target.dst_pc = dst_pc,
					_ => panic!("branch relocation points to a non-branch instruction"),
				},
				Reloc::BrTable { pc, idx } => match self.ins[pc as usize] {
					isa::Instruction::BrTable(ref mut targets) => targets[idx].dst_pc = dst_pc,
					_ => panic!("brtable relocation points to not brtable instruction"),
				}
			}
		}

		// Mark this label as resolved.
		self.labels[label.0] = Label::Resolved(dst_pc);
	}

	fn into_inner(self) -> Vec<isa::Instruction> {
		assert!(self.unresolved.is_empty());
		self.ins
	}
}
