use crate::model::{ExtDefinition, ExternalFunc};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};


pub fn codegen(ext_def: &ExtDefinition, to: &mut TokenStream) {
    let mut externals = TokenStream::new();
    let mut module_resolver = TokenStream::new();

    // TODO: Come up with a name.
    let mut new_name = "_WASMI_IMPLS_".to_string();
	new_name.push_str("NAME".to_string().trim_start_matches("r#"));
	let dummy_const = Ident::new(&new_name, Span::call_site());

    derive_externals(ext_def, &mut externals);
    derive_module_resolver(ext_def, &mut module_resolver);

    (quote! {
        const #dummy_const: () = {
			extern crate wasmi as _wasmi;

            use _wasmi::{
                Trap, RuntimeValue, RuntimeArgs, Externals,
                derive_support::WasmResult,
            };

            #externals
            #module_resolver
        };
    }).to_tokens(to);
}

fn gen_dispatch_func_arm(func: &ExternalFunc) -> TokenStream {
    let index = func.index as usize;
    let name = Ident::new(&func.name, Span::call_site());
    let return_ty_span = func.return_ty.clone().unwrap_or_else(|| Span::call_site());

    let mut args = vec![];
    let mut unmarshall_args = TokenStream::new();
    for (i, arg_span) in func.args.iter().cloned().enumerate() {
        let mut arg_name = "arg".to_string();
        arg_name.push_str(&i.to_string());
        let arg_name = Ident::new(&arg_name, arg_span.clone());

        (quote_spanned! {arg_span=>
            let #arg_name =
                args.next()
                    .and_then(|rt_val| rt_val.try_into())
                    .unwrap();
        }).to_tokens(&mut unmarshall_args);

        args.push(quote_spanned! {arg_span=> #arg_name });
    }

    let prologue = quote! {
        let mut args = args.as_ref().iter();
        #unmarshall_args
    };
    let epilogue = quote_spanned! {return_ty_span=>
        WasmResult::to_wasm_result(r)
    };

    (quote! {
        #index => {
            #prologue
            let r = self.#name( #(#args),* );
            #epilogue
        }
    })

    // let body = $crate::wasm_utils::constrain_closure::<
    //     <$returns as $crate::wasm_utils::ConvertibleToWasm>::NativeType, _
    // >(|| {
    //     unmarshall_args!($body, $objectname, $args_iter, $( $names : $params ),*)
    // });
    // let r = body()?;
    // return Ok(Some({ use $crate::wasm_utils::ConvertibleToWasm; r.to_runtime_value() }))
}

fn derive_externals(ext_def: &ExtDefinition, to: &mut TokenStream) {
    let (impl_generics, ty_generics, where_clause) = ext_def.generics.split_for_impl();
    let ty = &ext_def.ty;

    let mut match_arms = vec![];
    for func in &ext_def.funcs {
        match_arms.push(gen_dispatch_func_arm(func));
    }

    (quote::quote! {
        impl #impl_generics Externals for #ty #where_clause {
            fn invoke_index(
                &mut self,
                index: usize,
                args: RuntimeArgs,
            ) -> Result<Option<RuntimeValue>, Trap> {
                match index {
                    #(#match_arms),*
                    _ => panic!("fn with index {} is undefined", index),
                }
            }

            // ...
        }
    }).to_tokens(to);
}

fn derive_module_resolver(ext_def: &ExtDefinition, to: &mut TokenStream) {
    (quote::quote! {
        impl #impl_generics ModuleImportResolver for #ty #where_clause {
            fn invoke_index(
                &mut self,
                index: usize,
                args: RuntimeArgs,
            ) -> Result<Option<RuntimeValue>, Trap> {
                match index {
                    #(#match_arms),*
                    _ => panic!("fn with index {} is undefined", index),
                }
            }

            // ...
        }
    }).to_tokens(to);
}
