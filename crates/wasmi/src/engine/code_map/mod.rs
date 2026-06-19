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
    collections::arena::ArenaKey,
    core::{Fuel, FuelCostsProvider},
    engine::{ResumableOutOfFuelError, utils::unreachable_unchecked},
    errors::FuelError,
    ir::index::InternalFunc,
    module::{FuncIdx, ModuleHeader},
};
use alloc::boxed::Box;
use core::{
    cell::UnsafeCell,
    fmt,
    hint,
    iter,
    mem::{self, ManuallyDrop, MaybeUninit},
    pin::Pin,
    ptr::{self, NonNull},
    slice,
    sync::atomic::{AtomicU8, Ordering},
};
use spin::Mutex;
use wasmparser::{FuncToValidate, ValidatorResources, WasmFeatures};

#[cfg(doc)]
use crate::ir::Op;

/// How many functions are stored in the first bucket.
const LEN_BUCKET0_LOG2: usize = 5;

/// The maximum allowed number of functions in a [`CodeMap`].
const MAX_FUNCS: usize = 10_000_000;

/// The number of buckets required to satisfy [`LEN_BUCKET0_LOG2`] and [`MAX_FUNCS`].
const MAX_BUCKETS: usize = Funcs::required_buckets_for_len(MAX_FUNCS);

/// A data structure to store and manage [`FuncEntity`] definitions.
#[derive(Debug)]
pub struct CodeMap {
    /// The append-only storage for all [`FuncEntity`] definitions.
    funcs: Mutex<Funcs>,
    /// Shared Wasm features across all [`FuncEntity`] definitions within the [`CodeMap`].
    features: WasmFeatures,
}

/// An append-only collection for [`FuncEntity`] definitions.
pub struct Funcs {
    /// The buckets to store the [`FuncEntity`] definitions append-only.
    ///
    /// Each bucket is twice the size of its predecessor.
    buckets: [Option<RawFuncsBucket>; MAX_BUCKETS],
    /// The number of occupied [`FuncsBucket`] items in `buckets`.
    len_buckets: usize,
    /// The number of [`FuncEntity`] definitions stored across all `buckets`.
    len_funcs: usize,
}

impl fmt::Debug for Funcs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut n = 0;
        let buckets = self.buckets.map(|raw| {
            let len = Self::size_of_bucket_at(n);
            let bucket = unsafe { FuncsBucketRef::from_raw_parts(raw?, len) };
            n += 1;
            Some(bucket)
        });
        f.debug_struct("Funcs")
            .field("buckets", &buckets)
            .field("len_buckets", &self.len_buckets)
            .field("len_funcs", &self.len_funcs)
            .finish()
    }
}

/// A shared view of [`Funcs`] to avoid having to go through `Mutex` upon access.
///
/// # Note
///
/// - This represents a potentially stale view of a [`Funcs`] source.
/// - Since [`Funcs`] is append only, this stale view is never invalid but maybe outdated.
/// - Upon execution an [`Engine`] derives a [`FuncsRef`] view of the current [`Funcs`]
///   state and then uses that to drive most of the call-based executions to avoid mutex locks.
#[expect(dead_code)] // TODO: make use of this type
pub struct FuncsRef<'a> {
    /// The live buckets that store [`FuncEntity`] definitions.
    buckets: &'a [Option<RawFuncsBucket>],
    /// The number of [`FuncEntity`] definitions stored across all live `buckets`.
    len_funcs: usize,
}

impl<'a> From<&'a Funcs> for FuncsRef<'a> {
    fn from(funcs: &'a Funcs) -> Self {
        Self {
            buckets: &funcs.buckets[..funcs.len_buckets],
            len_funcs: funcs.len_funcs,
        }
    }
}

