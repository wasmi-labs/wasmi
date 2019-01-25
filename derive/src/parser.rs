use crate::error::CompileError;
use syn::{spanned::Spanned, FnArg, Ident, ImplItem, ImplItemMethod, ReturnType};

/// A parameter. This doesn't used for modeling `&self` or `&mut self` parameters.
#[derive(Clone)]
pub struct Param {
    /// A generated identifier used to name temporary variables
    /// used for storing this parameter in generated code.
    ///
    /// This ident is used primary used for its' span.
    pub ident: syn::Ident,
}

/// A function definition parsed from an impl block.
pub struct FuncDef {
    /// Assigned index of this function.
    pub index: u32,
    pub name: String,
    /// The parameter of this function. This excludes the `&self` or `&mut self`.
    pub params: Vec<Param>,
    pub return_ty: Option<proc_macro2::Span>,
}

/// This is the core data structure which contains the list of all defined functions
/// and the data required for the code generator (e.g. for implementing a trait).
pub struct ImplBlockDef {
    /// List of all defined external functions.
    pub funcs: Vec<FuncDef>,
    /// The generics required to implement a trait for this type.
    pub generics: syn::Generics,
    /// The type declaration to implement a trait, most typically
    /// represented by a structure.
    ///
    /// E.g.: `Foo<'a>`, `()`
    pub ty: Box<syn::Type>,
}

/// Parse an incoming stream of tokens into externalities definition.
pub fn parse(input: proc_macro2::TokenStream) -> Result<ImplBlockDef, CompileError> {
    let item_impl = syn::parse2::<syn::ItemImpl>(input)
        .map_err(|_| CompileError::new("failed to parse".to_string()))?;

    let mut funcs = vec![];

    for item in item_impl.items {
        match item {
            ImplItem::Method(ImplItemMethod { sig, .. }) => {
                let index = funcs.len() as u32;

                let params = sig
                    .decl
                    .inputs
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, input)| {
                        // The first parameter should be either &self or &mut self.
                        // This makes code generation simpler.
                        if idx == 0 {
                            match input {
                                FnArg::SelfRef(_) => return None,
                                _ => {
                                    return Some(Err(err_span!(
                                        input.span(),
                                        "only &self and &mut self supported as first argument"
                                    )));
                                }
                            }
                        }

                        let param_name = format!("arg{}", idx);
                        let ident = Ident::new(&param_name, input.span());
                        Some(Ok(Param { ident }))
                    })
                    .collect::<Result<Vec<Param>, CompileError>>()?;

                let return_ty = match sig.decl.output {
                    ReturnType::Default => None,
                    ReturnType::Type(_, ty) => Some(ty.span()),
                };

                funcs.push(FuncDef {
                    index,
                    name: sig.ident.to_string(),
                    params,
                    return_ty,
                });
            }
            _ => {}
        }
    }

    Ok(ImplBlockDef {
        funcs,
        generics: item_impl.generics.clone(),
        ty: item_impl.self_ty.clone(),
    })
}
