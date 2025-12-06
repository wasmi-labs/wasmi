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
    marker::PhantomData,
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

    pub fn take_done_reason(&mut self) -> DoneReason {
        let Some(reason) = self.done_reason.take() else {
            panic!("missing break reason")
        };
        reason
    }

    pub fn execution_outcome(&mut self) -> Result<Sp, ExecutionOutcome> {
        self.take_done_reason().into_execution_outcome()
    }
}

/// The reason why a Wasmi execution has halted.
///
/// # Note
///
/// This type lives in the [`VmState`] type and in case of a halt needs to be
/// updated manually which is a bit costly which is why the most common reason
/// which is a raised [`TrapCode`] is not included in this `enum` and was put
/// into the return type of execution handlers directly, instead.
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
    /// The execution halted due to a generic [`Error`].
    #[cold]
    #[inline]
    pub fn error(error: Error) -> Self {
        Self::Error(error)
    }

    /// The executed halted because a called host function yielded an error.
    ///
    /// # Note
    ///
    /// This needs special treatment due to resumable function calls.
    #[cold]
    #[inline]
    pub fn host_error(error: Error, func: Func, results: SlotSpan) -> Self {
        Self::Host(ResumableHostTrapError::new(error, func, results))
    }

    /// The executed halted because the execution ran out of fuel.
    ///
    /// # Note
    ///
    /// This needs special treatment due to resumable function calls.
    #[cold]
    #[inline]
    pub fn out_of_fuel(required_fuel: u64) -> Self {
        Self::OutOfFuel(ResumableOutOfFuelError::new(required_fuel))
    }

    /// Converts `self` into an [`ExecutionOutcome`].
    #[inline]
    pub fn into_execution_outcome(self) -> Result<Sp, ExecutionOutcome> {
        let outcome = match self {
            DoneReason::Return(sp) => return Ok(sp),
            DoneReason::Host(error) => error.into(),
            DoneReason::OutOfFuel(error) => error.into(),
            DoneReason::Error(error) => error.into(),
        };
        Err(outcome)
    }
}

/// A thin-wrapper around a non-owned [`InstanceEntity`].
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Inst {
    /// The underlying reference to the [`InstanceEntity`].
    value: NonNull<InstanceEntity>,
    /// Indicates to the compiler that this type is similar in behavior as
    /// a non-owning, non-lifetime restricted `*const InstanceEntity` type.
    marker: PhantomData<*const InstanceEntity>,
}

impl From<&'_ InstanceEntity> for Inst {
    fn from(entity: &'_ InstanceEntity) -> Self {
        Self {
            value: entity.into(),
            marker: PhantomData,
        }
    }
}

impl PartialEq for Inst {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl Eq for Inst {}

impl Inst {
    /// Returns a shared reference to the referenced [`InstanceEntity`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// - The [`Inst`] was constructed from a valid, properly aligned
    ///   `InstanceEntity` pointer.
    /// - The referenced [`InstanceEntity`] remains alive and is not
    ///   mutably accessed for the entire duration of the returned
    ///   reference.
    pub unsafe fn as_ref(&self) -> &InstanceEntity {
        unsafe { self.value.as_ref() }
    }
}

/// # Safety
///
/// It is safe to send `Inst` to another thread because:
/// - The `InstanceEntity` behind the pointer is itself `Send`.
/// - `Inst` only allows shared (`&`) access to the `InstanceEntity` through its API.
/// - There is no interior mutability that could cause data races.
unsafe impl Send for Inst {}

/// # Safety
///
/// It is safe to share `&Inst` across threads because:
/// - All access to the `InstanceEntity` through `Inst` is immutable.
/// - `InstanceEntity` is `Sync`.
/// - The pointer will not be mutated, preventing data races.
unsafe impl Sync for Inst {}

mod inst_tests {
    // Note: the `Send` and `Sync` impl for `Inst` is only valid if
    //       `InstanceEntity` is `Send` and `Sync`.
    //
    // Below are compile-time tests, thus they are not just run with
    // `cargo test` but with any compilation of the `wasmi` crate.
    // Compilation would fail if `InstanceEntity` no longer implements
    // `Send` or `Sync`.
    use super::*;