#[expect(dead_code)] // TODO: make use of this type
impl FuncsRef<'_> {
    /// Returns a shared reference to the [`FuncEntity`] of `func` if any.
    ///
    /// Returns `None` if `n` is out of bounds.
    pub fn get(&self, func: EngineFunc) -> Option<&FuncEntity> {
        if !self.contains(func) {
            return None;
        }
        let (bucket, slot) = Funcs::locate(func);
        self.bucket_ref_at(bucket)?.get(slot)
    }

    /// Returns the [`FuncsBucketRef`] at index `n` if any.
    fn bucket_ref_at(&self, n: usize) -> Option<FuncsBucketRef<'_>> {
        let raw = (*self.buckets.get(n)?).as_ref().copied()?;
        let len = Funcs::size_of_bucket_at(n);
        // Safety: bucket `n` was allocated with `size_of_bucket_at(n)` entities and is never freed
        //         until `Funcs` is dropped.
        Some(unsafe { FuncsBucketRef::from_raw_parts(raw, len) })
    }

    /// Returns `true` if `func` is contained in `self`.
    fn contains(&self, func: EngineFunc) -> bool {
        func.into_usize() < self.len_funcs
    }
}

#[test]
fn size_of_funcs_type() {
    const EXPECTED_SIZE: usize =
        (MAX_BUCKETS * mem::size_of::<*mut FuncEntity>()) + (2 * mem::size_of::<usize>());
    assert_eq!(core::mem::size_of::<Funcs>(), EXPECTED_SIZE);
}

impl Default for Funcs {
    fn default() -> Self {
        Self {
            buckets: [const { None }; MAX_BUCKETS],
            len_buckets: 0,
            len_funcs: 0,
        }
    }
}

impl Funcs {
    /// Allocates `n` new function entities to `self` and returns a span to them.
    ///
    /// # Note
    ///
    /// All function entities allocated this way are initialized to an undefined state.
    pub fn alloc_funcs(&mut self, n: usize) -> Result<EngineFuncSpan, Error> {
        let start = self.len_funcs;
        let end = start
            .checked_add(n)
            .filter(|&end| end <= MAX_FUNCS)
            .unwrap(); // TODO: proper error handling
        let needed = Self::required_buckets_for_len(end);
        while self.len_buckets < needed {
            let bucket = FuncsBucket::new(Self::size_of_bucket_at(self.len_buckets));
            let (raw, _len) = bucket.into_raw_parts();
            self.buckets[self.len_buckets] = Some(raw);
            self.len_buckets += 1;
        }
        self.len_funcs = end;
        Ok(EngineFuncSpan::new(
            EngineFunc::from(InternalFunc::from(start as u32)),
            EngineFunc::from(InternalFunc::from(end as u32)),
        ))
    }

    /// Returns a shared reference to the [`FuncEntity`] of `func` if any.
    ///
    /// Returns `None` if `n` is out of bounds.
    pub fn get(&self, func: EngineFunc) -> Option<&FuncEntity> {
        if !self.contains(func) {
            return None;
        }
        let (bucket, slot) = Self::locate(func);
        self.bucket_ref_at(bucket)?.get(slot)
    }

    /// Maps a global function `index` to its `(bucket, slot)` position.
    #[inline]
    fn locate(func: EngineFunc) -> (usize, usize) {
        let index = Self::func_to_index(func);
        let j = u64::from(index) + (1 << LEN_BUCKET0_LOG2);
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
        let j = (len as u64 - 1) + (1 << LEN_BUCKET0_LOG2);
        let msb = 63 - j.leading_zeros() as usize;
        (msb - LEN_BUCKET0_LOG2) + 1
    }

    /// Returns the number of functions stored in the bucket at index `n`.
    fn size_of_bucket_at(n: usize) -> usize {
        1usize << (LEN_BUCKET0_LOG2 + n)
    }

    /// Returns the [`FuncsBucketRef`] at index `n` if any.
    fn bucket_ref_at(&self, n: usize) -> Option<FuncsBucketRef<'_>> {
        let raw = (*self.buckets.get(n)?).as_ref().copied()?;
        // Safety: bucket `n` was allocated with `size_of_bucket_at(n)` entities and is never freed
        //         until `Funcs` is dropped.
        Some(unsafe { FuncsBucketRef::from_raw_parts(raw, Self::size_of_bucket_at(n)) })
    }

    /// Converts `func` into its underlying `u32` index.
    fn func_to_index(func: EngineFunc) -> u32 {
        u32::from(InternalFunc::from(func))
    }

    /// Returns `true` if `func` is contained in `self`.
    fn contains(&self, func: EngineFunc) -> bool {
        func.into_usize() < self.len_funcs
    }
}

