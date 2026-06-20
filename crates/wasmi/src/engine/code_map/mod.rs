//! Data structure storing information about compiled functions.
//!
//! # Note
//!
//! This is the data structure specialized to handle compiled
//! register machine based bytecode functions.

mod span;
mod utils;

pub use self::span::{EngineFunc, EngineFuncSpan, EngineFuncSpanIter};
use self::utils::SmallByteSlice;
use super::{FuncTranslationDriver, FuncTranslator, TranslationError, ValidatingFuncTranslator};
use crate::{
    Config,
    Error,
    TrapCode,
    core::{Fuel, FuelCostsProvider, hint},
    engine::{ResumableOutOfFuelError, utils::unreachable_unchecked},
    errors::FuelError,
    module::{FuncIdx, ModuleHeader},
};
use alloc::boxed::Box;
use core::{
    cell::UnsafeCell,
    fmt,
    iter,
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
    ptr::{self, NonNull},
    slice,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
};
use spin::Mutex;
use wasmparser::{FuncToValidate, ValidatorResources, WasmFeatures};

#[cfg(doc)]
use crate::ir::Op;

/// The number of functions stored in the first bucket represented as `log2`.
const LEN_BUCKET0_LOG2: usize = 5;

/// The maximum allowed number of functions in a [`CodeMap`].
const MAX_FUNCS: usize = 100_000_000;

/// The number of functions stored in the first bucket.
///
/// Derived from [`LEN_BUCKET0_LOG2`].
const LEN_BUCKET0: u64 = 1 << LEN_BUCKET0_LOG2;

/// The number of buckets required to satisfy [`LEN_BUCKET0_LOG2`] and [`MAX_FUNCS`].
///
/// Derived from [`MAX_FUNCS`].
const MAX_BUCKETS: usize = Funcs::required_buckets_for_len(MAX_FUNCS);

/// A data structure to store and manage [`FuncEntry`] definitions.
#[derive(Debug)]
pub struct CodeMap {
    /// The append-only, lock-free-readable storage for all [`FuncEntry`] definitions.
    funcs: Funcs,
    /// Serializes concurrent writers ([`Self::alloc_funcs`]); readers never take this lock.
    alloc_lock: Mutex<()>,
    /// Shared Wasm features across all [`FuncEntry`] definitions within the [`CodeMap`].
    features: WasmFeatures,
}

impl CodeMap {
    /// Creates a new [`CodeMap`].
    pub fn new(config: &Config) -> Self {
        Self {
            funcs: Funcs::default(),
            alloc_lock: Mutex::new(()),
            features: config.wasm_features(),
        }
    }

    /// Allocates `amount` new uninitialized [`EngineFunc`] to the [`CodeMap`].
    ///
    /// # Note
    ///
    /// Before using the [`CodeMap`] all [`EngineFunc`]s must be initialized with either of:
    ///
    /// - [`CodeMap::init_func_as_compiled`]
    /// - [`CodeMap::init_func_as_uncompiled`]
    pub fn alloc_funcs(&self, amount: usize) -> EngineFuncSpan {
        // The writer lock serializes concurrent allocators; readers never take it.
        let _guard = self.alloc_lock.lock();
        match self.funcs.alloc_funcs(amount) {
            Ok(span) => span,
            Err(err) => panic!("failed to alloc funcs: {err}"),
        }
    }

    /// Initializes the [`EngineFunc`] with its [`CompiledFuncEntry`].
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`EngineFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`EngineFunc`].
    pub fn init_func_as_compiled(&self, func: EngineFunc, entity: CompiledFuncEntry) {
        let func = match self.funcs.get(func) {
            Some(func) => func,
            None => panic!("missing function entry at: {func:?}"),
        };
        func.init_compiled(entity);
    }

    /// Initializes the [`EngineFunc`] for lazy translation.
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`EngineFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`EngineFunc`].
    pub fn init_func_as_uncompiled(
        &self,
        func: EngineFunc,
        func_idx: FuncIdx,
        bytes: &[u8],
        module: &ModuleHeader,
        func_to_validate: Option<FuncToValidate<ValidatorResources>>,
    ) {
        let func = match self.funcs.get(func) {
            Some(func) => func,
            None => panic!("missing function entry at: {func:?}"),
        };
        func.init_uncompiled(UncompiledFuncEntry::new(
            func_idx,
            bytes,
            module.clone(),
            func_to_validate,
        ));
    }

    /// Returns a cheap, lock-free [`CodeView`] snapshot of this [`CodeMap`].
    ///
    /// # Note
    ///
    /// The snapshot borrows `self` and caches the currently published function count; it is used by
    /// the executor to resolve calls without locking or per-call atomics.
    #[inline]
    pub fn view(&self) -> CodeView<'_> {
        // Note: a single `Acquire` load establishes the happens-before for every bucket published before
        //       this length; subsequent per-call reads through the snapshot are plain (non-atomic).
        let len_funcs = self.funcs.len_funcs.load(Ordering::Acquire);
        CodeView {
            code_map: self,
            len_funcs,
        }
    }
}

