use crate::{RawVal, TypedRawVal, ValType};
use core::{error::Error, fmt, fmt::Display, ptr::NonNull};

/// An error that may occur upon operating on global variables.
#[derive(Debug)]
#[non_exhaustive]
pub enum GlobalError {
    /// Occurs when trying to write to an immutable global variable.
    ImmutableWrite,
    /// Occurs when trying writing a value with mismatching type to a global variable.
    TypeMismatch,
}

impl Error for GlobalError {}

impl Display for GlobalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::ImmutableWrite => "tried to write to immutable global variable",
            Self::TypeMismatch => "tried to write value of non-matching type to global variable",
        };
        write!(f, "{message}")
    }
}

/// The mutability of a global variable.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mutability {
    /// The value of the global variable is a constant.
    Const,
    /// The value of the global variable is mutable.
    Var,
}

impl Mutability {
    /// Returns `true` if this mutability is [`Mutability::Const`].
    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const)
    }

    /// Returns `true` if this mutability is [`Mutability::Var`].
    pub fn is_mut(&self) -> bool {
        matches!(self, Self::Var)
    }
}

/// The type of a global variable.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GlobalType {
    /// The value type of the global variable.
    content: ValType,
    /// The mutability of the global variable.
    mutability: Mutability,
}

impl GlobalType {
    /// Creates a new [`GlobalType`] from the given [`ValType`] and [`Mutability`].
    pub fn new(content: ValType, mutability: Mutability) -> Self {
        Self {
            content,
            mutability,
        }
    }

    /// Returns the [`ValType`] of the global variable.
    pub fn content(&self) -> ValType {
        self.content
    }

    /// Returns the [`Mutability`] of the global variable.
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }
}

/// A global variable entity.
#[derive(Debug)]
pub struct Global {
    /// The current value of the global variable.
    value: RawVal,
    /// The type of the global variable.
    ty: GlobalType,
}

impl Global {
    /// Creates a new global variable with the given initial `value` and type `ty`.
    pub fn new(value: RawVal, ty: GlobalType) -> Self {
        Self { value, ty }
    }

    /// Returns the [`GlobalType`] of the global variable.
    pub fn ty(&self) -> GlobalType {
        self.ty
    }

    /// Sets a new value to the global variable.
    ///
    /// # Errors
    ///
    /// - If the [`Global`] is immutable.
    /// - If `new_value` does not match the type of the [`Global`].
    pub fn set(&mut self, new_value: TypedRawVal) -> Result<(), GlobalError> {
        if !self.ty().mutability().is_mut() {
            return Err(GlobalError::ImmutableWrite);
        }
        if self.ty().content() != new_value.ty() {
            return Err(GlobalError::TypeMismatch);
        }
        self.value = new_value.into();
        Ok(())
    }

    /// Returns the current [`TypedRawVal`] of the [`Global`].
    pub fn get(&self) -> TypedRawVal {
        TypedRawVal::new(self.ty().content(), self.value)
    }

    /// Returns the current [`RawVal`] of the [`Global`].
    pub fn get_raw(&self) -> &RawVal {
        &self.value
    }

    /// Returns a pointer to the [`RawVal`] of the [`Global`].
    /// 
    /// # Panics (Debug)
    ///
    /// If the underlying global value is immutable.
    pub fn get_raw_ptr(&mut self) -> NonNull<RawVal> {
        debug_assert!(matches!(self.ty.mutability(), Mutability::Var));
        NonNull::from(&mut self.value)
    }
}