impl Drop for Funcs {
    fn drop(&mut self) {
        let buckets = mem::replace(&mut self.buckets, [const { None }; MAX_BUCKETS]);
        for (n, raw_bucket) in buckets.into_iter().enumerate() {
            let Some(raw_bucket) = raw_bucket else { return };
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
    /// The raw pointer to the bucket's [`FuncEntity`] definitions.
    funcs: NonNull<FuncEntity>,
}

/// A fixed-size bucket of [`FuncEntity`] definitions.
pub struct FuncsBucket {
    /// The heap allocation that stores the [`FuncEntity`] definitions of this bucket.
    funcs: Box<[FuncEntity]>,
}

impl FuncsBucket {
    /// Creates a new [`FuncBucket`] with a fixed `size`.
    pub fn new(size: usize) -> Self {
        Self {
            funcs: iter::repeat_with(FuncEntity::uninit).take(size).collect(),
        }
    }

    /// Destructs `self` into its raw parts, a [`RawFuncsBucket`] and its length.
    pub fn into_raw_parts(self) -> (RawFuncsBucket, usize) {
        let len = self.funcs.len();
        // Safety: `Box` is guaranteed to be non-null.
        let funcs = unsafe { NonNull::new_unchecked(Box::into_raw(self.funcs) as *mut FuncEntity) };
        (RawFuncsBucket { funcs }, len)
    }

    /// Creates a [`FuncsBucket`] from its raw parts.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to provide a valid `raw` and `len` argument.
    pub unsafe fn from_raw_parts(raw: RawFuncsBucket, len: usize) -> FuncsBucket {
        let funcs = ptr::slice_from_raw_parts_mut(raw.funcs.as_ptr(), len);
        FuncsBucket {
            funcs: unsafe { Box::from_raw(funcs) },
        }
    }
}

/// A fixed-size bucket of [`FuncEntity`] definitions.
#[derive(Debug)]
pub struct FuncsBucketRef<'a> {
    /// The shared [`FuncEntity`] definitions of this bucket.
    funcs: &'a [FuncEntity],
}

impl<'a> FuncsBucketRef<'a> {
    /// Creates a [`FuncsBucketRef`] from its raw parts.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to provide a valid `raw` and `len` argument.
    pub unsafe fn from_raw_parts(raw: RawFuncsBucket, len: usize) -> FuncsBucketRef<'a> {
        let funcs = unsafe { slice::from_raw_parts(raw.funcs.as_ptr(), len) };
        Self { funcs }
    }

    /// Returns a shared reference to the [`FuncEntity`] at index `n` if any.
    pub fn get(&self, n: usize) -> Option<&'a FuncEntity> {
        self.funcs.get(n)
    }
}

/// A function entity in any of its various states.
///
/// # Dev. Note
///
/// We use `#[repr(C)]` to dictate field ordering which is important for the executor
/// since the executes uses direct pointers to [`FuncEntity`] instances once they are
/// known to be compiled.
#[repr(C)]
pub struct FuncEntity {
    /// Payload; which field is active (if any) is determined by `state`.
    data: UnsafeCell<FuncEntityData>,
    /// Synchronization authority *and* discriminant. One of the consts in `state` module below.
    state: AtomicU8,
}

impl Drop for FuncEntity {
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

impl fmt::Debug for FuncEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = unsafe { self.data_mut() };
        match self.state.load(Ordering::Acquire) {
            state::UNINIT => f.debug_struct("FuncEntity::Uninit").finish(),
            state::COMPILING => f.debug_struct("FuncEntity::Compiling").finish(),
            state::FAILED_TO_COMPILE => f.debug_struct("FuncEntity::FailedToCompile").finish(),
            state::COMPILED => f
                .debug_struct("FuncEntity::Compiled")
                .field("state", unsafe { &state.compiled })
                .finish(),
            state::UNCOMPILED => f
                .debug_struct("FuncEntity::Uncompiled")
                .field("state", unsafe { &state.uncompiled })
                .finish(),
            _ => unreachable!(),
        }
    }
}

impl FuncEntity {
    /// Creates an uninitialized [`FuncEntity`].
    pub fn uninit() -> Self {
        Self {
            data: UnsafeCell::new(FuncEntityData { undefined: () }),
            state: AtomicU8::new(state::UNINIT),
        }
    }

    /// Returns `true` if `self` has already been initialized.
    fn is_initialized(&self) -> bool {
        self.state.load(Ordering::Relaxed) != state::UNINIT
    }

