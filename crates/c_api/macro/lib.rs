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
        #[no_mangle]
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
        #[no_mangle]
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
        #[no_mangle]
        pub extern "C" fn #same(_a: &#ty, _b: &#ty) -> ::core::primitive::bool {
            #[cfg(feature = "std")]
            ::std::eprintln!("`{}` is not implemented", ::core::stringify!(#same));
            ::core::unimplemented!(::core::stringify!(#same))
        }

        #[doc = #get_host_info_docs]
        #[no_mangle]
        pub extern "C" fn #get_host_info(a: &#ty) -> *mut ::core::ffi::c_void {
            ::core::ptr::null_mut()
        }

        #[doc = #set_host_info_docs]
        #[no_mangle]
        pub extern "C" fn #set_host_info(a: &#ty, info: *mut ::core::ffi::c_void) {
            #[cfg(feature = "std")]
            ::std::eprintln!("`{}` is not implemented", ::core::stringify!(#set_host_info));
            ::core::unimplemented!(::core::stringify!(#set_host_info));
        }

        #[doc = #set_host_info_final_docs]
        #[no_mangle]
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
        #[no_mangle]
        pub extern "C" fn #as_ref(a: &#ty) -> ::alloc::boxed::Box<crate::wasm_ref_t> {
            #[cfg(feature = "std")]
            ::std::eprintln!("`{}` is not implemented", ::core::stringify!(#as_ref));
            ::core::unimplemented!(::core::stringify!(#as_ref));
        }

        #[doc = #as_ref_const_docs]
        #[no_mangle]
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
