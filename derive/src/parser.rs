use crate::model::{self, ExtDefinition, ExternalFunc, Param};
use syn::{ItemImpl, ImplItem, ImplItemMethod, FnArg, ReturnType, Ident};
use syn::spanned::Spanned;

/// Parse an incoming stream of tokens into externalities definition.
pub fn parse(input: proc_macro2::TokenStream) -> Result<ExtDefinition, ()> {
    let item_impl = syn::parse2::<syn::ItemImpl>(input).map_err(|_| ())?;

    let mut funcs = vec![];

    for item in item_impl.items {
        match item {
            ImplItem::Method(ImplItemMethod {
                sig,
                ..
            }) => {
                let index = funcs.len() as u32;

                // self TODO: handle this properly
                let args = sig.decl.inputs.iter().skip(1).enumerate().map(|(idx, input)| {
                    let param_name = format!("arg{}", idx);
                    let ident = Ident::new(&param_name, input.span());
                    Param {
                        ident,
                    }
                }).collect::<Vec<_>>();

                let return_ty = match sig.decl.output {
                    ReturnType::Default => None,
                    ReturnType::Type(_, ty) => Some(ty.span()),
                };

                funcs.push(ExternalFunc {
                    index,
                    name: sig.ident.to_string(),
                    args,
                    return_ty,
                });
            },
            _ => {},
        }
    }

    Ok(ExtDefinition {
        funcs,
        generics: item_impl.generics.clone(),
        ty: item_impl.self_ty.clone(),
    })
}
