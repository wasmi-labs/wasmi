//! Definitions for thin-wrapper index types.

use crate::Error;

macro_rules! for_each_index {
    ($mac:ident) => {
        $mac! {
            /// Used to query the [`Instruction`] of an [`InstrSequence`].
            ///
            /// [`Instruction`]: crate::Instruction
            /// [`InstrSequence`]: crate::InstrSequence
            Instr(pub(crate) u32);
            /// A Wasmi register.
            Reg(pub(crate) i16);
            /// A Wasm function index.
            Func(pub(crate) u32);
            /// A Wasm function type index.
            FuncType(pub(crate) u32);
            /// A Wasmi internal function index.
            InternalFunc(pub(crate) u32);
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

    /// Returns `true` if `self` represents a function local constant value.
    pub fn is_const(self) -> bool {
        self.0.is_negative()
    }
}

impl Instr {
    /// Creates an [`Instr`] from the given `usize` value.
    ///
    /// # Panics
    ///
    /// If the `value` exceeds limitations for [`Instr`].
    pub fn from_usize(index: usize) -> Self {
        let index = index.try_into().unwrap_or_else(|error| {
            panic!("invalid index {index} for instruction reference: {error}")
        });
        Self(index)
    }

    /// Returns the index underlying to `self` as `usize`.
    pub fn into_usize(self) -> usize {
        self.0 as usize
    }
}
