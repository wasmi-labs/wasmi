use crate::{
    core::{UntypedVal, ValType, F32, F64},
    ExternRef,
    Func,
    FuncRef,
};

/// Untyped instances that allow to be typed.
pub trait WithType {
    /// The typed output type.
    type Output;

    /// Converts `self` to [`Self::Output`] using `ty`.
    fn with_type(self, ty: ValType) -> Self::Output;
}

impl WithType for UntypedVal {
    type Output = Val;

    fn with_type(self, ty: ValType) -> Self::Output {
        match ty {
            ValType::I32 => Val::I32(self.into()),
            ValType::I64 => Val::I64(self.into()),
            ValType::F32 => Val::F32(self.into()),
            ValType::F64 => Val::F64(self.into()),
            ValType::FuncRef => Val::FuncRef(self.into()),
            ValType::ExternRef => Val::ExternRef(self.into()),
        }
    }
}

impl From<Val> for UntypedVal {
    fn from(value: Val) -> Self {
        match value {
            Val::I32(value) => value.into(),
            Val::I64(value) => value.into(),
            Val::F32(value) => value.into(),
            Val::F64(value) => value.into(),
            Val::FuncRef(value) => value.into(),
            Val::ExternRef(value) => value.into(),
        }
    }
}

/// Runtime representation of a Wasm value.
///
/// Wasm code manipulate values of the four basic value types:
/// integers and floating-point (IEEE 754-2008) data of 32 or 64 bit width each, respectively.
///
/// There is no distinction between signed and unsigned integer types. Instead, integers are
/// interpreted by respective operations as either unsigned or signed in two’s complement representation.
#[derive(Clone, Debug)]
pub enum Val {
    /// Value of 32-bit signed or unsigned integer.
    I32(i32),
    /// Value of 64-bit signed or unsigned integer.
    I64(i64),
    /// Value of 32-bit IEEE 754-2008 floating point number.
    F32(F32),
    /// Value of 64-bit IEEE 754-2008 floating point number.
    F64(F64),
    /// A nullable [`Func`][`crate::Func`] reference, a.k.a. [`FuncRef`].
    FuncRef(FuncRef),
    /// A nullable external object reference, a.k.a. [`ExternRef`].
    ExternRef(ExternRef),
}

impl Val {
    /// Creates new default value of given type.
    #[inline]
    pub fn default(value_type: ValType) -> Self {
        match value_type {
            ValType::I32 => Self::I32(0),
            ValType::I64 => Self::I64(0),
            ValType::F32 => Self::F32(0f32.into()),
            ValType::F64 => Self::F64(0f64.into()),
            ValType::FuncRef => Self::from(FuncRef::null()),
            ValType::ExternRef => Self::from(ExternRef::null()),
        }
    }

    /// Get variable type for this value.
    #[inline]
    pub fn ty(&self) -> ValType {
        match *self {
            Self::I32(_) => ValType::I32,
            Self::I64(_) => ValType::I64,
            Self::F32(_) => ValType::F32,
            Self::F64(_) => ValType::F64,
            Self::FuncRef(_) => ValType::FuncRef,
            Self::ExternRef(_) => ValType::ExternRef,
        }
    }

    /// Returns the underlying `i32` if the type matches otherwise returns `None`.
    pub fn i32(&self) -> Option<i32> {
        match self {
            Self::I32(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the underlying `i64` if the type matches otherwise returns `None`.
    pub fn i64(&self) -> Option<i64> {
        match self {
            Self::I64(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the underlying `f32` if the type matches otherwise returns `None`.
    pub fn f32(&self) -> Option<F32> {
        match self {
            Self::F32(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the underlying `f64` if the type matches otherwise returns `None`.
    pub fn f64(&self) -> Option<F64> {
        match self {
            Self::F64(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the underlying `funcref` if the type matches otherwise returns `None`.
    pub fn funcref(&self) -> Option<&FuncRef> {
        match self {
            Self::FuncRef(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the underlying `externref` if the type matches otherwise returns `None`.
    pub fn externref(&self) -> Option<&ExternRef> {
        match self {
            Self::ExternRef(value) => Some(value),
            _ => None,
        }
    }
}

impl From<i32> for Val {
    #[inline]
    fn from(val: i32) -> Self {
        Self::I32(val)
    }
}

impl From<i64> for Val {
    #[inline]
    fn from(val: i64) -> Self {
        Self::I64(val)
    }
}

impl From<F32> for Val {
    #[inline]
    fn from(val: F32) -> Self {
        Self::F32(val)
    }
}

impl From<F64> for Val {
    #[inline]
    fn from(val: F64) -> Self {
        Self::F64(val)
    }
}

impl From<FuncRef> for Val {
    #[inline]
    fn from(funcref: FuncRef) -> Self {
        Self::FuncRef(funcref)
    }
}

impl From<Func> for Val {
    #[inline]
    fn from(func: Func) -> Self {
        Self::FuncRef(FuncRef::new(func))
    }
}

impl From<ExternRef> for Val {
    #[inline]
    fn from(externref: ExternRef) -> Self {
        Self::ExternRef(externref)
    }
}
