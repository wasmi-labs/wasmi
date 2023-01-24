use super::{FuncIdx, GlobalIdx};
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

impl InitExpr {
    /// Creates a new [`InitExpr`] from the given Wasm constant expression.
    pub fn new(expr: wasmparser::ConstExpr<'_>) -> Self {
        let mut reader = expr.get_operators_reader();
        let wasm_op = reader.read().unwrap_or_else(|error| {
            panic!("expected valid Wasm const expression operand: {error}")
        });
        let op = InitExprOperand::new(wasm_op);
        let end_op = reader.read();
        assert!(
            matches!(end_op, Ok(wasmparser::Operator::End)),
            "expected the Wasm end operator but found {end_op:?}",
        );
        assert!(
            reader.ensure_end().is_ok(),
            "expected no more Wasm operands"
        );
        Self { op }
    }

    /// Convert the [`InitExpr`] into the underlying Wasm `elemexpr` if possible.
    ///
    /// Returns `None` if the function reference is `null`.
    ///
    /// # Panics
    ///
    /// If a non Wasm `elemexpr` operand is encountered.
    pub fn into_elemexpr(&self) -> Option<FuncIdx> {
        match self.op {
            InitExprOperand::RefNull => None,
            InitExprOperand::FuncRef(func_index) => Some(FuncIdx(func_index)),
            InitExprOperand::Const(_) | InitExprOperand::GlobalGet(_) => {
                panic!("encountered an unexpected Wasm elemexpr {:?}", self.op)
            }
        }
    }

    /// Return the `Const` [`InitExpr`] if any.
    ///
    /// Returns `None` if the underlying operand is not `Const`.
    ///
    /// # Panics
    ///
    /// If a non-const expression operand is encountered.
    pub fn into_const(&self) -> Option<Value> {
        match self.op {
            InitExprOperand::Const(value) => Some(value),
            // Note: We do not need to handle `global.get` since
            //       that is only allowed for imported non-mutable
            //       global variables which have a value that is only
            //       known post-instantiation time.
            InitExprOperand::GlobalGet(_)
            | InitExprOperand::RefNull
            | InitExprOperand::FuncRef(_) => None,
        }
    }

    /// Evaluates the [`InitExpr`] given the context for global variables.
    ///
    /// # Panics
    ///
    /// If a non-const expression operand is encountered.
    pub fn into_const_with_context(&self, global_get: impl Fn(u32) -> Value) -> Value {
        match self.op {
            InitExprOperand::Const(value) => value,
            InitExprOperand::GlobalGet(index) => global_get(index.into_u32()),
            ref error @ (InitExprOperand::FuncRef(_) | InitExprOperand::RefNull) => {
                panic!("encountered non-const expression operand: {error:?}")
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
    /// A Wasm `ref.null` value.
    RefNull,
    /// A Wasm `ref.func index` value.
    FuncRef(u32),
}

impl InitExprOperand {
    /// Creates a new [`InitExprOperand`] from the given Wasm operator.
    ///
    /// # Panics
    ///
    /// If the Wasm operator is not a valid [`InitExprOperand`].
    fn new(operator: wasmparser::Operator<'_>) -> Self {
        match operator {
            wasmparser::Operator::I32Const { value } => Self::constant(value),
            wasmparser::Operator::I64Const { value } => Self::constant(value),
            wasmparser::Operator::F32Const { value } => Self::constant(F32::from(value.bits())),
            wasmparser::Operator::F64Const { value } => Self::constant(F64::from(value.bits())),
            wasmparser::Operator::GlobalGet { global_index } => {
                Self::GlobalGet(GlobalIdx(global_index))
            }
            wasmparser::Operator::RefNull { .. } => Self::RefNull,
            wasmparser::Operator::RefFunc { function_index } => Self::FuncRef(function_index),
            operator => {
                panic!("encountered unsupported const expression operator: {operator:?}")
            }
        }
    }

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
            wasmparser::Operator::RefNull { .. } => Ok(InitExprOperand::RefNull),
            wasmparser::Operator::RefFunc { function_index } => {
                Ok(InitExprOperand::FuncRef(function_index))
            }
            operator => {
                panic!("encountered unsupported const expression operator: {operator:?}")
            }
        }
    }
}