/// A cheap, lock-free, stale-but-valid snapshot of a [`CodeMap`].
///
/// Borrows the source [`CodeMap`] and caches the number of published functions, so the executor
/// can resolve functions without locking and without any atomic load on the per-call path.
///
/// # Note
///
/// - Represents a potentially stale view: functions appended after the snapshot are not visible
///   until [`CodeView::refresh`] is called. Since [`Funcs`] is append-only and pointer-stable, a
///   stale view is never invalid, only outdated.
/// - Holding `&CodeMap` is sound under both Stacked and Tree Borrows because no `&mut CodeMap`
///   (or `&mut Funcs`) is ever formed while a view is alive: all publication goes through interior
///   mutability under the writer lock.
/// - Upon execution an [`Engine`] derives a [`CodeView`] of the current [`CodeMap`] state and uses
///   it to drive call-based executions without taking the writer lock. After a host function call
///   the view must be [`refresh`](CodeView::refresh)ed, since the host may have appended functions
///   (e.g. by compiling and instantiating new modules) that the resuming Wasm can reach.
///
/// [`Engine`]: crate::Engine
#[derive(Copy, Clone)]
pub struct CodeView<'a> {
    /// The source [`CodeMap`]. The bucket-array base address and `features` are stable for its
    /// lifetime; only the published function count grows (tracked by `len_funcs`).
    code_map: &'a CodeMap,
    /// Cached number of published [`FuncEntry`] definitions; bounds visibility.
    ///
    /// # Note
    ///
    /// This is only updated by [`CodeView::refresh`], so the per-call read path performs no atomic load.
    len_funcs: usize,
}

impl fmt::Debug for CodeView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodeView")
            .field("len_funcs", &self.len_funcs)
            .finish_non_exhaustive()
    }
}

impl<'a> CodeView<'a> {
    /// Re-materializes the snapshot to reflect functions published since it was last (re)created.
    ///
    /// Must be called after a host function call: the host may have compiled and/or instantiated
    /// new Wasm modules (appending to the [`CodeMap`]) that the resuming Wasm code can then reach.
    #[inline]
    pub fn refresh(&mut self) {
        self.len_funcs = self.code_map.funcs.len_funcs.load(Ordering::Acquire);
    }

    /// Returns a shared reference to the [`FuncEntry`] of `func` if visible in this snapshot.
    ///
    /// Returns `None` if `func` is not (yet) visible to this snapshot.
    #[inline]
    pub fn get_ref(&self, func: EngineFunc) -> Option<&'a FuncEntry> {
        // Safety: `len_funcs` is loaded via acquire upon creation of `self` so that `buckets` are published.
        unsafe { self.code_map.funcs.get_within(func, self.len_funcs) }
    }

    /// Returns the [`CompiledFuncRef`] of `func`, compiling it lazily if still uncompiled.
    ///
    /// # Errors
    ///
    /// - If translation or Wasm validation of `func` failed.
    /// - If `fuel` ran out in case fuel consumption is enabled.
    #[track_caller]
    #[inline]
    pub fn get(
        &self,
        fuel: Option<&mut Fuel>,
        func: EngineFunc,
    ) -> Result<CompiledFuncRef<'a>, Error> {
        let Some(entity) = self.get_ref(func) else {
            panic!("missing function entry at: {func:?}")
        };
        entity.get_or_compile(fuel, &self.code_map.features)
    }

    /// Returns the [`WasmFeatures`] of the underlying [`CodeMap`].
    pub fn features(&self) -> &WasmFeatures {
        &self.code_map.features
    }
}

/// An append-only collection for [`FuncEntry`] definitions.
pub struct Funcs {
    /// The buckets to store the [`FuncEntry`] definitions append-only.
    ///
    /// # Note
    ///
    /// - Each bucket is twice the size of its predecessor.
    /// - The first `required_buckets_for_len(len_funcs)` slots are `Some` and, once published,
    ///   are never written or moved again.
    /// - The `buckets` array lives behind an [`UnsafeCell`] so that new buckets can be
    ///   published through a shared `&Funcs` (no `&mut Funcs` is ever formed), which is what lets a
    ///   [`CodeView`] snapshot read `buckets` lock-free and without atomics on the per-call path.
    buckets: UnsafeCell<[Option<RawFuncsBucket>; MAX_BUCKETS]>,
    /// The number of [`FuncEntry`] definitions published across all `buckets`.
    ///
    /// This doubles as the publication flag:
    ///
    /// - writers stores the new bucket pointers before `Release`-storing `len_funcs`
    /// - readers `Acquire`-loads `len_funcs` before reading any bucket
    ///
    /// Combined, both establish the happens-before that makes the [`UnsafeCell`] reads data-race free.
    /// The number of occupied buckets is derived via [`Funcs::required_buckets_for_len`].
    len_funcs: AtomicUsize,
}

