use super::{EngineId, EngineOwned};
use crate::{
    FuncType,
    collections::arena::{ArenaKey, DedupArena},
};

/// A raw index to a function signature entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DedupFuncTypeIdx(u32);

impl ArenaKey for DedupFuncTypeIdx {
    fn into_usize(self) -> usize {
        self.0.into_usize()
    }

    fn from_usize(value: usize) -> Option<Self> {
        <_ as ArenaKey>::from_usize(value).map(Self)
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DedupFuncType(EngineOwned<DedupFuncTypeIdx>);

/// A [`FuncType`] registry that efficiently deduplicate stored function types.
///
/// Can also be used to later resolve deduplicated function types into their
/// original [`FuncType`] for inspecting their parameter and result types.
///
/// The big advantage of deduplicated [`FuncType`] entities is that we can use
/// this for indirect calls to speed up the signature checks since comparing
/// deduplicated [`FuncType`] instances is as fast as comparing integer values.
/// Also with respect to Wasmi bytecode deduplicated [`FuncType`] entities
/// require a lot less space to be stored.
#[derive(Debug)]
pub struct FuncTypeRegistry {
    /// A unique identifier for the associated engine.
    ///
    /// # Note
    ///
    /// This is used to guard against invalid entity indices.
    engine_id: EngineId,
    /// Deduplicated function types.
    ///
    /// # Note
    ///
    /// The engine deduplicates function types to make the equality
    /// comparison very fast. This helps to speed up indirect calls.
    func_types: DedupArena<DedupFuncTypeIdx, FuncType>,
}

impl FuncTypeRegistry {
    /// Creates a new [`FuncTypeRegistry`] using the given [`EngineId`].
    pub(crate) fn new(engine_id: EngineId) -> Self {
        Self {
            engine_id,
            func_types: DedupArena::default(),
        }
    }

    /// Unpacks the entity and checks if it is owned by the engine.
    ///
    /// # Panics
    ///
    /// If the guarded entity is not owned by the engine.
    fn unwrap_or_panic<T>(&self, func_type: EngineOwned<T>) -> T
    where
        T: ArenaKey,
    {
        self.engine_id.unwrap(func_type).unwrap_or_else(|| {
            panic!(
                "encountered foreign entity in func type registry: {:?}",
                self.engine_id,
            )
        })
    }

    /// Allocates a new function type to the engine.
    pub(crate) fn alloc_func_type(&mut self, func_type: FuncType) -> DedupFuncType {
        DedupFuncType(self.engine_id.wrap(self.func_types.alloc(func_type)))
    }

    /// Resolves a deduplicated function type into a [`FuncType`] entity.
    ///
    /// # Panics
    ///
    /// - If the deduplicated function type is not owned by the engine.
    /// - If the deduplicated function type cannot be resolved to its entity.
    pub(crate) fn resolve_func_type(&self, func_type: &DedupFuncType) -> &FuncType {
        let entity_index = self.unwrap_or_panic(func_type.0);
        self.func_types
            .get(entity_index)
            .unwrap_or_else(|| panic!("failed to resolve stored function type: {entity_index:?}"))
    }
}
