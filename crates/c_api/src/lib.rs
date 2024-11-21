//! Implements C-API support for the Wasmi WebAssembly interpreter.
//!
//! Namely implements the Wasm C-API proposal found here: <https://github.com/WebAssembly/wasm-c-api/>
//!
//! # Crate features
//!
//! ## The `mangle-symbols` feature
//! The `mangle-symbols` feature, when disabled, disables the `#[no_mangle]` attributes on exported symbols. This means that,
//! when the feature is disabled, the public symbols (such as `wasm_func_new`) in this library are not visible with the
//! plain "C" name, and the usual Rust mangling is operated instead.
//!
//! The rationale behind this feature is to allow (or facilitate) users that want to use multiple implementers of the
//! C-API proposal together to avoid duplicate symbols error. This feature is enabled by default, thus public symbols
//! are not mangled.
//!
//! ### Note
//! When disabled, the public symbols prefixed with `wasmi_` are mangled as well, even if they should not, in principle,
//! cause any duplicate symbol error when working with other implementers.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

extern crate alloc;
#[cfg(feature = "std")]
#[macro_use]
extern crate std;

pub use wasmi;

mod config;
mod engine;
mod error;
mod r#extern;
mod foreign;
mod frame;
mod func;
mod global;
mod instance;
mod memory;
mod module;
mod r#ref;
mod store;
mod table;
mod trap;
mod types;
mod utils;
mod val;
mod vec;

use self::utils::*;
pub use self::{
    config::*,
    engine::*,
    error::*,
    foreign::*,
    frame::*,
    func::*,
    global::*,
    instance::*,
    memory::*,
    module::*,
    r#extern::*,
    r#ref::*,
    store::*,
    table::*,
    trap::*,
    types::*,
    val::*,
    vec::*,
};
