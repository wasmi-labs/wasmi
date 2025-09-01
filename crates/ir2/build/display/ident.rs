use crate::build::{
    display::utils::{DisplayConcat, IntoDisplayMaybe as _},
    op::{
        BinaryOp,
        CmpBranchOp,
        CmpSelectOp,
        GenericOp,
        LoadOp,
        OperandKind,
        StoreOp,
        TableGetOp,
        TableSetOp,
        UnaryOp,
        V128LoadLaneOp,
        V128ReplaceLaneOp,
    },
    token::{Case, Ident, Sep, SnakeCase},
};
use core::fmt::{self, Display};

pub struct DisplayIdent<T> {
    pub value: T,
    pub case: Case,
}

impl<T> DisplayIdent<T> {
    pub fn camel(value: T) -> Self {
        Self {
            value,
            case: Case::Camel,
        }
    }

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

impl Display for DisplayIdent<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let op = self.value;
        let kind = op.kind;
        let ident = case.wrap(kind.ident());
        let sep = case.wrap(Sep);
        let ident_prefix = DisplayConcat((case.wrap(kind.result_ty()), sep));
        let ident_suffix = kind
            .is_conversion()
            .then_some(kind.value_ty())
            .map(|i| (sep, case.wrap(i)))
            .map(DisplayConcat)
            .display_maybe();
        let result_suffix = case.wrap(OperandKind::Stack);
        let value_suffix = SnakeCase(op.value);
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
        let result_suffix = case.wrap(OperandKind::Stack);
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
        let input_ident = case.wrap(cmp.ident_prefix());
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
        let input_ident = case.wrap(cmp.ident_prefix());
        let result_suffix = case.wrap(OperandKind::Stack);
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
        let result_suffix = case.wrap(OperandKind::Stack);
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
        let result_suffix = case.wrap(OperandKind::Stack);
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

impl Display for DisplayIdent<&'_ V128ReplaceLaneOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let op = self.value;
        let sep = case.wrap(Sep);
        let v128 = case.wrap(Ident::V128);
        let ident = case.wrap(Ident::ReplaceLane);
        let width = op.width;
        let result_suffix = case.wrap(OperandKind::Stack);
        let v128_suffix = SnakeCase(OperandKind::Stack);
        let value_suffix = SnakeCase(op.value);
        write!(
            f,
            "{v128}{sep}{ident}{width}_{result_suffix}{v128_suffix}{value_suffix}"
        )
    }
}

impl Display for DisplayIdent<&'_ V128LoadLaneOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let op = self.value;
        let sep = case.wrap(Sep);
        let v128 = case.wrap(Ident::V128);
        let ident = case.wrap(Ident::LoadLane);
        let width = u8::from(op.width);
        let result_suffix = case.wrap(OperandKind::Stack);
        let ptr_suffix = SnakeCase(op.ptr);
        let v128_suffix = SnakeCase(OperandKind::Stack);
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
            "{v128}{sep}{ident}{width}{mem0_ident}{offset16_ident}_{result_suffix}{ptr_suffix}{v128_suffix}"
        )
    }
}
