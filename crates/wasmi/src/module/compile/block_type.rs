use super::super::{utils::WasmiValueType, FuncTypeIdx, ModuleResources};
use crate::{core::ValueType, engine::DedupFuncType, Engine};

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
    Returns(ValueType),
    /// A general block type with parameters and results.
    FuncType(DedupFuncType),
}

impl BlockType {
    /// Creates a new [`BlockType`] from the given [`wasmparser::BlockType`].
    ///
    /// # Errors
    ///
    /// If the conversion is not valid or unsupported.
    pub fn new(block_type: wasmparser::BlockType, res: ModuleResources) -> Self {
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
    fn returns(return_type: ValueType) -> Self {
        Self::from_inner(BlockTypeInner::Returns(return_type))
    }

    /// Creates a [`BlockType`] with parameters and results.
    pub(crate) fn func_type(func_type: &DedupFuncType) -> Self {
        Self::from_inner(BlockTypeInner::FuncType(*func_type))
    }

    /// Returns the number of parameters of the [`BlockType`].
    pub fn len_params(&self, engine: &Engine) -> u32 {
        match &self.inner {
            BlockTypeInner::Empty | BlockTypeInner::Returns(_) => 0,
            BlockTypeInner::FuncType(func_type) => {
                engine.resolve_func_type(func_type, |func_type| func_type.params().len() as u32)
            }
        }
    }

    /// Returns the number of results of the [`BlockType`].
    pub fn len_results(&self, engine: &Engine) -> u32 {
        match &self.inner {
            BlockTypeInner::Empty => 0,
            BlockTypeInner::Returns(_) => 1,
            BlockTypeInner::FuncType(func_type) => {
                engine.resolve_func_type(func_type, |func_type| func_type.results().len() as u32)
            }
        }
    }

    /// Calls `f` for each block parameter type.
    pub fn foreach_param<F>(&self, engine: &Engine, mut f: F)
    where
        F: FnMut(ValueType),
    {
        match &self.inner {
            BlockTypeInner::Empty | BlockTypeInner::Returns(_) => (),
            BlockTypeInner::FuncType(func_type) => {
                engine.resolve_func_type(func_type, |func_type| {
                    for param in func_type.params() {
                        f(*param);
                    }
                })
            }
        }
    }

    /// Calls `f` for each block result type.
    pub fn foreach_result<F>(&self, engine: &Engine, mut f: F)
    where
        F: FnMut(ValueType),
    {
        match &self.inner {
            BlockTypeInner::Empty => (),
            BlockTypeInner::Returns(result) => {
                f(*result);
            }
            BlockTypeInner::FuncType(func_type) => {
                engine.resolve_func_type(func_type, |func_type| {
                    for result in func_type.results() {
                        f(*result);
                    }
                })
            }
        }
    }
}
