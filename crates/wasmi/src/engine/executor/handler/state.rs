use crate::{
    core::{ReadAs, UntypedVal, WriteAs},
    engine::{
        executor::{
            handler::{
                dispatch::{Control, ExecutionOutcome},
                utils::extract_mem0,
            },
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
    cmp,
    mem,
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
            DoneReason::Error(error) => error.into(),
        };
        Err(outcome)
    }
}

#[derive(Debug)]
pub enum DoneReason {
    /// The execution finished successfully with a result found at the [`Sp`].
    Return(Sp),
    /// A resumable error indicating an error returned by a called host function.
    Host(ResumableHostTrapError),
    /// A resumable error indicating that the execution ran out of fuel.
    OutOfFuel(ResumableOutOfFuelError),
    /// A non-resumable error.
    Error(Error),
}

impl DoneReason {
    #[cold]
    #[inline]
    pub fn error(error: Error) -> Self {
        Self::Error(error)
    }

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
            // Note: it is fine to use <= here because for zero sized frames
            //       we sometimes end up with `start == cells.len()` which isn't
            //       bad since in those cases `Sp` is never used.
            start <= cells.len(),
            "start = {}, cells.len() = {}",
            start,
            cells.len()
        );
        let value = unsafe { cells.as_mut_ptr().add(start) };
        Self { value }
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

