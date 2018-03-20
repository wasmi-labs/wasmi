use std::u32;
use std::iter::repeat;
use std::collections::HashMap;
use parity_wasm::elements::{Opcode, BlockType, ValueType, TableElementType, Func, FuncBody};
use common::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX};
use validation::context::ModuleContext;

use validation::Error;

use common::stack::StackWithLimit;
use common::{BlockFrame, BlockFrameType};

/// Maximum number of entries in value stack per function.
const DEFAULT_VALUE_STACK_LIMIT: usize = 16384;
/// Maximum number of entries in frame stack per function.
const DEFAULT_FRAME_STACK_LIMIT: usize = 16384;

/// Function validation context.
struct FunctionValidationContext<'a> {
	/// Wasm module
	module: &'a ModuleContext,
	/// Current instruction position.
	position: usize,
	/// Local variables.
	locals: &'a [ValueType],
	/// Value stack.
	value_stack: StackWithLimit<StackValueType>,
	/// Frame stack.
	frame_stack: StackWithLimit<BlockFrame>,
	/// Function return type. None if validating expression.
	return_type: Option<BlockType>,
	/// Labels positions.
	labels: HashMap<usize, usize>,
}

/// Value type on the stack.
#[derive(Debug, Clone, Copy)]
enum StackValueType {
	/// Any value type.
	Any,
	/// Concrete value type.
	Specific(ValueType),
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
	) -> Result<HashMap<usize, usize>, Error> {
		let (params, result_ty) = module.require_function_type(func.type_ref())?;

		// locals = (params + vars)
		let mut locals = params.to_vec();
		locals.extend(
			body.locals()
				.iter()
				.flat_map(|l| repeat(l.value_type())
				.take(l.count() as usize)
			),
		);

		let mut context = FunctionValidationContext::new(
			&module,
			&locals,
			DEFAULT_VALUE_STACK_LIMIT,
			DEFAULT_FRAME_STACK_LIMIT,
			result_ty,
		);

		context.push_label(BlockFrameType::Function, result_ty)?;
		Validator::validate_function_block(&mut context, body.code().elements())?;
		while !context.frame_stack.is_empty() {
			context.pop_label()?;
		}

		Ok(context.into_labels())
	}

	fn validate_function_block(context: &mut FunctionValidationContext, body: &[Opcode]) -> Result<(), Error> {
		let body_len = body.len();
		if body_len == 0 {
			return Err(Error("Non-empty function body expected".into()));
		}

		loop {
			let opcode = &body[context.position];
			match Validator::validate_instruction(context, opcode)? {
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
			Unreachable => Ok(InstructionOutcome::Unreachable),
			Nop => Ok(InstructionOutcome::ValidateNextInstruction),
			Block(block_type) => Validator::validate_block(context, block_type),
			Loop(block_type) => Validator::validate_loop(context, block_type),
			If(block_type) => Validator::validate_if(context, block_type),
			Else => Validator::validate_else(context),
			End => Validator::validate_end(context),
			Br(idx) => Validator::validate_br(context, idx),
			BrIf(idx) => Validator::validate_br_if(context, idx),
			BrTable(ref table, default) => Validator::validate_br_table(context, table, default),
			Return => Validator::validate_return(context),

			Call(index) => Validator::validate_call(context, index),
			CallIndirect(index, _reserved) => Validator::validate_call_indirect(context, index),

			Drop => Validator::validate_drop(context),
			Select => Validator::validate_select(context),

			GetLocal(index) => Validator::validate_get_local(context, index),
			SetLocal(index) => Validator::validate_set_local(context, index),
			TeeLocal(index) => Validator::validate_tee_local(context, index),
			GetGlobal(index) => Validator::validate_get_global(context, index),
			SetGlobal(index) => Validator::validate_set_global(context, index),

			I32Load(align, _) => Validator::validate_load(context, align, 4, ValueType::I32),
			I64Load(align, _) => Validator::validate_load(context, align, 8, ValueType::I64),
			F32Load(align, _) => Validator::validate_load(context, align, 4, ValueType::F32),
			F64Load(align, _) => Validator::validate_load(context, align, 8, ValueType::F64),
			I32Load8S(align, _) => Validator::validate_load(context, align, 1, ValueType::I32),
			I32Load8U(align, _) => Validator::validate_load(context, align, 1, ValueType::I32),
			I32Load16S(align, _) => Validator::validate_load(context, align, 2, ValueType::I32),
			I32Load16U(align, _) => Validator::validate_load(context, align, 2, ValueType::I32),
			I64Load8S(align, _) => Validator::validate_load(context, align, 1, ValueType::I64),
			I64Load8U(align, _) => Validator::validate_load(context, align, 1, ValueType::I64),
			I64Load16S(align, _) => Validator::validate_load(context, align, 2, ValueType::I64),
			I64Load16U(align, _) => Validator::validate_load(context, align, 2, ValueType::I64),
			I64Load32S(align, _) => Validator::validate_load(context, align, 4, ValueType::I64),
			I64Load32U(align, _) => Validator::validate_load(context, align, 4, ValueType::I64),

			I32Store(align, _) => Validator::validate_store(context, align, 4, ValueType::I32),
			I64Store(align, _) => Validator::validate_store(context, align, 8, ValueType::I64),
			F32Store(align, _) => Validator::validate_store(context, align, 4, ValueType::F32),
			F64Store(align, _) => Validator::validate_store(context, align, 8, ValueType::F64),
			I32Store8(align, _) => Validator::validate_store(context, align, 1, ValueType::I32),
			I32Store16(align, _) => Validator::validate_store(context, align, 2, ValueType::I32),
			I64Store8(align, _) => Validator::validate_store(context, align, 1, ValueType::I64),
			I64Store16(align, _) => Validator::validate_store(context, align, 2, ValueType::I64),
			I64Store32(align, _) => Validator::validate_store(context, align, 4, ValueType::I64),

			CurrentMemory(_) => Validator::validate_current_memory(context),
			GrowMemory(_) => Validator::validate_grow_memory(context),

			I32Const(_) => Validator::validate_const(context, ValueType::I32),
			I64Const(_) => Validator::validate_const(context, ValueType::I64),
			F32Const(_) => Validator::validate_const(context, ValueType::F32),
			F64Const(_) => Validator::validate_const(context, ValueType::F64),

			I32Eqz => Validator::validate_testop(context, ValueType::I32),
			I32Eq => Validator::validate_relop(context, ValueType::I32),
			I32Ne => Validator::validate_relop(context, ValueType::I32),
			I32LtS => Validator::validate_relop(context, ValueType::I32),
			I32LtU => Validator::validate_relop(context, ValueType::I32),
			I32GtS => Validator::validate_relop(context, ValueType::I32),
			I32GtU => Validator::validate_relop(context, ValueType::I32),
			I32LeS => Validator::validate_relop(context, ValueType::I32),
			I32LeU => Validator::validate_relop(context, ValueType::I32),
			I32GeS => Validator::validate_relop(context, ValueType::I32),
			I32GeU => Validator::validate_relop(context, ValueType::I32),

			I64Eqz => Validator::validate_testop(context, ValueType::I64),
			I64Eq => Validator::validate_relop(context, ValueType::I64),
			I64Ne => Validator::validate_relop(context, ValueType::I64),
			I64LtS => Validator::validate_relop(context, ValueType::I64),
			I64LtU => Validator::validate_relop(context, ValueType::I64),
			I64GtS => Validator::validate_relop(context, ValueType::I64),
			I64GtU => Validator::validate_relop(context, ValueType::I64),
			I64LeS => Validator::validate_relop(context, ValueType::I64),
			I64LeU => Validator::validate_relop(context, ValueType::I64),
			I64GeS => Validator::validate_relop(context, ValueType::I64),
			I64GeU => Validator::validate_relop(context, ValueType::I64),

			F32Eq => Validator::validate_relop(context, ValueType::F32),
			F32Ne => Validator::validate_relop(context, ValueType::F32),
			F32Lt => Validator::validate_relop(context, ValueType::F32),
			F32Gt => Validator::validate_relop(context, ValueType::F32),
			F32Le => Validator::validate_relop(context, ValueType::F32),
			F32Ge => Validator::validate_relop(context, ValueType::F32),

			F64Eq => Validator::validate_relop(context, ValueType::F64),
			F64Ne => Validator::validate_relop(context, ValueType::F64),
			F64Lt => Validator::validate_relop(context, ValueType::F64),
			F64Gt => Validator::validate_relop(context, ValueType::F64),
			F64Le => Validator::validate_relop(context, ValueType::F64),
			F64Ge => Validator::validate_relop(context, ValueType::F64),

			I32Clz => Validator::validate_unop(context, ValueType::I32),
			I32Ctz => Validator::validate_unop(context, ValueType::I32),
			I32Popcnt => Validator::validate_unop(context, ValueType::I32),
			I32Add => Validator::validate_binop(context, ValueType::I32),
			I32Sub => Validator::validate_binop(context, ValueType::I32),
			I32Mul => Validator::validate_binop(context, ValueType::I32),
			I32DivS => Validator::validate_binop(context, ValueType::I32),
			I32DivU => Validator::validate_binop(context, ValueType::I32),
			I32RemS => Validator::validate_binop(context, ValueType::I32),
			I32RemU => Validator::validate_binop(context, ValueType::I32),
			I32And => Validator::validate_binop(context, ValueType::I32),
			I32Or => Validator::validate_binop(context, ValueType::I32),
			I32Xor => Validator::validate_binop(context, ValueType::I32),
			I32Shl => Validator::validate_binop(context, ValueType::I32),
			I32ShrS => Validator::validate_binop(context, ValueType::I32),
			I32ShrU => Validator::validate_binop(context, ValueType::I32),
			I32Rotl => Validator::validate_binop(context, ValueType::I32),
			I32Rotr => Validator::validate_binop(context, ValueType::I32),

			I64Clz => Validator::validate_unop(context, ValueType::I64),
			I64Ctz => Validator::validate_unop(context, ValueType::I64),
			I64Popcnt => Validator::validate_unop(context, ValueType::I64),
			I64Add => Validator::validate_binop(context, ValueType::I64),
			I64Sub => Validator::validate_binop(context, ValueType::I64),
			I64Mul => Validator::validate_binop(context, ValueType::I64),
			I64DivS => Validator::validate_binop(context, ValueType::I64),
			I64DivU => Validator::validate_binop(context, ValueType::I64),
			I64RemS => Validator::validate_binop(context, ValueType::I64),
			I64RemU => Validator::validate_binop(context, ValueType::I64),
			I64And => Validator::validate_binop(context, ValueType::I64),
			I64Or => Validator::validate_binop(context, ValueType::I64),
			I64Xor => Validator::validate_binop(context, ValueType::I64),
			I64Shl => Validator::validate_binop(context, ValueType::I64),
			I64ShrS => Validator::validate_binop(context, ValueType::I64),
			I64ShrU => Validator::validate_binop(context, ValueType::I64),
			I64Rotl => Validator::validate_binop(context, ValueType::I64),
			I64Rotr => Validator::validate_binop(context, ValueType::I64),

			F32Abs => Validator::validate_unop(context, ValueType::F32),
			F32Neg => Validator::validate_unop(context, ValueType::F32),
			F32Ceil => Validator::validate_unop(context, ValueType::F32),
			F32Floor => Validator::validate_unop(context, ValueType::F32),
			F32Trunc => Validator::validate_unop(context, ValueType::F32),
			F32Nearest => Validator::validate_unop(context, ValueType::F32),
			F32Sqrt => Validator::validate_unop(context, ValueType::F32),
			F32Add => Validator::validate_binop(context, ValueType::F32),
			F32Sub => Validator::validate_binop(context, ValueType::F32),
			F32Mul => Validator::validate_binop(context, ValueType::F32),
			F32Div => Validator::validate_binop(context, ValueType::F32),
			F32Min => Validator::validate_binop(context, ValueType::F32),
			F32Max => Validator::validate_binop(context, ValueType::F32),
			F32Copysign => Validator::validate_binop(context, ValueType::F32),

			F64Abs => Validator::validate_unop(context, ValueType::F64),
			F64Neg => Validator::validate_unop(context, ValueType::F64),
			F64Ceil => Validator::validate_unop(context, ValueType::F64),
			F64Floor => Validator::validate_unop(context, ValueType::F64),
			F64Trunc => Validator::validate_unop(context, ValueType::F64),
			F64Nearest => Validator::validate_unop(context, ValueType::F64),
			F64Sqrt => Validator::validate_unop(context, ValueType::F64),
			F64Add => Validator::validate_binop(context, ValueType::F64),
			F64Sub => Validator::validate_binop(context, ValueType::F64),
			F64Mul => Validator::validate_binop(context, ValueType::F64),
			F64Div => Validator::validate_binop(context, ValueType::F64),
			F64Min => Validator::validate_binop(context, ValueType::F64),
			F64Max => Validator::validate_binop(context, ValueType::F64),
			F64Copysign => Validator::validate_binop(context, ValueType::F64),

			I32WrapI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::I32),
			I32TruncSF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::I32),
			I32TruncUF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::I32),
			I32TruncSF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::I32),
			I32TruncUF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::I32),
			I64ExtendSI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::I64),
			I64ExtendUI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::I64),
			I64TruncSF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::I64),
			I64TruncUF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::I64),
			I64TruncSF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::I64),
			I64TruncUF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::I64),
			F32ConvertSI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::F32),
			F32ConvertUI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::F32),
			F32ConvertSI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::F32),
			F32ConvertUI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::F32),
			F32DemoteF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::F32),
			F64ConvertSI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::F64),
			F64ConvertUI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::F64),
			F64ConvertSI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::F64),
			F64ConvertUI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::F64),
			F64PromoteF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::F64),

			I32ReinterpretF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::I32),
			I64ReinterpretF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::I64),
			F32ReinterpretI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::F32),
			F64ReinterpretI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::F64),
		}
	}

	fn validate_const(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		context.push_value(value_type.into())?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_unop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		context.pop_value(value_type.into())?;
		context.push_value(value_type.into())?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_binop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		context.pop_value(value_type.into())?;
		context.pop_value(value_type.into())?;
		context.push_value(value_type.into())?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_testop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		context.pop_value(value_type.into())?;
		context.push_value(ValueType::I32.into())?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_relop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		context.pop_value(value_type.into())?;
		context.pop_value(value_type.into())?;
		context.push_value(ValueType::I32.into())?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_cvtop(context: &mut FunctionValidationContext, value_type1: ValueType, value_type2: ValueType) -> Result<InstructionOutcome, Error> {
		context.pop_value(value_type1.into())?;
		context.push_value(value_type2.into())?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_drop(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		context.pop_value(StackValueType::Any).map(|_| ())?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_select(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		context.pop_value(ValueType::I32.into())?;
		let select_type = context.pop_value(StackValueType::Any)?;
		context.pop_value(select_type)?;
		context.push_value(select_type)?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_get_local(context: &mut FunctionValidationContext, index: u32) -> Result<InstructionOutcome, Error> {
		let local_type = context.require_local(index)?;
		context.push_value(local_type)?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_set_local(context: &mut FunctionValidationContext, index: u32) -> Result<InstructionOutcome, Error> {
		let local_type = context.require_local(index)?;
		let value_type = context.pop_value(StackValueType::Any)?;
		if local_type != value_type {
			return Err(Error(format!("Trying to update local {} of type {:?} with value of type {:?}", index, local_type, value_type)));
		}
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_tee_local(context: &mut FunctionValidationContext, index: u32) -> Result<InstructionOutcome, Error> {
		let local_type = context.require_local(index)?;
		context.tee_value(local_type)?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_get_global(context: &mut FunctionValidationContext, index: u32) -> Result<InstructionOutcome, Error> {
		let global_type: StackValueType = {
			let global = context.module.require_global(index, None)?;
			global.content_type().into()
		};
		context.push_value(global_type)?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_set_global(context: &mut FunctionValidationContext, index: u32) -> Result<InstructionOutcome, Error> {
		let global_type: StackValueType = {
			let global = context.module.require_global(index, Some(true))?;
			global.content_type().into()
		};
		let value_type = context.pop_value(StackValueType::Any)?;
		if global_type != value_type {
			return Err(Error(format!("Trying to update global {} of type {:?} with value of type {:?}", index, global_type, value_type)));
		}
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_load(context: &mut FunctionValidationContext, align: u32, max_align: u32, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		if 1u32.checked_shl(align).unwrap_or(u32::MAX) > max_align {
			return Err(Error(format!("Too large memory alignment 2^{} (expected at most {})", align, max_align)));
		}

		context.pop_value(ValueType::I32.into())?;
		context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.push_value(value_type.into())?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_store(context: &mut FunctionValidationContext, align: u32, max_align: u32, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		if 1u32.checked_shl(align).unwrap_or(u32::MAX) > max_align {
			return Err(Error(format!("Too large memory alignment 2^{} (expected at most {})", align, max_align)));
		}

		context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.pop_value(value_type.into())?;
		context.pop_value(ValueType::I32.into())?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_block(context: &mut FunctionValidationContext, block_type: BlockType) -> Result<InstructionOutcome, Error> {
		context.push_label(BlockFrameType::Block, block_type).map(|_| InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_loop(context: &mut FunctionValidationContext, block_type: BlockType) -> Result<InstructionOutcome, Error> {
		context.push_label(BlockFrameType::Loop, block_type).map(|_| InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_if(context: &mut FunctionValidationContext, block_type: BlockType) -> Result<InstructionOutcome, Error> {
		context.pop_value(ValueType::I32.into())?;
		context.push_label(BlockFrameType::IfTrue, block_type).map(|_| InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_else(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		let block_type = {
			let top_frame = context.top_label()?;
			if top_frame.frame_type != BlockFrameType::IfTrue {
				return Err(Error("Misplaced else instruction".into()));
			}
			top_frame.block_type
		};
		context.pop_label()?;

		if let BlockType::Value(value_type) = block_type {
			context.pop_value(value_type.into())?;
		}
		context.push_label(BlockFrameType::IfFalse, block_type).map(|_| InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_end(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		{
			let top_frame = context.top_label()?;
			if top_frame.frame_type == BlockFrameType::IfTrue {
				if top_frame.block_type != BlockType::NoResult {
					return Err(Error(format!("If block without else required to have NoResult block type. But it have {:?} type", top_frame.block_type)));
				}
			}
		}

		context.pop_label().map(|_| InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_br(context: &mut FunctionValidationContext, idx: u32) -> Result<InstructionOutcome, Error> {
		let (frame_type, frame_block_type) = {
			let frame = context.require_label(idx)?;
			(frame.frame_type, frame.block_type)
		};
		if frame_type != BlockFrameType::Loop {
			if let BlockType::Value(value_type) = frame_block_type {
				context.tee_value(value_type.into())?;
			}
		}
		Ok(InstructionOutcome::Unreachable)
	}

	fn validate_br_if(context: &mut FunctionValidationContext, idx: u32) -> Result<InstructionOutcome, Error> {
		context.pop_value(ValueType::I32.into())?;

		let (frame_type, frame_block_type) = {
			let frame = context.require_label(idx)?;
			(frame.frame_type, frame.block_type)
		};
		if frame_type != BlockFrameType::Loop {
			if let BlockType::Value(value_type) = frame_block_type {
				context.tee_value(value_type.into())?;
			}
		}
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_br_table(context: &mut FunctionValidationContext, table: &[u32], default: u32) -> Result<InstructionOutcome, Error> {
		let required_block_type: BlockType = {
			let default_block = context.require_label(default)?;
			let required_block_type = if default_block.frame_type != BlockFrameType::Loop {
				default_block.block_type
			} else {
				BlockType::NoResult
			};

			for label in table {
				let label_block = context.require_label(*label)?;
				let label_block_type = if label_block.frame_type != BlockFrameType::Loop {
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

		Ok(InstructionOutcome::Unreachable)
	}

	fn validate_return(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		if let BlockType::Value(value_type) = context.return_type()? {
			context.tee_value(value_type.into())?;
		}
		Ok(InstructionOutcome::Unreachable)
	}

	fn validate_call(context: &mut FunctionValidationContext, idx: u32) -> Result<InstructionOutcome, Error> {
		let (argument_types, return_type) = context.module.require_function(idx)?;
		for argument_type in argument_types.iter().rev() {
			context.pop_value((*argument_type).into())?;
		}
		if let BlockType::Value(value_type) = return_type {
			context.push_value(value_type.into())?;
		}
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_call_indirect(context: &mut FunctionValidationContext, idx: u32) -> Result<InstructionOutcome, Error> {
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
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_current_memory(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.push_value(ValueType::I32.into())?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn validate_grow_memory(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		context.module.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.pop_value(ValueType::I32.into())?;
		context.push_value(ValueType::I32.into())?;
		Ok(InstructionOutcome::ValidateNextInstruction)
	}
}

impl<'a> FunctionValidationContext<'a> {
	fn new(
		module: &'a ModuleContext,
		locals: &'a [ValueType],
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
			return_type: Some(return_type),
			labels: HashMap::new(),
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
			branch_position: self.position,
			end_position: self.position,
			value_stack_len: self.value_stack.len(),
			polymorphic_stack: false,
		})?)
	}

	fn pop_label(&mut self) -> Result<InstructionOutcome, Error> {
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

		if !self.frame_stack.is_empty() {
			self.labels.insert(frame.begin_position, self.position);
		}
		if let BlockType::Value(value_type) = frame.block_type {
			self.push_value(value_type.into())?;
		}

		Ok(InstructionOutcome::ValidateNextInstruction)
	}

	fn require_label(&self, idx: u32) -> Result<&BlockFrame, Error> {
		Ok(self.frame_stack.get(idx as usize)?)
	}

	fn return_type(&self) -> Result<BlockType, Error> {
		self.return_type.ok_or(Error("Trying to return from expression".into()))
	}

	fn require_local(&self, idx: u32) -> Result<StackValueType, Error> {
		self.locals.get(idx as usize)
			.cloned()
			.map(Into::into)
			.ok_or(Error(format!("Trying to access local with index {} when there are only {} locals", idx, self.locals.len())))
	}

	fn into_labels(self) -> HashMap<usize, usize> {
		self.labels
	}
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
