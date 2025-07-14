use core::slice;
use crate::{
    engine::DedupFuncType,
    module::{utils::WasmiValueType, FuncTypeIdx, ModuleHeader},
    Engine,
    FuncType,
    ValType,
};

/// The type of a Wasm control flow block.
#[derive(Debug, Copy, Clone)]
pub struct BlockType {
    inner: BlockTypeInner,
}

/// The inner workings of the [`BlockType`].
#[derive(Debug, Copy, Clone)]
pub enum BlockTypeInner {
    /// A block type with no parameters and no results.
    Empty,
    /// A block type with no parameters and exactly one result.
    Returns(ValType),
    /// A general block type with parameters and results.
    FuncType(DedupFuncType),
}

impl BlockType {
    /// Creates a new [`BlockType`] from the given [`wasmparser::BlockType`].
    ///
    /// # Errors
    ///
    /// If the conversion is not valid or unsupported.
    pub fn new(block_type: wasmparser::BlockType, res: &ModuleHeader) -> Self {
        match block_type {
            wasmparser::BlockType::Empty => Self::empty(),
            wasmparser::BlockType::Type(return_type) => {
                let return_type = WasmiValueType::from(return_type).into_inner();
                Self::returns(return_type)
            }
            wasmparser::BlockType::FuncType(func_type_idx) => {
                let dedup_func_type = res.get_func_type(FuncTypeIdx::from(func_type_idx));
                Self::func_type(dedup_func_type)
            }
        }
    }

    /// Creates a [`BlockType`] from the underlying type.
    fn from_inner(inner: BlockTypeInner) -> Self {
        Self { inner }
    }

    /// Creates a [`BlockType`] with no parameter and no results.
    fn empty() -> Self {
        Self::from_inner(BlockTypeInner::Empty)
    }

    /// Creates a [`BlockType`] with no parameters and a single result type.
    fn returns(return_type: ValType) -> Self {
        Self::from_inner(BlockTypeInner::Returns(return_type))
    }

    /// Creates a [`BlockType`] with parameters and results.
    pub(crate) fn func_type(func_type: &DedupFuncType) -> Self {
        Self::from_inner(BlockTypeInner::FuncType(*func_type))
    }

    /// Returns the number of parameters of the [`BlockType`].
    pub fn len_params(&self, engine: &Engine) -> u16 {
        match &self.inner {
            BlockTypeInner::Empty | BlockTypeInner::Returns(_) => 0,
            BlockTypeInner::FuncType(func_type) => {
                engine.resolve_func_type(func_type, FuncType::len_params)
            }
        }
    }

    /// Returns the number of results of the [`BlockType`].
    pub fn len_results(&self, engine: &Engine) -> u16 {
        match &self.inner {
            BlockTypeInner::Empty => 0,
            BlockTypeInner::Returns(_) => 1,
            BlockTypeInner::FuncType(func_type) => {
                engine.resolve_func_type(func_type, FuncType::len_results)
            }
        }
    }

    /// Applies `f` to `self`'s [`FuncType`] and returns the result.
    pub fn func_type_with<R>(&self, engine: &Engine, f: impl for<'a> FnOnce(&FuncType) -> R) -> R {
        match &self.inner {
            BlockTypeInner::Empty => f(&FuncType::new([], [])),
            BlockTypeInner::Returns(return_type) => f(&FuncType::new([], [*return_type])),
            BlockTypeInner::FuncType(func_type) => engine.resolve_func_type(func_type, f),
        }
    }
}
