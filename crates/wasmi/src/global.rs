use super::{AsContext, AsContextMut, Stored};
use crate::{core::ValueType, value::WithType, Value};
use core::{fmt, fmt::Display, ptr::NonNull};
use wasmi_arena::ArenaIndex;
use wasmi_core::UntypedValue;

/// A raw index to a global variable entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalIdx(u32);

impl ArenaIndex for GlobalIdx {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("index {value} is out of bounds as global index: {error}")
        });
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
    /// Occurs when a global type does not satisfy the constraints of another.
    UnsatisfyingGlobalType {
        /// The unsatisfying [`GlobalType`].
        unsatisfying: GlobalType,
        /// The required [`GlobalType`].
        required: GlobalType,
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
                    "type mismatch upon writing global variable. \
                    expected {expected:?} but encountered {encountered:?}.",
                )
            }
            Self::UnsatisfyingGlobalType {
                unsatisfying,
                required,
            } => {
                write!(
                    f,
                    "global type {unsatisfying:?} does not \
                    satisfy requirements of {required:?}",
                )
            }
        }
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
    content: ValueType,
    /// The mutability of the global variable.
    mutability: Mutability,
}

impl GlobalType {
    /// Creates a new [`GlobalType`] from the given [`ValueType`] and [`Mutability`].
    pub fn new(content: ValueType, mutability: Mutability) -> Self {
        Self {
            content,
            mutability,
        }
    }

    /// Returns the [`ValueType`] of the global variable.
    pub fn content(&self) -> ValueType {
        self.content
    }

    /// Returns the [`Mutability`] of the global variable.
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    /// Checks if `self` satisfies the given `GlobalType`.
    ///
    /// # Errors
    ///
    /// - If the initial limits of the `required` [`GlobalType`] are greater than `self`.
    /// - If the maximum limits of the `required` [`GlobalType`] are greater than `self`.
    pub(crate) fn satisfies(&self, required: &GlobalType) -> Result<(), GlobalError> {
        if self != required {
            return Err(GlobalError::UnsatisfyingGlobalType {
                unsatisfying: *self,
                required: *required,
            });
        }
        Ok(())
    }
}

/// A global variable entity.
#[derive(Debug)]
pub struct GlobalEntity {
    /// The current value of the global variable.
    value: UntypedValue,
    /// The type of the global variable.
    ty: GlobalType,
}

impl GlobalEntity {
    /// Creates a new global entity with the given initial value and mutability.
    pub fn new(initial_value: Value, mutability: Mutability) -> Self {
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
    /// - If the global variable is immutable.
    /// - If there is a type mismatch between the global variable and the new value.
    pub fn set(&mut self, new_value: Value) -> Result<(), GlobalError> {
        if !self.ty().mutability().is_mut() {
            return Err(GlobalError::ImmutableWrite);
        }
        if self.ty().content() != new_value.ty() {
            return Err(GlobalError::TypeMismatch {
                expected: self.ty().content(),
                encountered: new_value.ty(),
            });
        }
        self.set_untyped(new_value.into());
        Ok(())
    }

    /// Sets a new untyped value for the global variable.
    ///
    /// # Note
    ///
    /// This is an inherently unsafe API and only exists to allow
    /// for efficient `global.set` through the interpreter which is
    /// safe since the interpreter only handles validated Wasm code
    /// where the checks in [`Global::set`] cannot fail.
    pub(crate) fn set_untyped(&mut self, new_value: UntypedValue) {
        self.value = new_value;
    }

    /// Returns the current value of the global variable.
    pub fn get(&self) -> Value {
        self.get_untyped().with_type(self.ty().content())
    }

    /// Returns the current untyped value of the global variable.
    pub(crate) fn get_untyped(&self) -> UntypedValue {
        self.value
    }

    /// Returns a pointer to the untyped value of the global variable.
    pub(crate) fn get_untyped_ptr(&mut self) -> NonNull<UntypedValue> {
        NonNull::from(&mut self.value)
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
    /// [`Store`]: [`crate::Store`]
    pub(super) fn from_inner(stored: Stored<GlobalIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn as_inner(&self) -> &Stored<GlobalIdx> {
        &self.0
    }

    /// Creates a new global variable to the store.
    pub fn new(mut ctx: impl AsContextMut, initial_value: Value, mutability: Mutability) -> Self {
        ctx.as_context_mut()
            .store
            .inner
            .alloc_global(GlobalEntity::new(initial_value, mutability))
    }

    /// Returns the [`GlobalType`] of the global variable.
    pub fn ty(&self, ctx: impl AsContext) -> GlobalType {
        ctx.as_context().store.inner.resolve_global(self).ty()
    }

    /// Sets a new value to the global variable.
    ///
    /// # Errors
    ///
    /// - If the global variable is immutable.
    /// - If there is a type mismatch between the global variable and the new value.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Global`].
    pub fn set(&self, mut ctx: impl AsContextMut, new_value: Value) -> Result<(), GlobalError> {
        ctx.as_context_mut()
            .store
            .inner
            .resolve_global_mut(self)
            .set(new_value)
    }

    /// Returns the current value of the global variable.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Global`].
    pub fn get(&self, ctx: impl AsContext) -> Value {
        ctx.as_context().store.inner.resolve_global(self).get()
    }
}
