use std::mem;
use std::ops;
use std::{u32, usize};
use std::fmt::{self, Display};
use std::iter::repeat;
use std::collections::{HashMap, VecDeque};
use parity_wasm::elements::{Opcode, BlockType, Local};
use {Error, Trap, Signature};
use module::ModuleRef;
use func::{FuncRef, FuncInstance, FuncInstanceInternal};
use value::{
	RuntimeValue, TryInto, WrapInto, TryTruncateInto, ExtendInto,
	ArithmeticOps, Integer, Float, LittleEndianConvert, TransmuteInto,
};
use host::Externals;
use common::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX, BlockFrame, BlockFrameType};
use common::stack::StackWithLimit;

/// Function interpreter.
pub struct Interpreter<'a, E: Externals + 'a> {
	externals: &'a mut E,
}

/// Function execution context.
pub struct FunctionContext {
	/// Is context initialized.
	pub is_initialized: bool,
	/// Internal function reference.
	pub function: FuncRef,
	pub module: ModuleRef,
	/// Function return type.
	pub return_type: BlockType,
	/// Local variables.
	pub locals: Vec<RuntimeValue>,
	/// Values stack.
	pub value_stack: StackWithLimit<RuntimeValue>,
	/// Blocks frames stack.
	pub frame_stack: StackWithLimit<BlockFrame>,
	/// Current instruction position.
	pub position: usize,
}

/// Interpreter action to execute after executing instruction.
pub enum InstructionOutcome {
	/// Continue with next instruction.
	RunNextInstruction,
	/// Branch to given frame.
	Branch(usize),
	/// Execute function call.
	ExecuteCall(FuncRef),
	/// End current frame.
	End,
	/// Return from current function block.
	Return,
}

/// Function run result.
enum RunResult {
	/// Function has returned (optional) value.
	Return(Option<RuntimeValue>),
	/// Function is calling other function.
	NestedCall(FuncRef),
}