    /// Initializes the [`FuncEntity`] with a [`CompiledFuncEntity`].
    ///
    /// # Panics
    ///
    /// If `func` has already been initialized.
    pub fn init_compiled(&self, compiled: CompiledFuncEntity) {
        assert!(!self.is_initialized(), "func has already been initialized");
        // Safety: exclusive during build; previous union field is `undefined` (no Drop).
        unsafe { self.set_compiled(compiled) }
    }

    /// Initializes the [`FuncEntity`] to an uncompiled state.
    ///
    /// # Panics
    ///
    /// If `func` has already been initialized.
    pub fn init_uncompiled(&self, uncompiled: UncompiledFuncEntity) {
        assert!(!self.is_initialized(), "func has already been initialized");
        // Safety: exclusive during build; previous union field is `undefined` (no Drop).
        unsafe { self.set_uncompiled(uncompiled) }
    }

    /// Initializes the [`FuncEntity`] to an uncompiled state.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to assert that `self.data` is in the `undefined` state.
    unsafe fn set_compiled(&self, compiled: CompiledFuncEntity) {
        let data = unsafe { self.data_mut() };
        data.compiled = ManuallyDrop::new(compiled);
        self.state.store(state::COMPILED, Ordering::Release);
    }

    /// Initializes the [`FuncEntity`] to an uncompiled state.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to assert that `self.data` is in the `undefined` state.
    unsafe fn set_uncompiled(&self, uncompiled: UncompiledFuncEntity) {
        let data = unsafe { self.data_mut() };
        data.uncompiled = ManuallyDrop::new(uncompiled);
        self.state.store(state::UNCOMPILED, Ordering::Release);
    }

    /// Takes [`UncompiledFuncEntity`] from `self.data` and leaves behind an `undefined` state.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to ensure that `self.data` has been is in `uncompiled` state.
    unsafe fn take_uncompiled(&self) -> UncompiledFuncEntity {
        let data = unsafe { self.data_mut() };
        let uncompiled = unsafe { ManuallyDrop::take(&mut data.uncompiled) };
        data.undefined = ();
        uncompiled
    }

    /// Returns an exclusive reference to the [`FuncEntityData`] of `self`.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility that no other references to this data are alive.
    #[allow(clippy::mut_from_ref)] // same API as `UnsafeCell::as_mut_unchecked`
    unsafe fn data_mut(&self) -> &mut FuncEntityData {
        unsafe { &mut *self.data.get() }
    }

