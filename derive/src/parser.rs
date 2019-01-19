use crate::model::{self, ExtDefinition, ExternalFunc};
use syn::{ItemImpl, ImplItem, ImplItemMethod, FnArg, ReturnType};
use syn::spanned::Spanned;

/// Parse an incoming stream of tokens into a list of external functions.
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
                let args = sig.decl.inputs.iter().skip(1).enumerate().map(|input| {
                    input.span()
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
                    arity: sig.decl.inputs.len() - 1, // self TODO: handle this properly
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
