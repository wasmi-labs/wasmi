//! Definitions for thin-wrapper index types.

use crate::Error;

macro_rules! for_each_index {
    ($mac:ident) => {
        $mac! {
            /// A Wasmi register.
            Reg(pub(crate) i16);
            /// A Wasm function index.
            Func(pub(crate) u32);
            /// A Wasm function type index.
            FuncType(pub(crate) u32);
            /// A Wasmi internal function index.
            InternalFunc(pub(crate) u32);
            /// A Wasmi imported function index.
            ImportedFunc(pub(crate) u32);
            /// A Wasm global variable index.
            Global(pub(crate) u32);
            /// A Wasm table index.
            Table(pub(crate) u32);
            /// A Wasm data segment index.
            Data(pub(crate) u32);
            /// A Wasm element segment index.
            Elem(pub(crate) u32);
        }
    };
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

impl TryFrom<u32> for Reg {
    type Error = Error;

    fn try_from(local_index: u32) -> Result<Self, Self::Error> {
        i16::try_from(local_index)
            .map_err(|_| Error::RegisterOutOfBounds)
            .map(Self::from)
    }
}

impl Reg {
    /// Returns the [`Reg`] with the next contiguous index.
    pub fn next(self) -> Reg {
        Self(self.0.wrapping_add(1))
    }

    /// Returns the [`Reg`] with the previous contiguous index.
    pub fn prev(self) -> Reg {
        Self(self.0.wrapping_sub(1))
    }
}