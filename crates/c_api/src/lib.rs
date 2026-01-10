//! Implements C-API support for the Wasmi WebAssembly interpreter.
//!
//! Namely implements the Wasm C-API proposal found here: <https://github.com/WebAssembly/wasm-c-api/>
//!
//! # Crate features
//!
//! ## The `prefix-symbols` feature
//! Adds a `wasmi_` prefix to all the public symbols. This means that, for example, the function `wasm_store_delete`
//! will be given the public (not mangled) symbol `wasmi_wasm_store_delete`.
//!
//! ### Rationale
//! This feature allows users that need to separate multiple C-API implementers to segregate wasmi's C-API symbols,
//! avoiding symbol clashes.
//!
//! ### Note
//! It's important to notice that when the `prefix-symbols` feature is enabled, the symbols declared in the C-API header
//! are not given the prefix, introducing - by design, in order to keep the C-API header same as the actual
//! specification - an asymmetry. For example, Rust users that want to enable this feature, can use `bindgen` to
//! generate correct C-to-Rust interop code:
//!
//! ```ignore
//!    #[derive(Debug)]
//!    struct WasmiRenamer {}
//!
//!    impl ParseCallbacks for WasmiRenamer {
//!        /// This function will run for every extern variable and function. The returned value determines
//!        /// the link name in the bindings.
//!        fn generated_link_name_override(
//!            &self,
//!            item_info: bindgen::callbacks::ItemInfo<'_>,
//!        ) -> Option<String> {
//!            if item_info.name.starts_with("wasm") {
//!                let new_name = if cfg!(any(target_os = "macos", target_os = "ios")) {
//!                    format!("_wasmi_{}", item_info.name)
//!                } else {
//!                    format!("wasmi_{}", item_info.name)
//!                };
//!
//!                Some(new_name)
//!            } else {
//!                None
//!            }
//!        }
//!    }
//!
//!    let bindings = bindgen::Builder::default()
//!        .header(
//!            PathBuf::from(std::env::var("DEP_WASMI_C_API_INCLUDE").unwrap())
//!                .join("wasm.h")
//!                .to_string_lossy(),
//!        )
//!        .derive_default(true)
//!        .derive_debug(true)
//!        .parse_callbacks(Box::new(WasmiRenamer {}))
//!        .generate()
//!        .expect("Unable to generate bindings for `wasmi`!");
//! ```

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
    r#extern::*,
    foreign::*,
    frame::*,
    func::*,
    global::*,
    instance::*,
    memory::*,
    module::*,
    r#ref::*,
    store::*,
    table::*,
    trap::*,
    types::*,
    val::*,
    vec::*,
};
