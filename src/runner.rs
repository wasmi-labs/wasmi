#![allow(clippy::unnecessary_wraps)]

use crate::{
    func::{FuncInstance, FuncInstanceInternal, FuncRef},
    host::Externals,
    isa::{self, DropKeep, Keep},
    memory::MemoryRef,
    memory_units::Pages,
    module::ModuleRef,
    nan_preserving_float::{F32, F64},
    tracer::{
        etable::{ETable, RunInstructionTracePre},
        Tracer,
    },
    value::{
        ArithmeticOps,
        ExtendInto,
        Float,
        Integer,
        LittleEndianConvert,
        TransmuteInto,
        TryTruncateInto,
        WrapInto,
    },
    RuntimeValue,
    Signature,
    Trap,
    TrapCode,
    ValueType,
};
use alloc::{boxed::Box, vec::Vec};
use core::{cell::RefCell, fmt, ops, u32, usize};
use parity_wasm::elements::Local;
use specs::{
    external_host_call_table::ExternalHostCallSignature,
    itable::{BinOp, BitOp, InstructionTableEntry, RelOp, ShiftOp, UnaryOp},
    jtable::JumpTableEntry,
    mtable::{MemoryReadSize, MemoryStoreSize, VarType},
    step::StepInfo,
};
use std::rc::Rc;
use validation::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX};

/// Maximum number of bytes on the value stack.
/// wasmi's default value is 1024 * 1024,
/// ZKWASM: Maximum number of entries on the value stack.
/// we set 4096 to adapt zkWasm
pub const DEFAULT_VALUE_STACK_LIMIT: usize = 4096;

/// Maximum number of levels on the call stack.
pub const DEFAULT_CALL_STACK_LIMIT: usize = 128 * 1024;

/// This is a wrapper around u64 to allow us to treat runtime values as a tag-free `u64`
/// (where if the runtime value is <64 bits the upper bits are 0). This is safe, since
/// all of the possible runtime values are valid to create from 64 defined bits, so if
/// types don't line up we get a logic error (which will ideally be caught by the wasm
/// spec tests) and not undefined behaviour.
///
/// At the boundary between the interpreter and the outside world we convert to the public
/// `Value` type, which can then be matched on. We can create a `Value` from
/// a `ValueInternal` only when the type is statically known, which it always is
/// at these boundaries.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(transparent)]
pub struct ValueInternal(pub u64);

impl ValueInternal {
    pub fn with_type(self, ty: ValueType) -> RuntimeValue {
        match ty {
            ValueType::I32 => RuntimeValue::I32(<_>::from_value_internal(self)),
            ValueType::I64 => RuntimeValue::I64(<_>::from_value_internal(self)),
            ValueType::F32 => RuntimeValue::F32(<_>::from_value_internal(self)),
            ValueType::F64 => RuntimeValue::F64(<_>::from_value_internal(self)),
        }
    }
}

trait FromValueInternal
where
    Self: Sized,
{
    fn from_value_internal(val: ValueInternal) -> Self;
}

macro_rules! impl_from_value_internal {
	($($t:ty),*) =>	{
		$(
			impl FromValueInternal for $t {
				fn from_value_internal(
					ValueInternal(val): ValueInternal,
				) -> Self {
					val	as _
				}
			}

			impl From<$t> for ValueInternal {
				fn from(other: $t) -> Self {
					ValueInternal(other as _)
				}
			}
		)*
	};
}

macro_rules! impl_from_value_internal_float	{
	($($t:ty),*) =>	{
		$(
			impl FromValueInternal for $t {
				fn from_value_internal(
					ValueInternal(val): ValueInternal,
				) -> Self {
					<$t>::from_bits(val	as _)
				}
			}

			impl From<$t> for ValueInternal {
				fn from(other: $t) -> Self {
					ValueInternal(other.to_bits() as	_)
				}
			}
		)*
	};
}

impl_from_value_internal!(i8, u8, i16, u16, i32, u32, i64, u64);
impl_from_value_internal_float!(f32, f64, F32, F64);

pub fn from_value_internal_to_u64_with_typ(vtype: VarType, val: ValueInternal) -> u64 {
    match vtype {
        VarType::I32 => val.0 as u32 as u64,
        VarType::I64 => val.0 as u64,
    }
}

impl From<bool> for ValueInternal {
    fn from(other: bool) -> Self {
        (if other { 1 } else { 0 }).into()
    }
}

impl FromValueInternal for bool {
    fn from_value_internal(ValueInternal(val): ValueInternal) -> Self {
        val != 0
    }
}

impl From<RuntimeValue> for ValueInternal {
    fn from(other: RuntimeValue) -> Self {
        match other {
            RuntimeValue::I32(val) => val.into(),
            RuntimeValue::I64(val) => val.into(),
            RuntimeValue::F32(val) => val.into(),
            RuntimeValue::F64(val) => val.into(),
        }
    }
}

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
        matches!(self, InterpreterState::Resumable(_))
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
    call_stack: CallStack,
    return_type: Option<ValueType>,
    state: InterpreterState,
    scratch: Vec<RuntimeValue>,
    pub(crate) tracer: Option<Rc<RefCell<Tracer>>>,
    mask_tracer: Vec<u32>,
}

impl Interpreter {
    pub fn new(
        func: &FuncRef,
        args: &[RuntimeValue],
        mut stack_recycler: Option<&mut StackRecycler>,
    ) -> Result<Interpreter, Trap> {
        let mut value_stack = StackRecycler::recreate_value_stack(&mut stack_recycler);
        for &arg in args {
            let arg = arg.into();
            value_stack.push(arg).map_err(
                // There is not enough space for pushing initial arguments.
                // Weird, but bail out anyway.
                |_| Trap::from(TrapCode::StackOverflow),
            )?;
        }

        let mut call_stack = StackRecycler::recreate_call_stack(&mut stack_recycler);
        let initial_frame = FunctionContext::new(func.clone());
        call_stack.push(initial_frame);

        let return_type = func.signature().return_type();

        Ok(Interpreter {
            value_stack,
            call_stack,
            return_type,
            state: InterpreterState::Initialized,
            scratch: Vec::new(),
            tracer: None,
            mask_tracer: vec![],
        })
    }

    fn get_tracer_if_active(&self) -> Option<Rc<RefCell<Tracer>>> {
        if self.tracer.is_some() && self.mask_tracer.is_empty() {
            self.tracer.clone()
        } else {
            None
        }
    }

    pub fn state(&self) -> &InterpreterState {
        &self.state
    }

    pub fn start_execution<'a, E: Externals + 'a>(
        &mut self,
        externals: &'a mut E,
    ) -> Result<Option<RuntimeValue>, Trap> {
        // Ensure that the VM has not been executed. This is checked in `FuncInvocation::start_execution`.
        assert!(self.state == InterpreterState::Initialized);

        self.state = InterpreterState::Started;
        self.run_interpreter_loop(externals)?;

        let opt_return_value = self
            .return_type
            .map(|vt| self.value_stack.pop().with_type(vt));

        // Ensure that stack is empty after the execution. This is guaranteed by the validation properties.
        assert!(self.value_stack.len() == 0);

        Ok(opt_return_value)
    }

    pub fn resume_execution<'a, E: Externals + 'a>(
        &mut self,
        return_val: Option<RuntimeValue>,
        externals: &'a mut E,
    ) -> Result<Option<RuntimeValue>, Trap> {
        use core::mem::swap;

        // Ensure that the VM is resumable. This is checked in `FuncInvocation::resume_execution`.
        assert!(self.state.is_resumable());

        let mut resumable_state = InterpreterState::Started;
        swap(&mut self.state, &mut resumable_state);

        if let Some(return_val) = return_val {
            self.value_stack
                .push(return_val.into())
                .map_err(Trap::from)?;
        }

        self.run_interpreter_loop(externals)?;

        let opt_return_value = self
            .return_type
            .map(|vt| self.value_stack.pop().with_type(vt));

        // Ensure that stack is empty after the execution. This is guaranteed by the validation properties.
        assert!(self.value_stack.len() == 0);

        Ok(opt_return_value)
    }