/// Iterator over the occupied buckets of [`Funcs`].
pub struct Buckets<'a> {
    /// The slice of occupied buckets.
    buckets: &'a [Option<RawFuncsBucket>],
    /// The bucket index that is yielded next if any.
    n: usize,
}

impl<'a> Buckets<'a> {
    fn new(funcs: &'a Funcs) -> Self {
        let len_funcs = funcs.len_funcs.load(Ordering::Acquire);
        let len_buckets = Funcs::required_buckets_for_len(len_funcs);
        let base: *const Option<RawFuncsBucket> = funcs.buckets.get().cast();
        // Safety: the `Acquire` load above synchronizes-with the publication of buckets
        //         `[0, len_buckets)`, which are frozen (written once, never again). Building the
        //         slice from a raw pointer of exactly `len_buckets` confines the shared borrow to
        //         that frozen prefix, so it never aliases the tail a concurrent `alloc_funcs` may be
        //         writing.
        let buckets = unsafe { slice::from_raw_parts(base, len_buckets) };
        Self { buckets, n: 0 }
    }
}

impl<'a> Iterator for Buckets<'a> {
    type Item = FuncsBucketRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let bucket = self.buckets.get(self.n)?;
        let raw = bucket.as_ref().copied()?;
        let len_bucket = Funcs::size_of_bucket_at(self.n);
        let bucket = unsafe { FuncsBucketRef::from_raw_parts(raw, len_bucket) };
        self.n += 1;
        Some(bucket)
    }
}

/// [`Debug`](fmt::Debug) wrapper to show buckets as list for [`Funcs`].
pub struct DebugBuckets<'a>(&'a Funcs);
impl fmt::Debug for DebugBuckets<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_list();
        for bucket in self.0.buckets() {
            dbg.entry(&bucket);
        }
        dbg.finish()
    }
}

impl fmt::Debug for Funcs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buckets = DebugBuckets(self);
        f.debug_struct("Funcs")
            .field("buckets", &buckets)
            .field("len_funcs", &self.len_funcs)
            .finish()
    }
}

#[test]
fn size_of_funcs_type() {
    const EXPECTED_SIZE: usize = (MAX_BUCKETS * core::mem::size_of::<*mut FuncEntry>())
        + core::mem::size_of::<AtomicUsize>();
    assert_eq!(core::mem::size_of::<Funcs>(), EXPECTED_SIZE);
}

impl Default for Funcs {
    fn default() -> Self {
        Self {
            buckets: UnsafeCell::new([const { None }; MAX_BUCKETS]),
            len_funcs: AtomicUsize::new(0),
        }
    }
}

impl Funcs {
    /// Allocates `n` new function entities to `self` and returns a span to them.
    ///
    /// # Note
    ///
    /// All function entities allocated this way are initialized to an undefined state.
    ///
    /// # Synchronization
    ///
    /// Must be called under the [`CodeMap`] writer lock so that concurrent allocators are
    /// serialized. New bucket slots are written through a raw element pointer (never via
    /// `&mut Funcs` or `&mut [_]`) and are disjoint from any slot a concurrent reader may
    /// observe (reader slots are `< required_buckets_for_len(start)`).
    fn alloc_funcs(&self, n: usize) -> Result<EngineFuncSpan, Error> {
        let start = self.len_funcs.load(Ordering::Relaxed);
        let end = start
            .checked_add(n)
            .filter(|&end| end <= MAX_FUNCS)
            .unwrap(); // TODO: proper error handling
        let current_buckets = Self::required_buckets_for_len(start);
        let needed = Self::required_buckets_for_len(end);
        let base = self.buckets.get().cast::<Option<RawFuncsBucket>>();
        for n in current_buckets..needed {
            let bucket = FuncsBucket::new(Self::size_of_bucket_at(n));
            let (raw, _len) = bucket.into_raw_parts();
            // Safety: slot `n` is
            //   - freshly allocated
            //   - not yet published
            //   - disjoint from any slot a concurrent reader may observe
            unsafe { base.add(n).write(Some(raw)) };
        }
        // Finally, store the updated `len_funcs` to publish the `self.buckets` changes.
        self.len_funcs.store(end, Ordering::Release);
        Ok(EngineFuncSpan::new(
            EngineFunc::from(start as u32),
            EngineFunc::from(end as u32),
        ))
    }

    /// Returns a shared reference to the [`FuncEntry`] of `func` if published.
    ///
    /// Returns `None` if `func` is out of bounds.
    #[inline]
    pub fn get(&self, func: EngineFunc) -> Option<&FuncEntry> {
        let len_funcs = self.len_funcs.load(Ordering::Acquire);
        // Safety: we just loaded `len_funcs` via acquire so that `buckets` are published.
        unsafe { self.get_within(func, len_funcs) }
    }

