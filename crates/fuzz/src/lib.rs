pub mod config;
mod crash_inputs;
mod error;
mod module;
mod value;

#[cfg(all(
    feature = "differential",
    any(feature = "wasmi-v1", feature = "wasmtime",)
))]
pub mod oracle;
#[cfg(all(
    feature = "differential",
    not(any(feature = "wasmi-v1", feature = "wasmtime",))
))]
const _: () = {
    compile_error!(
        "differntial fuzzing: must have `wasmi-v1` or `wasmtime` crate feature enabled"
    );
};

pub use self::{
    config::{FuzzSmithConfig, FuzzWasmiConfig},
    crash_inputs::generate_crash_inputs,
    error::{FuzzError, TrapCode},
    module::{FuzzModule, WasmSource, WatSource},
    value::{FuzzVal, FuzzValType},
};
