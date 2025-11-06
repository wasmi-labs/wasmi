use crate::{
    core::{ReadAs, UntypedVal, WriteAs},
    engine::{
        executor::{
            handler::{dispatch::ExecutionOutcome, utils::extract_mem0},
            CodeMap,
        },
        utils::unreachable_unchecked,
        ResumableHostTrapError,
        ResumableOutOfFuelError,
        StackConfig,
    },
    func::FuncInOut,
    instance::InstanceEntity,
    ir::{self, BoundedSlotSpan, Slot, SlotSpan},
    store::PrunedStore,
    Error,
    Func,
    TrapCode,
};
use alloc::vec::Vec;
use core::{
    ptr::{self, NonNull},
    slice,
};

pub struct VmState<'vm> {
    pub store: &'vm mut PrunedStore,
    pub stack: &'vm mut Stack,
    pub code: &'vm CodeMap,
    done_reason: Option<DoneReason>,
}

impl<'vm> VmState<'vm> {
    pub fn new(store: &'vm mut PrunedStore, stack: &'vm mut Stack, code: &'vm CodeMap) -> Self {
        Self {
            store,
            stack,
            code,
            done_reason: None,
        }
    }

    pub fn done_with(&mut self, reason: impl FnOnce() -> DoneReason) {
        #[cold]
        #[inline(never)]
        fn err(prev: &DoneReason, reason: impl FnOnce() -> DoneReason) -> ! {
            panic!(
                "\
                tried to done with reason while reason already exists:\n\
                \t- new reason: {:?},\n\
                \t- old reason: {:?},\
                ",
                reason(),
                prev,
            )
        }

        if let Some(prev) = &self.done_reason {
            err(prev, reason)
        }
        self.done_reason = Some(reason());
    }

    pub fn into_execution_outcome(self) -> Result<Sp, ExecutionOutcome> {
        let Some(reason) = self.done_reason else {
            panic!("missing break reason")
        };
        let outcome = match reason {
            DoneReason::Return(sp) => return Ok(sp),
            DoneReason::Host(error) => error.into(),
            DoneReason::OutOfFuel(error) => error.into(),
            DoneReason::CompileError(error) => error.into(),
        };
        Err(outcome)
    }
}

#[derive(Debug)]
pub enum DoneReason {
    Return(Sp),
    Host(ResumableHostTrapError),
    OutOfFuel(ResumableOutOfFuelError),
    CompileError(Error),
}

impl DoneReason {
    #[cold]
    #[inline]
    pub fn host_error(error: Error, func: Func, results: SlotSpan) -> Self {
        Self::Host(ResumableHostTrapError::new(error, func, results))
    }