    /// Returns a shared reference to the [`FuncEntry`] of `func` if in bounds.
    ///
    ///  # Safety
    ///
    /// - It is the callers responsibility to make sure that `self.len_funcs` is loaded via `Acquire`
    ///   so the buckets backing funcs are guaranteed to be published.
    /// - Furthermore, the caller is responsible to provide `len_funcs` that matches the exact number
    ///   of occupied (`Some`) buckets in `self`.
    #[inline]
    pub unsafe fn get_within(&self, func: EngineFunc, len_funcs: usize) -> Option<&FuncEntry> {
        use crate::core::hint::unlikely;
        if unlikely(u32::from(func) as usize >= len_funcs) {
            return None;
        }
        let (bucket, slot) = Self::locate(func);
        // Safety: `func < len_funcs` implies `bucket` is published; a plain read suffices since the
        //         `Acquire` that produced `len_funcs` established the happens-before with publication.
        let base = self.buckets.get().cast::<Option<RawFuncsBucket>>();
        let Some(raw) = (unsafe { *base.add(bucket) }) else {
            unsafe { unreachable_unchecked!() } // bucket of a contained func is always published
        };
        let bucket_ref =
            unsafe { FuncsBucketRef::from_raw_parts(raw, Self::size_of_bucket_at(bucket)) };
        bucket_ref.get(slot)
    }

    /// Maps a global function `index` to its `(bucket, slot)` position.
    #[inline]
    fn locate(func: EngineFunc) -> (usize, usize) {
        let index = u32::from(func);
        let j = u64::from(index) + LEN_BUCKET0;
        let msb = 63 - j.leading_zeros() as usize; // floor(log2(j))
        let bucket = msb - LEN_BUCKET0_LOG2;
        let slot = (j - (1u64 << msb)) as usize;
        (bucket, slot)
    }

    /// Returns the number of [`FuncsBucket`] required to store `len` functions in total.
    #[inline]
    const fn required_buckets_for_len(len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        let j = (len as u64 - 1) + LEN_BUCKET0;
        let msb = 63 - j.leading_zeros() as usize;
        (msb - LEN_BUCKET0_LOG2) + 1
    }

    /// Returns the number of functions stored in the bucket at index `n`.
    #[inline]
    fn size_of_bucket_at(n: usize) -> usize {
        1usize << (LEN_BUCKET0_LOG2 + n)
    }

    /// Returns an iterator over the occupied buckets of `self`.
    fn buckets(&self) -> Buckets<'_> {
        Buckets::new(self)
    }
}

impl Drop for Funcs {
    fn drop(&mut self) {
        // We have exclusive access in `drop`, so no atomics/`UnsafeCell` synchronization is needed.
        let buckets = self.buckets.get_mut();
        for (n, raw_bucket) in buckets.iter_mut().enumerate() {
            let Some(raw_bucket) = raw_bucket.take() else {
                break;
            };
            let len = Self::size_of_bucket_at(n);
            let bucket = unsafe { FuncsBucket::from_raw_parts(raw_bucket, len) };
            drop(bucket)
        }
    }
}

/// A raw [`FuncsBucket`] that misses length information.
///
/// The information about its length needs to be fed and managed from outside.
#[derive(Copy, Clone)]
pub struct RawFuncsBucket {
    /// The raw pointer to the bucket's [`FuncEntry`] definitions.
    funcs: NonNull<FuncEntry>,
}

/// A fixed-size bucket of [`FuncEntry`] definitions.
pub struct FuncsBucket {
    /// The heap allocation that stores the [`FuncEntry`] definitions of this bucket.
    funcs: Box<[FuncEntry]>,
}

impl FuncsBucket {
    /// Creates a new [`FuncsBucket`] with a fixed `size`.
    #[inline]
    pub fn new(size: usize) -> Self {
        Self {
            funcs: iter::repeat_with(FuncEntry::uninit).take(size).collect(),
        }
    }

    /// Destructs `self` into its raw parts, a [`RawFuncsBucket`] and its length.
    pub fn into_raw_parts(self) -> (RawFuncsBucket, usize) {
        let len = self.funcs.len();
        // Safety: `Box` is guaranteed to be non-null.
        let funcs = unsafe { NonNull::new_unchecked(Box::into_raw(self.funcs) as *mut FuncEntry) };
        (RawFuncsBucket { funcs }, len)
    }

    /// Creates a [`FuncsBucket`] from its raw parts.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to provide a valid `raw` and `len` argument.
    #[inline]
    pub unsafe fn from_raw_parts(raw: RawFuncsBucket, len: usize) -> FuncsBucket {
        let funcs = ptr::slice_from_raw_parts_mut(raw.funcs.as_ptr(), len);
        FuncsBucket {
            funcs: unsafe { Box::from_raw(funcs) },
        }
    }
}

/// A fixed-size bucket of [`FuncEntry`] definitions.
#[derive(Debug)]
pub struct FuncsBucketRef<'a> {
    /// The shared [`FuncEntry`] definitions of this bucket.
    funcs: &'a [FuncEntry],
}

