use super::GlobalIdx;
use crate::errors::ModuleError;
use wasmi_core::{Value, F32, F64};

/// An initializer expression.
///
/// # Note
///
/// This is used by global variables, table element segments and
/// linear memory data segments.
#[derive(Debug)]
pub struct InitExpr {
    /// The operand of the initializer expression.
    ///
    /// # Note
    ///
    /// The Wasm MVP only supports initializer expressions with a single
    /// operand (besides the `End` operand). In later Wasm proposals this
    /// might change but until then we keep it as simple as possible.
    op: InitExprOperand,
}

impl TryFrom<wasmparser::ConstExpr<'_>> for InitExpr {
    type Error = ModuleError;

    fn try_from(init_expr: wasmparser::ConstExpr<'_>) -> Result<Self, Self::Error> {
        let mut reader = init_expr.get_operators_reader();
        let op = reader.read()?.try_into()?;
        if !matches!(reader.read()?, wasmparser::Operator::End) {
            return Err(ModuleError::unsupported(init_expr));
        }
        Ok(InitExpr { op })
    }
}

impl InitExpr {
    /// Returns a slice over the operators of the [`InitExpr`].
    pub fn operators(&self) -> &[InitExprOperand] {
        core::slice::from_ref(&self.op)
    }

    /// Evaluates the given constant [`InitExpr`] if possible.
    ///
    /// Returns `None` if it is not possible to constant evaluate `expr`.
    pub fn eval_const(&self) -> Option<Value> {
        match self.op {
            InitExprOperand::Const(value) => Some(value),
            InitExprOperand::GlobalGet(_) => {
                // Note: We do not need to handle `global.get` since
                //       that is only allowed for imported non-mutable
                //       global variables which have a value that is only
                //       known post-instantiation time.
                None
            }
        }
    }
}

/// A single operand of an initializer expression.
///
/// # Note
///
/// The Wasm MVP only supports `const` and `global.get` expressions
/// inside initializer expressions. In later Wasm proposals this might
/// change but until then we keep it as simple as possible.
#[derive(Debug)]
pub enum InitExprOperand {
    /// A constant value.
    Const(Value),
    /// The value of a global variable at the time of evaluation.
    ///
    /// # Note
    ///
    /// In the Wasm MVP only immutable globals are allowed to be evaluated.
    GlobalGet(GlobalIdx),
}

impl InitExprOperand {
    /// Creates a new constant [`InitExprOperand`].
    fn constant<T>(value: T) -> Self
    where
        T: Into<Value>,
    {
        Self::Const(value.into())
    }
}

impl TryFrom<wasmparser::Operator<'_>> for InitExprOperand {
    type Error = ModuleError;

    fn try_from(operator: wasmparser::Operator<'_>) -> Result<Self, Self::Error> {
        match operator {
            wasmparser::Operator::I32Const { value } => Ok(InitExprOperand::constant(value)),
            wasmparser::Operator::I64Const { value } => Ok(InitExprOperand::constant(value)),
            wasmparser::Operator::F32Const { value } => {
                Ok(InitExprOperand::constant(F32::from(value.bits())))
            }
            wasmparser::Operator::F64Const { value } => {
                Ok(InitExprOperand::constant(F64::from(value.bits())))
            }
            wasmparser::Operator::GlobalGet { global_index } => {
                Ok(InitExprOperand::GlobalGet(GlobalIdx(global_index)))
            }
            unsupported => Err(ModuleError::unsupported(unsupported)),
        }
    }
}
