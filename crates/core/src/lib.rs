#![no_std]
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

mod float;
mod fuel;
mod func_type;
pub mod hint;
mod host_error;
mod limiter;
mod memory;
mod trap;
mod typed;
mod untyped;
mod value;
pub mod wasm;

#[cfg(feature = "simd")]
pub mod simd;

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use self::value::{Float, Integer, SignExtendFrom, TruncateSaturateInto, TryTruncateInto};
pub use self::{
    limiter::{LimiterError, ResourceLimiter, ResourceLimiterRef},
    float::{F32, F64},
    fuel::{FuelCosts, FuelCostsProvider},
    func_type::{DynamicallyTyped, FuncType, FuncTypeError},
    host_error::HostError,
    trap::{Trap, TrapCode},
    typed::{Typed, TypedVal},
    untyped::{DecodeUntypedSlice, EncodeUntypedSlice, ReadAs, UntypedError, UntypedVal, WriteAs},
    value::{ValType, V128},
};
