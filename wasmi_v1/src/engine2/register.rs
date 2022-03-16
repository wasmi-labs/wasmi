//! Data structures to represent the Wasm value stack during execution.

use crate::core::{Value, ValueType, F32, F64};
use core::fmt::Debug;

/// A single entry or register in a register.
///
/// # Note
///
/// This is a thin-wrapper around [`u64`] to allow us to treat runtime values
/// as efficient tag-free [`u64`] values. Bits that are not required by the runtime
/// value are set to zero.
/// This is safe since all of the supported runtime values fit into [`u64`] and since
/// Wasm modules are validated before execution so that invalid representations do not
/// occur, e.g. interpreting a value of 42 as a [`bool`] value.
///
/// At the boundary between the interpreter and the outside world we convert the
/// stack entry value into the required `Value` type which can then be matched on.
/// It is only possible to convert a [`RegisterEntry`] into a [`Value`] if and only if
/// the type is statically known which always is the case at these boundaries.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct RegisterEntry(u64);

impl RegisterEntry {
    /// Returns the underlying bits of the [`RegisterEntry`].
    pub fn to_bits(self) -> u64 {
        self.0
    }

    /// Converts the untyped [`RegisterEntry`] value into a typed [`Value`].
    pub fn with_type(self, value_type: ValueType) -> Value {
        match value_type {
            ValueType::I32 => Value::I32(<_>::from_stack_entry(self)),
            ValueType::I64 => Value::I64(<_>::from_stack_entry(self)),
            ValueType::F32 => Value::F32(<_>::from_stack_entry(self)),
            ValueType::F64 => Value::F64(<_>::from_stack_entry(self)),
        }
    }
}

impl From<Value> for RegisterEntry {
    fn from(value: Value) -> Self {
        match value {
            Value::I32(value) => value.into(),
            Value::I64(value) => value.into(),
            Value::F32(value) => value.into(),
            Value::F64(value) => value.into(),
        }
    }
}

/// Trait used to convert untyped values of [`RegisterEntry`] into typed values.
pub trait FromRegisterEntry
where
    Self: Sized,
{
    /// Converts the untyped [`RegisterEntry`] into the typed `Self` value.
    ///
    /// # Note
    ///
    /// This heavily relies on the fact that executed Wasm is validated
    /// before execution and therefore might result in conversions that
    /// are only valid in a validated context, e.g. so that a stack entry
    /// with a value of 42 is not interpreted as [`bool`] which does not
    /// have a corresponding representation for 42.
    fn from_stack_entry(entry: RegisterEntry) -> Self;
}

macro_rules! impl_from_stack_entry_integer {
    ($($t:ty),* $(,)?) =>	{
        $(
            impl FromRegisterEntry for $t {
                fn from_stack_entry(entry: RegisterEntry) -> Self {
                    entry.to_bits() as _
                }
            }

            impl From<$t> for RegisterEntry {
                fn from(value: $t) -> Self {
                    Self(value as _)
                }
            }
        )*
    };
}
impl_from_stack_entry_integer!(i8, u8, i16, u16, i32, u32, i64, u64);

macro_rules! impl_from_stack_entry_float {
    ($($t:ty),*) =>	{
        $(
            impl FromRegisterEntry for $t {
                fn from_stack_entry(entry: RegisterEntry) -> Self {
                    Self::from_bits(entry.to_bits() as _)
                }
            }

            impl From<$t> for RegisterEntry {
                fn from(value: $t) -> Self {
                    Self(value.to_bits() as _)
                }
            }
        )*
    };
}
impl_from_stack_entry_float!(f32, f64, F32, F64);

impl From<bool> for RegisterEntry {
    fn from(value: bool) -> Self {
        Self(value as _)
    }
}

impl FromRegisterEntry for bool {
    fn from_stack_entry(entry: RegisterEntry) -> Self {
        entry.to_bits() != 0
    }
}
