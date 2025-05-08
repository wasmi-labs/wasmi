//! A set of convenience macros for the `wasmi_c_api_impl` crate.
//!
//! These are intended to mirror the macros in the `wasm.h` header file and
//! largely facilitate the `declare_own`, `declare_ty` and `declare_ref` macro.

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::quote;

fn extract_ident(input: proc_macro::TokenStream) -> Ident {
    let input = TokenStream::from(input);
    let ident = match input.into_iter().next().unwrap() {
        TokenTree::Ident(i) => i,
        unexpected => panic!("expected a valid Rust identifier but found: {unexpected:?}"),
    };
    let name = ident.to_string();
    assert!(name.ends_with("_t"));
    ident
}

#[proc_macro]
pub fn declare_own(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ty = extract_ident(input);
    let name = ty.to_string();
    let delete = quote::format_ident!("{}_delete", &name[..name.len() - 2]);
    let docs = format!("Deletes the [`{name}`].");

    (quote! {
        #[doc = #docs]
        #[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn #delete(_: ::alloc::boxed::Box<#ty>) {}
    })
    .into()
}

#[proc_macro]
pub fn declare_ty(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ty = extract_ident(input);
    let name = ty.to_string();
    let prefix = &name[..name.len() - 2];
    let copy = quote::format_ident!("{}_copy", &prefix);
    let docs = format!(
        "Creates a new [`{name}`] which matches the provided one.\n\n\
        The caller is responsible for deleting the returned value via [`{prefix}_delete`].\n\n\
    "
    );

    (quote! {
        ::wasmi_c_api_macros::declare_own!(#ty);

        #[doc = #docs]
        #[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn #copy(src: &#ty) -> ::alloc::boxed::Box<#ty> {
            ::alloc::boxed::Box::new(src.clone())
        }
    })
    .into()
}

#[proc_macro]
pub fn declare_ref(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ty = extract_ident(input);
    let name = ty.to_string();
    let prefix = &name[..name.len() - 2];
    let same = quote::format_ident!("{}_same", prefix);
    let same_docs = format!(
        "Returns `true` if the given references are pointing to the same [`{name}`].\n\n\
        This is not yet supported and aborts the process upon use."
    );
    let get_host_info = quote::format_ident!("{}_get_host_info", prefix);
    let get_host_info_docs = format!(
        "Returns the host information of the [`{name}`].\n\n\
        This is not yet supported and always returns `NULL`."
    );
    let set_host_info = quote::format_ident!("{}_set_host_info", prefix);
    let set_host_info_docs = format!(
        "Sets the host information of the [`{name}`].\n\n\
        This is not yet supported and aborts the process upon use."
    );
    let set_host_info_final = quote::format_ident!("{}_set_host_info_with_finalizer", prefix);
    let set_host_info_final_docs = format!(
        "Sets the host information finalizer of the [`{name}`].\n\n\
        This is not yet supported and aborts the process upon use."
    );
    let as_ref = quote::format_ident!("{}_as_ref", prefix);
    let as_ref_docs = format!(
        "Returns the [`{name}`] as mutable reference.\n\n\
        This is not yet supported and aborts the process upon use."
    );
    let as_ref_const = quote::format_ident!("{}_as_ref_const", prefix);
    let as_ref_const_docs = format!(
        "Returns the [`{name}`] as immutable reference.\n\n\
        This is not yet supported and aborts the process upon use."
    );

    (quote! {
        ::wasmi_c_api_macros::declare_ty!(#ty);

        #[doc = #same_docs]
        #[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn #same(_a: &#ty, _b: &#ty) -> ::core::primitive::bool {
            #[cfg(feature = "std")]
            ::std::eprintln!("`{}` is not implemented", ::core::stringify!(#same));
            ::core::unimplemented!(::core::stringify!(#same))
        }

        #[doc = #get_host_info_docs]
        #[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn #get_host_info(a: &#ty) -> *mut ::core::ffi::c_void {
            ::core::ptr::null_mut()
        }

        #[doc = #set_host_info_docs]
        #[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn #set_host_info(a: &#ty, info: *mut ::core::ffi::c_void) {
            #[cfg(feature = "std")]
            ::std::eprintln!("`{}` is not implemented", ::core::stringify!(#set_host_info));
            ::core::unimplemented!(::core::stringify!(#set_host_info));
        }

        #[doc = #set_host_info_final_docs]
        #[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn #set_host_info_final(
            a: &#ty,
            info: *mut ::core::ffi::c_void,
            finalizer: ::core::option::Option<extern "C" fn(*mut ::core::ffi::c_void)>,
        ) {
            #[cfg(feature = "std")]
            ::std::eprintln!("`{}` is not implemented", ::core::stringify!(#set_host_info_final));
            ::core::unimplemented!(::core::stringify!(#set_host_info_final));
        }

        #[doc = #as_ref_docs]
        #[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn #as_ref(a: &#ty) -> ::alloc::boxed::Box<crate::wasm_ref_t> {
            #[cfg(feature = "std")]
            ::std::eprintln!("`{}` is not implemented", ::core::stringify!(#as_ref));
            ::core::unimplemented!(::core::stringify!(#as_ref));
        }

        #[doc = #as_ref_const_docs]
        #[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn #as_ref_const(a: &#ty) -> ::alloc::boxed::Box<crate::wasm_ref_t> {
            #[cfg(feature = "std")]
            ::std::eprintln!("`{}` is not implemented", ::core::stringify!(#as_ref_const));
            ::core::unimplemented!(::core::stringify!(#as_ref_const));
        }

        // TODO: implement `wasm_ref_as_#name#`
        // TODO: implement `wasm_ref_as_#name#_const`
    })
    .into()
}

macro_rules! bail {
    ($message:literal) => {{
        return ::core::result::Result::Err(Error($message.into()));
    }};
}

/// An error with its error message.
struct Error(String);

impl Error {
    /// Converts the [`Error`] into a `compile_error!` token stream.
    fn to_compile_error(&self) -> proc_macro::TokenStream {
        let message = &self.0;
        quote! { ::core::compile_error!(#message) }.into()
    }
}

/// Applied on Rust `fn` items from the Wasm spec.
///
/// Annotates the given function with `#[export_name = $func_name]`
/// where `$func_name` is the name of the given function.
#[proc_macro_attribute]
pub fn prefix_symbol(
    attributes: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match prefix_symbol_impl(attributes.into(), input.into()) {
        Ok(result) => result.into(),
        Err(error) => error.to_compile_error(),
    }
}

fn prefix_symbol_impl(attributes: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    if !attributes.is_empty() {
        bail!("err(prefix_symbol): attributes must be empty")
    }
    let mut stream = input.clone().into_iter();
    let fn_token = stream.find(|tt| matches!(tt, TokenTree::Ident(ref ident) if *ident == "fn"));
    if fn_token.is_none() {
        bail!("can only apply on `fn` items")
    }
    let Some(TokenTree::Ident(fn_ident)) = stream.next() else {
        bail!("function name must follow `fn` keyword")
    };
    let fn_name = fn_ident.to_string();
    if !fn_name.starts_with("wasm_") {
        // No prefix needed since the function is not a part of the Wasm spec.
        return Ok(input);
    }
    let prefixed_fn_name = format!("wasmi_{fn_name}");
    Ok(quote! {
        #[export_name = #prefixed_fn_name]
        #input
    })
}
