/// Extension methods for [`struct@syn::Attribute`].
pub trait AttributeExt {
    /// Returns `true` if the [`struct@syn::Attribute`] is a Rust documentation attribute.
    fn is_doc(&self) -> bool;
}

impl AttributeExt for syn::Attribute {
    fn is_doc(&self) -> bool {
        self.path().is_ident("doc")
    }
}

/// Converts the `ident` to a snake-case raw [`Ident`].
pub fn to_snake_case_ident(ident: &syn::Ident) -> syn::Ident {
    let span = ident.span();
    let snake_name = heck::AsSnakeCase(ident.to_string()).to_string();
    syn::Ident::new_raw(&snake_name, span)
}