impl<'a> FuncsBucketRef<'a> {
    /// Creates a [`FuncsBucketRef`] from its raw parts.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to provide a valid `raw` and `len` argument.
    #[inline]
    pub unsafe fn from_raw_parts(raw: RawFuncsBucket, len: usize) -> FuncsBucketRef<'a> {
        let funcs = unsafe { slice::from_raw_parts(raw.funcs.as_ptr(), len) };
        Self { funcs }
    }

    /// Returns a shared reference to the [`FuncEntry`] at index `n` if any.
    #[inline]
    pub fn get(&self, n: usize) -> Option<&'a FuncEntry> {
        self.funcs.get(n)
    }
}

/// A function entity in any of its various states.
///
/// # Dev. Note
///
/// We use `#[repr(C)]` to dictate field ordering which is important for the executor
/// since the executes uses direct pointers to [`FuncEntry`] instances once they are
/// known to be compiled.
#[repr(C)]
pub struct FuncEntry {
    /// Payload; which field is active (if any) is determined by `state`.
    data: UnsafeCell<FuncEntryData>,
    /// Synchronization authority *and* discriminant. One of the consts in `state` module below.
    state: AtomicU8,
}

impl Drop for FuncEntry {
    fn drop(&mut self) {
        match self.state.load(Ordering::Acquire) {
            | state::UNINIT | state::COMPILING | state::FAILED_TO_COMPILE => {}
            | state::UNCOMPILED => {
                let data = self.data.get_mut();
                unsafe { ManuallyDrop::drop(&mut data.uncompiled) }
            }
            | state::COMPILED => {
                let data = self.data.get_mut();
                unsafe { ManuallyDrop::drop(&mut data.compiled) }
            }
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for FuncEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = unsafe { self.data_mut() };
        match self.state.load(Ordering::Acquire) {
            state::UNINIT => f.debug_struct("FuncEntry::Uninit").finish(),
            state::COMPILING => f.debug_struct("FuncEntry::Compiling").finish(),
            state::FAILED_TO_COMPILE => f.debug_struct("FuncEntry::FailedToCompile").finish(),
            state::COMPILED => f
                .debug_struct("FuncEntry::Compiled")
                .field("state", unsafe { &state.compiled })
                .finish(),
            state::UNCOMPILED => f
                .debug_struct("FuncEntry::Uncompiled")
                .field("state", unsafe { &state.uncompiled })
                .finish(),
            _ => unreachable!(),
        }
    }
}

impl FuncEntry {
    /// Creates an uninitialized [`FuncEntry`].
    #[inline]
    fn uninit() -> Self {
        Self {
            data: UnsafeCell::new(FuncEntryData { undefined: () }),
            state: AtomicU8::new(state::UNINIT),
        }
    }

    /// Returns `true` if `self` has already been initialized.
    fn is_initialized(&self) -> bool {
        self.state.load(Ordering::Relaxed) != state::UNINIT
    }

    /// Initializes the [`FuncEntry`] with a [`CompiledFuncEntry`].
    ///
    /// # Panics
    ///
    /// If `func` has already been initialized.
    fn init_compiled(&self, compiled: CompiledFuncEntry) {
        assert!(!self.is_initialized(), "func has already been initialized");
        // Safety: exclusive during build; previous union field is `undefined` (no Drop).
        unsafe { self.set_compiled(compiled) }
    }

    /// Initializes the [`FuncEntry`] to an uncompiled state.
    ///
    /// # Panics
    ///
    /// If `func` has already been initialized.
    fn init_uncompiled(&self, uncompiled: UncompiledFuncEntry) {
        assert!(!self.is_initialized(), "func has already been initialized");
        // Safety: exclusive during build; previous union field is `undefined` (no Drop).
        unsafe { self.set_uncompiled(uncompiled) }
    }

    /// Initializes the [`FuncEntry`] to an uncompiled state.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to assert that `self.data` is in the `undefined` state.
    unsafe fn set_compiled(&self, compiled: CompiledFuncEntry) {
        let data = unsafe { self.data_mut() };
        data.compiled = ManuallyDrop::new(compiled);
        self.state.store(state::COMPILED, Ordering::Release);
    }

    /// Initializes the [`FuncEntry`] to an uncompiled state.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to assert that `self.data` is in the `undefined` state.
    unsafe fn set_uncompiled(&self, uncompiled: UncompiledFuncEntry) {
        let data = unsafe { self.data_mut() };
        data.uncompiled = ManuallyDrop::new(uncompiled);
        self.state.store(state::UNCOMPILED, Ordering::Release);
    }

    /// Takes [`UncompiledFuncEntry`] from `self.data` and leaves behind an `undefined` state.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to ensure that `self.data` has been is in `uncompiled` state.
    unsafe fn take_uncompiled(&self) -> UncompiledFuncEntry {
        let data = unsafe { self.data_mut() };
        let uncompiled = unsafe { ManuallyDrop::take(&mut data.uncompiled) };
        data.undefined = ();
        uncompiled
    }

