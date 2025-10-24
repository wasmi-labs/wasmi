use crate::{
    collections::HeadVec,
    core::{ReadAs, UntypedVal, WriteAs},
    engine::{
        executor::{handler::utils::extract_mem0, CodeMap},
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

#[derive(Debug, Default, Copy, Clone)]
pub struct Done {
    _priv: (),
}

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
        assert!(self.done_reason.is_none());
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

    pub fn push_frame(
        &mut self,
        caller_ip: Option<Ip>,
        callee_ip: Ip,
        params: BoundedSlotSpan,
        size: usize,
        instance: Option<NonNull<InstanceEntity>>,
    ) -> Result<Sp, TrapCode> {
        let delta = usize::from(u16::from(params.span().head()));
        let len_params = params.len();
        let Some(start) = self.frames.top_start().checked_add(delta) else {
            return Err(TrapCode::StackOverflow);
        };
        let sp = self.values.push(start, size, len_params)?;
        self.frames.push(caller_ip, callee_ip, start, instance)?;
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

    fn push(&mut self, start: usize, size: usize, len_params: u16) -> Result<Sp, TrapCode> {
        debug_assert!(usize::from(len_params) <= size);
        let Some(end) = start.checked_add(size) else {
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
}

#[derive(Debug)]
pub struct CallStack {
    frames: Vec<Frame>,
    instances: HeadVec<NonNull<InstanceEntity>>,
    max_height: usize,
}

impl CallStack {
    fn new(max_height: usize) -> Self {
        Self {
            frames: Vec::new(),
            instances: HeadVec::default(),
            max_height,
        }
    }

    fn empty() -> Self {
        Self {
            frames: Vec::new(),
            instances: HeadVec::default(),
            max_height: 0,
        }
    }

    fn reset(&mut self) {
        self.frames.clear();
        self.instances.clear();
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
        let changes_instance = instance.is_some();
        if let Some(instance) = instance {
            self.instances.push(instance);
        }
        self.frames.push(Frame {
            ip: callee_ip,
            start,
            changes_instance,
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
        let instance = popped.changes_instance.then(|| {
            self.instances
                .pop()
                .expect("must have an instance if changed");
            self.instances
                .last()
                .copied()
                .expect("must have another instance since frame stack is non-empty")
        });
        Some((ip, start, instance))
    }
}

#[derive(Debug)]
pub struct Frame {
    pub ip: Ip,
    start: usize,
    changes_instance: bool,
}
