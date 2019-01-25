//! A derive macro for generation of simple `Externals`.
//!
//! ```nocompile
//! #// no compile because we can't depend on wasmi here, or otherwise it will be a circular dependency.
//! extern crate wasmi;
//! extern crate wasmi_derive;
//!
//! use std::fmt;
//! use wasmi::HostError;
//! use wasmi_derive::derive_externals;
//!
//! #[derive(Debug)]
//! struct NoInfoError;
//! impl HostError for NoInfoError {}
//! impl fmt::Display for NoInfoError {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         write!(f, "NoInfoError")
//!     }
//! }
//!
//! struct NonStaticExternals<'a> {
//!     state: &'a mut usize,
//! }
//!
//! #[derive_externals]
//! impl<'a> NonStaticExternals<'a> {
//!     pub fn hello(&self, a: u32, b: u32) -> u32 {
//!         a + b
//!     }
//!
//!     pub fn increment(&mut self) {
//!         *self.state += 1;
//!     }
//!
//!     pub fn traps(&self) -> Result<(), NoInfoError> {
//!         Err(NoInfoError)
//!     }
//! }
//! ```
//!

// We reached the `recursion_limit` in quote macro.
#![recursion_limit = "128"]

extern crate proc_macro;

mod codegen;
mod parser;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn derive_externals(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input: proc_macro2::TokenStream = input.into();

    let ext_def = parser::parse(input.clone()).unwrap();
    codegen::codegen(&ext_def, &mut input);

    input.into()
}
