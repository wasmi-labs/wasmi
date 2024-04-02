mod utils;

use crate::utils::{to_snake_case_ident, AttributeExt as _};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(WasmiProfiling)]
pub fn wasmi_profiling(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let syn::Data::Enum(data_enum) = &input.data else {
        panic!(
            "wasmi_profiling_macro: can only operate on `enum` but got: {:?}",
            &input.data
        );
    };
    let span = input.span();
    let ident = &input.ident;
    let data_type = generate_data_type(span, data_enum);
    let instr_data_type = generate_instr_data_type(span, data_enum);
    let expanded = quote! {
        const _: () = {
            #data_type
            #instr_data_type

            impl ::wasmi_profiling::WasmiProfiling for #ident {
                type Data = Data;

                fn data() -> Self::Data {
                    <Self::Data>::default()
                }
            }
        };
    };
    TokenStream::from(expanded)
}

fn generate_data_type(span: Span, data_enum: &syn::DataEnum) -> TokenStream2 {
    let select_instr = data_enum.variants.iter().map(|variant| {
        let span = variant.span();
        let snake_ident = to_snake_case_ident(&variant.ident);
        let docs = variant.attrs.iter().filter(|attr| attr.is_doc());
        quote_spanned!(span=>
            #( #docs )*
            #[inline]
            pub fn #snake_ident(self) -> ::wasmi_profiling::SelectedInstr<'a> {
                ::wasmi_profiling::SelectedInstr::new(
                    &mut self.data.dispatch,
                    &mut self.data.ticker,
                    &mut self.data.instr.#snake_ident,
                )
            }
        )
    });
    quote_spanned!(span=>
        #[derive(
            ::core::fmt::Debug,
            ::core::default::Default,
            ::core::marker::Copy,
            ::core::clone::Clone,
            ::wasmi_profiling::serde::Serialize,
        )]
        #[repr(transparent)]
        pub struct Data {
            data: ::wasmi_profiling::ProfilingData<InstrData>,
        }

        impl Data {
            /// Start profiling a Wasmi execution run.
            ///
            /// # Note
            ///
            /// This should be invoked right before the first instruction dispatch.
            pub fn start(&mut self) {
                self.data.start();
            }
        }

        impl ::wasmi_profiling::SelectInstr for Data {
            type Selector<'a> = InstrSelector<'a>;

            #[inline]
            fn instr(&mut self) -> Self::Selector<'_> {
                Self::Selector { data: &mut self.data }
            }
        }

        #[derive(Debug)]
        #[repr(transparent)]
        pub struct InstrSelector<'a> {
            data: &'a mut ::wasmi_profiling::ProfilingData<InstrData>,
        }

        impl<'a> InstrSelector<'a> {
            #( #select_instr )*
        }
    )
}

fn generate_instr_data_type(span: Span, data_enum: &syn::DataEnum) -> TokenStream2 {
    let fields = data_enum.variants.iter().map(|variant| {
        let span = variant.span();
        let snake_ident = to_snake_case_ident(&variant.ident);
        let docs = variant.attrs.iter().filter(|attr| attr.is_doc());
        quote_spanned!(span=>
            #[serde(skip_serializing_if = "::wasmi_profiling::InstrTracker::is_never_called")]
            #( #docs )*
            pub #snake_ident: ::wasmi_profiling::InstrTracker
        )
    });
    let total_time_impl = data_enum.variants.iter().map(|variant| {
        let span = variant.span();
        let snake_ident = to_snake_case_ident(&variant.ident);
        quote_spanned!(span=>
            self.#snake_ident.total_time
        )
    });
    let count_impl = data_enum.variants.iter().map(|variant| {
        let span = variant.span();
        let snake_ident = to_snake_case_ident(&variant.ident);
        quote_spanned!(span=>
            self.#snake_ident.count
        )
    });
    quote_spanned!(span=>
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
            fn instrs_count(&self) -> ::core::primitive::u64 {
                #( #count_impl )+*
            }
        }
    )
}
