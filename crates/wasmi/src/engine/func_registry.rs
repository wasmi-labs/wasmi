use super::{bytecode::Instruction, DedupFuncType, EngineIdx, Guarded};
use crate::func::HostFunc;
use alloc::vec::Vec;
use core::{fmt::Debug, num::NonZeroU32};
use wasmi_arena::{ArenaIndex, GuardedEntity};

/// Stores Wasm and host functions registered to an [`Engine`].
#[derive(Debug)]
pub struct FuncRegistry {
    /// The index of the [`Engine`].
    engine_idx: EngineIdx,
    /// All registered Wasm and host functions.
    funcs: Vec<FuncEntity>,
}

impl FuncRegistry {
    /// Creates a new [`FuncRegistry`] with the [`EngineIdx`].
    pub fn new(engine_idx: EngineIdx) -> Self {
        Self {
            engine_idx,
            funcs: Vec::new(),
        }
    }

    /// Wraps an entitiy `Idx` (index type) as a [`Guarded<Idx>`] type.
    ///
    /// # Note
    ///
    /// [`Stored<Idx>`] associates an `Idx` type with the internal store index.
    /// This way wrapped indices cannot be misused with incorrect [`Store`] instances.
    fn wrap_guarded<Idx>(&self, entity_idx: Idx) -> Guarded<Idx> {
        Guarded::new(self.engine_idx, entity_idx)
    }

    /// Unwraps the given [`Guarded<Idx>`] reference and returns the `Idx`.
    ///
    /// # Panics
    ///
    /// If the [`Stored<Idx>`] does not originate from this [`Store`].
    fn unwrap_guarded<Idx>(&self, stored: &Guarded<Idx>) -> Idx
    where
        Idx: ArenaIndex + Debug,
    {
        stored.entity_index(self.engine_idx).unwrap_or_else(|| {
            panic!(
                "entity reference ({:?}) does not belong to engine {:?}",
                stored, self.engine_idx,
            )
        })
    }

    /// Allocates a new Wasm function to the [`FuncRegistry`].
    ///
    /// Returns a [`Func`] reference to the allocated function.
    pub fn alloc_wasm<I>(
        &mut self,
        ty: DedupFuncType,
        len_locals: usize,
        stack_usage: usize,
        instrs: I,
    ) -> Func
    where
        I: IntoIterator<Item = Instruction>,
    {
        let instrs = instrs.into_iter().collect();
        let func_index = FuncIdx::from_usize(self.funcs.len());
        let header = WasmFuncEntity {
            instrs,
            ty,
            len_locals,
            stack_usage: len_locals + stack_usage,
        };
        self.funcs.push(FuncEntity::Wasm(header));
        Func::from_inner(self.wrap_guarded(func_index))
    }

    /// Allocates a new host function to the [`FuncRegistry`].
    ///
    /// Returns a [`Func`] reference to the allocated function.
    pub fn alloc_host(&mut self, ty: DedupFuncType, func: HostFunc) -> Func {
        let header = HostFuncEntity { ty, func };
        let func_index = FuncIdx::from_usize(self.funcs.len());
        self.funcs.push(FuncEntity::Host(header));
        Func::from_inner(self.wrap_guarded(func_index))
    }

    /// Resolves the [`FuncEntity`] of [`Func`].
    ///
    /// # Panics
    ///
    /// - If [`Func`] does not originate from the [`Engine`].
    /// - If [`Func`] is not registered by the [`Engine`].
    pub fn resolve(&self, func: Func) -> &FuncEntity {
        self.funcs
            .get(self.unwrap_guarded(func.as_inner()).into_usize())
            .unwrap_or_else(|| panic!("out of bounds function entity access {func:?}"))
    }
}

/// Pointer to an instruction of a Wasm function.
#[derive(Debug, Copy, Clone)]
pub struct InstructionPtr {
    iptr: *const Instruction,
}

impl From<*const Instruction> for InstructionPtr {
    #[inline]
    fn from(iptr: *const Instruction) -> Self {
        Self { iptr }
    }
}

/// It is safe to send an [`InstructionPtr`] to another thread.
///
/// The access to the pointed-to [`Instruction`] is read-only and
/// [`Instruction`] itself is [`Send`].
///
/// However, it is not safe to share an [`InstructionPtr`] between threads
/// due to their [`InstructionPtr::offset`] method which relinks the
/// internal pointer and is not synchronized.
unsafe impl Send for InstructionPtr {}

