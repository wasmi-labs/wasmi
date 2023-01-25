#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    clippy::cast_lossless,
    clippy::missing_errors_doc,
    clippy::used_underscore_binding,
    clippy::redundant_closure_for_method_calls,
    clippy::type_repetition_in_bounds,
    clippy::inconsistent_struct_constructor,
    clippy::default_trait_access,
    clippy::map_unwrap_or,
    clippy::items_after_statements
)]

mod host_error;
mod nan_preserving_float;
mod trap;
mod units;
mod untyped;
mod value;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std as alloc;

use self::value::{
    ArithmeticOps,
    ExtendInto,
    Float,
    Integer,
    LittleEndianConvert,
    SignExtendFrom,
    TruncateSaturateInto,
    TryTruncateInto,
    WrapInto,
};
pub use self::{
    host_error::HostError,
    nan_preserving_float::{F32, F64},
    trap::{Trap, TrapCode},
    units::Pages,
    untyped::{DecodeUntypedSlice, EncodeUntypedSlice, UntypedError, UntypedValue},
    value::ValueType,
};