    /// Returns an exclusive reference to the [`FuncEntryData`] of `self`.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility that no other references to this data are alive.
    #[allow(clippy::mut_from_ref)] // same API as `UnsafeCell::as_mut_unchecked`
    unsafe fn data_mut(&self) -> &mut FuncEntryData {
        unsafe { &mut *self.data.get() }
    }

    /// Compiles `self` and returns a view to the [`CompiledFuncRef`].
    ///
    /// # Errors
    ///
    /// - If translation or Wasm validation of `func` failed.
    /// - If `ctx` ran out of fuel in case fuel consumption is enabled.
    #[inline]
    pub fn get_or_compile(
        &self,
        fuel: Option<&mut Fuel>,
        features: &WasmFeatures,
    ) -> Result<CompiledFuncRef<'_>, Error> {
        use core::hint::spin_loop;
        'outer: loop {
            match self.state.load(Ordering::Acquire) {
                state::COMPILED => break 'outer,
                state::COMPILING => {
                    spin_loop();
                    continue 'outer;
                }
                state::FAILED_TO_COMPILE => {
                    hint::cold();
                    return Err(Error::from(TranslationError::LazyCompilationFailed));
                }
                state::UNCOMPILED => {
                    hint::cold();
                    if self
                        .state
                        .compare_exchange(
                            state::UNCOMPILED,
                            state::COMPILING,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_err()
                    {
                        // Case: lost the race -> re-observe state
                        spin_loop();
                        continue 'outer;
                    }
                    // Case: won the race -> take ownership of the uncompiled payload, leave `undefined`
                    let mut uncompiled = unsafe { self.take_uncompiled() };
                    match uncompiled.compile(fuel, features) {
                        Ok(compiled) => {
                            // Case: the function compiled successfully
                            unsafe { self.set_compiled(compiled) };
                            break 'outer;
                        }
                        Err(error) if matches!(error.as_trap_code(), Some(TrapCode::OutOfFuel)) => {
                            // Case: ran out of fuel during translation -> may retry
                            hint::cold();
                            unsafe { self.set_uncompiled(uncompiled) };
                            return Err(error);
                        }
                        Err(error) => {
                            // Case: translation failed unexpectedly -> no retry
                            hint::cold();
                            self.state
                                .store(state::FAILED_TO_COMPILE, Ordering::Release);
                            return Err(error);
                        }
                    };
                }
                _ => unsafe { unreachable_unchecked!() },
            }
        }
        Ok(unsafe { self.assume_compiled() })
    }

    /// Assumes `self` to be compiled and returns a [`CompiledFuncRef`].
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to only call this method on [`FuncEntry`]
    /// values that are guaranteed to have been successfully compiled prior.
    ///
    /// # Errors
    ///
    /// - If translation or Wasm validation of `func` failed.
    /// - If `self` is in any non-compiled state.
    #[inline]
    pub unsafe fn assume_compiled(&self) -> CompiledFuncRef<'_> {
        let func = unsafe { &*self.data.get() };
        let compiled = unsafe { &*func.compiled };
        CompiledFuncRef::from(compiled)
    }
}

/// The internal representation of a [`FuncEntry`] in its various states.
union FuncEntryData {
    /// Used in [`state::UNINIT`], [`state::COMPILING`] and [`state::FAILED_TO_COMPILE`] states.
    undefined: (),
    /// Used in the [`state::UNCOMPILED`] state.
    uncompiled: ManuallyDrop<UncompiledFuncEntry>,
    /// Used in the [`state::COMPILED`] state.
    compiled: ManuallyDrop<CompiledFuncEntry>,
}

// # Safety:
//
// `FuncEntry`, `Funcs`, `RawFuncsBucket` are `!Send`/`!Sync` by default.
// This is due to the `UnsafeCell`s in `FuncEntry` and `Funcs` and the `NonNull` in `RawFuncsBucket`.
//
// The impls below rely on three invariants upheld throughout this module:
//
// 1. Pointer-stable: buckets are only appended — never moved, reallocated, or freed until
//    `Funcs` is dropped — so every `FuncEntry` keeps a valid address for the store's life.
// 2. Synchronized mutation: a `FuncEntry`'s `data` is written only before the entity is shared,
//    or by the thread that won the `UNCOMPILED -> COMPILING` CAS, and always before a `Release`
//    to `state`; readers gate `data` behind an `Acquire` of `state`. No `&mut FuncEntry` is ever
//    formed after creation, and a `COMPILED` payload is immutable from then on.
// 3. Thread-agnostic payloads: `UncompiledFuncEntry`/`CompiledFuncEntry` own their data
//    (`Arc`-backed `ModuleHeader`, boxed bytecode, validator resources) with no thread affinity.
//
// - `FuncEntry` is owned data + an atomic discriminant: sending moves thread-agnostic data (3);
//   sharing is race-free because every interior mutation is published through `state` (2).
// - `Funcs` and `RawFuncsBucket` are owning handles into `Send + Sync` `FuncEntry` allocations (1),
//   so sharing them is like sharing `Box<[FuncEntry]>` or `&[FuncEntry]`.
// - New buckets are published through `Funcs` `UnsafeCell` under the writer lock and ordered against
//   readers by the `len_funcs` atomic, so a reader only touches frozen slots disjoint from any a
//   concurrent writer writes.
unsafe impl Send for FuncEntry {}
unsafe impl Sync for FuncEntry {}
unsafe impl Send for Funcs {}
unsafe impl Sync for Funcs {}
unsafe impl Send for RawFuncsBucket {}
unsafe impl Sync for RawFuncsBucket {}

