use super::{
    super::{Global, Memory},
    bytecode::{BrTable, FuncIdx, GlobalIdx, LocalIdx, Offset, SignatureIdx, VisitInstruction},
    AsContextMut,
    CallStack,
    DropKeep,
    EngineInner,
    ExecutionOutcome,
    FromStackEntry,
    FunctionExecutionOutcome,
    FunctionFrame,
    ResolvedFuncBody,
    StackEntry,
    Target,
    ValueStack,
};
use crate::{
    nan_preserving_float::{F32, F64},
    Trap,
    TrapKind,
};
use memory_units::wasm32::Pages;

/// Types that can be converted from and to little endian bytes.
pub trait LittleEndianConvert {
    /// The little endian bytes representation.
    type Bytes: Default + AsRef<[u8]> + AsMut<[u8]>;

    /// Converts `self` into little endian bytes.
    fn into_le_bytes(self) -> Self::Bytes;

    /// Converts little endian bytes into `Self`.
    fn from_le_bytes(bytes: Self::Bytes) -> Self;
}

macro_rules! impl_little_endian_convert_primitive {
    ( $($primitive:ty),* $(,)? ) => {
        $(
            impl LittleEndianConvert for $primitive {
                type Bytes = [::core::primitive::u8; ::core::mem::size_of::<$primitive>()];

                fn into_le_bytes(self) -> Self::Bytes {
                    <$primitive>::to_le_bytes(self)
                }

                fn from_le_bytes(bytes: Self::Bytes) -> Self {
                    <$primitive>::from_le_bytes(bytes)
                }
            }
        )*
    };
}
impl_little_endian_convert_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

macro_rules! impl_little_endian_convert_float {
    ( $( struct $float_ty:ident($uint_ty:ty); )* $(,)? ) => {
        $(
            impl LittleEndianConvert for $float_ty {
                type Bytes = <$uint_ty as LittleEndianConvert>::Bytes;

                fn into_le_bytes(self) -> Self::Bytes {
                    <$uint_ty>::into_le_bytes(self.to_bits())
                }

                fn from_le_bytes(bytes: Self::Bytes) -> Self {
                    Self::from_bits(<$uint_ty>::from_le_bytes(bytes))
                }
            }
        )*
    };
}
impl_little_endian_convert_float!(
    struct F32(u32);
    struct F64(u64);
);

/// Convert one type to another by wrapping.
pub trait WrapInto<T> {
    /// Convert one type to another by wrapping.
    fn wrap_into(self) -> T;
}

macro_rules! impl_wrap_into {
    ($from:ident, $into:ident) => {
        impl WrapInto<$into> for $from {
            fn wrap_into(self) -> $into {
                self as $into
            }
        }
    };
    ($from:ident, $intermediate:ident, $into:ident) => {
        impl WrapInto<$into> for $from {
            fn wrap_into(self) -> $into {
                <$into>::from(self as $intermediate)
            }
        }
    };
}

impl_wrap_into!(i32, i8);
impl_wrap_into!(i32, i16);
impl_wrap_into!(i64, i8);
impl_wrap_into!(i64, i16);
impl_wrap_into!(i64, i32);
impl_wrap_into!(i64, f32, F32);
impl_wrap_into!(u64, f32, F32);
// Casting from an f64 to an f32 will produce the closest possible value.
//
// Note:
// - The rounding strategy is unspecified.
// - Currently this will cause Undefined Behavior if the value is finite
//   but larger or smaller than the largest or smallest finite value
//   representable by f32. This is a bug and will be fixed.
impl_wrap_into!(f64, f32);

impl WrapInto<F32> for F64 {
    fn wrap_into(self) -> F32 {
        (f64::from(self) as f32).into()
    }
}

/// State that is used during Wasm function execution.
#[derive(Debug)]
pub struct ExecutionContext<'engine, 'func> {
    /// Stores the value stack of live values on the Wasm stack.
    value_stack: &'engine mut ValueStack,
    /// Stores the call stack of live function invocations.
    call_stack: &'engine mut CallStack,
    /// The function frame that is being executed.
    frame: &'func mut FunctionFrame,
    /// The resolved function body of the executed function frame.
    func_body: ResolvedFuncBody<'engine>,
}

