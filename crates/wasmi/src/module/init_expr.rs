//! Data structures to represents Wasm constant expressions.
//!
//! This has built-in support for the `extended-const` Wasm proposal.
//! The design of the execution mechanic was inspired by the [`s1vm`]
//! virtual machine architecture.
//!
//! [`s1vm`]: https://github.com/Neopallium/s1vm

use super::FuncIdx;
use crate::{ExternRef, FuncRef, Value};
use alloc::boxed::Box;
use core::fmt;
use smallvec::SmallVec;
use wasmi_core::{UntypedValue, F32, F64};

/// Types that allow evluation given an evaluation context.
pub trait Eval {
    /// Evaluates `self` given an [`EvalContext`].
    fn eval(&self, ctx: &dyn EvalContext) -> Option<UntypedValue>;
}

/// A [`ConstExpr`] evaluation context.
///
/// Required for evaluating a [`ConstExpr`].
pub trait EvalContext {
    /// Returns the [`Value`] of the global value at `index` if any.
    fn get_global(&self, index: u32) -> Option<Value>;
    /// Returns the [`FuncRef`] of the function at `index` if any.
    fn get_func(&self, index: u32) -> Option<FuncRef>;
}

/// An empty evaluation context.
pub struct EmptyEvalContext;

impl EvalContext for EmptyEvalContext {
    fn get_global(&self, _index: u32) -> Option<Value> {
        None
    }

    fn get_func(&self, _index: u32) -> Option<FuncRef> {
        None
    }
}

/// An input parameter to a [`ConstExpr`] operator.
#[derive(Debug)]
pub enum Op {
    /// A constant value.
    Const(ConstOp),
    /// The value of a global variable.
    Global(GlobalOp),
    /// A Wasm `ref.func index` value.
    FuncRef(FuncRefOp),
    /// An arbitrary expression.
    Expr(ExprOp),
}

/// A constant value operator.
///
/// This may represent the following Wasm operators:
///
/// - `i32.const`
/// - `i64.const`
/// - `f32.const`
/// - `f64.const`
/// - `ref.null`
#[derive(Debug)]
pub struct ConstOp {
    /// The underlying precomputed untyped value.
    value: UntypedValue,
}

impl Eval for ConstOp {
    fn eval(&self, _ctx: &dyn EvalContext) -> Option<UntypedValue> {
        Some(self.value)
    }
}

/// Represents a Wasm `global.get` operator.

#[derive(Debug)]
pub struct GlobalOp {
    /// The index of the global variable.
    global_index: u32,
}

impl Eval for GlobalOp {
    fn eval(&self, ctx: &dyn EvalContext) -> Option<UntypedValue> {
        ctx.get_global(self.global_index).map(UntypedValue::from)
    }
}

/// Represents a Wasm `func.ref` operator.

#[derive(Debug)]
pub struct FuncRefOp {
    /// The index of the function.
    function_index: u32,
}

impl Eval for FuncRefOp {
    fn eval(&self, ctx: &dyn EvalContext) -> Option<UntypedValue> {
        ctx.get_func(self.function_index).map(UntypedValue::from)
    }
}

/// A generic Wasm expression operator.
///
/// This may represent one of the following Wasm operators:
///
/// - `i32.add`
/// - `i32.sub`
/// - `i32.mul`
/// - `i64.add`
/// - `i64.sub`
/// - `i64.mul`
#[allow(clippy::type_complexity)]
pub struct ExprOp {
    /// The underlying closure that implements the expression.
    expr: Box<dyn Fn(&dyn EvalContext) -> Option<UntypedValue> + Send + Sync>,
}

impl fmt::Debug for ExprOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExprOp").finish()
    }
}

impl Eval for ExprOp {
    fn eval(&self, ctx: &dyn EvalContext) -> Option<UntypedValue> {
        (self.expr)(ctx)
    }
}

impl Op {
    /// Creates a new constant operator for the given `value`.
    pub fn constant<T>(value: T) -> Self
    where
        T: Into<Value>,
    {
        Self::Const(ConstOp {
            value: value.into().into(),
        })
    }

