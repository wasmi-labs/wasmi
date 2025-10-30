use crate::{
    core::{ReadAs, UntypedVal, WriteAs},
    engine::{
        executor::{handler::utils::extract_mem0, CodeMap},
        utils::unreachable_unchecked,
        StackConfig,
    },
    errors::HostError,
    instance::InstanceEntity,
    ir::{self, BoundedSlotSpan, Slot},
    store::PrunedStore,
    Error,
    TrapCode,
};
use alloc::{boxed::Box, vec::Vec};
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

    pub fn done(&mut self, reason: DoneReason) {
        debug_assert!(self.done_reason.is_none());
        self.done_reason = Some(reason);
    }

    pub fn done_reason(&self) -> Option<&DoneReason> {
        self.done_reason.as_ref()
    }

    pub fn into_done_reason(self) -> Option<DoneReason> {
        self.done_reason
    }
}

#[derive(Debug)]
pub enum DoneReason {
    Return(Sp),
    Trap(TrapCode),
    OutOfFuel { required_fuel: u64 },
    Host(Box<dyn HostError>),
    CompileError(Error),
}

impl From<TrapCode> for DoneReason {
    fn from(trap_code: TrapCode) -> Self {
        Self::Trap(trap_code)
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
        debug_assert!(start < cells.len());
        Self {
            value: unsafe { cells.as_mut_ptr().add(start) },
        }
    }

    pub fn null() -> Self {
        Self {
            value: ptr::null_mut(),
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

    #[inline(always)]
    pub fn push_frame(
        &mut self,
        caller_ip: Option<Ip>,
        callee_ip: Ip,
        callee_params: BoundedSlotSpan,
        callee_size: usize,
        callee_instance: Option<NonNull<InstanceEntity>>,
    ) -> Result<Sp, TrapCode> {
        let delta = usize::from(u16::from(callee_params.span().head()));
        let len_params = callee_params.len();
        let Some(start) = self.frames.top_start().checked_add(delta) else {
            return Err(TrapCode::StackOverflow);
        };
        let sp = self.values.push(start, callee_size, len_params)?;
        self.frames.push(caller_ip, callee_ip, start, callee_instance)?;
        Ok(sp)
    }

    pub fn pop_frame(
        &mut self,
        store: &mut PrunedStore,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: NonNull<InstanceEntity>,
    ) -> Option<(Ip, Sp, Mem0Ptr, Mem0Len, NonNull<InstanceEntity>)> {
        let (ip, start, changed_instance) = self.frames.pop()?;
        let sp = self.values.sp(start);
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
        callee_instance: Option<NonNull<InstanceEntity>>,
    ) -> Result<Sp, TrapCode> {
        let params_start = usize::from(u16::from(callee_params.span().head()));
        let params_len = callee_params.len();
        let start = self.frames.replace(callee_ip, callee_instance)?;
        let sp = self
            .values
            .replace(start, callee_size, params_start, params_len)?;
        Ok(sp)
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
        Sp::new(&mut self.cells, start)
    }

    #[inline(always)]
    fn push(&mut self, start: usize, len_slots: usize, len_params: u16) -> Result<Sp, TrapCode> {
        debug_assert!(usize::from(len_params) <= len_slots);
        if len_slots == 0 {
            return Ok(Sp::null());
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
        params_start: usize,
        params_len: u16,
    ) -> Result<Sp, TrapCode> {
        debug_assert!(params_start <= len_slots);
        debug_assert!(params_start + usize::from(params_len) <= len_slots);
        if len_slots == 0 {
            return Ok(Sp::null());
        }
        let Some(end) = start.checked_add(len_slots) else {
            return Err(TrapCode::StackOverflow);
        };
        if end > self.max_height {
            return Err(TrapCode::StackOverflow);
        }
        self.cells.resize_with(end, UntypedVal::default);
        let Some(cells) = self.cells.get_mut(start..end) else {
            unsafe { unreachable_unchecked!() }
        };
        let params_end = params_start.wrapping_add(usize::from(params_len));
        cells.copy_within(params_start..params_end, 0);
        let locals_start = start.wrapping_add(usize::from(params_len));
        cells[locals_start..].fill_with(UntypedVal::default);
        let sp = self.sp(start);
        Ok(sp)
    }
}

#[derive(Debug)]
pub struct CallStack {
    frames: Vec<Frame>,
    instance: Option<NonNull<InstanceEntity>>,
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

    #[inline(always)]
    fn push(
        &mut self,
        caller_ip: Option<Ip>,
        callee_ip: Ip,
        start: usize,
        instance: Option<NonNull<InstanceEntity>>,
    ) -> Result<(), TrapCode> {
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
        self.frames.push(Frame {
            ip: callee_ip,
            start,
            instance: prev_instance,
        });
        Ok(())
    }

    fn pop(&mut self) -> Option<(Ip, usize, Option<NonNull<InstanceEntity>>)> {
        let Some(popped) = self.frames.pop() else {
            panic!("unexpected empty frame stack") // TODO: return `Result` instead of panicking
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
    fn replace(
        &mut self,
        callee_ip: Ip,
        instance: Option<NonNull<InstanceEntity>>,
    ) -> Result<usize, TrapCode> {
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
    instance: Option<NonNull<InstanceEntity>>,
}
