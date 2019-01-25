use syn::spanned::Spanned;
use syn::{FnArg, Ident, ImplItem, ImplItemMethod, ItemImpl, ReturnType};

pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

pub struct Signature {
    pub params: Vec<ValueType>,
    pub return_ty: Option<ValueType>,
}

#[derive(Clone)]
pub struct Param {
    /// A generated identifier used to name temporary variables
    /// used for storing this parameter in generated code.
    pub ident: syn::Ident,
}

pub struct ExternalFunc {
    /// Assigned index of this function.
    pub index: u32,
    pub name: String,
    // TODO: Rename args to params
    pub args: Vec<Param>,
    pub return_ty: Option<proc_macro2::Span>,
}

/// The core structure that contains the list of all functions
/// and the data required for implementing a trait.
pub struct ExtDefinition {
    /// List of all external functions.
    pub funcs: Vec<ExternalFunc>,
    /// The generics required to implement a trait for this type.
    pub generics: syn::Generics,
    /// The type declaration to implement to implement a trait, most typically
    /// represented by a structure.
    pub ty: Box<syn::Type>,
}

/// Parse an incoming stream of tokens into externalities definition.
pub fn parse(input: proc_macro2::TokenStream) -> Result<ExtDefinition, ()> {
    let item_impl = syn::parse2::<syn::ItemImpl>(input).map_err(|_| ())?;

    let mut funcs = vec![];

    for item in item_impl.items {
        match item {
            ImplItem::Method(ImplItemMethod { sig, .. }) => {
                let index = funcs.len() as u32;

                // self TODO: handle this properly
                let args = sig
                    .decl
                    .inputs
                    .iter()
                    .skip(1)
                    .enumerate()
                    .map(|(idx, input)| {
                        let param_name = format!("arg{}", idx);
                        let ident = Ident::new(&param_name, input.span());
                        Param { ident }
                    })
                    .collect::<Vec<_>>();

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
            }
            _ => {}
        }
    }

    Ok(ExtDefinition {
        funcs,
        generics: item_impl.generics.clone(),
        ty: item_impl.self_ty.clone(),
    })
}
