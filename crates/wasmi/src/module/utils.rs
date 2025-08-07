use crate::{FuncType, GlobalType, MemoryType, Mutability, TableType, ValType};
use wasmparser::AbstractHeapType;

/// Types that can be created from `wasmparser` definitions.
pub(crate) trait FromWasmparser<T> {
    /// Create `Self` from the `wasmparser` definition.
    ///
    /// # Panics
    ///
    /// If creation of `Self` is not possible.
    fn from_wasmparser(value: T) -> Self;
}

impl FromWasmparser<wasmparser::TableType> for TableType {
    /// Creates a new [`TableType`] from the given `wasmparser` primitive.
    ///
    /// # Dev. Note
    ///
    /// We do not use the `From` trait here so that this conversion
    /// routine does not become part of the public API of [`TableType`].
    fn from_wasmparser(table_type: wasmparser::TableType) -> Self {
        let element = WasmiValueType::from(table_type.element_type).into_inner();
        let minimum: u64 = table_type.initial;
        let maximum: Option<u64> = table_type.maximum;
        match table_type.table64 {
            true => Self::new64(element, minimum, maximum),
            false => {
                let Ok(minimum) = u32::try_from(minimum) else {
                    panic!("invalid 32-bit table.minimum: {minimum}")
                };
                let Ok(maximum) = maximum.map(u32::try_from).transpose() else {
                    panic!("invalid 32-bit table.maximum: {maximum:?}")
                };
                Self::new(element, minimum, maximum)
            }
        }
    }
}

impl FromWasmparser<wasmparser::MemoryType> for MemoryType {
    /// Creates a new [`MemoryType`] from the given `wasmparser` primitive.
    ///
    /// # Dev. Note
    ///
    /// We do not use the `From` trait here so that this conversion
    /// routine does not become part of the public API of [`MemoryType`].
    fn from_wasmparser(memory_type: wasmparser::MemoryType) -> Self {
        assert!(
            !memory_type.shared,
            "wasmi does not support the `threads` Wasm proposal"
        );
        let mut b = Self::builder();
        b.min(memory_type.initial);
        b.max(memory_type.maximum);
        b.memory64(memory_type.memory64);
        if let Some(page_size_log2) = memory_type.page_size_log2 {
            let Ok(page_size_log2) = u8::try_from(page_size_log2) else {
                panic!("page size (in log2) must be a valid `u8` if any");
            };
            b.page_size_log2(page_size_log2);
        }
        b.build()
            .unwrap_or_else(|err| panic!("received invalid `MemoryType` from `wasmparser`: {err}"))
    }
}

impl FromWasmparser<wasmparser::GlobalType> for GlobalType {
    /// Creates a new [`GlobalType`] from the given `wasmparser` primitive.
    ///
    /// # Dev. Note
    ///
    /// We do not use the `From` trait here so that this conversion
    /// routine does not become part of the public API of [`GlobalType`].
    fn from_wasmparser(global_type: wasmparser::GlobalType) -> Self {
        let value_type = WasmiValueType::from(global_type.content_type).into_inner();
        let mutability = match global_type.mutable {
            true => Mutability::Var,
            false => Mutability::Const,
        };
        Self::new(value_type, mutability)
    }
}

impl FromWasmparser<&wasmparser::FuncType> for FuncType {
    /// Creates a new [`FuncType`] from the given `wasmparser` primitive.
    ///
    /// # Dev. Note
    ///
    /// We do not use the `From` trait here so that this conversion
    /// routine does not become part of the public API of [`FuncType`].
    fn from_wasmparser(func_type: &wasmparser::FuncType) -> Self {
        /// Returns the [`ValType`] from the given [`wasmparser::Type`].
        ///
        /// # Panics
        ///
        /// If the [`wasmparser::Type`] is not supported by Wasmi.
        fn extract_value_type(value_type: &wasmparser::ValType) -> ValType {
            WasmiValueType::from(*value_type).into_inner()
        }
        let params = func_type.params().iter().map(extract_value_type);
        let results = func_type.results().iter().map(extract_value_type);
        Self::new(params, results)
    }
}

/// A Wasmi [`ValType`].
///
/// # Note
///
/// This new-type wrapper exists so that we can implement the `From` trait.
pub struct WasmiValueType {
    inner: ValType,
}

impl WasmiValueType {
    /// Returns the inner [`ValType`].
    pub fn into_inner(self) -> ValType {
        self.inner
    }
}

impl From<ValType> for WasmiValueType {
    fn from(value: ValType) -> Self {
        Self { inner: value }
    }
}

impl From<wasmparser::HeapType> for WasmiValueType {
    fn from(heap_type: wasmparser::HeapType) -> Self {
        match heap_type {
            wasmparser::HeapType::Abstract {
                shared: false,
                ty: AbstractHeapType::Func,
            } => Self::from(ValType::FuncRef),
            wasmparser::HeapType::Abstract {
                shared: false,
                ty: AbstractHeapType::Extern,
            } => Self::from(ValType::ExternRef),
            unsupported => panic!("encountered unsupported heap type: {unsupported:?}"),
        }
    }
}

impl From<wasmparser::RefType> for WasmiValueType {
    fn from(ref_type: wasmparser::RefType) -> Self {
        match ref_type {
            wasmparser::RefType::FUNCREF => Self::from(ValType::FuncRef),
            wasmparser::RefType::EXTERNREF => Self::from(ValType::ExternRef),
            unsupported => panic!("encountered unsupported reference type: {unsupported:?}"),
        }
    }
}

impl From<wasmparser::ValType> for WasmiValueType {
    fn from(value_type: wasmparser::ValType) -> Self {
        match value_type {
            wasmparser::ValType::I32 => Self::from(ValType::I32),
            wasmparser::ValType::I64 => Self::from(ValType::I64),
            wasmparser::ValType::F32 => Self::from(ValType::F32),
            wasmparser::ValType::F64 => Self::from(ValType::F64),
            wasmparser::ValType::V128 => Self::from(ValType::V128),
            wasmparser::ValType::Ref(ref_type) => WasmiValueType::from(ref_type),
        }
    }
}
