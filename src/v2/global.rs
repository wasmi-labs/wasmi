use super::Index;
use super::{AsContext, AsContextMut, Stored};
use crate::{RuntimeValue, ValueType};
use core::fmt;
use core::fmt::Display;

/// A raw index to a global variable entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalIdx(usize);

impl Index for GlobalIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        Self(value)
    }
}

/// An error that may occur upon operating on global variables.
#[derive(Debug)]
#[non_exhaustive]
pub enum GlobalError {
    /// Occurs when trying to write to an immutable global variable.
    ImmutableWrite,
    /// Occurs when trying writing a value with mismatching type to a global variable.
    TypeMismatch {
        /// The type of the global variable.
        expected: ValueType,
        /// The type of the new value that mismatches the type of the global variable.
        encountered: ValueType,
    },
}

impl Display for GlobalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ImmutableWrite => write!(f, "tried to write to immutable global variable"),
            Self::TypeMismatch {
                expected,
                encountered,
            } => {
                write!(
                    f,
                    "type mismatch upon writing global variable. expected {} but encountered {}.",
                    expected, encountered,
                )
            }
        }
    }
}

/// The mutability of a global variable.
#[derive(Debug, Copy, Clone)]
pub enum Mutability {
    /// The value of the global variable is a constant.
    Const,
    /// The value of the global variable is mutable.
    Mutable,
}

/// A global variable entitiy.
#[derive(Debug)]
pub struct GlobalEntity {
    value: RuntimeValue,
    mutability: Mutability,
}

impl GlobalEntity {
    /// Creates a new global entity with the given initial value and mutability.
    pub fn new(initial_value: RuntimeValue, mutability: Mutability) -> Self {
        Self {
            value: initial_value,
            mutability,
        }
    }

    /// Returns `true` if the global variable is mutable.
    pub fn is_mutable(&self) -> bool {
        matches!(self.mutability, Mutability::Mutable)
    }

    /// Returns the type of the global variable value.
    pub fn value_type(&self) -> ValueType {
        self.value.value_type()
    }

    /// Sets a new value to the global variable.
    ///
    /// # Errors
    ///
    /// - If the global variable is immutable.
    /// - If there is a type mismatch between the global variable and the new value.
    pub fn set(&mut self, new_value: RuntimeValue) -> Result<(), GlobalError> {
        if !self.is_mutable() {
            return Err(GlobalError::ImmutableWrite);
        }
        if self.value_type() != new_value.value_type() {
            return Err(GlobalError::TypeMismatch {
                expected: self.value_type(),
                encountered: new_value.value_type(),
            });
        }
        self.value = new_value;
        Ok(())
    }

    /// Returns the current value of the global variable.
    pub fn get(&self) -> RuntimeValue {
        self.value
    }
}

/// A Wasm global variable reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Global(Stored<GlobalIdx>);

impl Global {
    /// Creates a new stored global variable reference.
    ///
    /// # Note
    ///
    /// This API is primarily used by the [`Store`] itself.
    ///
    /// [`Store`]: [`crate::v2::Store`]
    pub(super) fn from_inner(stored: Stored<GlobalIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn into_inner(self) -> Stored<GlobalIdx> {
        self.0
    }

    /// Creates a new global variable to the store.
    pub fn new(
        mut ctx: impl AsContextMut,
        initial_value: RuntimeValue,
        mutability: Mutability,
    ) -> Self {
        ctx.as_context_mut()
            .store
            .alloc_global(GlobalEntity::new(initial_value, mutability))
    }

    /// Returns `true` if the global variable is mutable.
    pub fn is_mutable(&self, ctx: impl AsContext) -> bool {
        ctx.as_context().store.resolve_global(*self).is_mutable()
    }

    /// Returns the type of the global variable value.
    pub fn value_type(&self, ctx: impl AsContext) -> ValueType {
        ctx.as_context().store.resolve_global(*self).value_type()
    }

    /// Sets a new value to the global variable.
    ///
    /// # Errors
    ///
    /// - If the global variable is immutable.
    /// - If there is a type mismatch between the global variable and the new value.
    pub fn set(
        &mut self,
        mut ctx: impl AsContextMut,
        new_value: RuntimeValue,
    ) -> Result<(), GlobalError> {
        ctx.as_context_mut()
            .store
            .resolve_global_mut(*self)
            .set(new_value)
    }

    /// Returns the current value of the global variable.
    pub fn get(&self, ctx: impl AsContext) -> RuntimeValue {
        ctx.as_context().store.resolve_global(*self).get()
    }
}