impl InstructionPtr {
    /// Offset the [`InstructionPtr`] by the given value.
    ///
    /// # Safety
    ///
    /// The caller is responsible for calling this method only with valid
    /// offset values so that the [`InstructionPtr`] never points out of valid
    /// bounds of the instructions of the same compiled Wasm function.
    #[inline(always)]
    pub unsafe fn offset(&mut self, by: isize) {
        self.iptr = self.iptr.offset(by);
    }

    /// Returns a shared reference to the currently pointed at [`Instruction`].
    ///
    /// # Safety
    ///
    /// The caller is responsible for calling this method only when it is
    /// guaranteed that the [`InstructionPtr`] is validly pointing inside
    /// the boundaries of its associated compiled Wasm function.
    #[inline(always)]
    pub unsafe fn get(&self) -> &Instruction {
        &*self.iptr
    }
}

/// An index uniquely identifying a Wasm or host function.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FuncIdx(NonZeroU32);

impl ArenaIndex for FuncIdx {
    fn into_usize(self) -> usize {
        self.0.get().wrapping_sub(1) as usize
    }

    fn from_usize(index: usize) -> Self {
        index
            .try_into()
            .ok()
            .map(|index: u32| index.wrapping_add(1))
            .and_then(NonZeroU32::new)
            .map(Self)
            .unwrap_or_else(|| panic!("out of bounds func index {index}"))
    }
}

/// A Wasm or host function reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Func(GuardedEntity<EngineIdx, FuncIdx>);

impl Func {
    /// Creates a new Wasm or host function reference from the guarded index.
    pub(super) fn from_inner(stored: GuardedEntity<EngineIdx, FuncIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying guarded index.
    pub(super) fn as_inner(&self) -> &GuardedEntity<EngineIdx, FuncIdx> {
        &self.0
    }
}

/// The header of a Wasm or host function.
#[derive(Debug, Clone)]
pub enum FuncEntity {
    /// A Wasm function entity.
    Wasm(WasmFuncEntity),
    /// A host function entity.
    Host(HostFuncEntity),
}

impl FuncEntity {
    /// Returns the deduplicated function type of the function.
    pub fn ty(&self) -> &DedupFuncType {
        match self {
            Self::Wasm(func) => func.ty(),
            Self::Host(func) => func.ty(),
        }
    }
}

/// A Wasm function.
#[derive(Debug, Clone)]
pub struct WasmFuncEntity {
    /// The function type of the Wasm function.
    ty: DedupFuncType,
    /// The instructions of the Wasm function.
    instrs: Box<[Instruction]>,
    /// The number of local variables of the Wasm function.
    len_locals: usize,
    /// The maximum stack height usage of the Wasm function during execution.
    stack_usage: usize,
}

impl WasmFuncEntity {
    /// Returns the deduplicated function type of the Wasm function.
    pub fn ty(&self) -> &DedupFuncType {
        &self.ty
    }

    /// Returns the instructions of the Wasm function.
    pub fn instrs(&self) -> &[Instruction] {
        &self.instrs[..]
    }

    /// Returns the [`InstructionPtr`] to the instructions of the Wasm function.
    pub fn iptr(&self) -> InstructionPtr {
        self.instrs().as_ptr().into()
    }

    /// Returns the amount of local variable of the Wasm function.
    pub fn len_locals(&self) -> usize {
        self.len_locals
    }

    /// Returns the amount of stack values used by the Wasm function.
    ///
    /// # Note
    ///
    /// This amount includes the amount of local variables but does
    /// _not_ include the amount of input parameters to the Wasm function.
    pub fn stack_usage(&self) -> usize {
        self.stack_usage
    }
}

/// A host function header.
#[derive(Debug, Copy, Clone)]
pub struct HostFuncEntity {
    /// The function type of the host function.
    ty: DedupFuncType,
    /// The host function of a [`Store`].
    ///
    /// # Note
    ///
    /// We cannot store host functions directly in the [`Engine`]
    /// since they are generic over the host state type.
    func: HostFunc,
}

impl HostFuncEntity {
    /// Returns the deduplicated function type of the host function.
    pub fn ty(&self) -> &DedupFuncType {
        &self.ty
    }

    /// Returns a reference to the underlying [`HostFunc`] in the [`Store`].
    pub fn host_func(&self) -> &HostFunc {
        &self.func
    }
}
