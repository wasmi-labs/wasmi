#[allow(unused_imports)]
use alloc::prelude::*;
use core::ops;
use core::{u32, usize};
use core::fmt;
use core::iter::repeat;
use parity_wasm::elements::Local;
use {Error, Trap, TrapKind, Signature};
use module::ModuleRef;
use memory::MemoryRef;
use func::{FuncRef, FuncInstance, FuncInstanceInternal};
use value::{
	RuntimeValue, FromRuntimeValue, WrapInto, TryTruncateInto, ExtendInto,
	ArithmeticOps, Integer, Float, LittleEndianConvert, TransmuteInto,
};
use host::Externals;
use common::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX};
use types::ValueType;
use memory_units::Pages;
use nan_preserving_float::{F32, F64};
use isa;

/// Maximum number of entries in value stack.
pub const DEFAULT_VALUE_STACK_LIMIT: usize = (1024 * 1024) / ::core::mem::size_of::<RuntimeValue>();

// TODO: Make these parameters changeble.
pub const DEFAULT_CALL_STACK_LIMIT: usize = 64 * 1024;

/// Interpreter action to execute after executing instruction.
pub enum InstructionOutcome {
	/// Continue with next instruction.
	RunNextInstruction,
	/// Branch to an instruction at the given position.
	Branch(isa::Target),
	/// Execute function call.
	ExecuteCall(FuncRef),
	/// Return from current function block.
	Return(isa::DropKeep),
}

#[derive(PartialEq, Eq)]
/// Function execution state, related to pause and resume.
pub enum InterpreterState {
	/// The interpreter has been created, but has not been executed.
	Initialized,
	/// The interpreter has started execution, and cannot be called again if it exits normally, or no Host traps happened.
	Started,
	/// The interpreter has been executed, and returned a Host trap. It can resume execution by providing back a return
	/// value.
	Resumable(Option<ValueType>),
}

impl InterpreterState {
	pub fn is_resumable(&self) -> bool {
		match self {
			&InterpreterState::Resumable(_) => true,
			_ => false,
		}
	}
}

/// Function run result.
enum RunResult {
	/// Function has returned.
	Return,
	/// Function is calling other function.
	NestedCall(FuncRef),
}

/// Function interpreter.
pub struct Interpreter {
	value_stack: ValueStack,
	call_stack: Vec<FunctionContext>,
	return_type: Option<ValueType>,
	state: InterpreterState,
}

impl Interpreter {
	pub fn new(func: &FuncRef, args: &[RuntimeValue]) -> Result<Interpreter, Trap> {
		let mut value_stack = ValueStack::with_limit(DEFAULT_VALUE_STACK_LIMIT);
		for arg in args {
			value_stack
				.push(*arg)
				.map_err(
					// There is not enough space for pushing initial arguments.
					// Weird, but bail out anyway.
					|_| Trap::from(TrapKind::StackOverflow)
				)?;
		}

		let mut call_stack = Vec::new();
		let initial_frame = FunctionContext::new(func.clone());
		call_stack.push(initial_frame);

		let return_type = func.signature().return_type();

		Ok(Interpreter {
			value_stack,
			call_stack,
			return_type,
			state: InterpreterState::Initialized,
		})
	}

	pub fn state(&self) -> &InterpreterState {
		&self.state
	}

	pub fn start_execution<'a, E: Externals + 'a>(&mut self, externals: &'a mut E) -> Result<Option<RuntimeValue>, Trap> {
		// Ensure that the VM has not been executed. This is checked in `FuncInvocation::start_execution`.
		assert!(self.state == InterpreterState::Initialized);

		self.state = InterpreterState::Started;
		self.run_interpreter_loop(externals)?;

		let opt_return_value = self.return_type.map(|_vt| {
			self.value_stack.pop()
		});

		// Ensure that stack is empty after the execution. This is guaranteed by the validation properties.
		assert!(self.value_stack.len() == 0);

		Ok(opt_return_value)
	}

	pub fn resume_execution<'a, E: Externals + 'a>(&mut self, return_val: Option<RuntimeValue>, externals: &'a mut E) -> Result<Option<RuntimeValue>, Trap> {
		use core::mem::swap;

		// Ensure that the VM is resumable. This is checked in `FuncInvocation::resume_execution`.
		assert!(self.state.is_resumable());

		let mut resumable_state = InterpreterState::Started;
		swap(&mut self.state, &mut resumable_state);
		let expected_ty = match resumable_state {
			InterpreterState::Resumable(ty) => ty,
			_ => unreachable!("Resumable arm is checked above is_resumable; qed"),
		};

		let value_ty = return_val.as_ref().map(|val| val.value_type());
		if value_ty != expected_ty {
			return Err(TrapKind::UnexpectedSignature.into());
		}

		if let Some(return_val) = return_val {
			self.value_stack.push(return_val).map_err(Trap::new)?;
		}

		self.run_interpreter_loop(externals)?;

		let opt_return_value = self.return_type.map(|_vt| {
			self.value_stack.pop()
		});

		// Ensure that stack is empty after the execution. This is guaranteed by the validation properties.
		assert!(self.value_stack.len() == 0);

		Ok(opt_return_value)
	}

