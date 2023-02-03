use crate::{FuncType, GlobalType, MemoryType, Mutability, TableType};
use wasmi_core::ValueType;

impl TableType {
    /// Creates a new [`TableType`] from the given `wasmparser` primitive.
    ///
    /// # Dev. Note
    ///
    /// We do not use the `From` trait here so that this conversion
    /// routine does not become part of the public API of [`TableType`].
    pub(crate) fn from_wasmparser(table_type: wasmparser::TableType) -> Self {
        let element = WasmiValueType::from(table_type.element_type).into_inner();
        let minimum = table_type.initial;
        let maximum = table_type.maximum;
        Self::new(element, minimum, maximum)
    }
}

impl MemoryType {
    /// Creates a new [`MemoryType`] from the given `wasmparser` primitive.
    ///
    /// # Dev. Note
    ///
    /// We do not use the `From` trait here so that this conversion
    /// routine does not become part of the public API of [`MemoryType`].
    pub(crate) fn from_wasmparser(memory_type: wasmparser::MemoryType) -> Self {
        assert!(
            !memory_type.memory64,
            "wasmi does not support the `memory64` Wasm proposal"
        );
        assert!(
            !memory_type.shared,
            "wasmi does not support the `threads` Wasm proposal"
        );
        let initial: u32 = memory_type
            .initial
            .try_into()
            .expect("wasm32 memories must have a valid u32 minimum size");
        let maximum: Option<u32> = memory_type
            .maximum
            .map(TryInto::try_into)
            .transpose()
            .expect("wasm32 memories must have a valid u32 maximum size if any");
        Self::new(initial, maximum)
            .expect("encountered invalid wasmparser::MemoryType after validation")
    }
}

impl GlobalType {
    /// Creates a new [`GlobalType`] from the given `wasmparser` primitive.
    ///
    /// # Dev. Note
    ///
    /// We do not use the `From` trait here so that this conversion
    /// routine does not become part of the public API of [`GlobalType`].
    pub(crate) fn from_wasmparser(global_type: wasmparser::GlobalType) -> Self {
        let value_type = WasmiValueType::from(global_type.content_type).into_inner();
        let mutability = match global_type.mutable {
            true => Mutability::Var,
            false => Mutability::Const,
        };
        Self::new(value_type, mutability)
    }
}

impl FuncType {
    /// Creates a new [`FuncType`] from the given `wasmparser` primitive.
    ///
    /// # Dev. Note
    ///
    /// We do not use the `From` trait here so that this conversion
    /// routine does not become part of the public API of [`FuncType`].
    pub(crate) fn from_wasmparser(func_type: wasmparser::FuncType) -> Self {
        /// Returns the [`ValueType`] from the given [`wasmparser::Type`].
        ///
        /// # Panics
        ///
        /// If the [`wasmparser::Type`] is not supported by `wasmi`.
        fn extract_value_type(value_type: &wasmparser::ValType) -> ValueType {
            WasmiValueType::from(*value_type).into_inner()
        }
        let params = func_type.params().iter().map(extract_value_type);
        let results = func_type.results().iter().map(extract_value_type);
        Self::new(params, results)
    }
}

/// A `wasmi` [`ValueType`].
///
/// # Note
///
/// This new-type wrapper exists so that we can implement the `From` trait.
pub struct WasmiValueType {
    inner: ValueType,
}

impl WasmiValueType {
    /// Returns the inner [`ValueType`].
    pub fn into_inner(self) -> ValueType {
        self.inner
    }
}

impl From<ValueType> for WasmiValueType {
    fn from(value: ValueType) -> Self {
        Self { inner: value }
    }
}

impl From<wasmparser::ValType> for WasmiValueType {
    fn from(value_type: wasmparser::ValType) -> Self {
        match value_type {
            wasmparser::ValType::I32 => Self::from(ValueType::I32),
            wasmparser::ValType::I64 => Self::from(ValueType::I64),
            wasmparser::ValType::F32 => Self::from(ValueType::F32),
            wasmparser::ValType::F64 => Self::from(ValueType::F64),
            wasmparser::ValType::V128 => panic!("wasmi does not support the `simd` Wasm proposal"),
            wasmparser::ValType::FuncRef => Self::from(ValueType::FuncRef),
            wasmparser::ValType::ExternRef => Self::from(ValueType::ExternRef),
        }
    }
}
