extern crate proc_macro;

mod model;
mod parser;
mod codegen;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn derive_externals(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input: proc_macro2::TokenStream = input.into();

    let ext_def = parser::parse(input.clone()).unwrap();
    codegen::codegen(&ext_def, &mut input);

    // We need to generate two types:
    // - Externals
    // - ModuleImportResolver

    // - for each of declared method collect it's name and it's signature.
    // - assign a method index for each method
    // - generate a switch for `Externals` that takes the input `index` and jumps
    //   on the corresponding match arm, which the wrapper.
    //   The wrapper decodes arguments, calls to the function and handles the result.
    // - generate a switch / ifs chain for `ModuleImportResolver`. In each arm it checks if the function
    //   has an appropriate arguments, and if so allocates a host function with the corresponding index.
    //
    // and we will then need to return both the original implementation and the generated implementation
    // of externals.

    // println!("{:?}", quote::quote! { #input }.to_string());
    let input = input.into();
    input
}