impl<'engine, 'func> ExecutionContext<'engine, 'func> {
    /// Creates an execution context for the given [`FunctionFrame`].
    pub fn new(engine: &'engine mut EngineInner, frame: &'func mut FunctionFrame) -> Self {
        let resolved = engine.code_map.resolve(frame.func_body);
        frame.initialize(resolved, &mut engine.value_stack);
        Self {
            value_stack: &mut engine.value_stack,
            call_stack: &mut engine.call_stack,
            frame,
            func_body: resolved,
        }
    }

    pub fn execute_frame(
        &mut self,
        mut ctx: impl AsContextMut,
    ) -> Result<FunctionExecutionOutcome, Trap> {
        'outer: loop {
            let pc = self.frame.inst_ptr;
            let inst_context = InstructionExecutionContext::new(
                self.value_stack,
                self.call_stack,
                self.frame,
                ctx.as_context_mut(),
            );
            match self.func_body.visit(pc, inst_context)? {
                ExecutionOutcome::Continue => {}
                ExecutionOutcome::Branch(target) => {
                    self.value_stack.drop_keep(target.drop_keep());
                }
                ExecutionOutcome::ExecuteCall(func) => {
                    return Ok(FunctionExecutionOutcome::NestedCall(func));
                }
                ExecutionOutcome::Return(drop_keep) => {
                    self.value_stack.drop_keep(drop_keep);
                    break 'outer;
                }
            }
        }
        Ok(FunctionExecutionOutcome::Return)
    }
}

/// An execution context for executing a single `wasmi` bytecode instruction.
#[derive(Debug)]
struct InstructionExecutionContext<'engine, 'func, Ctx> {
    /// Stores the value stack of live values on the Wasm stack.
    value_stack: &'engine mut ValueStack,
    /// Stores the call stack of live function invocations.
    call_stack: &'engine mut CallStack,
    /// The function frame that is being executed.
    frame: &'func mut FunctionFrame,
    /// A mutable [`Store`] context.
    ///
    /// [`Store`]: [`crate::v1::Store`]
    ctx: Ctx,
}

