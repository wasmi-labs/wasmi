//! Definitions for thin-wrapper index types.

use crate::Error;
use core::fmt::{self, Write};

macro_rules! for_each_index {
    ($mac:ident) => {
        $mac! {
            /// A Wasm function index.
            Func(pub(crate) u32);
            /// A Wasm function type index.
            FuncType(pub(crate) u32);
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

/// A raw-reference to an internal Wasmi function entry.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternalFunc(pub(crate) usize);

impl From<InternalFunc> for usize {
    #[inline]
    fn from(value: InternalFunc) -> usize {
        value.0
    }
}

impl From<usize> for InternalFunc {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl fmt::Debug for InternalFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InternalFunc")?;
        f.write_char('(')?;
        f.write_fmt(core::format_args!("0x{:X}", self.0))?;
        f.write_char(')')?;
        Ok(())
    }
}

impl Memory {
    /// Returns `true` if `self` refers to the default linear memory which always is at index 0.
    pub fn is_default(&self) -> bool {
        self.0 == 0
    }
}

impl From<Memory> for u32 {
    fn from(value: Memory) -> Self {
        u32::from(value.0)
    }
}

impl TryFrom<u32> for Memory {
    type Error = Error;

    fn try_from(index: u32) -> Result<Self, Self::Error> {
        u16::try_from(index)
            .map_err(|_| Error::MemoryIndexOutOfBounds)
            .map(Self::from)
    }
}

#[cfg(feature = "slot16")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawSlot(pub(crate) u16);

#[cfg(not(feature = "slot16"))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawSlot(pub(crate) u32);

/// The number of bytes in a Wasmi cell.
const CELL_BYTES: u32 = core::mem::size_of::<u64>() as u32;

#[cfg(feature = "slot16")]
impl From<u16> for RawSlot {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}

#[cfg(not(feature = "slot16"))]
impl From<u16> for RawSlot {
    #[inline]
    fn from(value: u16) -> Self {
        Self(u32::from(value) * CELL_BYTES)
    }
}

#[cfg(feature = "slot16")]
impl From<RawSlot> for u16 {
    #[inline]
    fn from(value: RawSlot) -> Self {
        value.0
    }
}

#[cfg(not(feature = "slot16"))]
impl From<RawSlot> for u16 {
    #[inline]
    fn from(value: RawSlot) -> Self {
        (value.0 / CELL_BYTES) as _
    }
}

#[cfg(feature = "slot16")]
impl RawSlot {
    /// Returns the byte offset of `self`.
    #[inline]
    fn byte_offset(self) -> usize {
        (u32::from(u16::from(self)) * CELL_BYTES) as _
    }
}

#[cfg(not(feature = "slot16"))]
impl RawSlot {
    /// Returns the byte offset of `self`.
    #[inline]
    fn byte_offset(self) -> usize {
        self.0 as _
    }
}

/// A Wasmi stack slot index.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Slot(pub(crate) RawSlot);

impl From<u16> for Slot {
    #[inline]
    fn from(value: u16) -> Self {
        Self(RawSlot::from(value))
    }
}

impl From<Slot> for u16 {
    #[inline]
    fn from(value: Slot) -> Self {
        value.0.into()
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
        let offset = RawSlot::from(n).0;
        let new_index = self.0.0.wrapping_add(offset);
        Self(RawSlot(new_index))
    }

    /// Returns the n-th previous [`Slot`] from `self` with contiguous index.
    ///
    /// # Note
    ///
    /// - Calling this with `n == 0` just returns `self`.
    /// - This has wrapping semantics with respect to the underlying index.
    pub fn prev_n(self, n: u16) -> Self {
        let offset = RawSlot::from(n).0;
        let new_index = self.0.0.wrapping_sub(offset);
        Self(RawSlot(new_index))
    }

    /// Returns the [`Slot`] with the next contiguous index.
    pub fn next(self) -> Self {
        self.next_n(1)
    }

    /// Returns the [`Slot`] with the previous contiguous index.
    pub fn prev(self) -> Self {
        self.prev_n(1)
    }

    /// Returns the byte offset of `self`.
    pub fn byte_offset(self) -> usize {
        self.0.byte_offset()
    }
}