	fn run_interpreter_loop<'a, E: Externals + 'a>(&mut self, externals: &'a mut E) -> Result<(), Trap> {
		loop {
			let mut function_context = self.call_stack
				.pop()
				.expect("on loop entry - not empty; on loop continue - checking for emptiness; qed");
			let function_ref = function_context.function.clone();
			let function_body = function_ref
				.body()
				.expect(
					"Host functions checked in function_return below; Internal functions always have a body; qed"
				);

			if !function_context.is_initialized() {
				// Initialize stack frame for the function call.
				function_context.initialize(&function_body.locals, &mut self.value_stack)?;
			}

			let function_return =
				self.do_run_function(
					&mut function_context,
					&function_body.code,
				).map_err(Trap::new)?;

			match function_return {
				RunResult::Return => {
					if self.call_stack.last().is_none() {
						// This was the last frame in the call stack. This means we
						// are done executing.
						return Ok(());
					}
				},
				RunResult::NestedCall(nested_func) => {
					if self.call_stack.len() + 1 >= DEFAULT_CALL_STACK_LIMIT {
						return Err(TrapKind::StackOverflow.into());
					}

					match *nested_func.as_internal() {
						FuncInstanceInternal::Internal { .. } => {
							let nested_context = FunctionContext::new(nested_func.clone());
							self.call_stack.push(function_context);
							self.call_stack.push(nested_context);
						},
						FuncInstanceInternal::Host { ref signature, .. } => {
							let args = prepare_function_args(signature, &mut self.value_stack);
							// We push the function context first. If the VM is not resumable, it does no harm. If it is, we then save the context here.
							self.call_stack.push(function_context);

							let return_val = match FuncInstance::invoke(&nested_func, &args, externals) {
								Ok(val) => val,
								Err(trap) => {
									if trap.kind().is_host() {
										self.state = InterpreterState::Resumable(nested_func.signature().return_type());
									}
									return Err(trap);
								},
							};

							// Check if `return_val` matches the signature.
							let value_ty = return_val.as_ref().map(|val| val.value_type());
							let expected_ty = nested_func.signature().return_type();
							if value_ty != expected_ty {
								return Err(TrapKind::UnexpectedSignature.into());
							}

							if let Some(return_val) = return_val {
								self.value_stack.push(return_val).map_err(Trap::new)?;
							}
						}
					}
				},
			}
		}
	}

	fn do_run_function(&mut self, function_context: &mut FunctionContext, instructions: &isa::Instructions)
		-> Result<RunResult, TrapKind>
	{
		let mut iter = instructions.iterate_from(function_context.position);
		loop {
			let instruction = iter.next().expect("instruction");

			match self.run_instruction(function_context, instruction)? {
				InstructionOutcome::RunNextInstruction => {},
				InstructionOutcome::Branch(target) => {
					iter = instructions.iterate_from(target.dst_pc);
					self.value_stack.drop_keep(target.drop_keep);
				},
				InstructionOutcome::ExecuteCall(func_ref) => {
					function_context.position = iter.position();
					return Ok(RunResult::NestedCall(func_ref));
				},
				InstructionOutcome::Return(drop_keep) => {
					self.value_stack.drop_keep(drop_keep);
					break;
				},
			}
		}

		Ok(RunResult::Return)
	}

