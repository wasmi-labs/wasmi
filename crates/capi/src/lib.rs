//! Implements C-API support for the Wasmi WebAssembly interpreter.
//! 
//! Namely implements the Wasm C-API proposal found here: <https://github.com/WebAssembly/wasm-c-api/>

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

extern crate alloc;

pub use wasmi;

mod config;
mod engine;

pub use self::{config::*, engine::*};