    const _: fn() = || {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<InstanceEntity>();
        assert_sync::<InstanceEntity>();
        assert_send::<Inst>();
        assert_sync::<Inst>();
    };
}

/// The data pointer to the default Wasm linear memory at index 0.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Mem0Ptr(*mut u8);

impl From<*mut u8> for Mem0Ptr {
    fn from(value: *mut u8) -> Self {
        Self(value)
    }
}

/// The length in bytes of the default Wasm linear memory at index 0.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Mem0Len(usize);

impl From<usize> for Mem0Len {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// Construct the default linear memory slice of bytes from its raw parts.
pub fn mem0_bytes<'a>(mem0: Mem0Ptr, mem0_len: Mem0Len) -> &'a mut [u8] {
    unsafe { slice::from_raw_parts_mut(mem0.0, mem0_len.0) }
}

/// The instruction pointer.
///
/// This always points to the currently executed instruction (or operator).
///
/// # Note
///
/// The pointer points to a `u8` since [`Op`](crate::ir::Op)s in Wasmi are
/// encoded and need to be decoded prior to execution.
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

impl Ip {
    /// Decodes a value of type `T` from the instruction stream at the [`Ip`].
    ///
    /// # Returns
    ///
    /// - This returns the advanced [`Ip`] together with the decoded value of type `T`.
    /// - The returned [`Ip`] points to the first byte immediately following the decoded value.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that [`Ip`] points to the start of a valid
    /// encoding of `T` and that the underlying instruction sequence remains
    /// readable for the full duration of the decode, including any bytes consumed
    /// by `T`.
    ///
    /// The behavior of this operation is undefined if:
    ///
    /// - The instruction sequence does not contain a valid encoding of `T` at [`Ip`].
    /// - Decoding `T` would read past the end of the instruction sequence.
    /// - The underlying memory is invalid, or no longer alive while decoding.
    #[inline]
    pub unsafe fn decode<T: ir::Decode>(self) -> (Ip, T) {
        struct IpDecoder(Ip);
        impl ir::Decoder for IpDecoder {
            fn read_bytes(&mut self, buffer: &mut [u8]) -> Result<(), ir::DecodeError> {
                let src = self.0.value;
                let dst = buffer.as_mut_ptr();
                let len = buffer.len();
                unsafe { ptr::copy_nonoverlapping(src, dst, len) };
                self.0 = unsafe { self.0.add(buffer.len()) };
                Ok(())
            }
        }

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

    /// Advances [`Ip`] past a value of type `T` without decoding it.
    ///
    /// # Note
    ///
    /// This is equivalent to calling [`Self::decode`] and discarding the decoded value,
    /// and may be used when the value is not needed.
    ///
    /// # Safety
    ///
    /// The caller must ensure that offsetting [`Ip`] by `delta` bytes does
    /// not move it outside the valid bounds of the instruction sequence
    /// and that any subsequent use of the returned [`Ip`] only reads from valid,
    /// alive memory.
    pub unsafe fn skip<T: ir::Decode>(self) -> Ip {
        let (ip, _) = unsafe { self.decode::<T>() };
        ip
    }

    /// Returns a new [`Ip`] offset by `delta` bytes from this one.
    ///
    /// # Note
    ///
    /// - This method performs no bounds checking.
    /// - A positive `delta` moves the pointer forward, a negative `delta` moves it backward.
    ///
    /// # Safety
    ///
    /// The caller must ensure that offsetting [`Ip`] by `delta` bytes does
    /// not move it outside the valid bounds of the instruction sequence
    /// and that any subsequent use of the returned [`Ip`] only reads from valid,
    /// alive memory.
    pub unsafe fn offset(self, delta: isize) -> Self {
        let value = unsafe { self.value.byte_offset(delta) };
        Self { value }
    }

    /// Returns a new [`Ip`] advanced by `delta` bytes.
    ///
    /// # Note
    ///
    /// This method performs no bounds checking.
    ///
    /// # Safety
    ///
    /// The caller must ensure that advancing [`Ip`] by `delta` bytes does
    /// not move it outside the valid bounds of the instruction sequence
    /// and that any subsequent use of the returned [`Ip`] only reads from valid,
    /// alive memory.
    pub unsafe fn add(self, delta: usize) -> Self {
        let value = unsafe { self.value.byte_add(delta) };
        Self { value }
    }
}

/// The stack pointer.
///
/// # Note
///
/// This always points to the beginning of the stack area reserved for the
/// currently executed function frame.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Sp {
    value: *mut UntypedVal,
}

impl Sp {
    /// Creates a new [`Sp`].
    #[inline]
    pub fn new(value: *mut UntypedVal) -> Self {
        Self { value }
    }

    /// Creates a new dangling [`Sp`].
    ///
    /// # Note
    ///
    /// The [`Sp`] returned by this method must never be dereferenced.
    /// This is used for cases where there are no frames on the call stack.
    pub fn dangling() -> Self {
        Self {
            value: ptr::dangling_mut(),
        }
    }

    /// Returns a value of type `T` at `slot`.
    pub fn get<T>(self, slot: Slot) -> T
    where
        UntypedVal: ReadAs<T>,
    {
        let index = usize::from(u16::from(slot));
        let value = unsafe { &*self.value.add(index) };
        <UntypedVal as ReadAs<T>>::read_as(value)
    }