    /// Compiles `self` and returns a view to the [`CompiledFuncRef`].
    ///
    /// # Note
    ///
    /// - If `self` has already been compiled `Ok(None)` is returned.
    /// - In this case the caller is supposed to use [`FuncEntity::get`]
    ///   to query the [`CompiledFuncRef`].
    /// - If this method returns with `Ok`, `self` can be assumed to be
    ///   compiled from that point on.
    ///
    /// # Errors
    ///
    /// - If translation or Wasm validation of `func` failed.
    /// - If `ctx` ran out of fuel in case fuel consumption is enabled.
    pub fn get_or_compile(
        &self,
        fuel: Option<&mut Fuel>,
        features: &WasmFeatures,
    ) -> Result<CompiledFuncRef<'_>, Error> {
        'outer: loop {
            match self.state.load(Ordering::Acquire) {
                state::COMPILED => break 'outer,
                state::COMPILING => {
                    hint::spin_loop();
                    continue 'outer;
                }
                state::FAILED_TO_COMPILE => {
                    return Err(Error::from(TranslationError::LazyCompilationFailed));
                }
                state::UNCOMPILED => {
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
                        hint::spin_loop();
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
                            unsafe { self.set_uncompiled(uncompiled) };
                            return Err(error);
                        }
                        Err(error) => {
                            // Case: translation failed unexpectedly -> no retry
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
    /// It is the caller's responsibility to only call this method on [`FuncEntity`]
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

/// The internal representation of a [`FuncEntity`] in its various states.
union FuncEntityData {
    /// Used in [`state::UNINIT`], [`state::COMPILING`] and [`state::FAILED_TO_COMPILE`] states.
    undefined: (),
    /// Used in the [`state::UNCOMPILED`] state.
    uncompiled: ManuallyDrop<UncompiledFuncEntity>,
    /// Used in the [`state::COMPILED`] state.
    compiled: ManuallyDrop<CompiledFuncEntity>,
}

// # Safety
//
// `FuncEntity`, `Funcs`, `RawFuncsBucket` and `FuncsRef` form an append-only,
// pointer-stable function store. They are `!Send`/`!Sync` by default (because of
// the `UnsafeCell` in `FuncEntity` and the `NonNull` in `RawFuncsBucket`), so the
// impls below are written by hand. Their soundness rests on three invariants that
// the rest of this module upholds:
//
// 1. Append-only & pointer-stable. Buckets are only ever appended; they are never
//    moved, reallocated, or freed until the entire `Funcs` is dropped. Hence every
//    `FuncEntity` keeps a stable address and any in-bounds reference/pointer stays
//    valid for as long as the store is alive.
//
// 2. Publication through the atomic discriminant. Every write to a `FuncEntity`'s
//    `data` payload happens either before the entity is reachable by another thread
//    (during module build) or by the single thread that won the
//    `UNCOMPILED -> COMPILING` compare-exchange, which holds exclusive logical
//    ownership of the payload for the duration of compilation. Each such write is
//    completed before a `Release` store to `state`, and every reader gates its
//    access to `data` behind an `Acquire` load of `state`. This release/acquire
//    pairing establishes the happens-before that makes the `UnsafeCell` accesses
//    free of data races, and a payload observed in the `COMPILED` state is
//    immutable from then on. No `&mut FuncEntity` is ever formed after creation;
//    all interior mutation goes through the `UnsafeCell` under this protocol.
//
// 3. Thread-agnostic payloads. The payloads (`UncompiledFuncEntity`,
//    `CompiledFuncEntity`) own their data (an `Arc`-backed `ModuleHeader`, boxed
//    bytecode, validator resources) and carry no thread affinity, so moving or
//    sharing them across threads is itself sound.
//
// Safety: `FuncEntity` is just owned data plus an atomic discriminant; sending it
//         transfers thread-agnostic owned data (invariant 3), and sharing
//         `&FuncEntity` is sound because every interior mutation is synchronized
//         through `state` via release/acquire (invariant 2).
unsafe impl Send for FuncEntity {}
unsafe impl Sync for FuncEntity {}

// Safety: `Funcs` and `RawFuncsBucket` are owning handles into heap allocations of
//         `FuncEntity`. Since `FuncEntity: Send + Sync` and the allocations have
//         no thread affinity, sending/sharing them is equivalent to sending/sharing
//         `Box<[FuncEntity]>` / `&[FuncEntity]` (invariant 1). `Funcs` is, in
//         addition, only ever mutated behind `CodeMap`'s `Mutex`.
unsafe impl Send for Funcs {}
unsafe impl Sync for Funcs {}
unsafe impl Send for RawFuncsBucket {}
unsafe impl Sync for RawFuncsBucket {}

// Safety: `FuncsRef` is a read-only (possibly stale) snapshot of the append-only
//         buckets. It only ever reads immutable bucket pointers and hands out
//         `&FuncEntity`, so it inherits the guarantees above (invariants 1 & 2).
unsafe impl Send for FuncsRef<'_> {}
unsafe impl Sync for FuncsRef<'_> {}

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

impl CodeMap {
    /// Creates a new [`CodeMap`].
    pub fn new(config: &Config) -> Self {
        Self {
            funcs: Mutex::new(Funcs::default()),
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
        match self.funcs.lock().alloc_funcs(amount) {
            Ok(span) => span,
            Err(err) => panic!("failed to alloc funcs: {err}"),
        }
    }

    /// Initializes the [`EngineFunc`] with its [`CompiledFuncEntity`].
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`EngineFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`EngineFunc`].
    pub fn init_func_as_compiled(&self, func: EngineFunc, entity: CompiledFuncEntity) {
        let funcs = self.funcs.lock();
        let func = match funcs.get(func) {
            Some(func) => func,
            None => panic!("failed to resolve function at {func:?}"),
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
        let funcs = self.funcs.lock();
        let func = match funcs.get(func) {
            Some(func) => func,
            None => panic!("failed to resolve function at {func:?}"),
        };
        func.init_uncompiled(UncompiledFuncEntity::new(
            func_idx,
            bytes,
            module.clone(),
            func_to_validate,
        ));
    }

    /// Returns a shared reference to the [`FuncEntity`] of `func` if contained by `self`.
    #[track_caller]
    #[inline]
    pub fn get_ref(&self, func: EngineFunc) -> Option<&FuncEntity> {
        let funcs = self.funcs.lock();
        let entity = funcs.get(func)?;
        // Safety: `Funcs` is append-only and entity buckets never move/free until engine drop.
        //         Therefore, the reference may outlive the guard.
        Some(Self::adjust_cref_lifetime(entity))
    }

    /// Returns the [`CompiledFuncRef`] of the `func`.
    ///
    /// This might compile `func` if it is still uncompiled.
    ///
    /// # Errors
    ///
    /// - If translation or Wasm validation of `func` failed.
    /// - If `ctx` ran out of fuel in case fuel consumption is enabled.
    #[track_caller]
    #[inline]
    pub fn get<'a>(
        &'a self,
        fuel: Option<&mut Fuel>,
        func: EngineFunc,
    ) -> Result<CompiledFuncRef<'a>, Error> {
        let Some(entity) = self.get_ref(func) else {
            panic!("invalid EngineFunc {func:?}")
        };
        entity.get_or_compile(fuel, &self.features)
    }

    /// Prolongs the lifetime of `cref` to `self`.
    ///
    /// # Safety
    ///
    /// This is safe since
    ///
    /// - [`CompiledFuncRef`] only references `Pin`ned data
    /// - [`CodeMap`] is an append-only data structure
    ///
    /// Thus any shared [`CompiledFuncRef`] can safely outlive the internal `Mutex` lock.
    #[inline]
    fn adjust_cref_lifetime<'a>(cref: &'_ FuncEntity) -> &'a FuncEntity {
        // Safety: we cast the lifetime of `cref` to match `&self` instead of the inner
        //         `MutexGuard` which is safe because `CodeMap` is append-only and the
        //         returned `CompiledFuncRef` only references `Pin`ned data.
        unsafe { mem::transmute::<&'_ FuncEntity, &'a FuncEntity>(cref) }
    }
}

/// A function type index into the Wasm module.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct TypeIndex(u32);

/// An internal uncompiled function entity.
pub struct UncompiledFuncEntity {
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
    /// This is `Some` if the [`UncompiledFuncEntity`] is to be validated upon compilation.
    validation: Option<(TypeIndex, ValidatorResources)>,
}

impl UncompiledFuncEntity {
    /// Creates a new [`UncompiledFuncEntity`].
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

    /// Compile the [`UncompiledFuncEntity`].
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
    ) -> Result<CompiledFuncEntity, Error> {
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

impl fmt::Debug for UncompiledFuncEntity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UncompiledFuncEntity")
            .field("func_idx", &self.func_index)
            .field("bytes", &self.bytes)
            .field("module", &self.module)
            .field("validate", &self.validation.is_some())
            .finish()
    }
}

/// Meta information about a [`EngineFunc`].
#[derive(Debug)]
pub struct CompiledFuncEntity {
    /// The sequence of [`Op`] of the [`CompiledFuncEntity`].
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

impl CompiledFuncEntity {
    /// Create a new initialized [`CompiledFuncEntity`].
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
        assert!(
            !ops.is_empty(),
            "compiled functions must have at least one instruction"
        );
        assert!(
            // Generally, Wasmi has no issues with more than `i32::MAX` instructions.
            // However, Wasmi's branch instructions can jump across at most `i32::MAX`
            // forwards or `i32::MIN` instructions backwards and thus having more than
            // `i32::MAX` instructions might introduce problems.
            ops.len() <= i32::MAX as usize,
            "compiled function has too many instructions: {}",
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
    /// The sequence of encoded [`Op`]s of the [`CompiledFuncEntity`].
    ops: Pin<&'a [u8]>,
    /// The total number of stack slots used for locals of the [`EngineFunc`].
    len_local_slots: u16,
    /// The number of stack slots used by the [`EngineFunc`].
    len_stack_slots: u16,
}

impl<'a> From<&'a CompiledFuncEntity> for CompiledFuncRef<'a> {
    #[inline]
    fn from(func: &'a CompiledFuncEntity) -> Self {
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