    fn run_interpreter_loop<'a, E: Externals + 'a>(
        &mut self,
        externals: &'a mut E,
    ) -> Result<(), Trap> {
        loop {
            let mut function_context = self.call_stack.pop().expect(
                "on loop entry - not empty; on loop continue - checking for emptiness; qed",
            );
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

            let function_return = self
                .do_run_function(&mut function_context, &function_body.code)
                .map_err(Trap::from)?;

            match function_return {
                RunResult::Return => {
                    if self.call_stack.is_empty() {
                        // This was the last frame in the call stack. This means we
                        // are done executing.
                        return Ok(());
                    }
                }
                RunResult::NestedCall(nested_func) => {
                    if self.call_stack.is_full() {
                        return Err(TrapCode::StackOverflow.into());
                    }

                    match *nested_func.as_internal() {
                        FuncInstanceInternal::Internal { .. } => {
                            let nested_context = FunctionContext::new(nested_func.clone());

                            if let Some(tracer) = self.get_tracer_if_active() {
                                let mut tracer = tracer.borrow_mut();
                                let callee_fid = tracer.lookup_function(&nested_func);

                                let eid = tracer.eid();
                                let last_jump_eid = tracer.last_jump_eid();

                                let inst = tracer.lookup_ientry(
                                    &function_context.function,
                                    function_context.position,
                                );

                                tracer.jtable.push(JumpTableEntry {
                                    eid,
                                    last_jump_eid,
                                    callee_fid,
                                    inst: Box::new(inst.into()),
                                });

                                tracer.push_frame();
                            }

                            if let Some(tracer) = self.tracer.clone() {
                                if tracer
                                    .borrow()
                                    .is_phantom_function(&nested_context.function)
                                {
                                    self.mask_tracer.push(self.value_stack.sp as u32);
                                }
                            }

                            self.call_stack.push(function_context);
                            self.call_stack.push(nested_context);
                        }
                        FuncInstanceInternal::Host { ref signature, .. } => {
                            prepare_function_args(
                                signature,
                                &mut self.value_stack,
                                &mut self.scratch,
                            );
                            // We push the function context first. If the VM is not resumable, it does no harm. If it is, we then save the context here.
                            self.call_stack.push(function_context);

                            let return_val = match FuncInstance::invoke(
                                &nested_func,
                                &self.scratch,
                                externals,
                            ) {
                                Ok(val) => val,
                                Err(trap) => {
                                    if trap.is_host() {
                                        self.state = InterpreterState::Resumable(
                                            nested_func.signature().return_type(),
                                        );
                                    }
                                    return Err(trap);
                                }
                            };

                            // Check if `return_val` matches the signature.
                            let value_ty = return_val.as_ref().map(|val| val.value_type());
                            let expected_ty = nested_func.signature().return_type();
                            if value_ty != expected_ty {
                                return Err(TrapCode::UnexpectedSignature.into());
                            }

                            if let Some(return_val) = return_val {
                                self.value_stack
                                    .push(return_val.into())
                                    .map_err(Trap::from)?;
                            }

                            if let Some(return_val) = return_val {
                                if let Some(tracer) = self.get_tracer_if_active() {
                                    let mut tracer = (*tracer).borrow_mut();

                                    let entry = tracer.etable.get_last_entry_mut().unwrap();

                                    match &entry.step_info {
                                        StepInfo::CallHost {
                                            plugin,
                                            host_function_idx,
                                            function_name,
                                            args,
                                            ret_val,
                                            signature,
                                            op_index_in_plugin,
                                        } => {
                                            assert!(ret_val.is_none());
                                            entry.step_info = StepInfo::CallHost {
                                                plugin: *plugin,
                                                host_function_idx: *host_function_idx,
                                                function_name: function_name.clone(),
                                                args: args.clone(),
                                                ret_val: Some(from_value_internal_to_u64_with_typ(
                                                    signature.return_type.unwrap().into(),
                                                    return_val.into(),
                                                )),
                                                signature: signature.clone(),
                                                op_index_in_plugin: *op_index_in_plugin,
                                            }
                                        }
                                        StepInfo::ExternalHostCall { op, sig, .. } => {
                                            if let ExternalHostCallSignature::Return = sig {
                                                entry.step_info = StepInfo::ExternalHostCall {
                                                    op: *op,
                                                    value: Some(
                                                        from_value_internal_to_u64_with_typ(
                                                            VarType::I64,
                                                            return_val.into(),
                                                        ),
                                                    ),
                                                    sig: *sig,
                                                }
                                            }
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn run_instruction_pre(
        &mut self,
        function_context: &FunctionContext,
        instructions: &isa::Instruction,
    ) -> Option<RunInstructionTracePre> {
        match *instructions {
            isa::Instruction::GetLocal(..) => None,
            isa::Instruction::SetLocal(depth, vtype) => {
                let value = self.value_stack.top();
                Some(RunInstructionTracePre::SetLocal {
                    depth,
                    value: value.clone(),
                    vtype,
                })
            }
            isa::Instruction::TeeLocal(..) => None,
            isa::Instruction::GetGlobal(..) => None,
            isa::Instruction::SetGlobal(idx) => {
                let value = self.value_stack.top();
                Some(RunInstructionTracePre::SetGlobal {
                    idx,
                    value: value.clone(),
                })
            }

            isa::Instruction::Br(_) => None,
            isa::Instruction::BrIfEqz(_) => Some(RunInstructionTracePre::BrIfEqz {
                value: <_>::from_value_internal(*self.value_stack.top()),
            }),
            isa::Instruction::BrIfNez(_) => Some(RunInstructionTracePre::BrIfNez {
                value: <_>::from_value_internal(*self.value_stack.top()),
            }),
            isa::Instruction::BrTable(_) => Some(RunInstructionTracePre::BrTable {
                index: <_>::from_value_internal(*self.value_stack.top()),
            }),

            isa::Instruction::Unreachable => None,
            isa::Instruction::Return(..) => None,

            isa::Instruction::Call(func_idx) => {
                let func = function_context
                    .module()
                    .func_by_index(func_idx)
                    .expect("Due to validation func should exists");

                let mut args = vec![];
                let len = func.signature().params().len();

                for i in 1..=len {
                    args.push(*self.value_stack.pick(i));
                }

                Some(RunInstructionTracePre::Call { args })
            }
            isa::Instruction::CallIndirect(type_idx) => {
                let table_idx = DEFAULT_TABLE_INDEX;
                let offset = <_>::from_value_internal(*self.value_stack.top());

                Some(RunInstructionTracePre::CallIndirect {
                    table_idx,
                    type_idx,
                    offset,
                })
            }

            isa::Instruction::Drop => Some(RunInstructionTracePre::Drop),
            isa::Instruction::Select(vtype) => Some(RunInstructionTracePre::Select {
                cond: from_value_internal_to_u64_with_typ(VarType::I32, *self.value_stack.pick(1)),
                val2: from_value_internal_to_u64_with_typ(vtype.into(), *self.value_stack.pick(2)),
                val1: from_value_internal_to_u64_with_typ(vtype.into(), *self.value_stack.pick(3)),
            }),

            isa::Instruction::I32Load(offset)
            | isa::Instruction::I32Load8S(offset)
            | isa::Instruction::I32Load8U(offset)
            | isa::Instruction::I32Load16S(offset)
            | isa::Instruction::I32Load16U(offset) => {
                let load_size = match *instructions {
                    isa::Instruction::I32Load(..) => MemoryReadSize::U32,
                    isa::Instruction::I32Load8S(..) => MemoryReadSize::S8,
                    isa::Instruction::I32Load8U(..) => MemoryReadSize::U8,
                    isa::Instruction::I32Load16S(..) => MemoryReadSize::S16,
                    isa::Instruction::I32Load16U(..) => MemoryReadSize::U16,
                    _ => unreachable!(),
                };

                let raw_address = <_>::from_value_internal(*self.value_stack.top());
                let address =
                    effective_address(offset, raw_address).map_or(None, |addr| Some(addr));

                Some(RunInstructionTracePre::Load {
                    offset,
                    raw_address,
                    effective_address: address,
                    vtype: parity_wasm::elements::ValueType::I32,
                    load_size,
                })
            }
            isa::Instruction::I64Load(offset)
            | isa::Instruction::I64Load8S(offset)
            | isa::Instruction::I64Load8U(offset)
            | isa::Instruction::I64Load16S(offset)
            | isa::Instruction::I64Load16U(offset)
            | isa::Instruction::I64Load32S(offset)
            | isa::Instruction::I64Load32U(offset) => {
                let load_size = match *instructions {
                    isa::Instruction::I64Load(..) => MemoryReadSize::I64,
                    isa::Instruction::I64Load8S(..) => MemoryReadSize::S8,
                    isa::Instruction::I64Load8U(..) => MemoryReadSize::U8,
                    isa::Instruction::I64Load16S(..) => MemoryReadSize::S16,
                    isa::Instruction::I64Load16U(..) => MemoryReadSize::U16,
                    isa::Instruction::I64Load32S(..) => MemoryReadSize::S32,
                    isa::Instruction::I64Load32U(..) => MemoryReadSize::U32,
                    _ => unreachable!(),
                };
                let raw_address = <_>::from_value_internal(*self.value_stack.top());
                let address =
                    effective_address(offset, raw_address).map_or(None, |addr| Some(addr));

                Some(RunInstructionTracePre::Load {
                    offset,
                    raw_address,
                    effective_address: address,
                    vtype: parity_wasm::elements::ValueType::I64,
                    load_size,
                })
            }
            isa::Instruction::I32Store(offset)
            | isa::Instruction::I32Store8(offset)
            | isa::Instruction::I32Store16(offset) => {
                let store_size = match *instructions {
                    isa::Instruction::I32Store8(_) => MemoryStoreSize::Byte8,
                    isa::Instruction::I32Store16(_) => MemoryStoreSize::Byte16,
                    isa::Instruction::I32Store(_) => MemoryStoreSize::Byte32,
                    _ => unreachable!(),
                };

                let value: u32 = <_>::from_value_internal(*self.value_stack.pick(1));
                let raw_address = <_>::from_value_internal(*self.value_stack.pick(2));
                let address =
                    effective_address(offset, raw_address).map_or(None, |addr| Some(addr));

                let pre_block_value1 = address.map(|address| {
                    let mut buf = [0u8; 8];
                    function_context
                        .memory
                        .clone()
                        .unwrap()
                        .get_into(address / 8 * 8, &mut buf)
                        .unwrap();
                    u64::from_le_bytes(buf)
                });

                let pre_block_value2 = address
                    .map(|address| {
                        if store_size.byte_size() as u32 + address % 8 > 8 {
                            let mut buf = [0u8; 8];
                            function_context
                                .memory
                                .clone()
                                .unwrap()
                                .get_into((address / 8 + 1) * 8, &mut buf)
                                .unwrap();
                            Some(u64::from_le_bytes(buf))
                        } else {
                            None
                        }
                    })
                    .flatten();

                Some(RunInstructionTracePre::Store {
                    offset,
                    raw_address,
                    effective_address: address,
                    value: value as u64,
                    vtype: parity_wasm::elements::ValueType::I32,
                    store_size,
                    pre_block_value1,
                    pre_block_value2,
                })
            }
            isa::Instruction::I64Store(offset)
            | isa::Instruction::I64Store8(offset)
            | isa::Instruction::I64Store16(offset)
            | isa::Instruction::I64Store32(offset) => {
                let store_size = match *instructions {
                    isa::Instruction::I64Store(..) => MemoryStoreSize::Byte64,
                    isa::Instruction::I64Store8(..) => MemoryStoreSize::Byte8,
                    isa::Instruction::I64Store16(..) => MemoryStoreSize::Byte16,
                    isa::Instruction::I64Store32(..) => MemoryStoreSize::Byte32,
                    _ => unreachable!(),
                };

                let value = <_>::from_value_internal(*self.value_stack.pick(1));
                let raw_address = <_>::from_value_internal(*self.value_stack.pick(2));
                let address =
                    effective_address(offset, raw_address).map_or(None, |addr| Some(addr));

                let pre_block_value1 = address.map(|address| {
                    let mut buf = [0u8; 8];
                    function_context
                        .memory
                        .clone()
                        .unwrap()
                        .get_into(address / 8 * 8, &mut buf)
                        .unwrap();
                    u64::from_le_bytes(buf)
                });

                let pre_block_value2 = address
                    .map(|address| {
                        if store_size.byte_size() as u32 + address % 8 > 8 {
                            let mut buf = [0u8; 8];
                            function_context
                                .memory
                                .clone()
                                .unwrap()
                                .get_into((address / 8 + 1) * 8, &mut buf)
                                .unwrap();
                            Some(u64::from_le_bytes(buf))
                        } else {
                            None
                        }
                    })
                    .flatten();

                Some(RunInstructionTracePre::Store {
                    offset,
                    raw_address,
                    effective_address: address,
                    value,
                    vtype: parity_wasm::elements::ValueType::I64,
                    store_size,
                    pre_block_value1,
                    pre_block_value2,
                })
            }

            isa::Instruction::CurrentMemory => None,
            isa::Instruction::GrowMemory => Some(RunInstructionTracePre::GrowMemory(
                <_>::from_value_internal(*self.value_stack.pick(1)),
            )),

            isa::Instruction::I32Const(_) => None,
            isa::Instruction::I64Const(_) => None,

            isa::Instruction::I32Eqz => Some(RunInstructionTracePre::I32Single(
                <_>::from_value_internal(*self.value_stack.pick(1)),
            )),
            isa::Instruction::I64Eqz => Some(RunInstructionTracePre::I64Single(
                <_>::from_value_internal(*self.value_stack.pick(1)),
            )),

            isa::Instruction::I32Eq
            | isa::Instruction::I32Ne
            | isa::Instruction::I32GtS
            | isa::Instruction::I32GtU
            | isa::Instruction::I32GeS
            | isa::Instruction::I32GeU
            | isa::Instruction::I32LtU
            | isa::Instruction::I32LeU
            | isa::Instruction::I32LtS
            | isa::Instruction::I32LeS => Some(RunInstructionTracePre::I32Comp {
                left: <_>::from_value_internal(*self.value_stack.pick(2)),
                right: <_>::from_value_internal(*self.value_stack.pick(1)),
            }),

            isa::Instruction::I64Eq
            | isa::Instruction::I64Ne
            | isa::Instruction::I64GtS
            | isa::Instruction::I64GtU
            | isa::Instruction::I64GeS
            | isa::Instruction::I64GeU
            | isa::Instruction::I64LtU
            | isa::Instruction::I64LeU
            | isa::Instruction::I64LtS
            | isa::Instruction::I64LeS => Some(RunInstructionTracePre::I64Comp {
                left: <_>::from_value_internal(*self.value_stack.pick(2)),
                right: <_>::from_value_internal(*self.value_stack.pick(1)),
            }),

            isa::Instruction::I32Add
            | isa::Instruction::I32Sub
            | isa::Instruction::I32Mul
            | isa::Instruction::I32DivS
            | isa::Instruction::I32DivU
            | isa::Instruction::I32RemS
            | isa::Instruction::I32RemU
            | isa::Instruction::I32Shl
            | isa::Instruction::I32ShrU
            | isa::Instruction::I32ShrS
            | isa::Instruction::I32And
            | isa::Instruction::I32Or
            | isa::Instruction::I32Xor
            | isa::Instruction::I32Rotl
            | isa::Instruction::I32Rotr => Some(RunInstructionTracePre::I32BinOp {
                left: <_>::from_value_internal(*self.value_stack.pick(2)),
                right: <_>::from_value_internal(*self.value_stack.pick(1)),
            }),

            isa::Instruction::I64Add
            | isa::Instruction::I64Sub
            | isa::Instruction::I64Mul
            | isa::Instruction::I64DivS
            | isa::Instruction::I64DivU
            | isa::Instruction::I64RemS
            | isa::Instruction::I64RemU
            | isa::Instruction::I64Shl
            | isa::Instruction::I64ShrU
            | isa::Instruction::I64ShrS
            | isa::Instruction::I64And
            | isa::Instruction::I64Or
            | isa::Instruction::I64Xor
            | isa::Instruction::I64Rotl
            | isa::Instruction::I64Rotr => Some(RunInstructionTracePre::I64BinOp {
                left: <_>::from_value_internal(*self.value_stack.pick(2)),
                right: <_>::from_value_internal(*self.value_stack.pick(1)),
            }),

            isa::Instruction::I32Ctz | isa::Instruction::I32Clz | isa::Instruction::I32Popcnt => {
                Some(RunInstructionTracePre::UnaryOp {
                    operand: from_value_internal_to_u64_with_typ(
                        VarType::I32,
                        *self.value_stack.pick(1),
                    ),
                    vtype: VarType::I32,
                })
            }
            isa::Instruction::I64Ctz | isa::Instruction::I64Clz | isa::Instruction::I64Popcnt => {
                Some(RunInstructionTracePre::UnaryOp {
                    operand: from_value_internal_to_u64_with_typ(
                        VarType::I64,
                        *self.value_stack.pick(1),
                    ),
                    vtype: VarType::I64,
                })
            }

            isa::Instruction::I32WrapI64 => Some(RunInstructionTracePre::I32WrapI64 {
                value: <_>::from_value_internal(*self.value_stack.pick(1)),
            }),
            isa::Instruction::I64ExtendUI32 => Some(RunInstructionTracePre::I64ExtendI32 {
                value: <_>::from_value_internal(*self.value_stack.pick(1)),
                sign: false,
            }),
            isa::Instruction::I64ExtendSI32 => Some(RunInstructionTracePre::I64ExtendI32 {
                value: <_>::from_value_internal(*self.value_stack.pick(1)),
                sign: true,
            }),
            isa::Instruction::I32Extend8S => Some(RunInstructionTracePre::I32SignExtendI8 {
                value: <_>::from_value_internal(*self.value_stack.pick(1)),
            }),
            isa::Instruction::I32Extend16S => Some(RunInstructionTracePre::I32SignExtendI16 {
                value: <_>::from_value_internal(*self.value_stack.pick(1)),
            }),
            isa::Instruction::I64Extend8S => Some(RunInstructionTracePre::I64SignExtendI8 {
                value: <_>::from_value_internal(*self.value_stack.pick(1)),
            }),
            isa::Instruction::I64Extend16S => Some(RunInstructionTracePre::I64SignExtendI16 {
                value: <_>::from_value_internal(*self.value_stack.pick(1)),
            }),
            isa::Instruction::I64Extend32S => Some(RunInstructionTracePre::I64SignExtendI32 {
                value: <_>::from_value_internal(*self.value_stack.pick(1)),
            }),

            _ => {
                println!("{:?}", *instructions);
                unimplemented!()
            }
        }
    }

    fn run_instruction_post(
        &mut self,
        pre_status: Option<RunInstructionTracePre>,
        context: &FunctionContext,
        instructions: &isa::Instruction,
    ) -> StepInfo {
        match *instructions {
            isa::Instruction::GetLocal(depth, vtype) => StepInfo::GetLocal {
                depth,
                value: from_value_internal_to_u64_with_typ(vtype.into(), *self.value_stack.top()),
                vtype: vtype.into(),
            },
            isa::Instruction::SetLocal(..) => {
                if let RunInstructionTracePre::SetLocal {
                    depth,
                    value,
                    vtype,
                } = pre_status.unwrap()
                {
                    StepInfo::SetLocal {
                        depth,
                        value: from_value_internal_to_u64_with_typ(vtype.into(), value),
                        vtype: vtype.into(),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::TeeLocal(depth, vtype) => StepInfo::TeeLocal {
                depth,
                value: from_value_internal_to_u64_with_typ(vtype.into(), *self.value_stack.top()),
                vtype: vtype.into(),
            },
            isa::Instruction::GetGlobal(idx) => {
                let global_ref = context.module().global_by_index(idx).unwrap();
                let is_mutable = global_ref.is_mutable();
                let vtype: VarType = global_ref.value_type().into_elements().into();
                let value = from_value_internal_to_u64_with_typ(
                    vtype.into(),
                    ValueInternal::from(global_ref.get()),
                );

                StepInfo::GetGlobal {
                    idx,
                    vtype,
                    is_mutable,
                    value,
                }
            }
            isa::Instruction::SetGlobal(idx) => {
                let global_ref = context.module().global_by_index(idx).unwrap();
                let is_mutable = global_ref.is_mutable();
                let vtype: VarType = global_ref.value_type().into_elements().into();
                let value = from_value_internal_to_u64_with_typ(
                    vtype.into(),
                    ValueInternal::from(global_ref.get()),
                );

                StepInfo::SetGlobal {
                    idx,
                    vtype,
                    is_mutable,
                    value,
                }
            }

            isa::Instruction::Br(target) => StepInfo::Br {
                dst_pc: target.dst_pc,
                drop: target.drop_keep.drop,
                keep: if let Keep::Single(t) = target.drop_keep.keep {
                    vec![t.into()]
                } else {
                    vec![]
                },
                keep_values: match target.drop_keep.keep {
                    Keep::Single(t) => vec![from_value_internal_to_u64_with_typ(
                        t.into(),
                        *self.value_stack.top(),
                    )],
                    Keep::None => vec![],
                },
            },
            isa::Instruction::BrIfEqz(target) => {
                if let RunInstructionTracePre::BrIfEqz { value } = pre_status.unwrap() {
                    StepInfo::BrIfEqz {
                        condition: value,
                        dst_pc: target.dst_pc,
                        drop: target.drop_keep.drop,
                        keep: if let Keep::Single(t) = target.drop_keep.keep {
                            vec![t.into()]
                        } else {
                            vec![]
                        },
                        keep_values: match target.drop_keep.keep {
                            Keep::Single(t) => vec![from_value_internal_to_u64_with_typ(
                                t.into(),
                                *self.value_stack.top(),
                            )],
                            Keep::None => vec![],
                        },
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::BrIfNez(target) => {
                if let RunInstructionTracePre::BrIfNez { value } = pre_status.unwrap() {
                    StepInfo::BrIfNez {
                        condition: value,
                        dst_pc: target.dst_pc,
                        drop: target.drop_keep.drop,
                        keep: if let Keep::Single(t) = target.drop_keep.keep {
                            vec![t.into()]
                        } else {
                            vec![]
                        },
                        keep_values: match target.drop_keep.keep {
                            Keep::Single(t) => vec![from_value_internal_to_u64_with_typ(
                                t.into(),
                                *self.value_stack.top(),
                            )],
                            Keep::None => vec![],
                        },
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::BrTable(targets) => {
                if let RunInstructionTracePre::BrTable { index } = pre_status.unwrap() {
                    StepInfo::BrTable {
                        index,
                        dst_pc: targets.get(index as u32).dst_pc,
                        drop: targets.get(index as u32).drop_keep.drop,
                        keep: if let Keep::Single(t) = targets.get(index as u32).drop_keep.keep {
                            vec![t.into()]
                        } else {
                            vec![]
                        },
                        keep_values: match targets.get(index as u32).drop_keep.keep {
                            Keep::Single(t) => vec![from_value_internal_to_u64_with_typ(
                                t.into(),
                                *self.value_stack.top(),
                            )],
                            Keep::None => vec![],
                        },
                    }
                } else {
                    unreachable!()
                }
            }

            isa::Instruction::Return(DropKeep { drop, keep }) => {
                let mut drop_values = vec![];

                for i in 1..=drop {
                    drop_values.push(*self.value_stack.pick(i as usize));
                }

                StepInfo::Return {
                    drop,
                    keep: if let Keep::Single(t) = keep {
                        vec![t.into()]
                    } else {
                        vec![]
                    },
                    keep_values: match keep {
                        Keep::Single(t) => vec![from_value_internal_to_u64_with_typ(
                            t.into(),
                            *self.value_stack.top(),
                        )],
                        Keep::None => vec![],
                    },
                }
            }

            isa::Instruction::Drop => {
                if let RunInstructionTracePre::Drop = pre_status.unwrap() {
                    StepInfo::Drop
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::Select(vtype) => {
                if let RunInstructionTracePre::Select { val1, val2, cond } = pre_status.unwrap() {
                    StepInfo::Select {
                        val1,
                        val2,
                        cond,
                        result: from_value_internal_to_u64_with_typ(
                            vtype.into(),
                            *self.value_stack.top(),
                        ),
                        vtype: vtype.into(),
                    }
                } else {
                    unreachable!()
                }
            }

            isa::Instruction::Call(index) => {
                if let RunInstructionTracePre::Call { args: _ } = pre_status.unwrap() {
                    let tracer = self.tracer.clone().unwrap();
                    let tracer = tracer.borrow();

                    let desc = tracer.function_index_translation.get(&index).unwrap();

                    match &desc.ftype {
                        specs::types::FunctionType::WasmFunction => StepInfo::Call {
                            index: desc.index_within_jtable,
                        },
                        specs::types::FunctionType::HostFunction {
                            plugin,
                            function_index: host_function_idx,
                            function_name,
                            op_index_in_plugin,
                        } => {
                            let params_len = desc.signature.params().len();
                            let mut args: Vec<u64> = vec![];
                            let signature: specs::host_function::Signature =
                                desc.signature.clone().into();
                            let params = signature.params.clone();

                            for i in 0..params_len {
                                args.push(from_value_internal_to_u64_with_typ(
                                    (params[i]).into(),
                                    *self.value_stack.pick(params_len - i),
                                ));
                            }
                            StepInfo::CallHost {
                                plugin: *plugin,
                                host_function_idx: *host_function_idx,
                                function_name: function_name.clone(),
                                args,
                                ret_val: None,
                                signature,
                                op_index_in_plugin: *op_index_in_plugin,
                            }
                        }
                        specs::types::FunctionType::HostFunctionExternal { op, sig, .. } => {
                            StepInfo::ExternalHostCall {
                                op: *op,
                                value: match sig {
                                    ExternalHostCallSignature::Argument => {
                                        Some(from_value_internal_to_u64_with_typ(
                                            VarType::I64,
                                            *self.value_stack.top(),
                                        ))
                                    }
                                    ExternalHostCallSignature::Return => None,
                                },
                                sig: *sig,
                            }
                        }
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::CallIndirect(_) => {
                if let RunInstructionTracePre::CallIndirect {
                    table_idx,
                    type_idx,
                    offset,
                } = pre_status.unwrap()
                {
                    let tracer = self.tracer.clone().unwrap();

                    let table = context
                        .module()
                        .table_by_index(DEFAULT_TABLE_INDEX)
                        .unwrap();
                    let func_ref = table.get(offset).unwrap().unwrap();

                    let func_idx = tracer.borrow().lookup_function(&func_ref);

                    StepInfo::CallIndirect {
                        table_index: table_idx,
                        type_index: type_idx,
                        offset,
                        func_index: func_idx,
                    }
                } else {
                    unreachable!()
                }
            }

            isa::Instruction::I32Load(..)
            | isa::Instruction::I32Load8U(..)
            | isa::Instruction::I32Load8S(..)
            | isa::Instruction::I32Load16U(..)
            | isa::Instruction::I32Load16S(..)
            | isa::Instruction::I64Load(..)
            | isa::Instruction::I64Load8U(..)
            | isa::Instruction::I64Load8S(..)
            | isa::Instruction::I64Load16U(..)
            | isa::Instruction::I64Load16S(..)
            | isa::Instruction::I64Load32U(..)
            | isa::Instruction::I64Load32S(..) => {
                if let RunInstructionTracePre::Load {
                    offset,
                    raw_address,
                    effective_address,
                    vtype,
                    load_size,
                } = pre_status.unwrap()
                {
                    let block_value1 = {
                        let mut buf = [0u8; 8];
                        context
                            .memory
                            .clone()
                            .unwrap()
                            .get_into(effective_address.unwrap() / 8 * 8, &mut buf)
                            .unwrap();
                        u64::from_le_bytes(buf)
                    };

                    let block_value2 =
                        if effective_address.unwrap() % 8 + load_size.byte_size() as u32 > 8 {
                            let mut buf = [0u8; 8];
                            context
                                .memory
                                .clone()
                                .unwrap()
                                .get_into((effective_address.unwrap() / 8 + 1) * 8, &mut buf)
                                .unwrap();
                            u64::from_le_bytes(buf)
                        } else {
                            0
                        };

                    StepInfo::Load {
                        vtype: vtype.into(),
                        load_size,
                        offset,
                        raw_address,
                        effective_address: effective_address.unwrap(),
                        value: from_value_internal_to_u64_with_typ(
                            vtype.into(),
                            *self.value_stack.top(),
                        ),
                        block_value1,
                        block_value2,
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Store(..)
            | isa::Instruction::I32Store8(..)
            | isa::Instruction::I32Store16(..)
            | isa::Instruction::I64Store(..)
            | isa::Instruction::I64Store8(..)
            | isa::Instruction::I64Store16(..)
            | isa::Instruction::I64Store32(..) => {
                if let RunInstructionTracePre::Store {
                    offset,
                    raw_address,
                    effective_address,
                    value,
                    vtype,
                    store_size,
                    pre_block_value1,
                    pre_block_value2,
                } = pre_status.unwrap()
                {
                    let updated_block_value1 = {
                        let mut buf = [0u8; 8];
                        context
                            .memory
                            .clone()
                            .unwrap()
                            .get_into(effective_address.unwrap() / 8 * 8, &mut buf)
                            .unwrap();
                        u64::from_le_bytes(buf)
                    };

                    let updated_block_value2 =
                        if effective_address.unwrap() % 8 + store_size.byte_size() as u32 > 8 {
                            let mut buf = [0u8; 8];
                            context
                                .memory
                                .clone()
                                .unwrap()
                                .get_into((effective_address.unwrap() / 8 + 1) * 8, &mut buf)
                                .unwrap();
                            u64::from_le_bytes(buf)
                        } else {
                            0
                        };

                    StepInfo::Store {
                        vtype: vtype.into(),
                        store_size,
                        offset,
                        raw_address,
                        effective_address: effective_address.unwrap(),
                        value: value as u64,
                        pre_block_value1: pre_block_value1.unwrap(),
                        pre_block_value2: pre_block_value2.unwrap_or(0u64),
                        updated_block_value1,
                        updated_block_value2,
                    }
                } else {
                    unreachable!()
                }
            }

            isa::Instruction::CurrentMemory => StepInfo::MemorySize,
            isa::Instruction::GrowMemory => {
                if let RunInstructionTracePre::GrowMemory(grow_size) = pre_status.unwrap() {
                    StepInfo::MemoryGrow {
                        grow_size,
                        result: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }

            isa::Instruction::I32Const(value) => StepInfo::I32Const { value },
            isa::Instruction::I64Const(value) => StepInfo::I64Const { value },

            isa::Instruction::I32Eqz => {
                if let RunInstructionTracePre::I32Single(value) = pre_status.unwrap() {
                    StepInfo::Test {
                        vtype: VarType::I32,
                        value: value as u32 as u64,
                        result: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Eq => {
                if let RunInstructionTracePre::I32Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I32Comp {
                        class: RelOp::Eq,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Ne => {
                if let RunInstructionTracePre::I32Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I32Comp {
                        class: RelOp::Ne,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32GtS => {
                if let RunInstructionTracePre::I32Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I32Comp {
                        class: RelOp::SignedGt,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32GtU => {
                if let RunInstructionTracePre::I32Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I32Comp {
                        class: RelOp::UnsignedGt,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32GeS => {
                if let RunInstructionTracePre::I32Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I32Comp {
                        class: RelOp::SignedGe,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32GeU => {
                if let RunInstructionTracePre::I32Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I32Comp {
                        class: RelOp::UnsignedGe,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32LtS => {
                if let RunInstructionTracePre::I32Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I32Comp {
                        class: RelOp::SignedLt,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32LtU => {
                if let RunInstructionTracePre::I32Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I32Comp {
                        class: RelOp::UnsignedLt,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32LeS => {
                if let RunInstructionTracePre::I32Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I32Comp {
                        class: RelOp::SignedLe,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32LeU => {
                if let RunInstructionTracePre::I32Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I32Comp {
                        class: RelOp::UnsignedLe,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }

            isa::Instruction::I64Eqz => {
                if let RunInstructionTracePre::I64Single(value) = pre_status.unwrap() {
                    StepInfo::Test {
                        vtype: VarType::I64,
                        value: value as u64,
                        result: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Eq => {
                if let RunInstructionTracePre::I64Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I64Comp {
                        class: RelOp::Eq,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Ne => {
                if let RunInstructionTracePre::I64Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I64Comp {
                        class: RelOp::Ne,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64GtS => {
                if let RunInstructionTracePre::I64Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I64Comp {
                        class: RelOp::SignedGt,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64GtU => {
                if let RunInstructionTracePre::I64Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I64Comp {
                        class: RelOp::UnsignedGt,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64LtU => {
                if let RunInstructionTracePre::I64Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I64Comp {
                        class: RelOp::UnsignedLt,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64LtS => {
                if let RunInstructionTracePre::I64Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I64Comp {
                        class: RelOp::SignedLt,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64LeU => {
                if let RunInstructionTracePre::I64Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I64Comp {
                        class: RelOp::UnsignedLe,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64LeS => {
                if let RunInstructionTracePre::I64Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I64Comp {
                        class: RelOp::SignedLe,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64GeU => {
                if let RunInstructionTracePre::I64Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I64Comp {
                        class: RelOp::UnsignedGe,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64GeS => {
                if let RunInstructionTracePre::I64Comp { left, right } = pre_status.unwrap() {
                    StepInfo::I64Comp {
                        class: RelOp::SignedGe,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }

            isa::Instruction::I32Add => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinOp {
                        class: BinOp::Add,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Sub => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinOp {
                        class: BinOp::Sub,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Mul => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinOp {
                        class: BinOp::Mul,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32DivU => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinOp {
                        class: BinOp::UnsignedDiv,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32RemU => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinOp {
                        class: BinOp::UnsignedRem,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32DivS => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinOp {
                        class: BinOp::SignedDiv,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32RemS => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinOp {
                        class: BinOp::SignedRem,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32And => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinBitOp {
                        class: BitOp::And,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Or => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinBitOp {
                        class: BitOp::Or,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Xor => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinBitOp {
                        class: BitOp::Xor,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Shl => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinShiftOp {
                        class: ShiftOp::Shl,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32ShrU => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinShiftOp {
                        class: ShiftOp::UnsignedShr,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32ShrS => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinShiftOp {
                        class: ShiftOp::SignedShr,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Rotl => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinShiftOp {
                        class: ShiftOp::Rotl,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Rotr => {
                if let RunInstructionTracePre::I32BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I32BinShiftOp {
                        class: ShiftOp::Rotr,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Add => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinOp {
                        class: BinOp::Add,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Sub => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinOp {
                        class: BinOp::Sub,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Mul => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinOp {
                        class: BinOp::Mul,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64DivU => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinOp {
                        class: BinOp::UnsignedDiv,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64RemU => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinOp {
                        class: BinOp::UnsignedRem,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64DivS => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinOp {
                        class: BinOp::SignedDiv,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64RemS => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinOp {
                        class: BinOp::SignedRem,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64And => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinBitOp {
                        class: BitOp::And,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Or => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinBitOp {
                        class: BitOp::Or,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Xor => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinBitOp {
                        class: BitOp::Xor,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Shl => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinShiftOp {
                        class: ShiftOp::Shl,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64ShrU => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinShiftOp {
                        class: ShiftOp::UnsignedShr,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64ShrS => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinShiftOp {
                        class: ShiftOp::SignedShr,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Rotl => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinShiftOp {
                        class: ShiftOp::Rotl,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Rotr => {
                if let RunInstructionTracePre::I64BinOp { left, right } = pre_status.unwrap() {
                    StepInfo::I64BinShiftOp {
                        class: ShiftOp::Rotr,
                        left,
                        right,
                        value: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }

            isa::Instruction::I32Ctz
            | isa::Instruction::I32Clz
            | isa::Instruction::I32Popcnt
            | isa::Instruction::I64Ctz
            | isa::Instruction::I64Clz
            | isa::Instruction::I64Popcnt => {
                if let RunInstructionTracePre::UnaryOp { operand, vtype } = pre_status.unwrap() {
                    StepInfo::UnaryOp {
                        class: UnaryOp::from(instructions.clone()),
                        vtype,
                        operand,
                        result: from_value_internal_to_u64_with_typ(vtype, *self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }

            isa::Instruction::I32WrapI64 => {
                if let RunInstructionTracePre::I32WrapI64 { value } = pre_status.unwrap() {
                    StepInfo::I32WrapI64 {
                        value,
                        result: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64ExtendSI32 | isa::Instruction::I64ExtendUI32 => {
                if let RunInstructionTracePre::I64ExtendI32 { value, sign } = pre_status.unwrap() {
                    StepInfo::I64ExtendI32 {
                        value,
                        result: <_>::from_value_internal(*self.value_stack.top()),
                        sign,
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Extend8S => {
                if let RunInstructionTracePre::I32SignExtendI8 { value } = pre_status.unwrap() {
                    StepInfo::I32SignExtendI8 {
                        value,
                        result: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I32Extend16S => {
                if let RunInstructionTracePre::I32SignExtendI16 { value } = pre_status.unwrap() {
                    StepInfo::I32SignExtendI16 {
                        value,
                        result: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Extend8S => {
                if let RunInstructionTracePre::I64SignExtendI8 { value } = pre_status.unwrap() {
                    StepInfo::I64SignExtendI8 {
                        value,
                        result: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Extend16S => {
                if let RunInstructionTracePre::I64SignExtendI16 { value } = pre_status.unwrap() {
                    StepInfo::I64SignExtendI16 {
                        value,
                        result: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }
            isa::Instruction::I64Extend32S => {
                if let RunInstructionTracePre::I64SignExtendI32 { value } = pre_status.unwrap() {
                    StepInfo::I64SignExtendI32 {
                        value,
                        result: <_>::from_value_internal(*self.value_stack.top()),
                    }
                } else {
                    unreachable!()
                }
            }

            _ => {
                println!("{:?}", instructions);
                unimplemented!()
            }
        }
    }

    fn do_run_function(
        &mut self,
        function_context: &mut FunctionContext,
        instructions: &isa::Instructions,
    ) -> Result<RunResult, TrapCode> {
        let mut iter = instructions.iterate_from(function_context.position);
        loop {
            let pc = iter.position();
            let sp = self.value_stack.sp;

            let instruction = iter.next().expect(
                "Ran out of instructions, this should be impossible \
                 since validation ensures that we either have an explicit \
                 return or an implicit block `end`.",
            );

            let pre_status = self.get_tracer_if_active().map_or(None, |_| {
                self.run_instruction_pre(function_context, &instruction)
            });

            let current_memory = {
                function_context
                    .memory()
                    .map_or(0usize, |m| m.current_size().0)
            };

            macro_rules! trace_post {
                () => {{
                    if let Some(tracer) = self.get_tracer_if_active() {
                        let post_status =
                            self.run_instruction_post(pre_status, function_context, &instruction);

                        let mut tracer = tracer.borrow_mut();

                        let instruction = { instruction.into(&tracer.function_index_translation) };

                        let function = tracer.lookup_function(&function_context.function);

                        let last_jump_eid = tracer.last_jump_eid();

                        let inst_entry = InstructionTableEntry {
                            fid: function,
                            iid: pc,
                            opcode: instruction,
                        };

                        tracer.etable.push(
                            inst_entry,
                            sp.try_into().unwrap(),
                            current_memory.try_into().unwrap(),
                            last_jump_eid,
                            post_status,
                        );
                    }
                }};
            }

            match self.run_instruction(function_context, &instruction)? {
                InstructionOutcome::RunNextInstruction => {
                    trace_post!();
                }
                InstructionOutcome::Branch(target) => {
                    trace_post!();
                    iter = instructions.iterate_from(target.dst_pc);
                    self.value_stack.drop_keep(target.drop_keep);
                }
                InstructionOutcome::ExecuteCall(func_ref) => {
                    // We don't record updated pc, the value should be recorded in the next trace log.
                    trace_post!();

                    function_context.position = iter.position();
                    return Ok(RunResult::NestedCall(func_ref));
                }
                InstructionOutcome::Return(drop_keep) => {
                    trace_post!();

                    if let Some(tracer) = self.tracer.clone() {
                        if tracer
                            .borrow()
                            .is_phantom_function(&function_context.function)
                        {
                            let sp_before = self.mask_tracer.pop().unwrap();

                            if self.mask_tracer.is_empty() {
                                let last_jump_eid = tracer.borrow().last_jump_eid();
                                let callee_fid =
                                    tracer.borrow().lookup_function(&function_context.function);
                                let wasm_input_function_idx =
                                    tracer.borrow().wasm_input_func_idx.unwrap();

                                tracer.borrow_mut().fill_trace(
                                    sp_before,
                                    current_memory as u32,
                                    last_jump_eid,
                                    callee_fid,
                                    function_context.function.signature(),
                                    wasm_input_function_idx,
                                    if let isa::Keep::Single(t) = drop_keep.keep {
                                        Some(from_value_internal_to_u64_with_typ(
                                            t.into(),
                                            *self.value_stack.top(),
                                        ))
                                    } else {
                                        None
                                    },
                                );
                            }
                        }
                    }

                    if let Some(tracer) = self.get_tracer_if_active() {
                        tracer.borrow_mut().pop_frame();
                    }

                    self.value_stack.drop_keep(drop_keep);
                    break;
                }
            }
        }

        Ok(RunResult::Return)
    }

    #[inline(always)]
    fn run_instruction(
        &mut self,
        context: &mut FunctionContext,
        instruction: &isa::Instruction,
    ) -> Result<InstructionOutcome, TrapCode> {
        match instruction {
            isa::Instruction::Unreachable => self.run_unreachable(context),

            isa::Instruction::Br(target) => self.run_br(context, *target),
            isa::Instruction::BrIfEqz(target) => self.run_br_eqz(*target),
            isa::Instruction::BrIfNez(target) => self.run_br_nez(*target),
            isa::Instruction::BrTable(targets) => self.run_br_table(*targets),
            isa::Instruction::Return(drop_keep, ..) => self.run_return(*drop_keep),

            isa::Instruction::Call(index) => self.run_call(context, *index),
            isa::Instruction::CallIndirect(index) => self.run_call_indirect(context, *index),

            isa::Instruction::Drop => self.run_drop(),
            isa::Instruction::Select(_) => self.run_select(),

            isa::Instruction::GetLocal(depth, ..) => self.run_get_local(*depth),
            isa::Instruction::SetLocal(depth, ..) => self.run_set_local(*depth),
            isa::Instruction::TeeLocal(depth, ..) => self.run_tee_local(*depth),
            isa::Instruction::GetGlobal(index) => self.run_get_global(context, *index),
            isa::Instruction::SetGlobal(index) => self.run_set_global(context, *index),

            isa::Instruction::I32Load(offset) => self.run_load::<i32>(context, *offset),
            isa::Instruction::I64Load(offset) => self.run_load::<i64>(context, *offset),
            isa::Instruction::F32Load(offset) => self.run_load::<F32>(context, *offset),
            isa::Instruction::F64Load(offset) => self.run_load::<F64>(context, *offset),
            isa::Instruction::I32Load8S(offset) => {
                self.run_load_extend::<i8, i32>(context, *offset)
            }
            isa::Instruction::I32Load8U(offset) => {
                self.run_load_extend::<u8, i32>(context, *offset)
            }
            isa::Instruction::I32Load16S(offset) => {
                self.run_load_extend::<i16, i32>(context, *offset)
            }
            isa::Instruction::I32Load16U(offset) => {
                self.run_load_extend::<u16, i32>(context, *offset)
            }
            isa::Instruction::I64Load8S(offset) => {
                self.run_load_extend::<i8, i64>(context, *offset)
            }
            isa::Instruction::I64Load8U(offset) => {
                self.run_load_extend::<u8, i64>(context, *offset)
            }
            isa::Instruction::I64Load16S(offset) => {
                self.run_load_extend::<i16, i64>(context, *offset)
            }
            isa::Instruction::I64Load16U(offset) => {
                self.run_load_extend::<u16, i64>(context, *offset)
            }
            isa::Instruction::I64Load32S(offset) => {
                self.run_load_extend::<i32, i64>(context, *offset)
            }
            isa::Instruction::I64Load32U(offset) => {
                self.run_load_extend::<u32, i64>(context, *offset)
            }

            isa::Instruction::I32Store(offset) => self.run_store::<i32>(context, *offset),
            isa::Instruction::I64Store(offset) => self.run_store::<i64>(context, *offset),
            isa::Instruction::F32Store(offset) => self.run_store::<F32>(context, *offset),
            isa::Instruction::F64Store(offset) => self.run_store::<F64>(context, *offset),
            isa::Instruction::I32Store8(offset) => self.run_store_wrap::<i32, i8>(context, *offset),
            isa::Instruction::I32Store16(offset) => {
                self.run_store_wrap::<i32, i16>(context, *offset)
            }
            isa::Instruction::I64Store8(offset) => self.run_store_wrap::<i64, i8>(context, *offset),
            isa::Instruction::I64Store16(offset) => {
                self.run_store_wrap::<i64, i16>(context, *offset)
            }
            isa::Instruction::I64Store32(offset) => {
                self.run_store_wrap::<i64, i32>(context, *offset)
            }

            isa::Instruction::CurrentMemory => self.run_current_memory(context),
            isa::Instruction::GrowMemory => self.run_grow_memory(context),

            isa::Instruction::I32Const(val) => self.run_const((*val).into()),
            isa::Instruction::I64Const(val) => self.run_const((*val).into()),
            isa::Instruction::F32Const(val) => self.run_const((*val).into()),
            isa::Instruction::F64Const(val) => self.run_const((*val).into()),

            isa::Instruction::I32Eqz => self.run_eqz::<i32>(),
            isa::Instruction::I32Eq => self.run_eq::<i32>(),
            isa::Instruction::I32Ne => self.run_ne::<i32>(),
            isa::Instruction::I32LtS => self.run_lt::<i32>(),
            isa::Instruction::I32LtU => self.run_lt::<u32>(),
            isa::Instruction::I32GtS => self.run_gt::<i32>(),
            isa::Instruction::I32GtU => self.run_gt::<u32>(),
            isa::Instruction::I32LeS => self.run_lte::<i32>(),
            isa::Instruction::I32LeU => self.run_lte::<u32>(),
            isa::Instruction::I32GeS => self.run_gte::<i32>(),
            isa::Instruction::I32GeU => self.run_gte::<u32>(),

            isa::Instruction::I64Eqz => self.run_eqz::<i64>(),
            isa::Instruction::I64Eq => self.run_eq::<i64>(),
            isa::Instruction::I64Ne => self.run_ne::<i64>(),
            isa::Instruction::I64LtS => self.run_lt::<i64>(),
            isa::Instruction::I64LtU => self.run_lt::<u64>(),
            isa::Instruction::I64GtS => self.run_gt::<i64>(),
            isa::Instruction::I64GtU => self.run_gt::<u64>(),
            isa::Instruction::I64LeS => self.run_lte::<i64>(),
            isa::Instruction::I64LeU => self.run_lte::<u64>(),
            isa::Instruction::I64GeS => self.run_gte::<i64>(),
            isa::Instruction::I64GeU => self.run_gte::<u64>(),

            isa::Instruction::F32Eq => self.run_eq::<F32>(),
            isa::Instruction::F32Ne => self.run_ne::<F32>(),
            isa::Instruction::F32Lt => self.run_lt::<F32>(),
            isa::Instruction::F32Gt => self.run_gt::<F32>(),
            isa::Instruction::F32Le => self.run_lte::<F32>(),
            isa::Instruction::F32Ge => self.run_gte::<F32>(),

            isa::Instruction::F64Eq => self.run_eq::<F64>(),
            isa::Instruction::F64Ne => self.run_ne::<F64>(),
            isa::Instruction::F64Lt => self.run_lt::<F64>(),
            isa::Instruction::F64Gt => self.run_gt::<F64>(),
            isa::Instruction::F64Le => self.run_lte::<F64>(),
            isa::Instruction::F64Ge => self.run_gte::<F64>(),

            isa::Instruction::I32Clz => self.run_clz::<i32>(),
            isa::Instruction::I32Ctz => self.run_ctz::<i32>(),
            isa::Instruction::I32Popcnt => self.run_popcnt::<i32>(),
            isa::Instruction::I32Add => self.run_add::<i32>(),
            isa::Instruction::I32Sub => self.run_sub::<i32>(),
            isa::Instruction::I32Mul => self.run_mul::<i32>(),
            isa::Instruction::I32DivS => self.run_div::<i32, i32>(),
            isa::Instruction::I32DivU => self.run_div::<i32, u32>(),
            isa::Instruction::I32RemS => self.run_rem::<i32, i32>(),
            isa::Instruction::I32RemU => self.run_rem::<i32, u32>(),
            isa::Instruction::I32And => self.run_and::<i32>(),
            isa::Instruction::I32Or => self.run_or::<i32>(),
            isa::Instruction::I32Xor => self.run_xor::<i32>(),
            isa::Instruction::I32Shl => self.run_shl::<i32>(0x1F),
            isa::Instruction::I32ShrS => self.run_shr::<i32, i32>(0x1F),
            isa::Instruction::I32ShrU => self.run_shr::<i32, u32>(0x1F),
            isa::Instruction::I32Rotl => self.run_rotl::<i32>(),
            isa::Instruction::I32Rotr => self.run_rotr::<i32>(),

            isa::Instruction::I64Clz => self.run_clz::<i64>(),
            isa::Instruction::I64Ctz => self.run_ctz::<i64>(),
            isa::Instruction::I64Popcnt => self.run_popcnt::<i64>(),
            isa::Instruction::I64Add => self.run_add::<i64>(),
            isa::Instruction::I64Sub => self.run_sub::<i64>(),
            isa::Instruction::I64Mul => self.run_mul::<i64>(),
            isa::Instruction::I64DivS => self.run_div::<i64, i64>(),
            isa::Instruction::I64DivU => self.run_div::<i64, u64>(),
            isa::Instruction::I64RemS => self.run_rem::<i64, i64>(),
            isa::Instruction::I64RemU => self.run_rem::<i64, u64>(),
            isa::Instruction::I64And => self.run_and::<i64>(),
            isa::Instruction::I64Or => self.run_or::<i64>(),
            isa::Instruction::I64Xor => self.run_xor::<i64>(),
            isa::Instruction::I64Shl => self.run_shl::<i64>(0x3F),
            isa::Instruction::I64ShrS => self.run_shr::<i64, i64>(0x3F),
            isa::Instruction::I64ShrU => self.run_shr::<i64, u64>(0x3F),
            isa::Instruction::I64Rotl => self.run_rotl::<i64>(),
            isa::Instruction::I64Rotr => self.run_rotr::<i64>(),

            isa::Instruction::F32Abs => self.run_abs::<F32>(),
            isa::Instruction::F32Neg => self.run_neg::<F32>(),
            isa::Instruction::F32Ceil => self.run_ceil::<F32>(),
            isa::Instruction::F32Floor => self.run_floor::<F32>(),
            isa::Instruction::F32Trunc => self.run_trunc::<F32>(),
            isa::Instruction::F32Nearest => self.run_nearest::<F32>(),
            isa::Instruction::F32Sqrt => self.run_sqrt::<F32>(),
            isa::Instruction::F32Add => self.run_add::<F32>(),
            isa::Instruction::F32Sub => self.run_sub::<F32>(),
            isa::Instruction::F32Mul => self.run_mul::<F32>(),
            isa::Instruction::F32Div => self.run_div::<F32, F32>(),
            isa::Instruction::F32Min => self.run_min::<F32>(),
            isa::Instruction::F32Max => self.run_max::<F32>(),
            isa::Instruction::F32Copysign => self.run_copysign::<F32>(),

            isa::Instruction::F64Abs => self.run_abs::<F64>(),
            isa::Instruction::F64Neg => self.run_neg::<F64>(),
            isa::Instruction::F64Ceil => self.run_ceil::<F64>(),
            isa::Instruction::F64Floor => self.run_floor::<F64>(),
            isa::Instruction::F64Trunc => self.run_trunc::<F64>(),
            isa::Instruction::F64Nearest => self.run_nearest::<F64>(),
            isa::Instruction::F64Sqrt => self.run_sqrt::<F64>(),
            isa::Instruction::F64Add => self.run_add::<F64>(),
            isa::Instruction::F64Sub => self.run_sub::<F64>(),
            isa::Instruction::F64Mul => self.run_mul::<F64>(),
            isa::Instruction::F64Div => self.run_div::<F64, F64>(),
            isa::Instruction::F64Min => self.run_min::<F64>(),
            isa::Instruction::F64Max => self.run_max::<F64>(),
            isa::Instruction::F64Copysign => self.run_copysign::<F64>(),

            isa::Instruction::I32WrapI64 => self.run_wrap::<i64, i32>(),
            isa::Instruction::I32TruncSF32 => self.run_trunc_to_int::<F32, i32, i32>(),
            isa::Instruction::I32TruncUF32 => self.run_trunc_to_int::<F32, u32, i32>(),
            isa::Instruction::I32TruncSF64 => self.run_trunc_to_int::<F64, i32, i32>(),
            isa::Instruction::I32TruncUF64 => self.run_trunc_to_int::<F64, u32, i32>(),
            isa::Instruction::I64ExtendSI32 => self.run_extend::<i32, i64, i64>(),
            isa::Instruction::I64ExtendUI32 => self.run_extend::<u32, u64, i64>(),
            isa::Instruction::I64TruncSF32 => self.run_trunc_to_int::<F32, i64, i64>(),
            isa::Instruction::I64TruncUF32 => self.run_trunc_to_int::<F32, u64, i64>(),
            isa::Instruction::I64TruncSF64 => self.run_trunc_to_int::<F64, i64, i64>(),
            isa::Instruction::I64TruncUF64 => self.run_trunc_to_int::<F64, u64, i64>(),
            isa::Instruction::F32ConvertSI32 => self.run_extend::<i32, F32, F32>(),
            isa::Instruction::F32ConvertUI32 => self.run_extend::<u32, F32, F32>(),
            isa::Instruction::F32ConvertSI64 => self.run_wrap::<i64, F32>(),
            isa::Instruction::F32ConvertUI64 => self.run_wrap::<u64, F32>(),
            isa::Instruction::F32DemoteF64 => self.run_wrap::<F64, F32>(),
            isa::Instruction::F64ConvertSI32 => self.run_extend::<i32, F64, F64>(),
            isa::Instruction::F64ConvertUI32 => self.run_extend::<u32, F64, F64>(),
            isa::Instruction::F64ConvertSI64 => self.run_extend::<i64, F64, F64>(),
            isa::Instruction::F64ConvertUI64 => self.run_extend::<u64, F64, F64>(),
            isa::Instruction::F64PromoteF32 => self.run_extend::<F32, F64, F64>(),

            isa::Instruction::I32ReinterpretF32 => self.run_reinterpret::<F32, i32>(),
            isa::Instruction::I64ReinterpretF64 => self.run_reinterpret::<F64, i64>(),
            isa::Instruction::F32ReinterpretI32 => self.run_reinterpret::<i32, F32>(),
            isa::Instruction::F64ReinterpretI64 => self.run_reinterpret::<i64, F64>(),

            isa::Instruction::I32Extend8S => self.run_extend::<i8, i32, i32>(),
            isa::Instruction::I32Extend16S => self.run_extend::<i16, i32, i32>(),
            isa::Instruction::I64Extend8S => self.run_extend::<i8, i64, i64>(),
            isa::Instruction::I64Extend16S => self.run_extend::<i16, i64, i64>(),
            isa::Instruction::I64Extend32S => self.run_extend::<i32, i64, i64>(),
        }
    }

    fn run_unreachable(
        &mut self,
        _context: &mut FunctionContext,
    ) -> Result<InstructionOutcome, TrapCode> {
        Err(TrapCode::Unreachable)
    }

    fn run_br(
        &mut self,
        _context: &mut FunctionContext,
        target: isa::Target,
    ) -> Result<InstructionOutcome, TrapCode> {
        Ok(InstructionOutcome::Branch(target))
    }

    fn run_br_nez(&mut self, target: isa::Target) -> Result<InstructionOutcome, TrapCode> {
        let condition = self.value_stack.pop_as();
        if condition {
            Ok(InstructionOutcome::Branch(target))
        } else {
            Ok(InstructionOutcome::RunNextInstruction)
        }
    }

    fn run_br_eqz(&mut self, target: isa::Target) -> Result<InstructionOutcome, TrapCode> {
        let condition = self.value_stack.pop_as();
        if condition {
            Ok(InstructionOutcome::RunNextInstruction)
        } else {
            Ok(InstructionOutcome::Branch(target))
        }
    }

    fn run_br_table(&mut self, targets: isa::BrTargets) -> Result<InstructionOutcome, TrapCode> {
        let index: u32 = self.value_stack.pop_as();

        let dst = targets.get(index);

        Ok(InstructionOutcome::Branch(dst))
    }

    fn run_return(&mut self, drop_keep: isa::DropKeep) -> Result<InstructionOutcome, TrapCode> {
        Ok(InstructionOutcome::Return(drop_keep))
    }

    fn run_call(
        &mut self,
        context: &mut FunctionContext,
        func_idx: u32,
    ) -> Result<InstructionOutcome, TrapCode> {
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
    ) -> Result<InstructionOutcome, TrapCode> {
        let table_func_idx: u32 = self.value_stack.pop_as();
        let table = context
            .module()
            .table_by_index(DEFAULT_TABLE_INDEX)
            .expect("Due to validation table should exists");
        let func_ref = table
            .get(table_func_idx)
            .map_err(|_| TrapCode::TableAccessOutOfBounds)?
            .ok_or(TrapCode::ElemUninitialized)?;

        {
            let actual_function_type = func_ref.signature();
            let required_function_type = context
                .module()
                .signature_by_index(signature_idx)
                .expect("Due to validation type should exists");

            if &*required_function_type != actual_function_type {
                return Err(TrapCode::UnexpectedSignature);
            }
        }

        Ok(InstructionOutcome::ExecuteCall(func_ref))
    }

    fn run_drop(&mut self) -> Result<InstructionOutcome, TrapCode> {
        let _ = self.value_stack.pop();
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_select(&mut self) -> Result<InstructionOutcome, TrapCode> {
        let (left, mid, right) = self.value_stack.pop_triple();

        let condition = <_>::from_value_internal(right);
        let val = if condition { left } else { mid };
        self.value_stack.push(val)?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_get_local(&mut self, index: u32) -> Result<InstructionOutcome, TrapCode> {
        let val = *self.value_stack.pick_mut(index as usize);
        self.value_stack.push(val)?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_set_local(&mut self, index: u32) -> Result<InstructionOutcome, TrapCode> {
        let val = self.value_stack.pop();
        *self.value_stack.pick_mut(index as usize) = val;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_tee_local(&mut self, index: u32) -> Result<InstructionOutcome, TrapCode> {
        let val = *self.value_stack.top();
        *self.value_stack.pick_mut(index as usize) = val;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_get_global(
        &mut self,
        context: &mut FunctionContext,
        index: u32,
    ) -> Result<InstructionOutcome, TrapCode> {
        let global = context
            .module()
            .global_by_index(index)
            .expect("Due to validation global should exists");
        let val = global.get();
        self.value_stack.push(val.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_set_global(
        &mut self,
        context: &mut FunctionContext,
        index: u32,
    ) -> Result<InstructionOutcome, TrapCode> {
        let val = self.value_stack.pop();
        let global = context
            .module()
            .global_by_index(index)
            .expect("Due to validation global should exists");
        global
            .set(val.with_type(global.value_type()))
            .expect("Due to validation set to a global should succeed");
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_load<T>(
        &mut self,
        context: &mut FunctionContext,
        offset: u32,
    ) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: LittleEndianConvert,
    {
        let raw_address = self.value_stack.pop_as();
        let address = effective_address(offset, raw_address)?;
        let m = context
            .memory()
            .expect("Due to validation memory should exists");
        let n: T = m
            .get_value(address)
            .map_err(|_| TrapCode::MemoryAccessOutOfBounds)?;
        self.value_stack.push(n.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_load_extend<T, U>(
        &mut self,
        context: &mut FunctionContext,
        offset: u32,
    ) -> Result<InstructionOutcome, TrapCode>
    where
        T: ExtendInto<U>,
        ValueInternal: From<U>,
        T: LittleEndianConvert,
    {
        let raw_address = self.value_stack.pop_as();
        let address = effective_address(offset, raw_address)?;
        let m = context
            .memory()
            .expect("Due to validation memory should exists");
        let v: T = m
            .get_value(address)
            .map_err(|_| TrapCode::MemoryAccessOutOfBounds)?;
        let stack_value: U = v.extend_into();
        self.value_stack
            .push(stack_value.into())
            .map_err(Into::into)
            .map(|_| InstructionOutcome::RunNextInstruction)
    }

    fn run_store<T>(
        &mut self,
        context: &mut FunctionContext,
        offset: u32,
    ) -> Result<InstructionOutcome, TrapCode>
    where
        T: FromValueInternal,
        T: LittleEndianConvert,
    {
        let stack_value = self.value_stack.pop_as::<T>();
        let raw_address = self.value_stack.pop_as::<u32>();
        let address = effective_address(offset, raw_address)?;

        let m = context
            .memory()
            .expect("Due to validation memory should exists");
        m.set_value(address, stack_value)
            .map_err(|_| TrapCode::MemoryAccessOutOfBounds)?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_store_wrap<T, U>(
        &mut self,
        context: &mut FunctionContext,
        offset: u32,
    ) -> Result<InstructionOutcome, TrapCode>
    where
        T: FromValueInternal,
        T: WrapInto<U>,
        U: LittleEndianConvert,
    {
        let stack_value: T = <_>::from_value_internal(self.value_stack.pop());
        let stack_value = stack_value.wrap_into();
        let raw_address = self.value_stack.pop_as::<u32>();
        let address = effective_address(offset, raw_address)?;
        let m = context
            .memory()
            .expect("Due to validation memory should exists");
        m.set_value(address, stack_value)
            .map_err(|_| TrapCode::MemoryAccessOutOfBounds)?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_current_memory(
        &mut self,
        context: &mut FunctionContext,
    ) -> Result<InstructionOutcome, TrapCode> {
        let m = context
            .memory()
            .expect("Due to validation memory should exists");
        let s = m.current_size().0;
        self.value_stack.push(ValueInternal(s as _))?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_grow_memory(
        &mut self,
        context: &mut FunctionContext,
    ) -> Result<InstructionOutcome, TrapCode> {
        let pages: u32 = self.value_stack.pop_as();
        let m = context
            .memory()
            .expect("Due to validation memory should exists");
        let m = match m.grow(Pages(pages as usize)) {
            Ok(Pages(new_size)) => new_size as u32,
            Err(_) => u32::MAX, // Returns -1 (or 0xFFFFFFFF) in case of error.
        };
        self.value_stack.push(ValueInternal(m as _))?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_const(&mut self, val: RuntimeValue) -> Result<InstructionOutcome, TrapCode> {
        self.value_stack
            .push(val.into())
            .map_err(Into::into)
            .map(|_| InstructionOutcome::RunNextInstruction)
    }

    fn run_relop<T, F>(&mut self, f: F) -> Result<InstructionOutcome, TrapCode>
    where
        T: FromValueInternal,
        F: FnOnce(T, T) -> bool,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = if f(left, right) {
            ValueInternal(1)
        } else {
            ValueInternal(0)
        };
        self.value_stack.push(v)?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_eqz<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        T: FromValueInternal,
        T: PartialEq<T> + Default,
    {
        let v = self.value_stack.pop_as::<T>();
        let v = ValueInternal(if v == Default::default() { 1 } else { 0 });
        self.value_stack.push(v)?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_eq<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        T: FromValueInternal + PartialEq<T>,
    {
        self.run_relop(|left: T, right: T| left == right)
    }

    fn run_ne<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        T: FromValueInternal + PartialEq<T>,
    {
        self.run_relop(|left: T, right: T| left != right)
    }

    fn run_lt<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        T: FromValueInternal + PartialOrd<T>,
    {
        self.run_relop(|left: T, right: T| left < right)
    }

    fn run_gt<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        T: FromValueInternal + PartialOrd<T>,
    {
        self.run_relop(|left: T, right: T| left > right)
    }

    fn run_lte<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        T: FromValueInternal + PartialOrd<T>,
    {
        self.run_relop(|left: T, right: T| left <= right)
    }

    fn run_gte<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        T: FromValueInternal + PartialOrd<T>,
    {
        self.run_relop(|left: T, right: T| left >= right)
    }

    fn run_unop<T, U, F>(&mut self, f: F) -> Result<InstructionOutcome, TrapCode>
    where
        F: FnOnce(T) -> U,
        T: FromValueInternal,
        ValueInternal: From<U>,
    {
        let v = self.value_stack.pop_as::<T>();
        let v = f(v);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_clz<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Integer<T> + FromValueInternal,
    {
        self.run_unop(|v: T| v.leading_zeros())
    }

    fn run_ctz<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Integer<T> + FromValueInternal,
    {
        self.run_unop(|v: T| v.trailing_zeros())
    }

    fn run_popcnt<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Integer<T> + FromValueInternal,
    {
        self.run_unop(|v: T| v.count_ones())
    }

    fn run_add<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: ArithmeticOps<T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.add(right);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_sub<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: ArithmeticOps<T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.sub(right);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_mul<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: ArithmeticOps<T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.mul(right);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_div<T, U>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: TransmuteInto<U> + FromValueInternal,
        U: ArithmeticOps<U> + TransmuteInto<T>,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let (left, right) = (left.transmute_into(), right.transmute_into());
        let v = left.div(right)?;
        let v = v.transmute_into();
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_rem<T, U>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: TransmuteInto<U> + FromValueInternal,
        U: Integer<U> + TransmuteInto<T>,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let (left, right) = (left.transmute_into(), right.transmute_into());
        let v = left.rem(right)?;
        let v = v.transmute_into();
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_and<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<<T as ops::BitAnd>::Output>,
        T: ops::BitAnd<T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.bitand(right);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_or<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<<T as ops::BitOr>::Output>,
        T: ops::BitOr<T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.bitor(right);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_xor<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<<T as ops::BitXor>::Output>,
        T: ops::BitXor<T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.bitxor(right);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_shl<T>(&mut self, mask: T) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<<T as ops::Shl<T>>::Output>,
        T: ops::Shl<T> + ops::BitAnd<T, Output = T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.shl(right & mask);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_shr<T, U>(&mut self, mask: U) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: TransmuteInto<U> + FromValueInternal,
        U: ops::Shr<U> + ops::BitAnd<U, Output = U>,
        <U as ops::Shr<U>>::Output: TransmuteInto<T>,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let (left, right) = (left.transmute_into(), right.transmute_into());
        let v = left.shr(right & mask);
        let v = v.transmute_into();
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_rotl<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Integer<T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.rotl(right);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_rotr<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Integer<T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.rotr(right);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_abs<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Float<T> + FromValueInternal,
    {
        self.run_unop(|v: T| v.abs())
    }

    fn run_neg<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<<T as ops::Neg>::Output>,
        T: ops::Neg + FromValueInternal,
    {
        self.run_unop(|v: T| v.neg())
    }

    fn run_ceil<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Float<T> + FromValueInternal,
    {
        self.run_unop(|v: T| v.ceil())
    }

    fn run_floor<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Float<T> + FromValueInternal,
    {
        self.run_unop(|v: T| v.floor())
    }

    fn run_trunc<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Float<T> + FromValueInternal,
    {
        self.run_unop(|v: T| v.trunc())
    }

    fn run_nearest<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Float<T> + FromValueInternal,
    {
        self.run_unop(|v: T| v.nearest())
    }

    fn run_sqrt<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Float<T> + FromValueInternal,
    {
        self.run_unop(|v: T| v.sqrt())
    }

    fn run_min<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Float<T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.min(right);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_max<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Float<T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.max(right);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_copysign<T>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<T>,
        T: Float<T> + FromValueInternal,
    {
        let (left, right) = self.value_stack.pop_pair_as::<T>();
        let v = left.copysign(right);
        self.value_stack.push(v.into())?;
        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_wrap<T, U>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<U>,
        T: WrapInto<U> + FromValueInternal,
    {
        self.run_unop(|v: T| v.wrap_into())
    }

    fn run_trunc_to_int<T, U, V>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<V>,
        T: TryTruncateInto<U, TrapCode> + FromValueInternal,
        U: TransmuteInto<V>,
    {
        let v = self.value_stack.pop_as::<T>();

        v.try_truncate_into()
            .map(|v| v.transmute_into())
            .map(|v| self.value_stack.push(v.into()))
            .map(|_| InstructionOutcome::RunNextInstruction)
    }

    fn run_extend<T, U, V>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<V>,
        T: ExtendInto<U> + FromValueInternal,
        U: TransmuteInto<V>,
    {
        let v = self.value_stack.pop_as::<T>();

        let v = v.extend_into().transmute_into();
        self.value_stack.push(v.into())?;

        Ok(InstructionOutcome::RunNextInstruction)
    }

    fn run_reinterpret<T, U>(&mut self) -> Result<InstructionOutcome, TrapCode>
    where
        ValueInternal: From<U>,
        T: FromValueInternal,
        T: TransmuteInto<U>,
    {
        let v = self.value_stack.pop_as::<T>();

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
        let module = match function.as_internal() {
			FuncInstanceInternal::Internal { module, .. } => module.upgrade().expect("module deallocated"),
			FuncInstanceInternal::Host { .. } => panic!("Host functions can't be called as internally defined functions; Thus FunctionContext can be created only with internally defined functions; qed"),
		};
        let memory = module.memory_by_index(DEFAULT_MEMORY_INDEX);
        FunctionContext {
            is_initialized: false,
            function,
            module: ModuleRef(module),
            memory,
            position: 0,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub fn initialize(
        &mut self,
        _locals: &[Local],
        _value_stack: &mut ValueStack,
    ) -> Result<(), TrapCode> {
        debug_assert!(!self.is_initialized);

        {
            /*
             * Since we have explicitly pushed local variables via T.const instruction,
             * we bypass extendind value_stack here.
             */
            // let num_locals = locals.iter().map(|l| l.count() as usize).sum();
            // value_stack.extend(num_locals)?;
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

fn effective_address(address: u32, offset: u32) -> Result<u32, TrapCode> {
    match offset.checked_add(address) {
        None => Err(TrapCode::MemoryAccessOutOfBounds),
        Some(address) => Ok(address),
    }
}

fn prepare_function_args(
    signature: &Signature,
    caller_stack: &mut ValueStack,
    host_args: &mut Vec<RuntimeValue>,
) {
    let req_args = signature.params();
    let len_args = req_args.len();
    let stack_args = caller_stack.pop_slice(len_args);
    assert_eq!(len_args, stack_args.len());
    host_args.clear();
    let prepared_args = req_args
        .iter()
        .zip(stack_args)
        .map(|(req_arg, stack_arg)| stack_arg.with_type(*req_arg));
    host_args.extend(prepared_args);
}

pub fn check_function_args(signature: &Signature, args: &[RuntimeValue]) -> Result<(), Trap> {
    if signature.params().len() != args.len() {
        return Err(TrapCode::UnexpectedSignature.into());
    }

    if signature
        .params()
        .iter()
        .zip(args.iter().map(|param_value| param_value.value_type()))
        .any(|(expected_type, actual_type)| &actual_type != expected_type)
    {
        return Err(TrapCode::UnexpectedSignature.into());
    }

    Ok(())
}

struct ValueStack {
    buf: Box<[ValueInternal]>,
    /// Index of the first free place in the stack.
    sp: usize,
}

impl core::fmt::Debug for ValueStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValueStack")
            .field("entries", &&self.buf[..self.sp])
            .field("stack_ptr", &self.sp)
            .finish()
    }
}

impl ValueStack {
    #[inline]
    fn drop_keep(&mut self, drop_keep: isa::DropKeep) {
        if let isa::Keep::Single(_) = drop_keep.keep {
            let top = *self.top();
            *self.pick_mut(drop_keep.drop as usize + 1) = top;
        }

        let cur_stack_len = self.len();
        self.sp = cur_stack_len - drop_keep.drop as usize;
    }

    #[inline]
    fn pop_as<T>(&mut self) -> T
    where
        T: FromValueInternal,
    {
        let value = self.pop();

        T::from_value_internal(value)
    }

    #[inline]
    fn pop_pair_as<T>(&mut self) -> (T, T)
    where
        T: FromValueInternal,
    {
        let right = self.pop_as();
        let left = self.pop_as();
        (left, right)
    }

    #[inline]
    fn pop_triple(&mut self) -> (ValueInternal, ValueInternal, ValueInternal) {
        let right = self.pop();
        let mid = self.pop();
        let left = self.pop();
        (left, mid, right)
    }

    #[inline]
    fn top(&self) -> &ValueInternal {
        self.pick(1)
    }

    fn pick(&self, depth: usize) -> &ValueInternal {
        &self.buf[self.sp - depth]
    }

    #[inline]
    fn pick_mut(&mut self, depth: usize) -> &mut ValueInternal {
        &mut self.buf[self.sp - depth]
    }

    #[inline]
    fn pop(&mut self) -> ValueInternal {
        self.sp -= 1;
        self.buf[self.sp]
    }

    #[inline]
    fn push(&mut self, value: ValueInternal) -> Result<(), TrapCode> {
        let cell = self.buf.get_mut(self.sp).ok_or(TrapCode::StackOverflow)?;
        *cell = value;
        self.sp += 1;
        Ok(())
    }

    #[allow(dead_code)]
    fn extend(&mut self, len: usize) -> Result<(), TrapCode> {
        let cells = self
            .buf
            .get_mut(self.sp..self.sp + len)
            .ok_or(TrapCode::StackOverflow)?;
        for cell in cells {
            *cell = Default::default();
        }
        self.sp += len;
        Ok(())
    }

    #[inline]
    fn len(&self) -> usize {
        self.sp
    }

    /// Pops the last `depth` stack entries and returns them as slice.
    ///
    /// Stack entries are returned in the order in which they got pushed
    /// onto the value stack.
    ///
    /// # Panics
    ///
    /// If the value stack does not have at least `depth` stack entries.
    pub fn pop_slice(&mut self, depth: usize) -> &[ValueInternal] {
        self.sp -= depth;
        let start = self.sp;
        let end = self.sp + depth;
        &self.buf[start..end]
    }
}

struct CallStack {
    buf: Vec<FunctionContext>,
    limit: usize,
}

impl CallStack {
    fn push(&mut self, ctx: FunctionContext) {
        self.buf.push(ctx);
    }

    fn pop(&mut self) -> Option<FunctionContext> {
        self.buf.pop()
    }

    fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    fn is_full(&self) -> bool {
        self.buf.len() + 1 >= self.limit
    }
}

/// Used to recycle stacks instead of allocating them repeatedly.
pub struct StackRecycler {
    value_stack_buf: Option<Box<[ValueInternal]>>,
    value_stack_limit: usize,
    call_stack_buf: Option<Vec<FunctionContext>>,
    call_stack_limit: usize,
}

impl StackRecycler {
    /// Limit stacks created by this recycler to
    /// - `value_stack_limit` entries for values and
    /// - `call_stack_limit` levels for calls.
    pub fn with_limits(value_stack_limit: usize, call_stack_limit: usize) -> Self {
        Self {
            value_stack_buf: None,
            value_stack_limit,
            call_stack_buf: None,
            call_stack_limit,
        }
    }

    /// Clears any values left on the stack to avoid
    /// leaking them to future export invocations.
    ///
    /// This is a secondary defense to prevent modules from
    /// exploiting faulty stack handling in the interpreter.
    ///
    /// Do note that there are additional channels that
    /// can leak information into an untrusted module.
    pub fn clear(&mut self) {
        if let Some(buf) = &mut self.value_stack_buf {
            for cell in buf.iter_mut() {
                *cell = ValueInternal(0);
            }
        }
    }

    fn recreate_value_stack(this: &mut Option<&mut Self>) -> ValueStack {
        let limit = this
            .as_ref()
            .map_or(DEFAULT_VALUE_STACK_LIMIT, |this| this.value_stack_limit);

        let buf = this
            .as_mut()
            .and_then(|this| this.value_stack_buf.take())
            .unwrap_or_else(|| {
                let mut buf = Vec::new();
                buf.reserve_exact(limit);
                buf.resize(limit, ValueInternal(0));
                buf.into_boxed_slice()
            });

        ValueStack { buf, sp: 0 }
    }

    fn recreate_call_stack(this: &mut Option<&mut Self>) -> CallStack {
        let limit = this
            .as_ref()
            .map_or(DEFAULT_CALL_STACK_LIMIT, |this| this.call_stack_limit);

        let buf = this
            .as_mut()
            .and_then(|this| this.call_stack_buf.take())
            .unwrap_or_default();

        CallStack { buf, limit }
    }

    pub(crate) fn recycle(&mut self, mut interpreter: Interpreter) {
        interpreter.call_stack.buf.clear();

        self.value_stack_buf = Some(interpreter.value_stack.buf);
        self.call_stack_buf = Some(interpreter.call_stack.buf);
    }
}

impl Default for StackRecycler {
    fn default() -> Self {
        Self::with_limits(DEFAULT_VALUE_STACK_LIMIT, DEFAULT_CALL_STACK_LIMIT)
    }
}
