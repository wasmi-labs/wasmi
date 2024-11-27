//! Implements C-API support for the Wasmi WebAssembly interpreter.
//!
//! Namely implements the Wasm C-API proposal found here: <https://github.com/WebAssembly/wasm-c-api/>

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