    /// Writes a `value` of type `T` at `slot`.
    pub fn set<T>(self, slot: Slot, value: T)
    where
        UntypedVal: WriteAs<T>,
    {
        let index = usize::from(u16::from(slot));
        let cell = unsafe { &mut *self.value.add(index) };
        <UntypedVal as WriteAs<T>>::write_as(cell, value);
    }

    /// Converts `self` to a slice of cells with length `len`.
    pub unsafe fn as_slice<'a>(self, len: usize) -> &'a [UntypedVal] {
        unsafe { core::slice::from_raw_parts(self.value, len) }
    }
}

/// The Wasmi stack.
///
/// This combines both value stack and call stack and provides a common API
/// to interact with both.
#[derive(Debug)]
pub struct Stack {
    /// The underlying value stack.
    values: ValueStack,
    /// The underlying call stack.
    frames: CallStack,
}

type ReturnCallHost = Control<(Ip, Sp, Inst), Sp>;

impl Stack {
    /// Creates a new [`Stack`] with the given [`StackConfig`] limits.
    pub fn new(config: &StackConfig) -> Self {
        Self {
            values: ValueStack::new(config.min_stack_height(), config.max_stack_height()),
            frames: CallStack::new(config.max_recursion_depth()),
        }
    }

    /// Creates a new [`Stack`] without heap allocations.
    pub fn empty() -> Self {
        Self {
            values: ValueStack::empty(),
            frames: CallStack::empty(),
        }
    }

    /// Resets `self` for reuse.
    pub fn reset(&mut self) {
        self.values.reset();
        self.frames.reset();
    }

    /// Returns the total number of heap allocated bytes of `self`.
    pub fn bytes_allocated(&self) -> usize {
        // Note: we use saturating add since this API is only used to separate
        //       heap allocating from non-heap allocating instances.
        self.values
            .bytes_allocates()
            .saturating_add(self.frames.bytes_allocates())
    }

    /// Synchronizes the [`Ip`] of the top-most function frame.
    ///
    /// # Note
    ///
    /// - Usually the current [`Ip`] is stored outside of the [`Stack`].
    /// - Synchronization is required when calling another function or when
    ///   finishing a resumable call in order to be able to resume execution
    ///   at that point later.
    pub fn sync_ip(&mut self, ip: Ip) {
        self.frames.sync_ip(ip);
    }

    /// Restores the top-most function frame and its [`Ip`], [`Sp`] and [`Inst`].
    ///
    /// # Note
    ///
    /// This is useful and required to resume a function execution that yielded back to the host.
    pub fn restore_frame(&mut self) -> (Ip, Sp, Inst) {
        let Some((ip, start, instance)) = self.frames.restore_frame() else {
            panic!("restore_frame: missing top-frame")
        };
        let sp = self.values.sp_or_dangling(start);
        (ip, sp, instance)
    }

    /// Prepares `self` for a host function tail call.
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

    /// Prepares `self` for a host function call.
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

    /// Adjusts `self` for a normal function call.
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

    /// Adjusts `self` after returning from a function.
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

    /// Adjusts `self` for a function tail call.
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

/// The value stack.
///
/// The Wasmi value stack is organized in 64-bit cells
/// where each is associated to a single function frame.
///
/// Cells can be read from and written to via [`Slot`]s.
///
/// # Note
///
/// - A [`ValueStack`] has a maximum height which it cannot exceed.
/// - A [`ValueStack`] can only grow (via [`ValueStack::grow_if_needed`]) and never shrink.
#[derive(Debug)]
pub struct ValueStack {
    /// The cells of the value stack.
    cells: Vec<UntypedVal>,
    /// The maximum height of the value stack.
    max_height: usize,
}

impl ValueStack {
    /// Create a new [`ValueStack`] with the minimum and maximum height limits.
    fn new(min_height: usize, max_height: usize) -> Self {
        debug_assert!(min_height <= max_height);
        // We need to convert from `size_of<Cell>`` to `size_of<u8>`:
        let sizeof_cell = mem::size_of::<UntypedVal>();
        let min_height = min_height / sizeof_cell;
        let max_height = max_height / sizeof_cell;
        let cells = Vec::with_capacity(min_height);
        Self { cells, max_height }
    }

    /// Create an empty [`ValueStack`] which uses no heap allocations.
    fn empty() -> Self {
        Self {
            cells: Vec::new(),
            max_height: 0,
        }
    }

    /// Reset `self` for reuse.
    fn reset(&mut self) {
        self.cells.clear();
    }

    /// Returns the number of heap allocated bytes of `self`.
    ///
    /// # Note
    ///
    /// This is mostly used to separate instances with and without heap allocations for caching.
    fn bytes_allocates(&self) -> usize {
        let bytes_per_frame = mem::size_of::<UntypedVal>();
        self.cells.capacity() * bytes_per_frame
    }