	#[inline(always)]
	fn run_instruction(&mut self, context: &mut FunctionContext, instruction: &isa::Instruction) -> Result<InstructionOutcome, TrapKind> {
		match instruction {
			&isa::Instruction::Unreachable => self.run_unreachable(context),

			&isa::Instruction::Br(ref target) => self.run_br(context, target.clone()),
			&isa::Instruction::BrIfEqz(ref target) => self.run_br_eqz(target.clone()),
			&isa::Instruction::BrIfNez(ref target) => self.run_br_nez(target.clone()),
			&isa::Instruction::BrTable(ref targets) => self.run_br_table(targets),
			&isa::Instruction::Return(drop_keep) => self.run_return(drop_keep),

			&isa::Instruction::Call(index) => self.run_call(context, index),
			&isa::Instruction::CallIndirect(index) => self.run_call_indirect(context, index),

			&isa::Instruction::Drop => self.run_drop(),
			&isa::Instruction::Select => self.run_select(),

			&isa::Instruction::GetLocal(depth) => self.run_get_local(depth),
			&isa::Instruction::SetLocal(depth) => self.run_set_local(depth),
			&isa::Instruction::TeeLocal(depth) => self.run_tee_local(depth),
			&isa::Instruction::GetGlobal(index) => self.run_get_global(context, index),
			&isa::Instruction::SetGlobal(index) => self.run_set_global(context, index),

			&isa::Instruction::I32Load(offset) => self.run_load::<i32>(context, offset),
			&isa::Instruction::I64Load(offset) => self.run_load::<i64>(context, offset),
			&isa::Instruction::F32Load(offset) => self.run_load::<F32>(context, offset),
			&isa::Instruction::F64Load(offset) => self.run_load::<F64>(context, offset),
			&isa::Instruction::I32Load8S(offset) => self.run_load_extend::<i8, i32>(context, offset),
			&isa::Instruction::I32Load8U(offset) => self.run_load_extend::<u8, i32>(context, offset),
			&isa::Instruction::I32Load16S(offset) => self.run_load_extend::<i16, i32>(context, offset),
			&isa::Instruction::I32Load16U(offset) => self.run_load_extend::<u16, i32>(context, offset),
			&isa::Instruction::I64Load8S(offset) => self.run_load_extend::<i8, i64>(context, offset),
			&isa::Instruction::I64Load8U(offset) => self.run_load_extend::<u8, i64>(context, offset),
			&isa::Instruction::I64Load16S(offset) => self.run_load_extend::<i16, i64>(context, offset),
			&isa::Instruction::I64Load16U(offset) => self.run_load_extend::<u16, i64>(context, offset),
			&isa::Instruction::I64Load32S(offset) => self.run_load_extend::<i32, i64>(context, offset),
			&isa::Instruction::I64Load32U(offset) => self.run_load_extend::<u32, i64>(context, offset),

			&isa::Instruction::I32Store(offset) => self.run_store::<i32>(context, offset),
			&isa::Instruction::I64Store(offset) => self.run_store::<i64>(context, offset),
			&isa::Instruction::F32Store(offset) => self.run_store::<F32>(context, offset),
			&isa::Instruction::F64Store(offset) => self.run_store::<F64>(context, offset),
			&isa::Instruction::I32Store8(offset) => self.run_store_wrap::<i32, i8>(context, offset),
			&isa::Instruction::I32Store16(offset) => self.run_store_wrap::<i32, i16>(context, offset),
			&isa::Instruction::I64Store8(offset) => self.run_store_wrap::<i64, i8>(context, offset),
			&isa::Instruction::I64Store16(offset) => self.run_store_wrap::<i64, i16>(context, offset),
			&isa::Instruction::I64Store32(offset) => self.run_store_wrap::<i64, i32>(context, offset),

			&isa::Instruction::CurrentMemory => self.run_current_memory(context),
			&isa::Instruction::GrowMemory => self.run_grow_memory(context),

			&isa::Instruction::I32Const(val) => self.run_const(val.into()),
			&isa::Instruction::I64Const(val) => self.run_const(val.into()),
			&isa::Instruction::F32Const(val) => self.run_const(RuntimeValue::decode_f32(val)),
			&isa::Instruction::F64Const(val) => self.run_const(RuntimeValue::decode_f64(val)),

			&isa::Instruction::I32Eqz => self.run_eqz::<i32>(),
			&isa::Instruction::I32Eq => self.run_eq::<i32>(),
			&isa::Instruction::I32Ne => self.run_ne::<i32>(),
			&isa::Instruction::I32LtS => self.run_lt::<i32>(),
			&isa::Instruction::I32LtU => self.run_lt::<u32>(),
			&isa::Instruction::I32GtS => self.run_gt::<i32>(),
			&isa::Instruction::I32GtU => self.run_gt::<u32>(),
			&isa::Instruction::I32LeS => self.run_lte::<i32>(),
			&isa::Instruction::I32LeU => self.run_lte::<u32>(),
			&isa::Instruction::I32GeS => self.run_gte::<i32>(),
			&isa::Instruction::I32GeU => self.run_gte::<u32>(),

			&isa::Instruction::I64Eqz => self.run_eqz::<i64>(),
			&isa::Instruction::I64Eq => self.run_eq::<i64>(),
			&isa::Instruction::I64Ne => self.run_ne::<i64>(),
			&isa::Instruction::I64LtS => self.run_lt::<i64>(),
			&isa::Instruction::I64LtU => self.run_lt::<u64>(),
			&isa::Instruction::I64GtS => self.run_gt::<i64>(),
			&isa::Instruction::I64GtU => self.run_gt::<u64>(),
			&isa::Instruction::I64LeS => self.run_lte::<i64>(),
			&isa::Instruction::I64LeU => self.run_lte::<u64>(),
			&isa::Instruction::I64GeS => self.run_gte::<i64>(),
			&isa::Instruction::I64GeU => self.run_gte::<u64>(),

			&isa::Instruction::F32Eq => self.run_eq::<F32>(),
			&isa::Instruction::F32Ne => self.run_ne::<F32>(),
			&isa::Instruction::F32Lt => self.run_lt::<F32>(),
			&isa::Instruction::F32Gt => self.run_gt::<F32>(),
			&isa::Instruction::F32Le => self.run_lte::<F32>(),
			&isa::Instruction::F32Ge => self.run_gte::<F32>(),

			&isa::Instruction::F64Eq => self.run_eq::<F64>(),
			&isa::Instruction::F64Ne => self.run_ne::<F64>(),
			&isa::Instruction::F64Lt => self.run_lt::<F64>(),
			&isa::Instruction::F64Gt => self.run_gt::<F64>(),
			&isa::Instruction::F64Le => self.run_lte::<F64>(),
			&isa::Instruction::F64Ge => self.run_gte::<F64>(),

			&isa::Instruction::I32Clz => self.run_clz::<i32>(),
			&isa::Instruction::I32Ctz => self.run_ctz::<i32>(),
			&isa::Instruction::I32Popcnt => self.run_popcnt::<i32>(),
			&isa::Instruction::I32Add => self.run_add::<i32>(),
			&isa::Instruction::I32Sub => self.run_sub::<i32>(),
			&isa::Instruction::I32Mul => self.run_mul::<i32>(),
			&isa::Instruction::I32DivS => self.run_div::<i32, i32>(),
			&isa::Instruction::I32DivU => self.run_div::<i32, u32>(),
			&isa::Instruction::I32RemS => self.run_rem::<i32, i32>(),
			&isa::Instruction::I32RemU => self.run_rem::<i32, u32>(),
			&isa::Instruction::I32And => self.run_and::<i32>(),
			&isa::Instruction::I32Or => self.run_or::<i32>(),
			&isa::Instruction::I32Xor => self.run_xor::<i32>(),
			&isa::Instruction::I32Shl => self.run_shl::<i32>(0x1F),
			&isa::Instruction::I32ShrS => self.run_shr::<i32, i32>(0x1F),
			&isa::Instruction::I32ShrU => self.run_shr::<i32, u32>(0x1F),
			&isa::Instruction::I32Rotl => self.run_rotl::<i32>(),
			&isa::Instruction::I32Rotr => self.run_rotr::<i32>(),

			&isa::Instruction::I64Clz => self.run_clz::<i64>(),
			&isa::Instruction::I64Ctz => self.run_ctz::<i64>(),
			&isa::Instruction::I64Popcnt => self.run_popcnt::<i64>(),
			&isa::Instruction::I64Add => self.run_add::<i64>(),
			&isa::Instruction::I64Sub => self.run_sub::<i64>(),
			&isa::Instruction::I64Mul => self.run_mul::<i64>(),
			&isa::Instruction::I64DivS => self.run_div::<i64, i64>(),
			&isa::Instruction::I64DivU => self.run_div::<i64, u64>(),
			&isa::Instruction::I64RemS => self.run_rem::<i64, i64>(),
			&isa::Instruction::I64RemU => self.run_rem::<i64, u64>(),
			&isa::Instruction::I64And => self.run_and::<i64>(),
			&isa::Instruction::I64Or => self.run_or::<i64>(),
			&isa::Instruction::I64Xor => self.run_xor::<i64>(),
			&isa::Instruction::I64Shl => self.run_shl::<i64>(0x3F),
			&isa::Instruction::I64ShrS => self.run_shr::<i64, i64>(0x3F),
			&isa::Instruction::I64ShrU => self.run_shr::<i64, u64>(0x3F),
			&isa::Instruction::I64Rotl => self.run_rotl::<i64>(),
			&isa::Instruction::I64Rotr => self.run_rotr::<i64>(),

			&isa::Instruction::F32Abs => self.run_abs::<F32>(),
			&isa::Instruction::F32Neg => self.run_neg::<F32>(),
			&isa::Instruction::F32Ceil => self.run_ceil::<F32>(),
			&isa::Instruction::F32Floor => self.run_floor::<F32>(),
			&isa::Instruction::F32Trunc => self.run_trunc::<F32>(),
			&isa::Instruction::F32Nearest => self.run_nearest::<F32>(),
			&isa::Instruction::F32Sqrt => self.run_sqrt::<F32>(),
			&isa::Instruction::F32Add => self.run_add::<F32>(),
			&isa::Instruction::F32Sub => self.run_sub::<F32>(),
			&isa::Instruction::F32Mul => self.run_mul::<F32>(),
			&isa::Instruction::F32Div => self.run_div::<F32, F32>(),
			&isa::Instruction::F32Min => self.run_min::<F32>(),
			&isa::Instruction::F32Max => self.run_max::<F32>(),
			&isa::Instruction::F32Copysign => self.run_copysign::<F32>(),

			&isa::Instruction::F64Abs => self.run_abs::<F64>(),
			&isa::Instruction::F64Neg => self.run_neg::<F64>(),
			&isa::Instruction::F64Ceil => self.run_ceil::<F64>(),
			&isa::Instruction::F64Floor => self.run_floor::<F64>(),
			&isa::Instruction::F64Trunc => self.run_trunc::<F64>(),
			&isa::Instruction::F64Nearest => self.run_nearest::<F64>(),
			&isa::Instruction::F64Sqrt => self.run_sqrt::<F64>(),
			&isa::Instruction::F64Add => self.run_add::<F64>(),
			&isa::Instruction::F64Sub => self.run_sub::<F64>(),
			&isa::Instruction::F64Mul => self.run_mul::<F64>(),
			&isa::Instruction::F64Div => self.run_div::<F64, F64>(),
			&isa::Instruction::F64Min => self.run_min::<F64>(),
			&isa::Instruction::F64Max => self.run_max::<F64>(),
			&isa::Instruction::F64Copysign => self.run_copysign::<F64>(),

			&isa::Instruction::I32WrapI64 => self.run_wrap::<i64, i32>(),
			&isa::Instruction::I32TruncSF32 => self.run_trunc_to_int::<F32, i32, i32>(),
			&isa::Instruction::I32TruncUF32 => self.run_trunc_to_int::<F32, u32, i32>(),
			&isa::Instruction::I32TruncSF64 => self.run_trunc_to_int::<F64, i32, i32>(),
			&isa::Instruction::I32TruncUF64 => self.run_trunc_to_int::<F64, u32, i32>(),
			&isa::Instruction::I64ExtendSI32 => self.run_extend::<i32, i64, i64>(),
			&isa::Instruction::I64ExtendUI32 => self.run_extend::<u32, u64, i64>(),
			&isa::Instruction::I64TruncSF32 => self.run_trunc_to_int::<F32, i64, i64>(),
			&isa::Instruction::I64TruncUF32 => self.run_trunc_to_int::<F32, u64, i64>(),
			&isa::Instruction::I64TruncSF64 => self.run_trunc_to_int::<F64, i64, i64>(),
			&isa::Instruction::I64TruncUF64 => self.run_trunc_to_int::<F64, u64, i64>(),
			&isa::Instruction::F32ConvertSI32 => self.run_extend::<i32, F32, F32>(),
			&isa::Instruction::F32ConvertUI32 => self.run_extend::<u32, F32, F32>(),
			&isa::Instruction::F32ConvertSI64 => self.run_wrap::<i64, F32>(),
			&isa::Instruction::F32ConvertUI64 => self.run_wrap::<u64, F32>(),
			&isa::Instruction::F32DemoteF64 => self.run_wrap::<F64, F32>(),
			&isa::Instruction::F64ConvertSI32 => self.run_extend::<i32, F64, F64>(),
			&isa::Instruction::F64ConvertUI32 => self.run_extend::<u32, F64, F64>(),
			&isa::Instruction::F64ConvertSI64 => self.run_extend::<i64, F64, F64>(),
			&isa::Instruction::F64ConvertUI64 => self.run_extend::<u64, F64, F64>(),
			&isa::Instruction::F64PromoteF32 => self.run_extend::<F32, F64, F64>(),

			&isa::Instruction::I32ReinterpretF32 => self.run_reinterpret::<F32, i32>(),
			&isa::Instruction::I64ReinterpretF64 => self.run_reinterpret::<F64, i64>(),
			&isa::Instruction::F32ReinterpretI32 => self.run_reinterpret::<i32, F32>(),
			&isa::Instruction::F64ReinterpretI64 => self.run_reinterpret::<i64, F64>(),
		}
	}

