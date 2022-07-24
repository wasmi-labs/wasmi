use crate::{pwasm::PwasmCompat, ValueType};
use alloc::borrow::Cow;
use parity_wasm::elements::{FunctionType, GlobalType, MemoryType, TableType};

/// Signature of a [function].
///
/// Signature of a function consists of zero or more parameter [types][type] and zero or one return [type].
///
/// Two signatures are considered equal if they have equal list of parameters and equal return types.
///
/// [type]: enum.ValueType.html
/// [function]: struct.FuncInstance.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    params: Cow<'static, [ValueType]>,
    return_type: Option<ValueType>,
}

impl Signature {
    /// Creates new signature with givens
    /// parameter types and optional return type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wasmi::{Signature, ValueType};
    ///
    /// // s1: (i32) -> ()
    /// let s1 = Signature::new(&[ValueType::I32][..], None);
    ///
    /// // s2: () -> i32
    /// let s2 = Signature::new(&[][..], Some(ValueType::I32));
    ///
    /// // s3: (I64) -> ()
    /// let dynamic_params = vec![ValueType::I64];
    /// let s3 = Signature::new(dynamic_params, None);
    /// ```
    pub fn new<C: Into<Cow<'static, [ValueType]>>>(
        params: C,
        return_type: Option<ValueType>,
    ) -> Signature {
        Signature {
            params: params.into(),
            return_type,
        }
    }

    /// Returns parameter types of this signature.
    pub fn params(&self) -> &[ValueType] {
        self.params.as_ref()
    }

    /// Returns return type of this signature.
    pub fn return_type(&self) -> Option<ValueType> {
        self.return_type
    }

    pub(crate) fn from_elements(func_type: &FunctionType) -> Signature {
        Signature {
            params: func_type
                .params()
                .iter()
                .cloned()
                .map(ValueType::from_elements)
                .collect(),
            return_type: func_type
                .results()
                .first()
                .map(|vty| ValueType::from_elements(*vty)),
        }
    }
}

/// Description of a global variable.
///
/// Primarly used to describe imports of global variables.
/// See [`ImportResolver`] for details.
///
/// [`ImportResolver`]: trait.ImportResolver.html
pub struct GlobalDescriptor {
    value_type: ValueType,
    mutable: bool,
}

impl GlobalDescriptor {
    pub(crate) fn from_elements(global_type: &GlobalType) -> GlobalDescriptor {
        GlobalDescriptor {
            value_type: ValueType::from_elements(global_type.content_type()),
            mutable: global_type.is_mutable(),
        }
    }

    /// Returns [`ValueType`] of the requested global.
    ///
    /// [`ValueType`]: enum.ValueType.html
    pub fn value_type(&self) -> ValueType {
        self.value_type
    }

    /// Returns whether the requested global mutable.
    pub fn is_mutable(&self) -> bool {
        self.mutable
    }
}

/// Description of a table.
///
/// Primarly used to describe imports of tables.
/// See [`ImportResolver`] for details.
///
/// [`ImportResolver`]: trait.ImportResolver.html
pub struct TableDescriptor {
    initial: u32,
    maximum: Option<u32>,
}

impl TableDescriptor {
    pub(crate) fn from_elements(table_type: &TableType) -> TableDescriptor {
        TableDescriptor {
            initial: table_type.limits().initial(),
            maximum: table_type.limits().maximum(),
        }
    }

    /// Returns initial size of the requested table.
    pub fn initial(&self) -> u32 {
        self.initial
    }

    /// Returns maximum size of the requested table.
    pub fn maximum(&self) -> Option<u32> {
        self.maximum
    }
}

/// Description of a linear memory.
///
/// Primarly used to describe imports of linear memories.
/// See [`ImportResolver`] for details.
///
/// [`ImportResolver`]: trait.ImportResolver.html
pub struct MemoryDescriptor {
    initial: u32,
    maximum: Option<u32>,
}

impl MemoryDescriptor {
    pub(crate) fn from_elements(memory_type: &MemoryType) -> MemoryDescriptor {
        MemoryDescriptor {
            initial: memory_type.limits().initial(),
            maximum: memory_type.limits().maximum(),
        }
    }

    /// Returns initial size (in pages) of the requested memory.
    pub fn initial(&self) -> u32 {
        self.initial
    }

    /// Returns maximum size (in pages) of the requested memory.
    pub fn maximum(&self) -> Option<u32> {
        self.maximum
    }
}
