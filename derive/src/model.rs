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

pub struct Param {
    span: proc_macro2::Span,
    generated_name: String,
}

pub struct ExternalFunc {
    /// Assigned index of this function.
    pub index: u32,
    pub name: String,
    // TODO: Rename args to params
    pub args: Vec<Param>,
    pub return_ty: Option<proc_macro2::Span>,
    // TODO: remove
    pub arity: usize,
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