impl<'engine, 'func, Ctx> InstructionExecutionContext<'engine, 'func, Ctx>
where
    Ctx: AsContextMut,
{
    /// Creates a new [`InstructionExecutionContext`] for executing a single `wasmi` bytecode instruction.
    pub fn new(
        value_stack: &'engine mut ValueStack,
        call_stack: &'engine mut CallStack,
        frame: &'func mut FunctionFrame,
        ctx: Ctx,
    ) -> Self {
        Self {
            value_stack,
            call_stack,
            frame,
            ctx,
        }
    }

    /// Returns the default linear memory.
    ///
    /// # Panics
    ///
    /// If there is no default linear memory.
    fn default_memory(&self) -> Memory {
        self.frame
            .default_memory
            .expect("missing default memory for function frame")
    }

    /// Returns the global variable at the given index.
    ///
    /// # Panics
    ///
    /// If there is no global variable at the given index.
    fn global(&self, global_index: GlobalIdx) -> Global {
        self.frame
            .instance
            .get_global(self.ctx.as_context(), global_index.into_inner())
            .unwrap_or_else(|| panic!("missing global at index {:?}", global_index))
    }

    /// Returns the local depth as `usize`.
    fn convert_local_depth(local_depth: LocalIdx) -> usize {
        // TODO: calculate the -1 offset at module compilation time.
        (local_depth.into_inner() - 1) as usize
    }

    /// Calculates the effective address of a linear memory access.
    ///
    /// # Errors
    ///
    /// If the resulting effective address overflows.
    fn effective_address(offset: Offset, address: u32) -> Result<usize, TrapKind> {
        offset
            .into_inner()
            .checked_add(address)
            .map(|address| address as usize)
            .ok_or(TrapKind::MemoryAccessOutOfBounds)
    }

    /// Loads a value of type `T` from the default memory at the given address offset.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `i32.load`
    /// - `i64.load`
    /// - `f32.load`
    /// - `f64.load`
    fn load<T>(&mut self, offset: Offset) -> Result<ExecutionOutcome, TrapKind>
    where
        StackEntry: From<T>,
        T: LittleEndianConvert,
    {
        let memory = self.default_memory();
        let entry = self.value_stack.last_mut();
        let raw_address = u32::from_stack_entry(*entry);
        let address = Self::effective_address(offset, raw_address)?;
        let mut bytes = <<T as LittleEndianConvert>::Bytes as Default>::default();
        memory
            .read(self.ctx.as_context(), address, bytes.as_mut())
            .map_err(|_| TrapKind::MemoryAccessOutOfBounds)?;
        let value = <T as LittleEndianConvert>::from_le_bytes(bytes);
        *entry = value.into();
        Ok(ExecutionOutcome::Continue)
    }

    /// Stores a value of type `T` into the default memory at the given address offset.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `i32.store`
    /// - `i64.store`
    /// - `f32.store`
    /// - `f64.store`
    fn store<T>(&mut self, offset: Offset) -> Result<ExecutionOutcome, TrapKind>
    where
        T: LittleEndianConvert + FromStackEntry,
    {
        let stack_value = self.value_stack.pop_as::<T>();
        let raw_address = self.value_stack.pop_as::<u32>();
        let address = Self::effective_address(offset, raw_address)?;
        let memory = self.default_memory();
        let bytes = <T as LittleEndianConvert>::into_le_bytes(stack_value);
        memory
            .write(self.ctx.as_context_mut(), address, bytes.as_ref())
            .map_err(|_| TrapKind::MemoryAccessOutOfBounds)?;
        Ok(ExecutionOutcome::Continue)
    }

    /// Stores a value of type `T` wrapped to type `U` into the default memory at the given address offset.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `i32.store8`
    /// - `i32.store16`
    /// - `i64.store8`
    /// - `i64.store16`
    /// - `i64.store32`
    fn store_wrap<T, U>(&mut self, offset: Offset) -> Result<ExecutionOutcome, TrapKind>
    where
        T: WrapInto<U> + FromStackEntry,
        U: LittleEndianConvert,
    {
        let wrapped_value = self.value_stack.pop_as::<T>().wrap_into();
        let raw_address = self.value_stack.pop_as::<u32>();
        let address = Self::effective_address(offset, raw_address)?;
        let memory = self.default_memory();
        let bytes = <U as LittleEndianConvert>::into_le_bytes(wrapped_value);
        memory
            .write(self.ctx.as_context_mut(), address, bytes.as_ref())
            .map_err(|_| TrapKind::MemoryAccessOutOfBounds)?;
        Ok(ExecutionOutcome::Continue)
    }
}