	fn run_unreachable(&mut self, _context: &mut FunctionContext) -> Result<InstructionOutcome, TrapKind> {
		Err(TrapKind::Unreachable)
	}

	fn run_br(&mut self, _context: &mut FunctionContext, target: isa::Target) -> Result<InstructionOutcome, TrapKind> {
		Ok(InstructionOutcome::Branch(target))
	}

	fn run_br_nez(&mut self, target: isa::Target) -> Result<InstructionOutcome, TrapKind> {
		let condition = self.value_stack.pop_as();
		if condition {
			Ok(InstructionOutcome::Branch(target))
		} else {
			Ok(InstructionOutcome::RunNextInstruction)
		}
	}

	fn run_br_eqz(&mut self, target: isa::Target) -> Result<InstructionOutcome, TrapKind> {
		let condition = self.value_stack.pop_as();
		if condition {
			Ok(InstructionOutcome::RunNextInstruction)
		} else {

			Ok(InstructionOutcome::Branch(target))
		}
	}

	fn run_br_table(&mut self, table: &[isa::Target]) -> Result<InstructionOutcome, TrapKind> {
		let index: u32 = self.value_stack.pop_as();

		let dst = if (index as usize) < table.len() - 1 {
			table[index as usize].clone()
		} else {
			table
				.last()
				.expect("Due to validation there should be at least one label")
				.clone()
		};
		Ok(InstructionOutcome::Branch(dst))
	}

