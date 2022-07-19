#![cfg_attr(not(feature = "std"), no_std)]

mod host_error;
mod nan_preserving_float;
mod trap;
mod untyped;
mod value;

#[cfg(feature = "virtual_memory")]
mod vmem;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std as alloc;

#[cfg(feature = "virtual_memory")]
pub use self::vmem::{VirtualMemory, VirtualMemoryError};

/// WebAssembly-specific sizes and units.
pub mod memory_units {
    pub use memory_units::{size_of, wasm32::*, ByteSize, Bytes, RoundUpTo};
}

pub use self::{
    host_error::HostError,
    nan_preserving_float::{F32, F64},
    trap::{Trap, TrapCode},
    untyped::{DecodeUntypedSlice, EncodeUntypedSlice, UntypedError, UntypedValue},
    value::{
        ArithmeticOps,
        ExtendInto,
        Float,
        FromValue,
        Integer,
        LittleEndianConvert,
        SignExtendFrom,
        TransmuteInto,
        TruncateSaturateInto,
        TryTruncateInto,
        Value,
        ValueType,
        WrapInto,
    },
};
