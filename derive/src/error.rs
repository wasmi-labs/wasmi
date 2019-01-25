use proc_macro2::{TokenStream, Span};
use quote::{quote_spanned, ToTokens};

macro_rules! err_span {
    ($span:expr, $($msg:tt)*) => (
        $crate::error::CompileError::new_spanned(&$span, format!($($msg)*))
    )
}

pub struct CompileError {
    msg: String,
    span: Option<Span>,
}

impl CompileError {
    pub fn new_spanned(span: &Span, msg: String) -> Self {
        CompileError {
            span: Some(*span),
            msg,
        }
    }

    pub fn new(msg: String) -> Self {
        CompileError {
            span: None,
            msg,
        }
    }
}

impl ToTokens for CompileError {
    fn to_tokens(&self, dst: &mut TokenStream) {
        let msg = &self.msg;
        let span = self.span.unwrap_or_else(|| Span::call_site());
        (quote_spanned! { span=>
            compile_error!(#msg);
        }).to_tokens(dst);
    }
}