    /// Creates a new global operator with the given index.
    pub fn global(global_index: u32) -> Self {
        Self::Global(GlobalOp { global_index })
    }

    /// Creates a new global operator with the given index.
    pub fn funcref(function_index: u32) -> Self {
        Self::FuncRef(FuncRefOp { function_index })
    }

    /// Creates a new expression operator for the given `expr`.
    pub fn expr<T>(expr: T) -> Self
    where
        T: Fn(&dyn EvalContext) -> Option<UntypedValue> + Send + Sync + 'static,
    {
        Self::Expr(ExprOp {
            expr: Box::new(expr),
        })
    }
}

impl Eval for Op {
    fn eval(&self, ctx: &dyn EvalContext) -> Option<UntypedValue> {
        match self {
            Op::Const(op) => op.eval(ctx),
            Op::Global(op) => op.eval(ctx),
            Op::FuncRef(op) => op.eval(ctx),
            Op::Expr(op) => op.eval(ctx),
        }
    }
}

/// A Wasm constant expression.
///
/// These are used to determine the offsets of memory data
/// and table element segments as well as the initial value
/// of global variables.
#[derive(Debug)]
pub struct ConstExpr {
    /// The root operator of the [`ConstExpr`].
    op: Op,
}

impl Eval for ConstExpr {
    fn eval(&self, ctx: &dyn EvalContext) -> Option<UntypedValue> {
        self.op.eval(ctx)
    }
}

macro_rules! def_expr {
    ($lhs:ident, $rhs:ident, $expr:expr) => {{
        Op::expr(move |ctx: &dyn EvalContext| -> Option<UntypedValue> {
            let lhs = $lhs.eval(ctx)?;
            let rhs = $rhs.eval(ctx)?;
            Some($expr(lhs, rhs))
        })
    }};
}