mod state {
    /// The function has not been allocated but not initialized, yet.
    pub const UNINIT: u8 = 0;
    /// The function has been allocated and initialized but not compiled, yet.
    pub const UNCOMPILED: u8 = 1;
    /// The function is currently compiled.
    pub const COMPILING: u8 = 2;
    /// The function failed to compile.
    pub const FAILED_TO_COMPILE: u8 = 3;
    /// The function has been allocated, initialized and compiled.
    pub const COMPILED: u8 = 4;
}

/// A function type index into the Wasm module.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct TypeIndex(u32);

/// An internal uncompiled function entity.
struct UncompiledFuncEntry {
    /// The index of the function within the Wasm module.
    func_index: FuncIdx,
    /// The Wasm binary bytes.
    bytes: SmallByteSlice,
    /// The Wasm module of the Wasm function.
    ///
    /// This is required for Wasm module related information in order
    /// to compile the Wasm function body.
    module: ModuleHeader,
    /// Optional Wasm validation information.
    ///
    /// This is `Some` if the [`UncompiledFuncEntry`] is to be validated upon compilation.
    validation: Option<(TypeIndex, ValidatorResources)>,
}

impl UncompiledFuncEntry {
    /// Creates a new [`UncompiledFuncEntry`].
    pub fn new(
        func_index: FuncIdx,
        bytes: &[u8],
        module: ModuleHeader,
        func_to_validate: impl Into<Option<FuncToValidate<ValidatorResources>>>,
    ) -> Self {
        let validation = func_to_validate.into().map(|func_to_validate| {
            assert_eq!(
                func_to_validate.index,
                func_index.into_u32(),
                "Wasmi function index ({}) does not match with Wasm validation function index ({})",
                func_to_validate.index,
                func_index.into_u32(),
            );
            (TypeIndex(func_to_validate.ty), func_to_validate.resources)
        });
        let bytes = bytes.into();
        Self {
            func_index,
            bytes,
            module,
            validation,
        }
    }

    /// Compile the [`UncompiledFuncEntry`].
    ///
    /// # Panics
    ///
    /// - If the `func` unexpectedly has already been compiled.
    /// - If the `engine` unexpectedly no longer exists due to weak referencing.
    ///
    /// # Errors
    ///
    /// - If function translation failed.
    /// - If `ctx` ran out of fuel in case fuel consumption is enabled.
    fn compile(
        &mut self,
        fuel: Option<&mut Fuel>,
        features: &WasmFeatures,
    ) -> Result<CompiledFuncEntry, Error> {
        /// The amount of fuel required to compile a function body per byte.
        ///
        /// This does _not_ include validation.
        ///
        /// # Note
        ///
        /// This fuel amount was chosen after extensive worst-case translation benchmarking.
        const COMPILE_FUEL_PER_BYTE: u64 = 7;
        /// The amount of fuel required to validate a function body per byte.
        ///
        /// This does _not_ include compilation.
        ///
        /// # Note
        ///
        /// This fuel amount was chosen after extensive worst-case translation benchmarking.
        const VALIDATE_FUEL_PER_BYTE: u64 = 2;
        /// The amount of fuel required to validate and compile a function body per byte.
        const VALIDATE_AND_COMPILE_FUEL_PER_BYTE: u64 =
            VALIDATE_FUEL_PER_BYTE + COMPILE_FUEL_PER_BYTE;

        let func_idx = self.func_index;
        let wasm_bytes = self.bytes.as_slice();
        let needs_validation = self.validation.is_some();
        let compilation_fuel = |_costs: &FuelCostsProvider| {
            let len_bytes = wasm_bytes.len() as u64;
            let fuel_per_byte = match needs_validation {
                false => COMPILE_FUEL_PER_BYTE,
                true => VALIDATE_AND_COMPILE_FUEL_PER_BYTE,
            };
            len_bytes.saturating_mul(fuel_per_byte)
        };
        if let Some(fuel) = fuel {
            match fuel.consume_fuel(compilation_fuel) {
                Err(FuelError::OutOfFuel { required_fuel }) => {
                    return Err(Error::from(ResumableOutOfFuelError::new(required_fuel)));
                }
                Ok(_) | Err(FuelError::FuelMeteringDisabled) => {}
            }
        }
        let module = self.module.clone();
        let Some(engine) = module.engine().upgrade() else {
            panic!(
                "cannot compile function lazily since engine does no longer exist: {:?}",
                module.engine()
            )
        };
        let mut result = MaybeUninit::uninit();
        match self.validation.take() {
            Some((type_index, resources)) => {
                let allocs = engine.get_allocs();
                let translator = FuncTranslator::new(func_idx, module, allocs.0)?;
                let func_to_validate = FuncToValidate {
                    resources,
                    index: func_idx.into_u32(),
                    ty: type_index.0,
                    features: *features,
                };
                let validator = func_to_validate.into_validator(allocs.1);
                let translator = ValidatingFuncTranslator::new(validator, translator)?;
                let allocs = FuncTranslationDriver::new(0, wasm_bytes, translator)?.translate(
                    |compiled_func| {
                        result.write(compiled_func);
                    },
                )?;
                engine.recycle_allocs(allocs.translation, allocs.validation);
            }
            None => {
                let allocs = engine.get_translation_allocs();
                let translator = FuncTranslator::new(func_idx, module, allocs)?;
                let allocs = FuncTranslationDriver::new(0, wasm_bytes, translator)?.translate(
                    |compiled_func| {
                        result.write(compiled_func);
                    },
                )?;
                engine.recycle_translation_allocs(allocs);
            }
        };
        Ok(unsafe { result.assume_init() })
    }
}