	fn run_return(&mut self, drop_keep: isa::DropKeep) -> Result<InstructionOutcome, TrapKind> {
		Ok(InstructionOutcome::Return(drop_keep))
	}

	fn run_call(
		&mut self,
		context: &mut FunctionContext,
		func_idx: u32,
	) -> Result<InstructionOutcome, TrapKind> {
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
	) -> Result<InstructionOutcome, TrapKind> {
		let table_func_idx: u32 = self.value_stack.pop_as();
		let table = context
			.module()
			.table_by_index(DEFAULT_TABLE_INDEX)
			.expect("Due to validation table should exists");
		let func_ref = table.get(table_func_idx)
			.map_err(|_| TrapKind::TableAccessOutOfBounds)?
			.ok_or_else(|| TrapKind::ElemUninitialized)?;

		{
			let actual_function_type = func_ref.signature();
			let required_function_type = context
				.module()
				.signature_by_index(signature_idx)
				.expect("Due to validation type should exists");

			if &*required_function_type != actual_function_type {
				return Err(TrapKind::UnexpectedSignature);
			}
		}

		Ok(InstructionOutcome::ExecuteCall(func_ref))
	}

	fn run_drop(&mut self) -> Result<InstructionOutcome, TrapKind> {
		let _ = self.value_stack.pop();
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_select(&mut self) -> Result<InstructionOutcome, TrapKind> {
		let (left, mid, right) = self
			.value_stack
			.pop_triple();

		let condition = right
			.try_into()
			.expect("Due to validation stack top should be I32");
		let val = if condition { left } else { mid };
		self.value_stack.push(val)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_get_local(&mut self, index: u32) -> Result<InstructionOutcome, TrapKind> {
		let val = *self.value_stack.pick_mut(index as usize);
		self.value_stack.push(val)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_set_local(&mut self, index: u32) -> Result<InstructionOutcome, TrapKind> {
		let val = self
			.value_stack
			.pop();
		*self.value_stack.pick_mut(index as usize) = val;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_tee_local(&mut self, index: u32) -> Result<InstructionOutcome, TrapKind> {
		let val = self
			.value_stack
			.top()
			.clone();
		*self.value_stack.pick_mut(index as usize) = val;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_get_global(
		&mut self,
		context: &mut FunctionContext,
		index: u32,
	) -> Result<InstructionOutcome, TrapKind> {
		let global = context
			.module()
			.global_by_index(index)
			.expect("Due to validation global should exists");
		let val = global.get();
		self.value_stack.push(val)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_set_global(
		&mut self,
		context: &mut FunctionContext,
		index: u32,
	) -> Result<InstructionOutcome, TrapKind> {
		let val = self
			.value_stack
			.pop();
		let global = context
			.module()
			.global_by_index(index)
			.expect("Due to validation global should exists");
		global.set(val).expect("Due to validation set to a global should succeed");
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_load<T>(&mut self, context: &mut FunctionContext, offset: u32) -> Result<InstructionOutcome, TrapKind>
		where RuntimeValue: From<T>, T: LittleEndianConvert {
		let raw_address = self
			.value_stack
			.pop_as();
		let address =
			effective_address(
				offset,
				raw_address,
			)?;
		let m = context
			.memory()
			.expect("Due to validation memory should exists");
		let n: T = m.get_value(address)
			.map_err(|_| TrapKind::MemoryAccessOutOfBounds)?;
		self.value_stack.push(n.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_load_extend<T, U>(&mut self, context: &mut FunctionContext, offset: u32) -> Result<InstructionOutcome, TrapKind>
		where T: ExtendInto<U>, RuntimeValue: From<U>, T: LittleEndianConvert {
		let raw_address = self
			.value_stack
			.pop_as();
		let address =
			effective_address(
				offset,
				raw_address,
			)?;
		let m = context
			.memory()
			.expect("Due to validation memory should exists");
		let v: T = m.get_value(address)
			.map_err(|_| TrapKind::MemoryAccessOutOfBounds)?;
		let stack_value: U = v.extend_into();
		self
			.value_stack
			.push(stack_value.into())
			.map_err(Into::into)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_store<T>(&mut self, context: &mut FunctionContext, offset: u32) -> Result<InstructionOutcome, TrapKind>
		where T: FromRuntimeValue, T: LittleEndianConvert {
		let stack_value = self
			.value_stack
			.pop_as::<T>();
		let raw_address = self
			.value_stack
			.pop_as::<u32>();
		let address =
			effective_address(
				offset,
				raw_address,
			)?;

		let m = context
			.memory()
			.expect("Due to validation memory should exists");
		m.set_value(address, stack_value)
			.map_err(|_| TrapKind::MemoryAccessOutOfBounds)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_store_wrap<T, U>(
		&mut self,
		context: &mut FunctionContext,
		offset: u32,
	) -> Result<InstructionOutcome, TrapKind>
	where
		T: FromRuntimeValue,
		T: WrapInto<U>,
		U: LittleEndianConvert,
	{
		let stack_value: T = self
			.value_stack
			.pop()
			.try_into()
			.expect("Due to validation value should be of proper type");
		let stack_value = stack_value.wrap_into();
		let raw_address = self
			.value_stack
			.pop_as::<u32>();
		let address =
			effective_address(
				offset,
				raw_address,
			)?;
		let m = context
			.memory()
			.expect("Due to validation memory should exists");
		m.set_value(address, stack_value)
			.map_err(|_| TrapKind::MemoryAccessOutOfBounds)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_current_memory(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, TrapKind> {
		let m = context
			.memory()
			.expect("Due to validation memory should exists");
		let s = m.current_size().0;
		self
			.value_stack
			.push(RuntimeValue::I32(s as i32))?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_grow_memory(&mut self, context: &mut FunctionContext) -> Result<InstructionOutcome, TrapKind> {
		let pages: u32 = self
			.value_stack
			.pop_as();
		let m = context
			.memory()
			.expect("Due to validation memory should exists");
		let m = match m.grow(Pages(pages as usize)) {
			Ok(Pages(new_size)) => new_size as u32,
			Err(_) => u32::MAX, // Returns -1 (or 0xFFFFFFFF) in case of error.
		};
		self
			.value_stack
			.push(RuntimeValue::I32(m as i32))?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_const(&mut self, val: RuntimeValue) -> Result<InstructionOutcome, TrapKind> {
		self
			.value_stack
			.push(val)
			.map_err(Into::into)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_relop<T, F>(&mut self, f: F) -> Result<InstructionOutcome, TrapKind>
	where
		T: FromRuntimeValue,
		F: FnOnce(T, T) -> bool,
	{
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = if f(left, right) {
			RuntimeValue::I32(1)
		} else {
			RuntimeValue::I32(0)
		};
		self.value_stack.push(v)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_eqz<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
		where T: FromRuntimeValue, T: PartialEq<T> + Default {
		let v = self
			.value_stack
			.pop_as::<T>();
		let v = RuntimeValue::I32(if v == Default::default() { 1 } else { 0 });
		self.value_stack.push(v)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_eq<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where T: FromRuntimeValue + PartialEq<T>
	{
		self.run_relop(|left: T, right: T| left == right)
	}

	fn run_ne<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
		where T: FromRuntimeValue + PartialEq<T> {
		self.run_relop(|left: T, right: T| left != right)
	}

	fn run_lt<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
		where T: FromRuntimeValue + PartialOrd<T> {
		self.run_relop(|left: T, right: T| left < right)
	}

	fn run_gt<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
		where T: FromRuntimeValue + PartialOrd<T> {
		self.run_relop(|left: T, right: T| left > right)
	}

	fn run_lte<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
		where T: FromRuntimeValue + PartialOrd<T> {
		self.run_relop(|left: T, right: T| left <= right)
	}

	fn run_gte<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
		where T: FromRuntimeValue + PartialOrd<T> {
		self.run_relop(|left: T, right: T| left >= right)
	}

	fn run_unop<T, U, F>(&mut self, f: F) -> Result<InstructionOutcome, TrapKind>
	where
		F: FnOnce(T) -> U,
		T: FromRuntimeValue,
		RuntimeValue: From<U>
	{
		let v = self
			.value_stack
			.pop_as::<T>();
		let v = f(v);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_clz<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Integer<T> + FromRuntimeValue {
		self.run_unop(|v: T| v.leading_zeros())
	}

	fn run_ctz<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Integer<T> + FromRuntimeValue {
		self.run_unop(|v: T| v.trailing_zeros())
	}

	fn run_popcnt<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Integer<T> + FromRuntimeValue {
		self.run_unop(|v: T| v.count_ones())
	}

	fn run_add<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
		where RuntimeValue: From<T>, T: ArithmeticOps<T> + FromRuntimeValue {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.add(right);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_sub<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
		where RuntimeValue: From<T>, T: ArithmeticOps<T> + FromRuntimeValue {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.sub(right);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_mul<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: ArithmeticOps<T> + FromRuntimeValue {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.mul(right);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_div<T, U>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: TransmuteInto<U> + FromRuntimeValue, U: ArithmeticOps<U> + TransmuteInto<T> {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let (left, right) = (left.transmute_into(), right.transmute_into());
		let v = left.div(right)?;
		let v = v.transmute_into();
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_rem<T, U>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: TransmuteInto<U> + FromRuntimeValue, U: Integer<U> + TransmuteInto<T> {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let (left, right) = (left.transmute_into(), right.transmute_into());
		let v = left.rem(right)?;
		let v = v.transmute_into();
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_and<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<<T as ops::BitAnd>::Output>, T: ops::BitAnd<T> + FromRuntimeValue {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.bitand(right);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_or<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<<T as ops::BitOr>::Output>, T: ops::BitOr<T> + FromRuntimeValue {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.bitor(right);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_xor<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<<T as ops::BitXor>::Output>, T: ops::BitXor<T> + FromRuntimeValue {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.bitxor(right);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_shl<T>(&mut self, mask: T) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<<T as ops::Shl<T>>::Output>, T: ops::Shl<T> + ops::BitAnd<T, Output=T> + FromRuntimeValue {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.shl(right & mask);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_shr<T, U>(&mut self, mask: U) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: TransmuteInto<U> + FromRuntimeValue, U: ops::Shr<U> + ops::BitAnd<U, Output=U>, <U as ops::Shr<U>>::Output: TransmuteInto<T> {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let (left, right) = (left.transmute_into(), right.transmute_into());
		let v = left.shr(right & mask);
		let v = v.transmute_into();
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_rotl<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Integer<T> + FromRuntimeValue {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.rotl(right);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_rotr<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Integer<T> + FromRuntimeValue
	{
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.rotr(right);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_abs<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Float<T> + FromRuntimeValue
	{
		self.run_unop(|v: T| v.abs())
	}

	fn run_neg<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where
		RuntimeValue: From<<T as ops::Neg>::Output>,
		T: ops::Neg + FromRuntimeValue
	{
		self.run_unop(|v: T| v.neg())
	}

	fn run_ceil<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Float<T> + FromRuntimeValue
	{
		self.run_unop(|v: T| v.ceil())
	}

	fn run_floor<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Float<T> + FromRuntimeValue
	{
		self.run_unop(|v: T| v.floor())
	}

	fn run_trunc<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Float<T> + FromRuntimeValue
	{
		self.run_unop(|v: T| v.trunc())
	}

	fn run_nearest<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Float<T> + FromRuntimeValue
	{
		self.run_unop(|v: T| v.nearest())
	}

	fn run_sqrt<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Float<T> + FromRuntimeValue
	{
		self.run_unop(|v: T| v.sqrt())
	}

	fn run_min<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Float<T> + FromRuntimeValue
	{
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.min(right);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_max<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Float<T> + FromRuntimeValue {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.max(right);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_copysign<T>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<T>, T: Float<T> + FromRuntimeValue {
		let (left, right) = self
			.value_stack
			.pop_pair_as::<T>();
		let v = left.copysign(right);
		self.value_stack.push(v.into())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_wrap<T, U>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where RuntimeValue: From<U>, T: WrapInto<U> + FromRuntimeValue {
		self.run_unop(|v: T| v.wrap_into())
	}

	fn run_trunc_to_int<T, U, V>(&mut self) -> Result<InstructionOutcome, TrapKind>
		where RuntimeValue: From<V>, T: TryTruncateInto<U, TrapKind> + FromRuntimeValue, U: TransmuteInto<V>,  {
		let v = self
			.value_stack
			.pop_as::<T>();

		v.try_truncate_into()
			.map(|v| v.transmute_into())
			.map(|v| self.value_stack.push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_extend<T, U, V>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where
		RuntimeValue: From<V>, T: ExtendInto<U> + FromRuntimeValue, U: TransmuteInto<V>
	{
		let v = self
			.value_stack
			.pop_as::<T>();

		let v = v.extend_into().transmute_into();
		self.value_stack.push(v.into())?;

		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_reinterpret<T, U>(&mut self) -> Result<InstructionOutcome, TrapKind>
	where
		RuntimeValue: From<U>, T: FromRuntimeValue, T: TransmuteInto<U>
	{
		let v = self
			.value_stack
			.pop_as::<T>();

		let v = v.transmute_into();
		self.value_stack.push(v.into())?;

		Ok(InstructionOutcome::RunNextInstruction)
	}
}

/// Function execution context.
struct FunctionContext {
	/// Is context initialized.
	pub is_initialized: bool,
	/// Internal function reference.
	pub function: FuncRef,
	pub module: ModuleRef,
	pub memory: Option<MemoryRef>,
	/// Current instruction position.
	pub position: u32,
}

impl FunctionContext {
	pub fn new(function: FuncRef) -> Self {
		let module = match *function.as_internal() {
			FuncInstanceInternal::Internal { ref module, .. } => module.upgrade().expect("module deallocated"),
			FuncInstanceInternal::Host { .. } => panic!("Host functions can't be called as internally defined functions; Thus FunctionContext can be created only with internally defined functions; qed"),
		};
		let memory = module.memory_by_index(DEFAULT_MEMORY_INDEX);
		FunctionContext {
			is_initialized: false,
			function: function,
			module: ModuleRef(module),
			memory: memory,
			position: 0,
		}
	}

	pub fn is_initialized(&self) -> bool {
		self.is_initialized
	}

	pub fn initialize(&mut self, locals: &[Local], value_stack: &mut ValueStack) -> Result<(), TrapKind> {
		debug_assert!(!self.is_initialized);

		let locals = locals.iter()
			.flat_map(|l| repeat(l.value_type()).take(l.count() as usize))
			.map(::types::ValueType::from_elements)
			.map(RuntimeValue::default)
			.collect::<Vec<_>>();

		// TODO: Replace with extend.
		for local in locals {
			value_stack.push(local)
				.map_err(|_| TrapKind::StackOverflow)?;
		}

		self.is_initialized = true;
		Ok(())
	}

	pub fn module(&self) -> ModuleRef {
		self.module.clone()
	}

	pub fn memory(&self) -> Option<&MemoryRef> {
		self.memory.as_ref()
	}
}

impl fmt::Debug for FunctionContext {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "FunctionContext")
	}
}

fn effective_address(address: u32, offset: u32) -> Result<u32, TrapKind> {
	match offset.checked_add(address) {
		None => Err(TrapKind::MemoryAccessOutOfBounds),
		Some(address) => Ok(address),
	}
}

fn prepare_function_args(
	signature: &Signature,
	caller_stack: &mut ValueStack,
) -> Vec<RuntimeValue> {
	let mut args = signature
		.params()
		.iter()
		.map(|_| caller_stack.pop())
		.collect::<Vec<RuntimeValue>>();
	args.reverse();
	check_function_args(signature, &args).expect("Due to validation arguments should match");
	args
}

pub fn check_function_args(signature: &Signature, args: &[RuntimeValue]) -> Result<(), Error> {
	if signature.params().len() != args.len() {
		return Err(
			Error::Function(
				format!(
					"not enough arguments, given {} but expected: {}",
					args.len(),
					signature.params().len(),
				)
			)
		);
	}

	signature.params().iter().cloned().zip(args).map(|(expected_type, param_value)| {
		let actual_type = param_value.value_type();
		if actual_type != expected_type {
			return Err(Error::Function(format!("invalid parameter type {:?} when expected {:?}", actual_type, expected_type)));
		}
		Ok(())
	}).collect::<Result<Vec<_>, _>>()?;

	Ok(())
}

#[derive(Debug)]
struct ValueStack {
	buf: Box<[RuntimeValue]>,
	/// Index of the first free place in the stack.
	sp: usize,
}

impl ValueStack {
	fn with_limit(limit: usize) -> ValueStack {
		let mut buf = Vec::new();
		buf.resize(limit, RuntimeValue::I32(0));

		ValueStack {
			buf: buf.into_boxed_slice(),
			sp: 0,
		}
	}

	#[inline]
	fn drop_keep(&mut self, drop_keep: isa::DropKeep) {
		if drop_keep.keep == isa::Keep::Single {
			let top = *self.top();
			*self.pick_mut(drop_keep.drop as usize + 1) = top;
		}

		let cur_stack_len = self.len();
		self.sp = cur_stack_len - drop_keep.drop as usize;
	}

	#[inline]
	fn pop_as<T>(&mut self) -> T
	where
		T: FromRuntimeValue,
	{
		let value = self.pop();
		value.try_into().expect("Due to validation stack top's type should match")
	}

	#[inline]
	fn pop_pair_as<T>(&mut self) -> (T, T)
	where
		T: FromRuntimeValue,
	{
		let right = self.pop_as();
		let left = self.pop_as();
		(left, right)
	}

	#[inline]
	fn pop_triple(&mut self) -> (RuntimeValue, RuntimeValue, RuntimeValue) {
		let right = self.pop();
		let mid = self.pop();
		let left = self.pop();
		(left, mid, right)
	}

	#[inline]
	fn top(&self) -> &RuntimeValue {
		self.pick(1)
	}

	fn pick(&self, depth: usize) -> &RuntimeValue {
		&self.buf[self.sp - depth]
	}

	#[inline]
	fn pick_mut(&mut self, depth: usize) -> &mut RuntimeValue {
		&mut self.buf[self.sp - depth]
	}

	#[inline]
	fn pop(&mut self) -> RuntimeValue {
		self.sp -= 1;
		self.buf[self.sp]
	}

	#[inline]
	fn push(&mut self, value: RuntimeValue) -> Result<(), TrapKind> {
		let cell = self.buf.get_mut(self.sp).ok_or_else(|| TrapKind::StackOverflow)?;
		*cell = value;
		self.sp += 1;
		Ok(())
	}

	#[inline]
	fn len(&self) -> usize {
		self.sp
	}
}