impl ConstExpr {
    /// Creates a new [`ConstExpr`] from the given Wasm [`ConstExpr`].
    ///
    /// # Note
    ///
    /// The constructor assumes that Wasm validation already succeeded
    /// on the input Wasm [`ConstExpr`].
    pub fn new(expr: wasmparser::ConstExpr<'_>) -> Self {
        /// A buffer required for translation of Wasm const expressions.
        type TranslationBuffer = SmallVec<[Op; 3]>;
        /// Convenience function to create the various expression operators.
        fn expr_op(
            stack: &mut TranslationBuffer,
            expr: fn(UntypedValue, UntypedValue) -> UntypedValue,
        ) {
            let rhs = stack
                .pop()
                .expect("must have rhs operator on the stack due to Wasm validation");
            let lhs = stack
                .pop()
                .expect("must have lhs operator on the stack due to Wasm validation");
            let op = match (lhs, rhs) {
                (Op::Const(lhs), Op::Const(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Const(lhs), Op::Global(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Const(lhs), Op::FuncRef(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Const(lhs), Op::Expr(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Global(lhs), Op::Const(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Global(lhs), Op::Global(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Global(lhs), Op::FuncRef(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Global(lhs), Op::Expr(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::FuncRef(lhs), Op::Const(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::FuncRef(lhs), Op::Global(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::FuncRef(lhs), Op::FuncRef(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::FuncRef(lhs), Op::Expr(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Expr(lhs), Op::Const(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Expr(lhs), Op::Global(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Expr(lhs), Op::FuncRef(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Expr(lhs), Op::Expr(rhs)) => def_expr!(lhs, rhs, expr),
            };
            stack.push(op);
        }

        let mut reader = expr.get_operators_reader();
        // TODO: we might want to avoid heap allocation in the simple cases that
        //       only have one operator via the small vector data structure.
        let mut stack = TranslationBuffer::new();
        loop {
            let op = reader.read().unwrap_or_else(|error| {
                panic!("unexpectedly encountered invalid const expression operator: {error}")
            });
            match op {
                wasmparser::Operator::I32Const { value } => {
                    stack.push(Op::constant(value));
                }
                wasmparser::Operator::I64Const { value } => {
                    stack.push(Op::constant(value));
                }
                wasmparser::Operator::F32Const { value } => {
                    stack.push(Op::constant(F32::from(value.bits())));
                }
                wasmparser::Operator::F64Const { value } => {
                    stack.push(Op::constant(F64::from(value.bits())));
                }
                wasmparser::Operator::GlobalGet { global_index } => {
                    stack.push(Op::global(global_index));
                }
                wasmparser::Operator::RefNull { ty } => {
                    let value = match ty {
                        wasmparser::ValType::FuncRef => Value::from(FuncRef::null()),
                        wasmparser::ValType::ExternRef => Value::from(ExternRef::null()),
                        ty => panic!("encountered invalid value type for RefNull: {ty:?}"),
                    };
                    stack.push(Op::constant(value));
                }
                wasmparser::Operator::RefFunc { function_index } => {
                    stack.push(Op::funcref(function_index));
                }
                wasmparser::Operator::I32Add => expr_op(&mut stack, UntypedValue::i32_add),
                wasmparser::Operator::I32Sub => expr_op(&mut stack, UntypedValue::i32_sub),
                wasmparser::Operator::I32Mul => expr_op(&mut stack, UntypedValue::i32_mul),
                wasmparser::Operator::I64Add => expr_op(&mut stack, UntypedValue::i64_add),
                wasmparser::Operator::I64Sub => expr_op(&mut stack, UntypedValue::i64_sub),
                wasmparser::Operator::I64Mul => expr_op(&mut stack, UntypedValue::i64_mul),
                wasmparser::Operator::End => break,
                op => panic!("encountered invalid Wasm const expression operator: {op:?}"),
            };
        }
        reader
            .ensure_end()
            .expect("due to Wasm validation this is guaranteed to suceed");
        let op = stack
            .pop()
            .expect("due to Wasm validation must have one operator on the stack");
        assert!(
            stack.is_empty(),
            "due to Wasm validation operator stack must be empty now"
        );
        Self { op }
    }

    /// Create a new `ref.func x` [`ConstExpr`].
    ///
    /// # Note
    ///
    /// Required for setting up table elements.
    pub fn new_funcref(function_index: u32) -> Self {
        Self {
            op: Op::FuncRef(FuncRefOp { function_index }),
        }
    }

    /// Returns `Some(index)` if the [`ConstExpr`] is a `funcref(index)`.
    ///
    /// Otherwise returns `None`.
    pub fn funcref(&self) -> Option<FuncIdx> {
        if let Op::FuncRef(op) = &self.op {
            return Some(FuncIdx::from(op.function_index));
        }
        None
    }

    /// Evaluates the [`ConstExpr`] in a constant evaluation context.
    ///
    /// # Note
    ///
    /// This is useful for evaluations during Wasm translation to
    /// perform optimizations on the translated bytecode.
    pub fn eval_const(&self) -> Option<UntypedValue> {
        self.eval(&EmptyEvalContext)
    }

    /// Evaluates the [`ConstExpr`] given a context for globals and functions.
    ///
    /// Returns `None` if a non-const expression operand is encountered
    /// or the provided globals and functions context returns `None`.
    ///
    /// # Note
    ///
    /// This is useful for evaluation of [`ConstExpr`] during bytecode execution.
    pub fn eval_with_context<G, F>(&self, global_get: G, func_get: F) -> Option<UntypedValue>
    where
        G: Fn(u32) -> Value,
        F: Fn(u32) -> FuncRef,
    {
        /// Context that wraps closures representing partial evaluation contexts.
        struct WrappedEvalContext<G, F> {
            /// Wrapped context for global variables.
            global_get: G,
            /// Wrapped context for functions.
            func_get: F,
        }
        impl<G, F> EvalContext for WrappedEvalContext<G, F>
        where
            G: Fn(u32) -> Value,
            F: Fn(u32) -> FuncRef,
        {
            fn get_global(&self, index: u32) -> Option<Value> {
                Some((self.global_get)(index))
            }

            fn get_func(&self, index: u32) -> Option<FuncRef> {
                Some((self.func_get)(index))
            }
        }
        self.eval(&WrappedEvalContext::<G, F> {
            global_get,
            func_get,
        })
    }
}
