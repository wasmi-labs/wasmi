use crate::{TypedVal, UntypedVal, ValType};
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
    value: UntypedVal,
    /// The type of the global variable.
    ty: GlobalType,
}

impl Global {
    /// Creates a new global entity with the given initial value and mutability.
    pub fn new(initial_value: TypedVal, mutability: Mutability) -> Self {
        Self {
            ty: GlobalType::new(initial_value.ty(), mutability),
            value: initial_value.into(),
        }
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
    pub fn set(&mut self, new_value: TypedVal) -> Result<(), GlobalError> {
        if !self.ty().mutability().is_mut() {
            return Err(GlobalError::ImmutableWrite);
        }
        if self.ty().content() != new_value.ty() {
            return Err(GlobalError::TypeMismatch);
        }
        self.value = new_value.into();
        Ok(())
    }

    /// Returns the current [`TypedVal`] of the [`Global`].
    pub fn get(&self) -> TypedVal {
        TypedVal::new(self.ty().content(), self.value)
    }

    /// Returns the current [`UntypedVal`] of the [`Global`].
    pub fn get_untyped(&self) -> &UntypedVal {
        &self.value
    }

    /// Returns a pointer to the [`UntypedVal`] of the [`Global`].
    pub fn get_untyped_ptr(&mut self) -> NonNull<UntypedVal> {
        NonNull::from(&mut self.value)
    }
}