impl<'a, E: Externals> Interpreter<'a, E> {
	pub fn new(externals: &'a mut E) -> Interpreter<'a, E> {
		Interpreter {
			externals,
		}
	}

	pub fn run_function(&mut self, function_context: FunctionContext) -> Result<Option<RuntimeValue>, Error> {
		let mut function_stack = VecDeque::new();
		function_stack.push_back(function_context);

		loop {
			let mut function_context = function_stack.pop_back().expect("on loop entry - not empty; on loop continue - checking for emptiness; qed");
			let function_ref = function_context.function.clone();
			let function_body = function_ref
				.body()
				.expect(
					"Host functions checked in function_return below; Internal functions always have a body; qed"
				);
			if !function_context.is_initialized() {
				let return_type = function_context.return_type;
				function_context.initialize(&function_body.locals);
				function_context.push_frame(&function_body.labels, BlockFrameType::Function, return_type)?;
			}

			let function_return = self.do_run_function(&mut function_context, function_body.opcodes.elements(), &function_body.labels)?;

			match function_return {
				RunResult::Return(return_value) => {
					match function_stack.back_mut() {
						Some(caller_context) => if let Some(return_value) = return_value {
							caller_context.value_stack_mut().push(return_value)?;
						},
						None => return Ok(return_value),
					}
				},
				RunResult::NestedCall(nested_func) => {
					match *nested_func.as_internal() {
						FuncInstanceInternal::Internal { .. } => {
							let nested_context = function_context.nested(nested_func.clone())?;
							function_stack.push_back(function_context);
							function_stack.push_back(nested_context);
						},
						FuncInstanceInternal::Host { ref signature, .. } => {
							let args = prepare_function_args(signature, &mut function_context.value_stack)?;
							let return_val = FuncInstance::invoke(&nested_func, &args, self.externals)?;
							if let Some(return_val) = return_val {
								function_context.value_stack_mut().push(return_val)?;
							}
							function_stack.push_back(function_context);
						}
					}
				},
			}
		}
	}

	fn do_run_function(&mut self, function_context: &mut FunctionContext, function_body: &[Opcode], function_labels: &HashMap<usize, usize>) -> Result<RunResult, Error> {
		loop {
			let instruction = &function_body[function_context.position];

			match self.run_instruction(function_context, function_labels, instruction)? {
				InstructionOutcome::RunNextInstruction => function_context.position += 1,
				InstructionOutcome::Branch(mut index) => {
					// discard index - 1 blocks
					while index >= 1 {
						function_context.discard_frame()?;
						index -= 1;
					}

					function_context.pop_frame(true)?;
					if function_context.frame_stack().is_empty() {
						break;
					}
				},
				InstructionOutcome::ExecuteCall(func_ref) => {
					function_context.position += 1;
					return Ok(RunResult::NestedCall(func_ref));
				},
				InstructionOutcome::End => {
					if function_context.frame_stack().is_empty() {
						break;
					}
				},
				InstructionOutcome::Return => break,
			}
		}

		Ok(RunResult::Return(match function_context.return_type {
			BlockType::Value(_) => Some(function_context.value_stack_mut().pop()?),
			BlockType::NoResult => None,
		}))
	}

	fn run_instruction(&mut self, context: &mut FunctionContext, labels: &HashMap<usize, usize>, opcode: &Opcode) -> Result<InstructionOutcome, Error> {
		match opcode {
			&Opcode::Unreachable => self.run_unreachable(context),
			&Opcode::Nop => self.run_nop(context),
			&Opcode::Block(block_type) => self.run_block(context, labels, block_type),
			&Opcode::Loop(block_type) => self.run_loop(context, labels, block_type),
			&Opcode::If(block_type) => self.run_if(context, labels, block_type),
			&Opcode::Else => self.run_else(context, labels),
			&Opcode::End => self.run_end(context),
			&Opcode::Br(idx) => self.run_br(context, idx),
			&Opcode::BrIf(idx) => self.run_br_if(context, idx),
			&Opcode::BrTable(ref table, default) => self.run_br_table(context, table, default),
			&Opcode::Return => self.run_return(context),

			&Opcode::Call(index) => self.run_call(context, index),
			&Opcode::CallIndirect(index, _reserved) => self.run_call_indirect(context, index),

			&Opcode::Drop => self.run_drop(context),
			&Opcode::Select => self.run_select(context),

			&Opcode::GetLocal(index) => self.run_get_local(context, index),
			&Opcode::SetLocal(index) => self.run_set_local(context, index),
			&Opcode::TeeLocal(index) => self.run_tee_local(context, index),
			&Opcode::GetGlobal(index) => self.run_get_global(context, index),
			&Opcode::SetGlobal(index) => self.run_set_global(context, index),

			&Opcode::I32Load(align, offset) => self.run_load::<i32>(context, align, offset),
			&Opcode::I64Load(align, offset) => self.run_load::<i64>(context, align, offset),
			&Opcode::F32Load(align, offset) => self.run_load::<f32>(context, align, offset),
			&Opcode::F64Load(align, offset) => self.run_load::<f64>(context, align, offset),
			&Opcode::I32Load8S(align, offset) => self.run_load_extend::<i8, i32>(context, align, offset),
			&Opcode::I32Load8U(align, offset) => self.run_load_extend::<u8, i32>(context, align, offset),
			&Opcode::I32Load16S(align, offset) => self.run_load_extend::<i16, i32>(context, align, offset),
			&Opcode::I32Load16U(align, offset) => self.run_load_extend::<u16, i32>(context, align, offset),
			&Opcode::I64Load8S(align, offset) => self.run_load_extend::<i8, i64>(context, align, offset),
			&Opcode::I64Load8U(align, offset) => self.run_load_extend::<u8, i64>(context, align, offset),
			&Opcode::I64Load16S(align, offset) => self.run_load_extend::<i16, i64>(context, align, offset),
			&Opcode::I64Load16U(align, offset) => self.run_load_extend::<u16, i64>(context, align, offset),
			&Opcode::I64Load32S(align, offset) => self.run_load_extend::<i32, i64>(context, align, offset),
			&Opcode::I64Load32U(align, offset) => self.run_load_extend::<u32, i64>(context, align, offset),

			&Opcode::I32Store(align, offset) => self.run_store::<i32>(context, align, offset),
			&Opcode::I64Store(align, offset) => self.run_store::<i64>(context, align, offset),
			&Opcode::F32Store(align, offset) => self.run_store::<f32>(context, align, offset),
			&Opcode::F64Store(align, offset) => self.run_store::<f64>(context, align, offset),
			&Opcode::I32Store8(align, offset) => self.run_store_wrap::<i32, i8>(context, align, offset),
			&Opcode::I32Store16(align, offset) => self.run_store_wrap::<i32, i16>(context, align, offset),
			&Opcode::I64Store8(align, offset) => self.run_store_wrap::<i64, i8>(context, align, offset),
			&Opcode::I64Store16(align, offset) => self.run_store_wrap::<i64, i16>(context, align, offset),
			&Opcode::I64Store32(align, offset) => self.run_store_wrap::<i64, i32>(context, align, offset),

			&Opcode::CurrentMemory(_) => self.run_current_memory(context),
			&Opcode::GrowMemory(_) => self.run_grow_memory(context),

			&Opcode::I32Const(val) => self.run_const(context, val.into()),
			&Opcode::I64Const(val) => self.run_const(context, val.into()),
			&Opcode::F32Const(val) => self.run_const(context, RuntimeValue::decode_f32(val)),
			&Opcode::F64Const(val) => self.run_const(context, RuntimeValue::decode_f64(val)),

			&Opcode::I32Eqz => self.run_eqz::<i32>(context),
			&Opcode::I32Eq => self.run_eq::<i32>(context),
			&Opcode::I32Ne => self.run_ne::<i32>(context),
			&Opcode::I32LtS => self.run_lt::<i32>(context),
			&Opcode::I32LtU => self.run_lt::<u32>(context),
			&Opcode::I32GtS => self.run_gt::<i32>(context),
			&Opcode::I32GtU => self.run_gt::<u32>(context),
			&Opcode::I32LeS => self.run_lte::<i32>(context),
			&Opcode::I32LeU => self.run_lte::<u32>(context),
			&Opcode::I32GeS => self.run_gte::<i32>(context),
			&Opcode::I32GeU => self.run_gte::<u32>(context),

			&Opcode::I64Eqz => self.run_eqz::<i64>(context),
			&Opcode::I64Eq => self.run_eq::<i64>(context),
			&Opcode::I64Ne => self.run_ne::<i64>(context),
			&Opcode::I64LtS => self.run_lt::<i64>(context),
			&Opcode::I64LtU => self.run_lt::<u64>(context),
			&Opcode::I64GtS => self.run_gt::<i64>(context),
			&Opcode::I64GtU => self.run_gt::<u64>(context),
			&Opcode::I64LeS => self.run_lte::<i64>(context),
			&Opcode::I64LeU => self.run_lte::<u64>(context),
			&Opcode::I64GeS => self.run_gte::<i64>(context),
			&Opcode::I64GeU => self.run_gte::<u64>(context),

			&Opcode::F32Eq => self.run_eq::<f32>(context),
			&Opcode::F32Ne => self.run_ne::<f32>(context),
			&Opcode::F32Lt => self.run_lt::<f32>(context),
			&Opcode::F32Gt => self.run_gt::<f32>(context),
			&Opcode::F32Le => self.run_lte::<f32>(context),
			&Opcode::F32Ge => self.run_gte::<f32>(context),

			&Opcode::F64Eq => self.run_eq::<f64>(context),
			&Opcode::F64Ne => self.run_ne::<f64>(context),
			&Opcode::F64Lt => self.run_lt::<f64>(context),
			&Opcode::F64Gt => self.run_gt::<f64>(context),
			&Opcode::F64Le => self.run_lte::<f64>(context),
			&Opcode::F64Ge => self.run_gte::<f64>(context),

			&Opcode::I32Clz => self.run_clz::<i32>(context),
			&Opcode::I32Ctz => self.run_ctz::<i32>(context),
			&Opcode::I32Popcnt => self.run_popcnt::<i32>(context),
			&Opcode::I32Add => self.run_add::<i32>(context),
			&Opcode::I32Sub => self.run_sub::<i32>(context),
			&Opcode::I32Mul => self.run_mul::<i32>(context),
			&Opcode::I32DivS => self.run_div::<i32, i32>(context),
			&Opcode::I32DivU => self.run_div::<i32, u32>(context),
			&Opcode::I32RemS => self.run_rem::<i32, i32>(context),
			&Opcode::I32RemU => self.run_rem::<i32, u32>(context),
			&Opcode::I32And => self.run_and::<i32>(context),
			&Opcode::I32Or => self.run_or::<i32>(context),
			&Opcode::I32Xor => self.run_xor::<i32>(context),
			&Opcode::I32Shl => self.run_shl::<i32>(context, 0x1F),
			&Opcode::I32ShrS => self.run_shr::<i32, i32>(context, 0x1F),
			&Opcode::I32ShrU => self.run_shr::<i32, u32>(context, 0x1F),
			&Opcode::I32Rotl => self.run_rotl::<i32>(context),
			&Opcode::I32Rotr => self.run_rotr::<i32>(context),

			&Opcode::I64Clz => self.run_clz::<i64>(context),
			&Opcode::I64Ctz => self.run_ctz::<i64>(context),
			&Opcode::I64Popcnt => self.run_popcnt::<i64>(context),
			&Opcode::I64Add => self.run_add::<i64>(context),
			&Opcode::I64Sub => self.run_sub::<i64>(context),
			&Opcode::I64Mul => self.run_mul::<i64>(context),
			&Opcode::I64DivS => self.run_div::<i64, i64>(context),
			&Opcode::I64DivU => self.run_div::<i64, u64>(context),
			&Opcode::I64RemS => self.run_rem::<i64, i64>(context),
			&Opcode::I64RemU => self.run_rem::<i64, u64>(context),
			&Opcode::I64And => self.run_and::<i64>(context),
			&Opcode::I64Or => self.run_or::<i64>(context),
			&Opcode::I64Xor => self.run_xor::<i64>(context),
			&Opcode::I64Shl => self.run_shl::<i64>(context, 0x3F),
			&Opcode::I64ShrS => self.run_shr::<i64, i64>(context, 0x3F),
			&Opcode::I64ShrU => self.run_shr::<i64, u64>(context, 0x3F),
			&Opcode::I64Rotl => self.run_rotl::<i64>(context),
			&Opcode::I64Rotr => self.run_rotr::<i64>(context),

			&Opcode::F32Abs => self.run_abs::<f32>(context),
			&Opcode::F32Neg => self.run_neg::<f32>(context),
			&Opcode::F32Ceil => self.run_ceil::<f32>(context),
			&Opcode::F32Floor => self.run_floor::<f32>(context),
			&Opcode::F32Trunc => self.run_trunc::<f32>(context),
			&Opcode::F32Nearest => self.run_nearest::<f32>(context),
			&Opcode::F32Sqrt => self.run_sqrt::<f32>(context),
			&Opcode::F32Add => self.run_add::<f32>(context),
			&Opcode::F32Sub => self.run_sub::<f32>(context),
			&Opcode::F32Mul => self.run_mul::<f32>(context),
			&Opcode::F32Div => self.run_div::<f32, f32>(context),
			&Opcode::F32Min => self.run_min::<f32>(context),
			&Opcode::F32Max => self.run_max::<f32>(context),
			&Opcode::F32Copysign => self.run_copysign::<f32>(context),

			&Opcode::F64Abs => self.run_abs::<f64>(context),
			&Opcode::F64Neg => self.run_neg::<f64>(context),
			&Opcode::F64Ceil => self.run_ceil::<f64>(context),
			&Opcode::F64Floor => self.run_floor::<f64>(context),
			&Opcode::F64Trunc => self.run_trunc::<f64>(context),
			&Opcode::F64Nearest => self.run_nearest::<f64>(context),
			&Opcode::F64Sqrt => self.run_sqrt::<f64>(context),
			&Opcode::F64Add => self.run_add::<f64>(context),
			&Opcode::F64Sub => self.run_sub::<f64>(context),
			&Opcode::F64Mul => self.run_mul::<f64>(context),
			&Opcode::F64Div => self.run_div::<f64, f64>(context),
			&Opcode::F64Min => self.run_min::<f64>(context),
			&Opcode::F64Max => self.run_max::<f64>(context),
			&Opcode::F64Copysign => self.run_copysign::<f64>(context),

			&Opcode::I32WrapI64 => self.run_wrap::<i64, i32>(context),
			&Opcode::I32TruncSF32 => self.run_trunc_to_int::<f32, i32, i32>(context),
			&Opcode::I32TruncUF32 => self.run_trunc_to_int::<f32, u32, i32>(context),
			&Opcode::I32TruncSF64 => self.run_trunc_to_int::<f64, i32, i32>(context),
			&Opcode::I32TruncUF64 => self.run_trunc_to_int::<f64, u32, i32>(context),
			&Opcode::I64ExtendSI32 => self.run_extend::<i32, i64, i64>(context),
			&Opcode::I64ExtendUI32 => self.run_extend::<u32, u64, i64>(context),
			&Opcode::I64TruncSF32 => self.run_trunc_to_int::<f32, i64, i64>(context),
			&Opcode::I64TruncUF32 => self.run_trunc_to_int::<f32, u64, i64>(context),
			&Opcode::I64TruncSF64 => self.run_trunc_to_int::<f64, i64, i64>(context),
			&Opcode::I64TruncUF64 => self.run_trunc_to_int::<f64, u64, i64>(context),
			&Opcode::F32ConvertSI32 => self.run_extend::<i32, f32, f32>(context),
			&Opcode::F32ConvertUI32 => self.run_extend::<u32, f32, f32>(context),
			&Opcode::F32ConvertSI64 => self.run_wrap::<i64, f32>(context),
			&Opcode::F32ConvertUI64 => self.run_wrap::<u64, f32>(context),
			&Opcode::F32DemoteF64 => self.run_wrap::<f64, f32>(context),
			&Opcode::F64ConvertSI32 => self.run_extend::<i32, f64, f64>(context),
			&Opcode::F64ConvertUI32 => self.run_extend::<u32, f64, f64>(context),
			&Opcode::F64ConvertSI64 => self.run_extend::<i64, f64, f64>(context),
			&Opcode::F64ConvertUI64 => self.run_extend::<u64, f64, f64>(context),
			&Opcode::F64PromoteF32 => self.run_extend::<f32, f64, f64>(context),

			&Opcode::I32ReinterpretF32 => self.run_reinterpret::<f32, i32>(context),
			&Opcode::I64ReinterpretF64 => self.run_reinterpret::<f64, i64>(context),
			&Opcode::F32ReinterpretI32 => self.run_reinterpret::<i32, f32>(context),
			&Opcode::F64ReinterpretI64 => self.run_reinterpret::<i64, f64>(context),
		}
	}

	fn run_unreachable(&mut self, _context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		Err(Error::Trap(Trap::Unreachable))
	}

	fn run_nop(&mut self, _context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_block(&mut self, context: &mut FunctionContext, labels: &HashMap<usize, usize>, block_type: BlockType) -> Result<InstructionOutcome, Error> {
		context.push_frame(labels, BlockFrameType::Block, block_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_loop(&mut self, context: &mut FunctionContext, labels: &HashMap<usize, usize>, block_type: BlockType) -> Result<InstructionOutcome, Error> {
		context.push_frame(labels, BlockFrameType::Loop, block_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_if(&mut self, context: &mut FunctionContext, labels: &HashMap<usize, usize>, block_type: BlockType) -> Result<InstructionOutcome, Error> {
		let branch = context.value_stack_mut().pop_as()?;
		let block_frame_type = if branch { BlockFrameType::IfTrue } else {
			let else_pos = labels[&context.position];
			if !labels.contains_key(&else_pos) {
				context.position = else_pos;
				return Ok(InstructionOutcome::RunNextInstruction);
			}

			context.position = else_pos;
			BlockFrameType::IfFalse
		};
		context.push_frame(labels, block_frame_type, block_type).map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_else(&mut self, context: &mut FunctionContext, labels: &HashMap<usize, usize>) -> Result<InstructionOutcome, Error> {
		let end_pos = labels[&context.position];
		context.pop_frame(false)?;
		context.position = end_pos;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_end(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		context.pop_frame(false)?;
		Ok(InstructionOutcome::End)
	}

	fn run_br(&mut self, _context: &mut FunctionContext, label_idx: u32) -> Result<InstructionOutcome, Error> {
		Ok(InstructionOutcome::Branch(label_idx as usize))
	}

	fn run_br_if(&mut self, context: &mut FunctionContext, label_idx: u32) -> Result<InstructionOutcome, Error> {
		if context.value_stack_mut().pop_as()? {
			Ok(InstructionOutcome::Branch(label_idx as usize))
		} else {
			Ok(InstructionOutcome::RunNextInstruction)
		}
	}

	fn run_br_table(&mut self, context: &mut FunctionContext, table: &[u32], default: u32) -> Result<InstructionOutcome, Error> {
		let index: u32 = context.value_stack_mut().pop_as()?;
		Ok(InstructionOutcome::Branch(table.get(index as usize).cloned().unwrap_or(default) as usize))
	}

	fn run_return(&mut self, _context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		Ok(InstructionOutcome::Return)
	}

	fn run_call(
		&mut self,
		context: &mut FunctionContext,
		func_idx: u32,
	) -> Result<InstructionOutcome, Error> {
		let func = context
			.module()
			.func_by_index(func_idx)
			.expect("Due to validation func should exists");
		Ok(InstructionOutcome::ExecuteCall(func))
	}

	fn run_call_indirect(
		&mut self,
		context: &mut FunctionContext,
		signature_idx: u32,
	) -> Result<InstructionOutcome, Error> {
		let table_func_idx: u32 = context.value_stack_mut().pop_as()?;
		let table = context
			.module()
			.table_by_index(DEFAULT_TABLE_INDEX)
			.expect("Due to validation table should exists");
		let func_ref = table.get(table_func_idx)?;

		{
			let actual_function_type = func_ref.signature();
			let required_function_type = context
				.module()
				.signature_by_index(signature_idx)
				.expect("Due to validation type should exists");

			if &*required_function_type != actual_function_type {
				return Err(Error::Function(format!(
					"expected function with signature ({:?}) -> {:?} when got with ({:?}) -> {:?}",
					required_function_type.params(),
					required_function_type.return_type(),
					actual_function_type.params(),
					actual_function_type.return_type()
				)));
			}
		}

		Ok(InstructionOutcome::ExecuteCall(func_ref))
	}

	fn run_drop(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		context
			.value_stack_mut()
			.pop()
			.map_err(Into::into)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_select(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		context
			.value_stack_mut()
			.pop_triple()
			.and_then(|(left, mid, right)| {
				let right: Result<_, Error> = right.try_into();
				match (left, mid, right) {
					(left, mid, Ok(condition)) => Ok((left, mid, condition)),
					_ => Err(Error::Stack("expected to get int value from stack".into()))
				}
			})
			.map(|(left, mid, condition)| if condition { left } else { mid })
			.map(|val| context.value_stack_mut().push(val))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_get_local(&mut self, context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome, Error> {
		let value = context.get_local(index as usize);
		context.value_stack_mut().push(value)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_set_local(&mut self, context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome, Error> {
		let arg = context.value_stack_mut().pop()?;
		context.set_local(index as usize, arg)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_tee_local(&mut self, context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome, Error> {
		let arg = context.value_stack().top()?.clone();
		context.set_local(index as usize, arg)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_get_global(
		&mut self,
		context: &mut FunctionContext,
		index: u32,
	) -> Result<InstructionOutcome, Error> {
		let global = context
			.module()
			.global_by_index(index)
			.expect("Due to validation global should exists");
		let val = global.get();
		context.value_stack_mut().push(val)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_set_global(
		&mut self,
		context: &mut FunctionContext,
		index: u32,
	) -> Result<InstructionOutcome, Error> {
		let val = context.value_stack_mut().pop()?;

		let global = context
			.module()
			.global_by_index(index)
			.expect("Due to validation global should exists");
		global.set(val).expect("Due to validation set to a global should succeed");
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_load<T>(&mut self, context: &mut FunctionContext, _align: u32, offset: u32) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T>, T: LittleEndianConvert {
		let address =
			effective_address(
				offset,
				context.value_stack_mut().pop_as()?
			)
			.map_err(Error::Trap)?;
		let m = context.module()
			.memory_by_index(DEFAULT_MEMORY_INDEX)
			.expect("Due to validation memory should exists");
		let b = m.get(address, mem::size_of::<T>())
			.map_err(|_| Error::Trap(Trap::MemoryAccessOutOfBounds))?;
		let n = T::from_little_endian(&b)
			.expect("Can't fail since buffer length should be size_of::<T>");
		context.value_stack_mut().push(n.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_load_extend<T, U>(&mut self, context: &mut FunctionContext, _align: u32, offset: u32) -> Result<InstructionOutcome, Error>
		where T: ExtendInto<U>, RuntimeValue: From<U>, T: LittleEndianConvert {
		let address =
			effective_address(
				offset,
				context.value_stack_mut().pop_as()?
			)
			.map_err(Error::Trap)?;
		let m = context.module()
			.memory_by_index(DEFAULT_MEMORY_INDEX)
			.expect("Due to validation memory should exists");
		let b = m.get(address, mem::size_of::<T>())
			.map_err(|_| Error::Trap(Trap::MemoryAccessOutOfBounds))?;
		let v = T::from_little_endian(&b)
			.expect("Can't fail since buffer length should be size_of::<T>");
		let stack_value: U = v.extend_into();
		context
			.value_stack_mut()
			.push(stack_value.into())
			.map_err(Into::into)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_store<T>(&mut self, context: &mut FunctionContext, _align: u32, offset: u32) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: LittleEndianConvert {
		let stack_value = context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|n| n.into_little_endian())?;
		let address =
			effective_address(
				offset,
				context.value_stack_mut().pop_as::<u32>()?
			)
			.map_err(Error::Trap)?;

		let m = context.module()
			.memory_by_index(DEFAULT_MEMORY_INDEX)
			.expect("Due to validation memory should exists");
		m.set(address, &stack_value)
			.map_err(|_| Error::Trap(Trap::MemoryAccessOutOfBounds))?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_store_wrap<T, U>(
		&mut self,
		context: &mut FunctionContext,
		_align: u32,
		offset: u32,
	) -> Result<InstructionOutcome, Error>
	where
		RuntimeValue: TryInto<T, Error>,
		T: WrapInto<U>,
		U: LittleEndianConvert,
	{
		let stack_value: T = context
			.value_stack_mut()
			.pop()
			.map_err(Into::into)
			.and_then(|v| v.try_into())?;
		let stack_value = stack_value.wrap_into().into_little_endian();
		let address =
			effective_address(
				offset,
				context.value_stack_mut().pop_as::<u32>()?
			)
			.map_err(Error::Trap)?;
		let m = context.module()
			.memory_by_index(DEFAULT_MEMORY_INDEX)
			.expect("Due to validation memory should exists");
		m.set(address, &stack_value)
			.map_err(|_| Error::Trap(Trap::MemoryAccessOutOfBounds))?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_current_memory(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		let m = context.module()
			.memory_by_index(DEFAULT_MEMORY_INDEX)
			.expect("Due to validation memory should exists");
		let s = m.size();
		context
			.value_stack_mut()
			.push(RuntimeValue::I32(s as i32))?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_grow_memory(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		let pages: u32 = context.value_stack_mut().pop_as()?;
		let m = context.module()
			.memory_by_index(DEFAULT_MEMORY_INDEX)
			.expect("Due to validation memory should exists");
		// Pushes -1 if allocation fails or previous memory size, if succeeds.
		let m = m.grow(pages)
			.unwrap_or(u32::MAX);
		context
			.value_stack_mut()
			.push(RuntimeValue::I32(m as i32))?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_const(&mut self, context: &mut FunctionContext, val: RuntimeValue) -> Result<InstructionOutcome, Error> {
		context
			.value_stack_mut()
			.push(val)
			.map_err(Into::into)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_eqz<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialEq<T> + Default {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| RuntimeValue::I32(if v == Default::default() { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_eq<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialEq<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left == right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_ne<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialEq<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left != right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_lt<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> + Display {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left < right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_gt<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left > right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_lte<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left <= right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_gte<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left >= right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_clz<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.leading_zeros())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_ctz<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.trailing_zeros())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_popcnt<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.count_ones())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_add<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: ArithmeticOps<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.add(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_sub<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: ArithmeticOps<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.sub(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_mul<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: ArithmeticOps<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.mul(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_div<T, U>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: TransmuteInto<U> + Display, U: ArithmeticOps<U> + TransmuteInto<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| (left.transmute_into(), right.transmute_into()))
			.map(|(left, right)| left.div(right))?
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_rem<T, U>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: TransmuteInto<U>, U: Integer<U> + TransmuteInto<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| (left.transmute_into(), right.transmute_into()))
			.map(|(left, right)| left.rem(right))?
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_and<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<<T as ops::BitAnd>::Output> + TryInto<T, Error>, T: ops::BitAnd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.bitand(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_or<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<<T as ops::BitOr>::Output> + TryInto<T, Error>, T: ops::BitOr<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.bitor(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_xor<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<<T as ops::BitXor>::Output> + TryInto<T, Error>, T: ops::BitXor<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.bitxor(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_shl<T>(&mut self, context: &mut FunctionContext, mask: T) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<<T as ops::Shl<T>>::Output> + TryInto<T, Error>, T: ops::Shl<T> + ops::BitAnd<T, Output=T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.shl(right & mask))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_shr<T, U>(&mut self, context: &mut FunctionContext, mask: U) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: TransmuteInto<U>, U: ops::Shr<U> + ops::BitAnd<U, Output=U>, <U as ops::Shr<U>>::Output: TransmuteInto<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| (left.transmute_into(), right.transmute_into()))
			.map(|(left, right)| left.shr(right & mask))
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_rotl<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.rotl(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_rotr<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.rotr(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_abs<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.abs())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_neg<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<<T as ops::Neg>::Output> + TryInto<T, Error>, T: ops::Neg {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.neg())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_ceil<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.ceil())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_floor<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.floor())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_trunc<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.trunc())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_nearest<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.nearest())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_sqrt<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.sqrt())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_min<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.min(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_max<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.max(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_copysign<T>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.copysign(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_wrap<T, U>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<U> + TryInto<T, Error>, T: WrapInto<U> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.wrap_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_trunc_to_int<T, U, V>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<V> + TryInto<T, Error>, T: TryTruncateInto<U, Error>, U: TransmuteInto<V>,  {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.and_then(|v| v.try_truncate_into())
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_extend<T, U, V>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<V> + TryInto<T, Error>, T: ExtendInto<U>, U: TransmuteInto<V> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map_err(Error::into)
			.map(|v| v.extend_into())
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_reinterpret<T, U>(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<U>, RuntimeValue: TryInto<T, Error>, T: TransmuteInto<U> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(TransmuteInto::transmute_into)
			.and_then(|val| context.value_stack_mut().push(val.into()).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}
}

impl FunctionContext {
	pub fn new(function: FuncRef, value_stack_limit: usize, frame_stack_limit: usize, signature: &Signature, args: Vec<RuntimeValue>) -> Self {
		let module = match *function.as_internal() {
			FuncInstanceInternal::Internal { ref module, .. } => module.upgrade().expect("module deallocated"),
			FuncInstanceInternal::Host { .. } => panic!("Host functions can't be called as internally defined functions; Thus FunctionContext can be created only with internally defined functions; qed"),
		};
		FunctionContext {
			is_initialized: false,
			function: function,
			module: ModuleRef(module),
			return_type: signature.return_type().map(|vt| BlockType::Value(vt.into_elements())).unwrap_or(BlockType::NoResult),
			value_stack: StackWithLimit::with_limit(value_stack_limit),
			frame_stack: StackWithLimit::with_limit(frame_stack_limit),
			locals: args,
			position: 0,
		}
	}

	pub fn nested(&mut self, function: FuncRef) -> Result<Self, Error> {
		let (function_locals, module, function_return_type) = {
			let module = match *function.as_internal() {
				FuncInstanceInternal::Internal { ref module, .. } => module.upgrade().expect("module deallocated"),
				FuncInstanceInternal::Host { .. } => panic!("Host functions can't be called as internally defined functions; Thus FunctionContext can be created only with internally defined functions; qed"),
			};
			let function_type = function.signature();
			let function_return_type = function_type.return_type().map(|vt| BlockType::Value(vt.into_elements())).unwrap_or(BlockType::NoResult);
			let function_locals = prepare_function_args(function_type, &mut self.value_stack)?;
			(function_locals, module, function_return_type)
		};

		Ok(FunctionContext {
			is_initialized: false,
			function: function,
			module: ModuleRef(module),
			return_type: function_return_type,
			value_stack: StackWithLimit::with_limit(self.value_stack.limit() - self.value_stack.len()),
			frame_stack: StackWithLimit::with_limit(self.frame_stack.limit() - self.frame_stack.len()),
			locals: function_locals,
			position: 0,
		})
	}

	pub fn is_initialized(&self) -> bool {
		self.is_initialized
	}

	pub fn initialize(&mut self, locals: &[Local]) {
		debug_assert!(!self.is_initialized);
		self.is_initialized = true;

		let locals = locals.iter()
			.flat_map(|l| repeat(l.value_type()).take(l.count() as usize))
			.map(RuntimeValue::default)
			.collect::<Vec<_>>();
		self.locals.extend(locals);
	}

	pub fn module(&self) -> ModuleRef {
		self.module.clone()
	}

	pub fn set_local(&mut self, index: usize, value: RuntimeValue) -> Result<InstructionOutcome, Error> {
		let l = self.locals.get_mut(index).expect("Due to validation local should exists");

		*l = value;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	pub fn get_local(&mut self, index: usize) -> RuntimeValue {
		self.locals.get(index)
			.cloned()
			.expect("Due to validation local should exists")
	}

	pub fn value_stack(&self) -> &StackWithLimit<RuntimeValue> {
		&self.value_stack
	}

	pub fn value_stack_mut(&mut self) -> &mut StackWithLimit<RuntimeValue> {
		&mut self.value_stack
	}

	pub fn frame_stack(&self) -> &StackWithLimit<BlockFrame> {
		&self.frame_stack
	}

	pub fn push_frame(&mut self, labels: &HashMap<usize, usize>, frame_type: BlockFrameType, block_type: BlockType) -> Result<(), Error> {
		let begin_position = self.position;
		let branch_position = match frame_type {
			BlockFrameType::Function => usize::MAX,
			BlockFrameType::Loop => begin_position,
			BlockFrameType::IfTrue => {
				let else_pos = labels[&begin_position];
				1usize + match labels.get(&else_pos) {
					Some(end_pos) => *end_pos,
					None => else_pos,
				}
			},
			_ => labels[&begin_position] + 1,
		};
		let end_position = match frame_type {
			BlockFrameType::Function => usize::MAX,
			_ => labels[&begin_position] + 1,
		};
		Ok(self.frame_stack.push(BlockFrame {
			frame_type: frame_type,
			block_type: block_type,
			begin_position: begin_position,
			branch_position: branch_position,
			end_position: end_position,
			value_stack_len: self.value_stack.len(),
		})?)
	}

	pub fn discard_frame(&mut self) -> Result<(), Error> {
		Ok(self.frame_stack.pop().map(|_| ())?)
	}

	pub fn pop_frame(&mut self, is_branch: bool) -> Result<(), Error> {
		let frame = self.frame_stack.pop()?;
		if frame.value_stack_len > self.value_stack.len() {
			return Err(Error::Stack("invalid stack len".into()));
		}

		let frame_value = match frame.block_type {
			BlockType::Value(_) if frame.frame_type != BlockFrameType::Loop || !is_branch => Some(self.value_stack.pop()?),
			_ => None,
		};
		self.value_stack.resize(frame.value_stack_len, RuntimeValue::I32(0));
		self.position = if is_branch { frame.branch_position } else { frame.end_position };
		if let Some(frame_value) = frame_value {
			self.value_stack.push(frame_value)?;
		}

		Ok(())
	}
}

impl fmt::Debug for FunctionContext {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "FunctionContext")
	}
}

fn effective_address(address: u32, offset: u32) -> Result<u32, Trap> {
	match offset.checked_add(address) {
		None => Err(Trap::MemoryAccessOutOfBounds),
		Some(address) => Ok(address),
	}
}

pub fn prepare_function_args(signature: &Signature, caller_stack: &mut StackWithLimit<RuntimeValue>) -> Result<Vec<RuntimeValue>, Error> {
	let mut args = signature.params().iter().cloned().rev().map(|expected_type| {
		let param_value = caller_stack.pop()?;
		let actual_type = param_value.value_type();
		if actual_type != expected_type {
			return Err(Error::Function(format!("invalid parameter type {:?} when expected {:?}", actual_type, expected_type)));
		}

		Ok(param_value)
	}).collect::<Result<Vec<_>, _>>()?;
	args.reverse();
	Ok(args)
}

impl StackWithLimit<RuntimeValue> {
	fn pop_as<T>(&mut self) -> Result<T, Error>
	where
		RuntimeValue: TryInto<T, Error>,
	{
		let value = self.pop()?;
		TryInto::try_into(value)
	}

	fn pop_pair_as<T>(&mut self) -> Result<(T, T), Error>
	where
		RuntimeValue: TryInto<T, Error>,
	{
		let right = self.pop_as()?;
		let left = self.pop_as()?;
		Ok((left, right))
	}

	fn pop_triple(&mut self) -> Result<(RuntimeValue, RuntimeValue, RuntimeValue), Error> {
		let right = self.pop()?;
		let mid = self.pop()?;
		let left = self.pop()?;
		Ok((left, mid, right))
	}
}
