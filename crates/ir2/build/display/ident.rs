use crate::build::{
    display::utils::{DisplayConcat, IntoDisplayMaybe as _},
    op::{Input, LoadOp, StoreOp},
    token::{Case, Ident, Sep, SnakeCase},
};
use core::fmt::{self, Display};

pub struct DisplayIdent<T> {
    value: T,
    case: Case,
}

impl<T> DisplayIdent<T> {
    pub fn camel(value: T) -> Self {
        Self {
            value,
            case: Case::Camel,
        }
    }

    #[expect(dead_code)]
    pub fn snake(value: T) -> Self {
        Self {
            value,
            case: Case::Snake,
        }
    }
}

impl Display for DisplayIdent<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let kind = self.value.kind;
        let ident = case.wrap(kind.ident());
        let result_suffix = case.wrap(Input::Stack);
        let ptr_suffix = SnakeCase(self.value.ptr);
        let sep = case.wrap(Sep);
        let ident_prefix = self
            .value
            .kind
            .ident_prefix()
            .map(|v| (case.wrap(v), sep))
            .map(DisplayConcat)
            .display_maybe();
        let mem0_ident = self
            .value
            .mem0
            .then_some(Ident::Mem0)
            .map(|v| (sep, case.wrap(v)))
            .map(DisplayConcat)
            .display_maybe();
        let offset16_ident = self
            .value
            .offset16
            .then_some(Ident::Offset16)
            .map(|v| (sep, case.wrap(v)))
            .map(DisplayConcat)
            .display_maybe();
        write!(
            f,
            "{ident_prefix}{ident}{mem0_ident}{offset16_ident}_{result_suffix}{ptr_suffix}",
        )
    }
}

impl Display for DisplayIdent<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let kind = self.value.kind;
        let ident = case.wrap(kind.ident());
        let ptr_suffix = case.wrap(self.value.ptr);
        let value_suffix = SnakeCase(self.value.value);
        let sep = case.wrap(Sep);
        let ident_prefix = self
            .value
            .kind
            .ident_prefix()
            .map(|v| (case.wrap(v), sep))
            .map(DisplayConcat)
            .display_maybe();
        let mem0_ident = self
            .value
            .mem0
            .then_some(Ident::Mem0)
            .map(|v| (sep, case.wrap(v)))
            .map(DisplayConcat)
            .display_maybe();
        let offset16_ident = self
            .value
            .offset16
            .then_some(Ident::Offset16)
            .map(|v| (sep, case.wrap(v)))
            .map(DisplayConcat)
            .display_maybe();
        write!(
            f,
            "{ident_prefix}{ident}{mem0_ident}{offset16_ident}_{ptr_suffix}{value_suffix}",
        )
    }
}