impl fmt::Debug for UncompiledFuncEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UncompiledFuncEntry")
            .field("func_idx", &self.func_index)
            .field("bytes", &self.bytes)
            .field("module", &self.module)
            .field("validate", &self.validation.is_some())
            .finish()
    }
}

/// Meta information about a [`EngineFunc`].
#[derive(Debug)]
pub struct CompiledFuncEntry {
    /// The sequence of [`Op`] of the [`CompiledFuncEntry`].
    ops: Pin<Box<[u8]>>,
    /// The total number of stack slots for locals for the [`EngineFunc`].
    ///
    /// # Note
    ///
    /// This includes defined Wasm function parameters and locals.
    len_local_slots: u16,
    /// The total number of stack slots used by the [`EngineFunc`].
    ///
    /// # Note
    ///
    /// This includes stack slots to store the function local constant values,
    /// function parameters, function locals and dynamically used stack slots.
    len_stack_slots: u16,
}

impl CompiledFuncEntry {
    /// Create a new initialized [`CompiledFuncEntry`].
    ///
    /// # Note
    ///
    /// - `len_locals` is the total number of stack slots used for function parameters or locals.
    /// - `len_slots`: is the total number of stack slots used by the function.
    ///
    /// # Panics
    ///
    /// - If `ops` is empty.
    /// - If `ops` contains more than `i32::MAX` encoded bytes.
    pub fn new(len_local_slots: u16, len_stack_slots: u16, ops: &[u8]) -> Self {
        debug_assert!(len_local_slots <= len_stack_slots);
        let ops: Pin<Box<[u8]>> = Pin::new(ops.into());
        debug_assert!(
            !ops.is_empty(),
            "compiled functions must have at least one instruction"
        );
        debug_assert!(
            // Wasmi's branch instructions can jump across at most
            //  - `i32::MAX` bytes forwards
            //  - `i32::MIN` bytes backwards
            // Therefore, having more than `i32::MAX` bytes of operators allows
            // for the existence of branches that overflow these spans.
            ops.len() <= i32::MAX as usize,
            "compiled function has too many operators: {}",
            ops.len(),
        );
        Self {
            ops,
            len_local_slots,
            len_stack_slots,
        }
    }
}

/// A shared reference to the data of a [`EngineFunc`].
#[derive(Debug, Copy, Clone)]
pub struct CompiledFuncRef<'a> {
    /// The sequence of encoded [`Op`]s of the [`CompiledFuncEntry`].
    ops: Pin<&'a [u8]>,
    /// The total number of stack slots used for locals of the [`EngineFunc`].
    len_local_slots: u16,
    /// The number of stack slots used by the [`EngineFunc`].
    len_stack_slots: u16,
}

impl<'a> From<&'a CompiledFuncEntry> for CompiledFuncRef<'a> {
    #[inline]
    fn from(func: &'a CompiledFuncEntry) -> Self {
        Self {
            ops: func.ops.as_ref(),
            len_local_slots: func.len_local_slots,
            len_stack_slots: func.len_stack_slots,
        }
    }
}

impl<'a> CompiledFuncRef<'a> {
    /// Returns the sequence of encoded [`Op`]s of the [`EngineFunc`].
    #[inline]
    pub fn ops(&self) -> &'a [u8] {
        self.ops.get_ref()
    }

    /// Returns the total number of stack slots used for locals of the [`EngineFunc`].
    #[inline]
    pub fn len_local_slots(&self) -> u16 {
        self.len_local_slots
    }

    /// Returns the total number of stack slots used by the [`EngineFunc`].
    #[inline]
    pub fn len_stack_slots(&self) -> u16 {
        self.len_stack_slots
    }
}
