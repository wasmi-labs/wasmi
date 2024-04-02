use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(WasmiProfiling)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let syn::Data::Enum(data_enum) = &input.data else {
        panic!(
            "wasmi_profiling_macro: can only operate on `enum` but got: {:?}",
            &input.data
        );
    };
    let ident = &input.ident;
    let profiling_type = generate_profiling_type(data_enum);
    let expanded = quote! {
        const _: () = {
            #profiling_type

            impl ::wasmi_profiling::Profiling for #ident {
                type InstrData = InstrData;

                fn new() -> ::wasmi_profiling::ProfilingData<Self::InstrData> {
                    <::wasmi_profiling::ProfilingData<Self::InstrData> as ::core::default::Default>::default()
                }
            }
        };
    };
    TokenStream::from(expanded)
}

fn generate_profiling_type(data_enum: &syn::DataEnum) -> TokenStream2 {
    let fields = data_enum.variants.iter().map(|variant| {
        let span = variant.span();
        let snake_ident = to_snake_case_ident(&variant.ident);
        quote_spanned!(span=>
            pub #snake_ident: ::wasmi_profiling::InstrTracker
        )
    });
    let total_time_impl = data_enum.variants.iter().map(|variant| {
        let span = variant.span();
        let snake_ident = to_snake_case_ident(&variant.ident);
        quote_spanned!(span=>
            ::wasmi_profiling::InstrTracker::total_time(&self.#snake_ident)
        )
    });
    let count_impl = data_enum.variants.iter().map(|variant| {
        let span = variant.span();
        let snake_ident = to_snake_case_ident(&variant.ident);
        quote_spanned!(span=>
            ::wasmi_profiling::InstrTracker::count(&self.#snake_ident)
        )
    });
    quote! {
        #[derive(
            ::core::fmt::Debug,
            ::core::default::Default,
            ::core::marker::Copy,
            ::core::clone::Clone,
            ::wasmi_profiling::serde::Serialize,
        )]
        pub struct InstrData {
            #( #fields ),*
        }

        impl ::wasmi_profiling::InstrsTotalTime for InstrData {
            fn instrs_total_time(&self) -> ::core::time::Duration {
                #( #total_time_impl )+*
            }
        }

        impl ::wasmi_profiling::InstrsCount for InstrData {
            fn instrs_total_time(&self) -> ::core::primitive::u64 {
                #( #count_impl )+*
            }
        }
    }
}

fn to_snake_case_ident(ident: &syn::Ident) -> syn::Ident {
    let snake_name = heck::AsSnakeCase(ident.to_string()).to_string();
    quote::format_ident!("{}", &snake_name)
}
