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
mod global;
pub mod hint;
mod host_error;
mod index_ty;
mod limiter;
mod memory;
mod raw;
mod table;
mod trap;
mod typed;
mod value;
pub mod wasm;

#[cfg(feature = "simd")]
pub mod simd;

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use self::value::{Float, Integer, SignExtendFrom, TruncateSaturateInto, TryTruncateInto};
pub use self::{
    float::{F32, F64},
    fuel::{Fuel, FuelCosts, FuelCostsProvider, FuelError},
    func_type::{FuncType, FuncTypeError},
    global::{Global, GlobalError, GlobalType, Mutability},
    host_error::HostError,
    index_ty::IndexType,
    limiter::{LimiterError, ResourceLimiter, ResourceLimiterRef},
    memory::{Memory, MemoryError, MemoryType, MemoryTypeBuilder},
    raw::{RawVal, ReadAs, WriteAs},
    table::{
        ElementSegment,
        ElementSegmentRef,
        RefType,
        Table,
        TableError,
        TableType,
        TypedRef,
        UntypedRef,
    },
    trap::{Trap, TrapCode},
    typed::{Typed, TypedVal},
    value::{V128, ValType},
};
