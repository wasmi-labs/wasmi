use super::{utils::WasmiValueType, FuncIdx, GlobalIdx};
use crate::{ExternRef, FuncRef, Value};
use wasmi_core::{ValueType, F32, F64};

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

    /// Create a new `ref.func x` [`InitExpr`].
    pub fn new_funcref(func_index: u32) -> Self {
        Self {
            op: InitExprOperand::FuncRef(func_index),
        }
    }

    /// Convert the [`InitExpr`] into a Wasm `funcexpr` index if possible.
    ///
    /// Returns `None` if the function reference is `null`.
    ///
    /// # Panics
    ///
    /// If the [`InitExpr`] cannot be interpreted as `funcref` index.
    pub fn as_funcref(&self) -> Option<FuncIdx> {
        match self.op {
            InitExprOperand::RefNull { .. } => None,
            InitExprOperand::FuncRef(func_index) => Some(FuncIdx::from(func_index)),
            InitExprOperand::Const(_) | InitExprOperand::GlobalGet(_) => {
                panic!("encountered non-funcref Wasm expression {:?}", self.op)
            }
        }
    }

    /// Converts the [`InitExpr`] into an `externref`.
    ///
    /// # Panics
    ///
    /// If the [`InitExpr`] cannot be interpreted as `externref`.
    pub fn as_externref(&self) -> ExternRef {
        match self.op {
            InitExprOperand::RefNull {
                ty: ValueType::ExternRef,
            } => ExternRef::null(),
            _ => panic!("encountered non-externref Wasm expression: {:?}", self.op),
        }
    }

    /// Return the `Const` [`InitExpr`] if any.
    ///
    /// Returns `None` if the underlying operand is not `Const`.
    ///
    /// # Panics
    ///
    /// If a non-const expression operand is encountered.
    pub fn to_const(&self) -> Option<Value> {
        match &self.op {
            InitExprOperand::Const(value) => Some(value.clone()),
            InitExprOperand::RefNull { ty } => match ty {
                ValueType::FuncRef => Some(Value::from(FuncRef::null())),
                ValueType::ExternRef => Some(Value::from(ExternRef::null())),
                _ => panic!("cannot have null reference for non-reftype but found {ty:?}"),
            },
            InitExprOperand::GlobalGet(_) => {
                // Note: We do not need to handle `global.get` since
                //       that is only allowed for imported non-mutable
                //       global variables which have a value that is only
                //       known post-instantiation time.
                None
            }
            InitExprOperand::FuncRef(_) => {
                // Note: We do not need to handle `global.get` here
                //       since we can do this somewhere else where we
                //       can properly handle the constant `func.ref`.
                //       In the function builder we want to replace
                //       `global.get` of constant `FuncRef(x)` with
                //       the Wasm `ref.func x` instruction.
                None
            }
        }
    }

    /// Returns `Some(index)` if the [`InitExpr`] is a `FuncRef(index)`.
    ///
    /// Otherwise returns `None`.
    pub fn func_ref(&self) -> Option<FuncIdx> {
        if let InitExprOperand::FuncRef(index) = self.op {
            return Some(FuncIdx::from(index));
        }
        None
    }

    /// Evaluates the [`InitExpr`] given the context for global variables.
    ///
    /// # Panics
    ///
    /// If a non-const expression operand is encountered.
    pub fn to_const_with_context(
        &self,
        global_get: impl Fn(u32) -> Value,
        func_get: impl Fn(u32) -> Value,
    ) -> Value {
        match &self.op {
            InitExprOperand::Const(value) => value.clone(),
            InitExprOperand::GlobalGet(index) => global_get(index.into_u32()),
            InitExprOperand::RefNull { ty } => match ty {
                ValueType::FuncRef => Value::from(FuncRef::null()),
                ValueType::ExternRef => Value::from(ExternRef::null()),
                _ => panic!("expected reftype for InitExpr but found {ty:?}"),
            },
            InitExprOperand::FuncRef(index) => func_get(*index),
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
    RefNull { ty: ValueType },
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
                Self::GlobalGet(GlobalIdx::from(global_index))
            }
            wasmparser::Operator::RefNull { hty } => Self::RefNull {
                ty: WasmiValueType::from(hty).into_inner(),
            },
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
