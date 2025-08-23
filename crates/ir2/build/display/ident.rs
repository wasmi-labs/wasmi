use crate::build::{
    display::utils::{DisplayConcat, IntoDisplayMaybe as _},
    op::{
        BinaryOp,
        CmpBranchOp,
        CmpSelectOp,
        GenericOp,
        Input,
        LoadOp,
        Op,
        StoreOp,
        TableGetOp,
        TableSetOp,
        UnaryOp,
    },
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

    pub fn map<V>(&self, value: V) -> DisplayIdent<V> {
        DisplayIdent {
            value,
            case: self.case,
        }
    }
}

impl Display for DisplayIdent<&'_ Op> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Op::Unary(op) => self.map(op).fmt(f),
            Op::Binary(op) => self.map(op).fmt(f),
            Op::CmpBranch(op) => self.map(op).fmt(f),
            Op::CmpSelect(op) => self.map(op).fmt(f),
            Op::Load(op) => self.map(op).fmt(f),
            Op::Store(op) => self.map(op).fmt(f),
            Op::TableGet(op) => self.map(op).fmt(f),
            Op::TableSet(op) => self.map(op).fmt(f),
            Op::Generic0(op) => self.map(op).fmt(f),
            Op::Generic1(op) => self.map(op).fmt(f),
            Op::Generic2(op) => self.map(op).fmt(f),
            Op::Generic3(op) => self.map(op).fmt(f),
            Op::Generic4(op) => self.map(op).fmt(f),
            Op::Generic5(op) => self.map(op).fmt(f),
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

impl Display for DisplayIdent<&'_ CmpBranchOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let sep = case.wrap(Sep);
        let cmp = self.value.cmp;
        let branch = case.wrap(Ident::Branch);
        let ident = case.wrap(cmp.ident());
        let input_ident = case.wrap(Ident::from(cmp.input_ty()));
        let lhs_suffix = case.wrap(self.value.lhs);
        let rhs_suffix = SnakeCase(self.value.rhs);
        write!(
            f,
            "{branch}{sep}{input_ident}{sep}{ident}_{lhs_suffix}{rhs_suffix}"
        )
    }
}

impl Display for DisplayIdent<&'_ CmpSelectOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let cmp = self.value.cmp;
        let select = case.wrap(Ident::Select);
        let ident = case.wrap(cmp.ident());
        let input_ident = case.wrap(Ident::from(cmp.input_ty()));
        let result_suffix = case.wrap(Input::Stack);
        let lhs_suffix = SnakeCase(self.value.lhs);
        let rhs_suffix = SnakeCase(self.value.rhs);
        write!(
            f,
            "{select}{input_ident}{ident}_{result_suffix}{lhs_suffix}{rhs_suffix}"
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

impl<const N: usize> Display for DisplayIdent<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ident = self.case.wrap(self.value.ident);
        write!(f, "{ident}")
    }
}

impl Display for DisplayIdent<&'_ TableGetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let ident = case.wrap(Ident::TableGet);
        let result_suffix = case.wrap(Input::Stack);
        let index_suffix = SnakeCase(self.value.index);
        write!(f, "{ident}_{result_suffix}{index_suffix}")
    }
}

impl Display for DisplayIdent<&'_ TableSetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let ident = case.wrap(Ident::TableSet);
        let index_suffix = case.wrap(self.value.index);
        let value_suffix = SnakeCase(self.value.value);
        write!(f, "{ident}_{index_suffix}{value_suffix}")
    }
}
