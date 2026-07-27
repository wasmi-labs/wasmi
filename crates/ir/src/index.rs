//! Definitions for thin-wrapper index types.

use crate::Error;
use core::fmt::{self, Write};

macro_rules! for_each_index {
    ($mac:ident) => {
        $mac! {
            /// A Wasmi function instance address.
            FuncAddr(pub(crate) u32);
            /// A deduplicated Wasmi function type.
            ///
            /// # Note
            ///
            /// This is the raw representation of an engine's deduplicated function type and
            /// not an index into the Wasm module's type section. Its engine association is
            /// implied by the function body storing it.
            FuncType(pub(crate) u32);
            /// A Wasmi global variable instance address.
            GlobalAddr(pub(crate) u32);
            /// A Wasmi linear memory instance address.
            MemoryAddr(pub(crate) u16);
            /// A Wasmi table instance address.
            TableAddr(pub(crate) u32);
            /// A Wasmi data segment instance address.
            DataAddr(pub(crate) u32);
            /// A Wasmi element segment instance address.
            ElemAddr(pub(crate) u32);
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

impl From<MemoryAddr> for u32 {
    fn from(value: MemoryAddr) -> Self {
        u32::from(value.0)
    }
}

impl TryFrom<u32> for MemoryAddr {
    type Error = Error;

    fn try_from(index: u32) -> Result<Self, Self::Error> {
        u16::try_from(index)
            .map_err(|_| Error::MemoryIndexOutOfBounds)
            .map(Self::from)
    }
}

#[cfg(feature = "slot16")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RawSlot(pub(crate) u16);

#[cfg(not(feature = "slot16"))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RawSlot(pub(crate) u32);

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
