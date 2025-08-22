use crate::build::{
    display::utils::{DisplayConcat, IntoDisplayMaybe as _},
    op::{BinaryOp, Input, LoadOp, StoreOp, UnaryOp},
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

impl Display for DisplayIdent<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let kind = self.value.kind;
        let ident = case.wrap(kind.ident());
        let sep = case.wrap(Sep);
        let ident_prefix = DisplayConcat((case.wrap(Ident::from(kind.result_ty())), sep));
        let ident_suffix = self
            .value
            .kind
            .is_conversion()
            .then_some(Ident::from(kind.input_ty()))
            .map(|i| (sep, case.wrap(i)))
            .map(DisplayConcat)
            .display_maybe();
        let result_suffix = case.wrap(Input::Stack);
        let value_suffix = SnakeCase(Input::Stack);
        write!(
            f,
            "{ident_prefix}{ident}{ident_suffix}_{result_suffix}{value_suffix}"
        )
    }
}

impl Display for DisplayIdent<&'_ BinaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let sep = case.wrap(Sep);
        let kind = self.value.kind;
        let ident = case.wrap(kind.ident());
        let ident_prefix = case.wrap(kind.ident_prefix());
        let result_suffix = case.wrap(Input::Stack);
        let lhs_suffix = SnakeCase(self.value.lhs);
        let rhs_suffix = SnakeCase(self.value.rhs);
        write!(
            f,
            "{ident_prefix}{sep}{ident}_{result_suffix}{lhs_suffix}{rhs_suffix}"
        )
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
