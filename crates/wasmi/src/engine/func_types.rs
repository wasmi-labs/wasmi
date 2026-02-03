use crate::{
    FuncType,
    collections::arena::{ArenaKey, DedupArena},
    engine::{EngineId, EngineOwned},
    store::RawHandle,
};

define_handle! {
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
    #[derive(PartialEq, Eq)]
    struct DedupFuncType(u32, EngineOwned) => FuncType;
}

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
    func_types: DedupArena<RawHandle<DedupFuncType>, FuncType>,
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
        let key = match self.func_types.alloc(func_type) {
            Ok(key) => key,
            Err(err) => panic!("failed to alloc func type: {err}"),
        };
        DedupFuncType(self.engine_id.wrap(key))
    }

    /// Resolves a deduplicated function type into a [`FuncType`] entity.
    ///
    /// # Panics
    ///
    /// - If the deduplicated function type is not owned by the engine.
    /// - If the deduplicated function type cannot be resolved to its entity.
    pub(crate) fn resolve_func_type(&self, key: &DedupFuncType) -> &FuncType {
        let raw_key = self.unwrap_or_panic(key.0);
        self.func_types
            .get(raw_key)
            .unwrap_or_else(|err| panic!("failed to resolve function type at {key:?}: {err}"))
    }
}