impl<'engine, 'func, Ctx> VisitInstruction for InstructionExecutionContext<'engine, 'func, Ctx>
where
    Ctx: AsContextMut,
{
    type Outcome = Result<ExecutionOutcome, TrapKind>;

    fn visit_unreachable(&mut self) -> Self::Outcome {
        Err(TrapKind::Unreachable)
    }

    fn visit_br(&mut self, target: Target) -> Self::Outcome {
        Ok(ExecutionOutcome::Branch(target))
    }

    fn visit_br_if_eqz(&mut self, target: Target) -> Self::Outcome {
        let condition = self.value_stack.pop_as();
        if condition {
            Ok(ExecutionOutcome::Continue)
        } else {
            Ok(ExecutionOutcome::Branch(target))
        }
    }

    fn visit_br_if_nez(&mut self, target: Target) -> Self::Outcome {
        let condition = self.value_stack.pop_as();
        if condition {
            Ok(ExecutionOutcome::Branch(target))
        } else {
            Ok(ExecutionOutcome::Continue)
        }
    }

    fn visit_br_table(&mut self, br_table: BrTable) -> Self::Outcome {
        let index: u32 = self.value_stack.pop_as();
        let target = br_table.target_or_default(index as usize);
        Ok(ExecutionOutcome::Branch(*target))
    }

    fn visit_ret(&mut self, drop_keep: DropKeep) -> Self::Outcome {
        Ok(ExecutionOutcome::Return(drop_keep))
    }

    fn visit_get_local(&mut self, local_depth: LocalIdx) -> Self::Outcome {
        let local_depth = Self::convert_local_depth(local_depth);
        let value = self.value_stack.peek(local_depth);
        self.value_stack.push(value);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_set_local(&mut self, local_depth: LocalIdx) -> Self::Outcome {
        let local_depth = Self::convert_local_depth(local_depth);
        let new_value = self.value_stack.pop();
        *self.value_stack.peek_mut(local_depth) = new_value;
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_tee_local(&mut self, local_depth: LocalIdx) -> Self::Outcome {
        let local_depth = Self::convert_local_depth(local_depth);
        let new_value = self.value_stack.last();
        *self.value_stack.peek_mut(local_depth) = new_value;
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_get_global(&mut self, global_index: GlobalIdx) -> Self::Outcome {
        let global_value = self.global(global_index).get(self.ctx.as_context());
        self.value_stack.push(global_value);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_set_global(&mut self, global_index: GlobalIdx) -> Self::Outcome {
        let global = self.global(global_index);
        let new_value = self
            .value_stack
            .pop()
            .with_type(global.value_type(self.ctx.as_context()));
        global
            .set(self.ctx.as_context_mut(), new_value)
            .unwrap_or_else(|error| panic!("encountered type mismatch upon global_set: {}", error));
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_call(&mut self, func_index: FuncIdx) -> Self::Outcome {
        let func = self
            .frame
            .instance
            .get_func(self.ctx.as_context_mut(), func_index.into_inner())
            .unwrap_or_else(|| panic!("missing function at index {:?}", func_index));
        Ok(ExecutionOutcome::ExecuteCall(func))
    }

    fn visit_call_indirect(&mut self, signature_index: SignatureIdx) -> Self::Outcome {
        let func_index: u32 = self.value_stack.pop_as();
        let table = self
            .frame
            .default_table
            .expect("encountered call_indirect without table");
        let func = table
            .get(self.ctx.as_context(), func_index as usize)
            .map_err(|_| TrapKind::TableAccessOutOfBounds)?
            .ok_or(TrapKind::ElemUninitialized)?;
        let actual_signature = func.signature(self.ctx.as_context());
        let expected_signature = self
            .frame
            .instance
            .get_signature(self.ctx.as_context(), signature_index.into_inner())
            .unwrap_or_else(|| {
                panic!(
                    "missing signature for call_indirect at index: {:?}",
                    signature_index,
                )
            });
        if actual_signature != expected_signature {
            return Err(TrapKind::UnexpectedSignature);
        }
        Ok(ExecutionOutcome::ExecuteCall(func))
    }

    fn visit_i32_const(&mut self, value: i32) -> Self::Outcome {
        self.value_stack.push(value);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_i64_const(&mut self, value: i64) -> Self::Outcome {
        self.value_stack.push(value);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_f32_const(&mut self, value: F32) -> Self::Outcome {
        self.value_stack.push(value);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_f64_const(&mut self, value: F64) -> Self::Outcome {
        self.value_stack.push(value);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_drop(&mut self) -> Self::Outcome {
        let _ = self.value_stack.pop();
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_select(&mut self) -> Self::Outcome {
        self.value_stack.pop2_eval(|e1, e2, e3| {
            let condition = FromStackEntry::from_stack_entry(e3);
            let result = if condition { *e1 } else { e2 };
            *e1 = result;
        });
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_current_memory(&mut self) -> Self::Outcome {
        let memory = self.default_memory();
        let result = memory.current_pages(self.ctx.as_context()).0 as u32;
        self.value_stack.push(result);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_grow_memory(&mut self) -> Self::Outcome {
        let pages: u32 = self.value_stack.pop_as();
        let memory = self.default_memory();
        let new_size = match memory.grow(self.ctx.as_context_mut(), Pages(pages as usize)) {
            Ok(Pages(old_size)) => old_size as u32,
            Err(_) => {
                // Note: The WebAssembly spec demands to return `0xFFFF_FFFF`
                //       in case of failure for this instruction.
                u32::MAX
            }
        };
        self.value_stack.push(new_size);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_i32_load(&mut self, offset: Offset) -> Self::Outcome {
        self.load::<i32>(offset)
    }

    fn visit_i64_load(&mut self, offset: Offset) -> Self::Outcome {
        self.load::<i64>(offset)
    }

    fn visit_f32_load(&mut self, offset: Offset) -> Self::Outcome {
        self.load::<F32>(offset)
    }

    fn visit_f64_load(&mut self, offset: Offset) -> Self::Outcome {
        self.load::<F64>(offset)
    }

    fn visit_i32_load_i8(&mut self, _offset: Offset) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_load_u8(&mut self, _offset: Offset) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_load_i16(&mut self, _offset: Offset) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_load_u16(&mut self, _offset: Offset) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_load_i8(&mut self, _offset: Offset) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_load_u8(&mut self, _offset: Offset) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_load_i16(&mut self, _offset: Offset) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_load_u16(&mut self, _offset: Offset) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_load_i32(&mut self, _offset: Offset) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_load_u32(&mut self, _offset: Offset) -> Self::Outcome {
        todo!()
    }

    fn visit_i32_store(&mut self, offset: Offset) -> Self::Outcome {
        self.store::<i32>(offset)
    }

    fn visit_i64_store(&mut self, offset: Offset) -> Self::Outcome {
        self.store::<i64>(offset)
    }

    fn visit_f32_store(&mut self, offset: Offset) -> Self::Outcome {
        self.store::<F32>(offset)
    }

    fn visit_f64_store(&mut self, offset: Offset) -> Self::Outcome {
        self.store::<F64>(offset)
    }

    fn visit_i32_store_8(&mut self, offset: Offset) -> Self::Outcome {
        self.store_wrap::<i32, i8>(offset)
    }

    fn visit_i32_store_16(&mut self, offset: Offset) -> Self::Outcome {
        self.store_wrap::<i32, i16>(offset)
    }

    fn visit_i64_store_8(&mut self, offset: Offset) -> Self::Outcome {
        self.store_wrap::<i64, i8>(offset)
    }

    fn visit_i64_store_16(&mut self, offset: Offset) -> Self::Outcome {
        self.store_wrap::<i64, i16>(offset)
    }

    fn visit_i64_store_32(&mut self, offset: Offset) -> Self::Outcome {
        self.store_wrap::<i64, i32>(offset)
    }

    fn visit_i32_eqz(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_eq(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_ne(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_lt_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_lt_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_gt_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_gt_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_le_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_le_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_ge_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_ge_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_eqz(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_eq(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_ne(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_lt_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_lt_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_gt_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_gt_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_le_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_le_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_ge_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_ge_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_eq(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_ne(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_lt(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_gt(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_le(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_ge(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_eq(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_ne(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_lt(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_gt(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_le(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_ge(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_clz(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_ctz(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_popcnt(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_add(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_sub(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_mul(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_div_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_div_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_rem_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_rem_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_and(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_or(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_xor(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_shl(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_shr_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_shr_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_rotl(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_rotr(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_clz(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_ctz(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_popcnt(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_add(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_sub(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_mul(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_div_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_div_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_rem_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_rem_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_and(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_or(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_xor(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_shl(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_shr_s(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_shr_u(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_rotl(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_rotr(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_abs(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_neg(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_ceil(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_floor(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_trunc(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_nearest(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_sqrt(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_add(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_sub(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_mul(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_div(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_min(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_max(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_copysign(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_abs(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_neg(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_ceil(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_floor(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_trunc(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_nearest(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_sqrt(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_add(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_sub(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_mul(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_div(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_min(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_max(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_copysign(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_wrap_i64(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_trunc_f32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_u32_trunc_f32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_trunc_f64(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_u32_trunc_f64(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_extend_i32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_extend_u32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_trunc_f32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_u64_trunc_f32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_trunc_f64(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_u64_trunc_f64(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_convert_i32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_convert_u32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_convert_i64(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_convert_u64(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_demote_f64(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_convert_i32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_convert_u32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_convert_i64(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_convert_u64(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_promote_f32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i32_reinterpret_f32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_i64_reinterpret_f64(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f32_reinterpret_i32(&mut self) -> Self::Outcome {
        todo!()
    }
    fn visit_f64_reinterpret_i64(&mut self) -> Self::Outcome {
        todo!()
    }
}