type ReturnCallHost = Control<(Ip, Sp, Inst), Sp>;

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

    pub fn sync_ip(&mut self, ip: Ip) {
        self.frames.sync_ip(ip);
    }

    pub fn restore_frame(&mut self) -> (Ip, Sp, Inst) {
        let Some((ip, start, instance)) = self.frames.restore_frame() else {
            panic!("restore_frame: missing top-frame")
        };
        let sp = self.values.sp_or_dangling(start);
        (ip, sp, instance)
    }

    pub fn return_prepare_host_frame<'a>(
        &'a mut self,
        callee_params: BoundedSlotSpan,
        results_len: u16,
        caller_instance: Inst,
    ) -> Result<(ReturnCallHost, FuncInOut<'a>), TrapCode> {
        let (callee_start, caller) = self.frames.return_prepare_host_frame(caller_instance);
        self.values
            .return_prepare_host_frame(caller, callee_start, callee_params, results_len)
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
        debug_assert!(min_height <= max_height);
        // We need to convert from `size_of<Cell>`` to `size_of<u8>`:
        let sizeof_cell = mem::size_of::<UntypedVal>();
        let min_height = min_height / sizeof_cell;
        let max_height = max_height / sizeof_cell;
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

    fn sp_or_dangling(&mut self, start: usize) -> Sp {
        match self.cells.is_empty() {
            true => {
                debug_assert_eq!(start, 0);
                Sp::dangling()
            }
            false => self.sp(start),
        }
    }

    /// Grows the number of cells to `new_len` if the current number is less than `new_len`.
    ///
    /// Does nothing if the number of cells is already at least `new_len`.
    ///
    /// # Errors
    ///
    /// - Returns [`TrapCode::OutOfSystemMemory`] if the machine ran out of memory.
    /// - Returns [`TrapCode::StackOverflow`] if this exceeds the stack's predefined limits.
    fn grow_if_needed(&mut self, new_len: usize) -> Result<(), TrapCode> {
        if new_len > self.max_height {
            return Err(TrapCode::StackOverflow);
        }
        let capacity = self.cells.capacity();
        let len = self.cells.len();
        if new_len > capacity {
            debug_assert!(
                self.cells.len() <= self.cells.capacity(),
                "capacity must always be larger or equal to the actual number of the cells"
            );
            let additional = new_len - len;
            self.cells
                .try_reserve(additional)
                .map_err(|_| TrapCode::OutOfSystemMemory)?;
            debug_assert!(
                self.cells.capacity() >= new_len,
                "capacity must now be at least as large as `new_len` ({new_len}) but found {}",
                self.cells.capacity()
            );
        }
        let max_len = cmp::max(new_len, len);
        // Safety: there is no need to initialize the cells since we are operating
        //         on `UntypedVal` which only has valid bit patterns.
        // Note: non-security related initialization of function parameters
        //       and zero-initialization of function locals happens elsewhere.
        unsafe { self.cells.set_len(max_len) };
        Ok(())
    }

    /// # Note
    ///
    /// In the following code, `callee` represents the called host function frame
    /// and `caller` represents the caller of the caller of the host function, a.k.a.
    /// the caller's caller.
    fn return_prepare_host_frame<'a>(
        &'a mut self,
        caller: Option<(Ip, usize, Inst)>,
        callee_start: usize,
        callee_params: BoundedSlotSpan,
        results_len: u16,
    ) -> Result<(ReturnCallHost, FuncInOut<'a>), TrapCode> {
        let caller_start = caller.map(|(_, start, _)| start).unwrap_or(0);
        let params_offset = usize::from(u16::from(callee_params.span().head()));
        let params_len = usize::from(callee_params.len());
        let results_len = usize::from(results_len);
        let callee_size = params_len.max(results_len);
        if callee_size == 0 {
            let sp = match caller {
                Some(_) if caller_start != callee_start => self.sp(caller_start),
                _ => Sp::dangling(),
            };
            let inout = FuncInOut::new(&mut [], 0, 0);
            let control = match caller {
                Some((ip, _, instance)) => ReturnCallHost::Continue((ip, sp, instance)),
                None => ReturnCallHost::Break(sp),
            };
            return Ok((control, inout));
        }
        let Some(params_start) = callee_start.checked_add(params_offset) else {
            return Err(TrapCode::StackOverflow);
        };
        let Some(params_end) = params_start.checked_add(params_len) else {
            return Err(TrapCode::StackOverflow);
        };
        self.cells
            .copy_within(params_start..params_end, callee_start);
        let Some(callee_end) = callee_start.checked_add(callee_size) else {
            return Err(TrapCode::StackOverflow);
        };
        self.grow_if_needed(callee_end)?;
        let caller_sp = self.sp(caller_start);
        let cells = &mut self.cells[callee_start..callee_end];
        let inout = FuncInOut::new(cells, params_len, results_len);
        let control = match caller {
            Some((ip, _, instance)) => ReturnCallHost::Continue((ip, caller_sp, instance)),
            None => ReturnCallHost::Break(caller_sp),
        };
        Ok((control, inout))
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
        self.grow_if_needed(callee_end)?;
        let sp = self.sp(caller_start);
        let cells = &mut self.cells[callee_start..callee_end];
        let inout = FuncInOut::new(cells, params_len, results_len);
        Ok((sp, inout))
    }

    #[inline(always)]
    fn push(&mut self, start: usize, len_slots: usize, len_params: u16) -> Result<Sp, TrapCode> {
        let len_params = usize::from(len_params);
        debug_assert!(len_params <= len_slots);
        if len_slots == 0 {
            return Ok(Sp::dangling());
        }
        let Some(end) = start.checked_add(len_slots) else {
            return Err(TrapCode::StackOverflow);
        };
        self.grow_if_needed(end)?;
        let start_locals = start.wrapping_add(len_params);
        self.cells[start_locals..end].fill_with(UntypedVal::default);
        let sp = self.sp(start);
        Ok(sp)
    }

    #[inline(always)]
    fn replace(
        &mut self,
        callee_start: usize,
        callee_size: usize,
        callee_params: BoundedSlotSpan,
    ) -> Result<Sp, TrapCode> {
        let params_len = usize::from(callee_params.len());
        let params_start = usize::from(u16::from(callee_params.span().head()));
        let params_end = params_start.wrapping_add(params_len);
        if callee_size == 0 {
            return Ok(Sp::dangling());
        }
        let Some(callee_end) = callee_start.checked_add(callee_size) else {
            return Err(TrapCode::StackOverflow);
        };
        self.grow_if_needed(callee_end)?;
        let Some(callee_cells) = self.cells.get_mut(callee_start..) else {
            unsafe { unreachable_unchecked!("ValueStack::replace: out of bounds callee cells") }
        };
        callee_cells.copy_within(params_start..params_end, 0);
        callee_cells[params_len..callee_size].fill_with(UntypedVal::default);
        let sp = self.sp(callee_start);
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

    fn restore_frame(&self) -> Option<(Ip, usize, Inst)> {
        let instance = self.instance?;
        let top = self.top()?;
        Some((top.ip, top.start, instance))
    }

    fn prepare_host_frame(&mut self, caller_ip: Option<Ip>) -> usize {
        if let Some(caller_ip) = caller_ip {
            self.sync_ip(caller_ip);
        }
        self.top_start()
    }

    /// # Note
    ///
    /// In the following code, `callee` represents the called host function frame
    /// and `caller` represents the caller of the caller of the host function, a.k.a.
    /// the caller's caller.
    pub fn return_prepare_host_frame(
        &mut self,
        callee_instance: Inst,
    ) -> (usize, Option<(Ip, usize, Inst)>) {
        let callee_start = self.top_start();
        let caller = match self.pop() {
            Some((ip, start, instance)) => {
                let instance = instance.unwrap_or(callee_instance);
                Some((ip, start, instance))
            }
            None => None,
        };
        (callee_start, caller)
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
