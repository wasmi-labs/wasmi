use super::{stack::ValueStack, Provider, TypedProvider};
use crate::{
    core::IndexType,
    ir::{BoundedRegSpan, Const16, Const32, Reg, RegSpan},
    Error,
};

/// Bail out early in case the current code is unreachable.
///
/// # Note
///
/// - This should be prepended to most Wasm operator translation procedures.
/// - If we are in unreachable code most Wasm translation is skipped. Only
///   certain control flow operators such as `End` are going through the
///   translation process. In particular the `End` operator may end unreachable
///   code blocks.
macro_rules! bail_unreachable {
    ($this:ident) => {{
        if !$this.is_reachable() {
            return Ok(());
        }
    }};
}

macro_rules! impl_provider_new_const16 {
    ($ty:ty) => {
        impl Provider<Const16<$ty>> {
            /// Creates a new `table` or `memory` index [`Provider`] from the general [`TypedProvider`].
            ///
            /// # Note
            ///
            /// This is a convenience function and used by translation
            /// procedures for certain Wasm `table` instructions.
            pub fn new(provider: TypedProvider, stack: &mut ValueStack) -> Result<Self, Error> {
                match provider {
                    TypedProvider::Const(value) => match Const16::try_from(<$ty>::from(value)).ok()
                    {
                        Some(value) => Ok(Self::Const(value)),
                        None => {
                            let register = stack.alloc_const(value)?;
                            Ok(Self::Register(register))
                        }
                    },
                    TypedProvider::Register(index) => Ok(Self::Register(index)),
                }
            }
        }
    };
}
impl_provider_new_const16!(u32);
impl_provider_new_const16!(u64);

impl super::FuncTranslator {
    /// Converts the `provider` to a 16-bit index-type constant value.
    ///
    /// # Note
    ///
    /// - Turns immediates that cannot be 16-bit encoded into function local constants.
    /// - The behavior is different whether `memory64` is enabled or disabled.
    pub(super) fn as_index_type_const16(
        &mut self,
        provider: TypedProvider,
        index_type: IndexType,
    ) -> Result<Provider<Const16<u64>>, Error> {
        let value = match provider {
            Provider::Register(reg) => return Ok(Provider::Register(reg)),
            Provider::Const(value) => value,
        };
        match index_type {
            IndexType::I64 => {
                if let Ok(value) = Const16::try_from(u64::from(value)) {
                    return Ok(Provider::Const(value));
                }
            }
            IndexType::I32 => {
                if let Ok(value) = Const16::try_from(u32::from(value)) {
                    return Ok(Provider::Const(<Const16<u64>>::cast(value)));
                }
            }
        }
        let register = self.stack.alloc_const(value)?;
        Ok(Provider::Register(register))
    }

    /// Converts the `provider` to a 32-bit index-type constant value.
    ///
    /// # Note
    ///
    /// - Turns immediates that cannot be 32-bit encoded into function local constants.
    /// - The behavior is different whether `memory64` is enabled or disabled.
    pub(super) fn as_index_type_const32(
        &mut self,
        provider: TypedProvider,
        index_type: IndexType,
    ) -> Result<Provider<Const32<u64>>, Error> {
        let value = match provider {
            Provider::Register(reg) => return Ok(Provider::Register(reg)),
            Provider::Const(value) => value,
        };
        match index_type {
            IndexType::I64 => {
                if let Ok(value) = Const32::try_from(u64::from(value)) {
                    return Ok(Provider::Const(value));
                }
            }
            IndexType::I32 => {
                let value = Const32::from(u32::from(value));
                return Ok(Provider::Const(<Const32<u64>>::cast(value)));
            }
        }
        let register = self.stack.alloc_const(value)?;
        Ok(Provider::Register(register))
    }
}

impl TypedProvider {
    /// Returns the `i16` [`Reg`] index if the [`TypedProvider`] is a [`Reg`].
    fn register_index(&self) -> Option<i16> {
        match self {
            TypedProvider::Register(index) => Some(i16::from(*index)),
            TypedProvider::Const(_) => None,
        }
    }
}

/// Extension trait to create a [`BoundedRegSpan`] from a slice of [`TypedProvider`]s.
pub trait FromProviders: Sized {
    /// Creates a [`BoundedRegSpan`] from the given slice of [`TypedProvider`] if possible.
    ///
    /// All [`TypedProvider`] must be [`Reg`] and have
    /// contiguous indices for the conversion to succeed.
    ///
    /// Returns `None` if the `providers` slice is empty.
    fn from_providers(providers: &[TypedProvider]) -> Option<Self>;
}

impl FromProviders for BoundedRegSpan {
    fn from_providers(providers: &[TypedProvider]) -> Option<Self> {
        let (first, rest) = providers.split_first()?;
        let first_index = first.register_index()?;
        let mut prev_index = first_index;
        for next in rest {
            let next_index = next.register_index()?;
            if next_index.checked_sub(prev_index)? != 1 {
                return None;
            }
            prev_index = next_index;
        }
        let end_index = prev_index.checked_add(1)?;
        let len = (end_index - first_index) as u16;
        Some(Self::new(RegSpan::new(Reg::from(first_index)), len))
    }
}