    #[cold]
    #[inline]
    pub fn out_of_fuel(required_fuel: u64) -> Self {
        Self::OutOfFuel(ResumableOutOfFuelError::new(required_fuel))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Inst(NonNull<InstanceEntity>);

impl From<&'_ InstanceEntity> for Inst {
    fn from(entity: &'_ InstanceEntity) -> Self {
        Self(entity.into())
    }
}

impl PartialEq for Inst {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Inst {}

impl Inst {
    pub unsafe fn as_ref(&self) -> &InstanceEntity {
        unsafe { self.0.as_ref() }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Mem0Ptr(*mut u8);

impl From<*mut u8> for Mem0Ptr {
    fn from(value: *mut u8) -> Self {
        Self(value)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Mem0Len(usize);

impl From<usize> for Mem0Len {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

pub fn mem0_bytes<'a>(mem0: Mem0Ptr, mem0_len: Mem0Len) -> &'a mut [u8] {
    unsafe { slice::from_raw_parts_mut(mem0.0, mem0_len.0) }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Ip {
    value: *const u8,
}

impl<'a> From<&'a [u8]> for Ip {
    fn from(ops: &'a [u8]) -> Self {
        Self {
            value: ops.as_ptr(),
        }
    }
}

struct IpDecoder(Ip);
impl ir::Decoder for IpDecoder {
    fn read_bytes(&mut self, buffer: &mut [u8]) -> Result<(), ir::DecodeError> {
        unsafe { ptr::copy_nonoverlapping(self.0.value, buffer.as_mut_ptr(), buffer.len()) };
        self.0 = unsafe { self.0.add(buffer.len()) };
        Ok(())
    }
}

impl Ip {
    #[inline]
    pub unsafe fn decode<T: ir::Decode>(self) -> (Ip, T) {
        let mut ip = IpDecoder(self);
        let decoded = match <T as ir::Decode>::decode(&mut ip) {
            Ok(decoded) => decoded,
            Err(error) => unsafe {
                crate::engine::utils::unreachable_unchecked!(
                    "failed to decode `OpCode` or op-handler: {error}"
                )
            },
        };
        (ip.0, decoded)
    }

    pub unsafe fn skip<T: ir::Decode>(self) -> Ip {
        let (ip, _) = unsafe { self.decode::<T>() };
        ip
    }

    pub unsafe fn offset(self, delta: isize) -> Self {
        let value = unsafe { self.value.byte_offset(delta) };
        Self { value }
    }

    pub unsafe fn add(self, delta: usize) -> Self {
        let value = unsafe { self.value.byte_add(delta) };
        Self { value }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Sp {
    value: *mut UntypedVal,
}

impl Sp {
    pub fn new(cells: &mut Vec<UntypedVal>, start: usize) -> Self {
        debug_assert!(
            start < cells.len(),
            "start = {}, cells.len() = {}",
            start,
            cells.len()
        );
        Self {
            value: unsafe { cells.as_mut_ptr().add(start) },
        }
    }

    pub fn dangling() -> Self {
        Self {
            value: ptr::dangling_mut(),
        }
    }

    pub fn get<T>(self, slot: Slot) -> T
    where
        UntypedVal: ReadAs<T>,
    {
        let index = usize::from(u16::from(slot));
        let value = unsafe { &*self.value.add(index) };
        <UntypedVal as ReadAs<T>>::read_as(value)
    }

    pub fn set<T>(self, slot: Slot, value: T)
    where
        UntypedVal: WriteAs<T>,
    {
        let index = usize::from(u16::from(slot));
        let cell = unsafe { &mut *self.value.add(index) };
        <UntypedVal as WriteAs<T>>::write_as(cell, value);
    }

    pub unsafe fn as_slice<'a>(self, len: usize) -> &'a [UntypedVal] {
        unsafe { core::slice::from_raw_parts(self.value, len) }
    }
}

#[derive(Debug)]
pub struct Stack {
    values: ValueStack,
    frames: CallStack,
}

impl Stack {
    pub fn new(config: &StackConfig) -> Self {
        Self {
            values: ValueStack::new(config.min_stack_height(), config.max_stack_height()),
            frames: CallStack::new(config.max_recursion_depth()),
        }
    }

    pub fn empty() -> Self {
        Self {
            values: ValueStack::empty(),
            frames: CallStack::empty(),
        }
    }

    pub fn reset(&mut self) {
        self.values.reset();
        self.frames.reset();
    }

    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    pub fn prepare_host_frame<'a>(
        &'a mut self,
        caller_ip: Option<Ip>,
        callee_params: BoundedSlotSpan,
        results_len: u16,
    ) -> Result<(Sp, FuncInOut<'a>), TrapCode> {
        let caller_start = self.frames.prepare_host_frame(caller_ip);
        self.values
            .prepare_host_frame(caller_start, callee_params, results_len)
    }

    #[inline(always)]
    pub fn push_frame(
        &mut self,
        caller_ip: Option<Ip>,
        callee_ip: Ip,
        callee_params: BoundedSlotSpan,
        callee_size: usize,
        callee_instance: Option<Inst>,
    ) -> Result<Sp, TrapCode> {
        let start = self
            .frames
            .push(caller_ip, callee_ip, callee_params, callee_instance)?;
        self.values.push(start, callee_size, callee_params.len())
    }

    pub fn pop_frame(
        &mut self,
        store: &mut PrunedStore,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
    ) -> Option<(Ip, Sp, Mem0Ptr, Mem0Len, Inst)> {
        let (ip, start, changed_instance) = self.frames.pop()?;
        let sp = self.values.sp_or_dangling(start);
        let (mem0, mem0_len, instance) = match changed_instance {
            Some(instance) => {
                let (mem0, mem0_len) = extract_mem0(store, instance);
                (mem0, mem0_len, instance)
            }
            None => (mem0, mem0_len, instance),
        };
        Some((ip, sp, mem0, mem0_len, instance))
    }

    #[inline(always)]
    pub fn replace_frame(
        &mut self,
        callee_ip: Ip,
        callee_params: BoundedSlotSpan,
        callee_size: usize,
        callee_instance: Option<Inst>,
    ) -> Result<Sp, TrapCode> {
        let start = self.frames.replace(callee_ip, callee_instance)?;
        self.values.replace(start, callee_size, callee_params)
    }
}

#[derive(Debug)]
pub struct ValueStack {
    cells: Vec<UntypedVal>,
    max_height: usize,
}

impl ValueStack {
    fn new(min_height: usize, max_height: usize) -> Self {
        let cells = Vec::with_capacity(min_height);
        Self { cells, max_height }
    }

    fn empty() -> Self {
        Self {
            cells: Vec::new(),
            max_height: 0,
        }
    }

    fn reset(&mut self) {
        self.cells.clear();
    }

    fn capacity(&self) -> usize {
        self.cells.capacity()
    }

    fn sp(&mut self, start: usize) -> Sp {
        debug_assert!(!self.cells.is_empty());
        Sp::new(&mut self.cells, start)
    }

    fn sp_or_dangling(&mut self, start: usize) -> Sp {
        match self.cells.is_empty() {
            true => {
                debug_assert_eq!(start, 0);
                Sp::dangling()
            }
            false => self.sp(start),
        }
    }

    fn prepare_host_frame<'a>(
        &'a mut self,
        caller_start: usize,
        callee_params: BoundedSlotSpan,
        results_len: u16,
    ) -> Result<(Sp, FuncInOut<'a>), TrapCode> {
        let params_offset = usize::from(u16::from(callee_params.span().head()));
        let params_len = usize::from(callee_params.len());
        let results_len = usize::from(results_len);
        let callee_size = params_len.max(results_len);
        let Some(callee_start) = caller_start.checked_add(params_offset) else {
            return Err(TrapCode::StackOverflow);
        };
        let Some(callee_end) = callee_start.checked_add(callee_size) else {
            return Err(TrapCode::StackOverflow);
        };
        self.cells.resize(callee_end, UntypedVal::default());
        let sp = self.sp(caller_start);
        let cells = &mut self.cells[callee_start..];
        let inout = FuncInOut::new(cells, params_len, results_len);
        Ok((sp, inout))
    }

    #[inline(always)]
    fn push(&mut self, start: usize, len_slots: usize, len_params: u16) -> Result<Sp, TrapCode> {
        debug_assert!(usize::from(len_params) <= len_slots);
        if len_slots == 0 {
            return Ok(Sp::dangling());
        }
        let Some(end) = start.checked_add(len_slots) else {
            return Err(TrapCode::StackOverflow);
        };
        if end > self.max_height {
            return Err(TrapCode::StackOverflow);
        }
        self.cells.resize_with(end, UntypedVal::default);
        let start_locals = start.wrapping_add(usize::from(len_params));
        self.cells[start_locals..end].fill_with(UntypedVal::default);
        let sp = self.sp(start);
        Ok(sp)
    }

    #[inline(always)]
    fn replace(
        &mut self,
        start: usize,
        len_slots: usize,
        callee_params: BoundedSlotSpan,
    ) -> Result<Sp, TrapCode> {
        let params_len = callee_params.len();
        let params_offset = usize::from(u16::from(callee_params.span().head()));
        debug_assert!(usize::from(params_len) <= len_slots);
        if len_slots == 0 {
            return Ok(Sp::dangling());
        }
        let Some(end) = start.checked_add(len_slots) else {
            return Err(TrapCode::StackOverflow);
        };
        if end > self.max_height {
            return Err(TrapCode::StackOverflow);
        }
        let Some(cells) = self.cells.get_mut(start..) else {
            unsafe { unreachable_unchecked!() }
        };
        let params_end = params_offset.wrapping_add(usize::from(params_len));
        cells.copy_within(params_offset..params_end, 0);
        self.cells.resize_with(end, UntypedVal::default);
        let Some(cells) = self.cells.get_mut(start..end) else {
            unsafe { unreachable_unchecked!() }
        };
        let locals_start = start.wrapping_add(usize::from(params_len));
        cells[locals_start..].fill_with(UntypedVal::default);
        let sp = self.sp(start);
        Ok(sp)
    }
}

#[derive(Debug)]
pub struct CallStack {
    frames: Vec<Frame>,
    instance: Option<Inst>,
    max_height: usize,
}

impl CallStack {
    fn new(max_height: usize) -> Self {
        Self {
            frames: Vec::new(),
            instance: None,
            max_height,
        }
    }

    fn empty() -> Self {
        Self::new(0)
    }

    fn reset(&mut self) {
        self.frames.clear();
        self.instance = None;
    }

    fn top_start(&self) -> usize {
        let Some(top) = self.top() else { return 0 };
        top.start
    }

    fn top(&self) -> Option<&Frame> {
        self.frames.last()
    }

    fn sync_ip(&mut self, ip: Ip) {
        let Some(top) = self.frames.last_mut() else {
            panic!("must have top call frame")
        };
        top.ip = ip;
    }

    fn prepare_host_frame(&mut self, caller_ip: Option<Ip>) -> usize {
        if let Some(caller_ip) = caller_ip {
            self.sync_ip(caller_ip);
        }
        self.top_start()
    }

    #[inline(always)]
    fn push(
        &mut self,
        caller_ip: Option<Ip>,
        callee_ip: Ip,
        callee_params: BoundedSlotSpan,
        instance: Option<Inst>,
    ) -> Result<usize, TrapCode> {
        if self.frames.len() == self.max_height {
            return Err(TrapCode::StackOverflow);
        }
        match caller_ip {
            Some(caller_ip) => self.sync_ip(caller_ip),
            None => debug_assert!(self.frames.is_empty()),
        }
        let prev_instance = match instance {
            Some(instance) => self.instance.replace(instance),
            None => self.instance,
        };
        let params_offset = usize::from(u16::from(callee_params.span().head()));
        let Some(start) = self.top_start().checked_add(params_offset) else {
            return Err(TrapCode::StackOverflow);
        };
        self.frames.push(Frame {
            ip: callee_ip,
            start,
            instance: prev_instance,
        });
        Ok(start)
    }

    fn pop(&mut self) -> Option<(Ip, usize, Option<Inst>)> {
        let Some(popped) = self.frames.pop() else {
            unsafe { unreachable_unchecked!("call stack must not be empty") }
        };
        let top = self.top()?;
        let ip = top.ip;
        let start = top.start;
        if let Some(instance) = popped.instance {
            self.instance = Some(instance);
        }
        Some((ip, start, popped.instance))
    }

    #[inline(always)]
    fn replace(&mut self, callee_ip: Ip, instance: Option<Inst>) -> Result<usize, TrapCode> {
        let Some(caller_frame) = self.frames.last_mut() else {
            unsafe { unreachable_unchecked!("missing caller frame on the call stack") }
        };
        let prev_instance = match instance {
            Some(instance) => self.instance.replace(instance),
            None => self.instance,
        };
        let start = caller_frame.start;
        *caller_frame = Frame {
            start,
            ip: callee_ip,
            instance: prev_instance,
        };
        Ok(start)
    }
}

#[derive(Debug)]
pub struct Frame {
    pub ip: Ip,
    start: usize,
    instance: Option<Inst>,
}
