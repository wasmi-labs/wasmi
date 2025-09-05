//! Definitions for thin-wrapper index types.

use crate::Error;

macro_rules! for_each_index {
    ($mac:ident) => {
        $mac! {
            /// A Wasmi stack slot.
            Slot(pub(crate) u16);
            /// A Wasm function index.
            Func(pub(crate) u32);
            /// A Wasm function type index.
            FuncType(pub(crate) u32);
            /// A Wasmi internal function index.
            InternalFunc(pub(crate) u32);
            /// A Wasm global variable index.
            Global(pub(crate) u32);
            /// A Wasm linear memory index.
            Memory(pub(crate) u16);
            /// A Wasm table index.
            Table(pub(crate) u32);
            /// A Wasm data segment index.
            Data(pub(crate) u32);
            /// A Wasm element segment index.
            Elem(pub(crate) u32);
        }
    };
}

impl Memory {
    /// Returns `true` if `self` refers to the default linear memory which always is at index 0.
    pub fn is_default(&self) -> bool {
        self.0 == 0
    }
}

macro_rules! define_index {
    (
        $(
            $( #[$docs:meta] )*
            $name:ident($vis:vis $ty:ty)
        );* $(;)?
    ) => {
        $(
            $( #[$docs] )*
            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $name($vis $ty);

            impl From<$name> for $ty {
                fn from(value: $name) -> $ty {
                    value.0
                }
            }

            impl From<$ty> for $name {
                fn from(value: $ty) -> Self {
                    Self(value)
                }
            }
        )*
    };
}
for_each_index!(define_index);

impl TryFrom<u32> for Slot {
    type Error = Error;

    fn try_from(local_index: u32) -> Result<Self, Self::Error> {
        u16::try_from(local_index)
            .map_err(|_| Error::StackSlotOutOfBounds)
            .map(Self::from)
    }
}

impl Slot {
    /// Returns the n-th next [`Slot`] from `self` with contiguous index.
    ///
    /// # Note
    ///
    /// - Calling this with `n == 0` just returns `self`.
    /// - This has wrapping semantics with respect to the underlying index.
    pub fn next_n(self, n: u16) -> Self {
        Self(self.0.wrapping_add(n))
    }

    /// Returns the n-th previous [`Slot`] from `self` with contiguous index.
    ///
    /// # Note
    ///
    /// - Calling this with `n == 0` just returns `self`.
    /// - This has wrapping semantics with respect to the underlying index.
    pub fn prev_n(self, n: u16) -> Self {
        Self(self.0.wrapping_sub(n))
    }

    /// Returns the [`Slot`] with the next contiguous index.
    pub fn next(self) -> Self {
        self.next_n(1)
    }

    /// Returns the [`Slot`] with the previous contiguous index.
    pub fn prev(self) -> Self {
        self.prev_n(1)
    }
}
