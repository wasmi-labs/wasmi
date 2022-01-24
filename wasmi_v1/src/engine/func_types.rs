use wasmi_core::ValueType;

use super::{EngineIdx, Guarded};
use crate::{
    arena::{DedupArena, GuardedEntity},
    FuncType,
    Index,
    Store,
};

/// A raw index to a function signature entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DedupFuncTypeIdx(usize);

impl Index for DedupFuncTypeIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        Self(value)
    }
}

/// A deduplicated Wasm [`FuncType`].
///
/// # Note
///
/// Advantages over a non-deduplicated [`FuncType`] are:
///
/// - Comparison for equality is as fast as an integer value comparison.
///     - With this we can speed up indirect calls in the engine.
/// - Requires a lot less memory footprint to be stored somewhere compared
///   to a full fledged [`FuncType`].
///
/// Disadvantages compared to non-deduplicated [`FuncType`] are:
///
/// - Requires another indirection to acquire information such as parameter
///   or result types of the underlying [`FuncType`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct DedupFuncType(GuardedEntity<EngineIdx, DedupFuncTypeIdx>);

impl DedupFuncType {
    /// Creates a new function signature reference.
    pub(super) fn from_inner(stored: GuardedEntity<EngineIdx, DedupFuncTypeIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn into_inner(self) -> GuardedEntity<EngineIdx, DedupFuncTypeIdx> {
        self.0
    }

    /// Creates a new function signature to the store.
    pub fn new<T, I, O>(ctx: &mut Store<T>, inputs: I, outputs: O) -> Self
    where
        I: IntoIterator<Item = ValueType>,
        O: IntoIterator<Item = ValueType>,
    {
        ctx.alloc_func_type(FuncType::new(inputs, outputs))
    }
}

/// A [`FuncType`] registry that efficiently deduplicate stored function types.
///
/// Can also be used to later resolve deduplicated function types into their
/// original [`FuncType`] for inspecting their parameter and result types.
///
/// The big advantage of deduplicated [`FuncType`] entities is that we can use
/// this for indirect calls to speed up the signature checks since comparing
/// deduplicated [`FuncType`] instances is as fast as comparing integer values.
/// Also with respect to `wasmi` bytecode deduplicated [`FuncType`] entities
/// require a lot less space to be stored.
#[derive(Debug)]
pub struct FuncTypeRegistry {
    /// A unique identifier for the associated engine.
    ///
    /// # Note
    ///
    /// This is used to guard against invalid entity indices.
    engine_idx: EngineIdx,
    /// Deduplicated function types.
    ///
    /// # Note
    ///
    /// The engine deduplicates function types to make the equality
    /// comparison very fast. This helps to speed up indirect calls.
    func_types: DedupArena<DedupFuncTypeIdx, FuncType>,
}

impl FuncTypeRegistry {
    /// Creates a new [`FuncTypeRegistry`] using the given [`EngineIdx`].
    pub(crate) fn new(engine_idx: EngineIdx) -> Self {
        Self {
            engine_idx,
            func_types: DedupArena::default(),
        }
    }

    /// Unpacks the entity and checks if it is owned by the engine.
    ///
    /// # Panics
    ///
    /// If the guarded entity is not owned by the engine.
    fn unwrap_index<Idx>(&self, func_type: Guarded<Idx>) -> Idx
    where
        Idx: Index,
    {
        func_type.entity_index(self.engine_idx).unwrap_or_else(|| {
            panic!(
                "encountered foreign entity in func type registry: {}",
                self.engine_idx.into_usize()
            )
        })
    }

    /// Allocates a new function type to the engine.
    pub(crate) fn alloc_func_type(&mut self, func_type: FuncType) -> DedupFuncType {
        DedupFuncType::from_inner(Guarded::new(
            self.engine_idx,
            self.func_types.alloc(func_type),
        ))
    }

    /// Resolves a deduplicated function type into a [`FuncType`] entity.
    ///
    /// # Panics
    ///
    /// - If the deduplicated function type is not owned by the engine.
    /// - If the deduplicated function type cannot be resolved to its entity.
    pub(crate) fn resolve_func_type(&self, func_type: DedupFuncType) -> &FuncType {
        let entity_index = self.unwrap_index(func_type.into_inner());
        self.func_types
            .get(entity_index)
            .unwrap_or_else(|| panic!("failed to resolve stored function type: {:?}", entity_index))
    }
}
