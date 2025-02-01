use wasmparser::AbstractHeapType;

use crate::{core::ValType, FuncType, GlobalType, MemoryType, Mutability, TableType};

impl TableType {
    /// Creates a new [`TableType`] from the given `wasmparser` primitive.
    ///
    /// # Dev. Note
    ///
    /// We do not use the `From` trait here so that this conversion
    /// routine does not become part of the public API of [`TableType`].
    pub(crate) fn from_wasmparser(table_type: wasmparser::TableType) -> Self {
        let element = WasmiValueType::from(table_type.element_type).into_inner();
        let minimum: u32 = table_type
            .initial
            .try_into()
            .unwrap_or_else(|_err| panic!("out of bounds minimum value: {}", table_type.initial));
        let maximum: Option<u32> = match table_type.maximum {
            Some(maximum) => {
                let maximum = maximum
                    .try_into()
                    .unwrap_or_else(|_err| panic!("out of bounds maximum value: {}", maximum));
                Some(maximum)
            }
            None => None,
        };
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
        let minimum: u32 = memory_type
            .initial
            .try_into()
            .expect("minimum linear memory pages must be a valid `u32`");
        let maximum: Option<u32> = memory_type
            .maximum
            .map(u32::try_from)
            .transpose()
            .expect("maximum linear memory pages must be a valid `u32` if any");
        let page_size_log2: Option<u8> = memory_type
            .page_size_log2
            .map(u8::try_from)
            .transpose()
            .expect("page size (in log2) must be a valid `u8` if any");
        let mut b = Self::builder();
        b.min(minimum);
        b.max(maximum);
        if let Some(page_size_log2) = page_size_log2 {
            b.page_size_log2(page_size_log2);
        }
        b.build()
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
    pub(crate) fn from_wasmparser(func_type: &wasmparser::FuncType) -> Self {
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
            wasmparser::ValType::V128 => panic!("wasmi does not support the `simd` Wasm proposal"),
            wasmparser::ValType::Ref(ref_type) => WasmiValueType::from(ref_type),
        }
    }
}