    /// Returns an [`Sp`] pointing to the cell at the `start` index.
    fn sp(&mut self, start: usize) -> Sp {
        debug_assert!(
            // Note: it is fine to use <= here because for zero sized frames
            //       we sometimes end up with `start == cells.len()` which isn't
            //       bad since in those cases `Sp` is never used.
            start <= self.cells.len(),
            "start = {}, cells.len() = {}",
            start,
            self.cells.len()
        );
        let value = unsafe { self.cells.as_mut_ptr().add(start) };
        Sp::new(value)
    }

    /// Returns an [`Sp`] pointing to the cell at the `start` index if `self` is non-empty.
    ///
    /// Otherwise returns a dangling [`Sp`] that must not be derefenced.
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

    /// Prepares `self` for a host function tail call.
    ///
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

    /// Prepares `self` for a host function call.
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

    /// Adjusts `self` for a normal function call.
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

    /// Adjusts `self` for a function tail call.
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

/// The Wasmi call stack.
///
/// This holds all the information about function frames that are on the call stack.
/// Additionally it keeps track of the [`Inst`] that is currently in use.
///
/// # Note
///
/// - A [`CallStack`] has a maximum height which it cannot exceed.
#[derive(Debug)]
pub struct CallStack {
    /// The stack of function frames.
    frames: Vec<Frame>,
    /// The currently used [`Inst`] if any.
    ///
    /// This may be `None`, for example if the [`CallStack`] is empty.
    instance: Option<Inst>,
    /// The maximum height of the call stack.
    max_height: usize,
}

impl CallStack {
    /// Creates a new [`CallStack`] with the given maximum height.
    fn new(max_height: usize) -> Self {
        Self {
            frames: Vec::new(),
            instance: None,
            max_height,
        }
    }

    /// Returns the number of heap allocated bytes of `self`.
    ///
    /// # Note
    ///
    /// This is mostly used to separate instances with and without heap allocations for caching.
    fn bytes_allocates(&self) -> usize {
        let bytes_per_frame = mem::size_of::<Frame>();
        self.frames.capacity() * bytes_per_frame
    }

    /// Creates an empty [`CallStack`] which uses no heap allocations.
    fn empty() -> Self {
        Self::new(0)
    }

    /// Resets `self` for reuse.
    fn reset(&mut self) {
        self.frames.clear();
        self.instance = None;
    }

    /// Returns the `start` index of the top-most function frame.
    ///
    /// Returns 0 if `self` is empty.
    fn top_start(&self) -> usize {
        let Some(top) = self.top() else { return 0 };
        top.start
    }

    /// Returns a shared reference to the top-most function frame if any.
    ///
    /// Returns `None` if `self` is empty.
    fn top(&self) -> Option<&Frame> {
        self.frames.last()
    }

    /// Synchronizes the [`Ip`] of the top-most function frame.
    ///
    /// # Note
    ///
    /// - Usually the current [`Ip`] is stored outside of the [`CallStack`].
    /// - Synchronization is required when calling another function or when
    ///   finishing a resumable call in order to be able to resume execution
    ///   at that point later.
    fn sync_ip(&mut self, ip: Ip) {
        let Some(top) = self.frames.last_mut() else {
            panic!("must have top call frame")
        };
        top.ip = ip;
    }

    /// Restores the top-most function frame and its [`Ip`], `start` index and [`Inst`].
    ///
    /// # Note
    ///
    /// This is useful and required to resume a function execution that yielded back to the host.
    fn restore_frame(&self) -> Option<(Ip, usize, Inst)> {
        let instance = self.instance?;
        let top = self.top()?;
        Some((top.ip, top.start, instance))
    }

    /// Prepares `self` for a host function call.
    fn prepare_host_frame(&mut self, caller_ip: Option<Ip>) -> usize {
        if let Some(caller_ip) = caller_ip {
            self.sync_ip(caller_ip);
        }
        self.top_start()
    }

    /// Prepares `self` for a host function tail call.
    ///
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

    /// Adjusts `self` for a normal function call.
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

    /// Adjusts `self` after returning from a function.
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

    /// Adjusts `self` for a function tail call.
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

/// The state of a single function frame.
#[derive(Debug)]
pub struct Frame {
    /// The functions [`Ip`].
    ///
    /// # Note
    ///
    /// This needs to be kept in sync for example when calling another function
    /// or yielding back to the host in for resumable calls.
    pub ip: Ip,
    /// The start index on the value stack for this function frame.
    start: usize,
    /// The [`Inst`] used if any.
    ///
    /// # Note
    ///
    /// This is only `Some` if [`Frame`] and its caller originate from different
    /// Wasm instances and thus execution needs to change the currently used [`Inst`].
    instance: Option<Inst>,
}
