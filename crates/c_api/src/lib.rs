//! Implements C-API support for the Wasmi WebAssembly interpreter.
//!
//! Namely implements the Wasm C-API proposal found here: <https://github.com/WebAssembly/wasm-c-api/>

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)] // TODO: remove when done implementing Wasmi C-API

extern crate alloc;

pub use wasmi;

mod config;
mod engine;
mod error;
mod r#extern;
mod foreign;
mod frame;
mod global;
mod memory;
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
    global::*,
    memory::*,
    r#extern::*,
    r#ref::*,
    store::*,
    table::*,
    trap::*,
    types::*,
    val::*,
    vec::*,
};

// TODO: remove type alias place-holders once types are fully implemented
pub type wasm_func_t = core::ffi::c_void;
pub type wasm_module_t = core::ffi::c_void;
pub type wasm_instance_t = core::ffi::c_void;
