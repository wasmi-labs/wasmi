//! Data structures to represents Wasm constant expressions.
//!
//! This has built-in support for the `extended-const` Wasm proposal.
//! The design of the execution mechanic was inspired by the [`s1vm`]
//! virtual machine architecture.
//!
//! [`s1vm`]: https://github.com/Neopallium/s1vm

use super::FuncIdx;
use crate::{ExternRef, F32, F64, Func, Nullable, RefType, Val, core::wasm};
use alloc::{boxed::Box, vec::Vec};
use core::{fmt, mem};
use wasmparser::AbstractHeapType;

#[cfg(feature = "simd")]
use crate::V128;

/// Types that allow evaluation given an evaluation context.
pub trait Eval {
    /// Evaluates `self` given an [`EvalContext`].
    fn eval(&self, ctx: &dyn EvalContext) -> Option<Val>;
}

/// A [`ConstExpr`] evaluation context.
///
/// Required for evaluating a [`ConstExpr`].
pub trait EvalContext {
    /// Returns the value of the global at `index` within the context `self` if any.
    ///
    /// Returns `None` if `self` cannot find or resolve the global at `index`.
    fn get_global(&self, index: u32) -> Option<Val>;

    /// Returns the function at `index` within the context `self` if any.
    ///
    /// Returns `None` if `self` cannot find or resolve the function at `index`.
    fn get_func(&self, index: u32) -> Option<Nullable<Func>>;
}

/// An empty evaluation context.
pub struct EmptyEvalContext;

impl EvalContext for EmptyEvalContext {
    fn get_global(&self, _index: u32) -> Option<Val> {
        None
    }

    fn get_func(&self, _index: u32) -> Option<Nullable<Func>> {
        None
    }
}

/// An input parameter to a [`ConstExpr`] operator.
#[derive(Debug)]
pub enum Op {
    /// A constant value.
    Const(ConstOp),
    /// The value of a global variable.
    GlobalGet(GlobalGetOp),
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
    /// The underlying precomputed raw value.
    value: ConstVal,
}

impl Eval for ConstOp {
    #[inline]
    fn eval(&self, _ctx: &dyn EvalContext) -> Option<Val> {
        Some(self.value.into())
    }
}

/// Represents a Wasm `global.get` operator.

#[derive(Debug)]
pub struct GlobalGetOp {
    /// The index of the global variable.
    index: u32,
}

impl Eval for GlobalGetOp {
    #[inline]
    fn eval(&self, ctx: &dyn EvalContext) -> Option<Val> {
        ctx.get_global(self.index)
    }
}

/// Represents a Wasm `func.ref` operator.

#[derive(Debug)]
pub struct FuncRefOp {
    /// The index of the function.
    index: u32,
}

impl Eval for FuncRefOp {
    #[inline]
    fn eval(&self, ctx: &dyn EvalContext) -> Option<Val> {
        ctx.get_func(self.index).map(Val::from)
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
    expr: Box<dyn Fn(&dyn EvalContext) -> Option<Val> + Send + Sync>,
}

impl fmt::Debug for ExprOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExprOp").finish()
    }
}

impl Eval for ExprOp {
    fn eval(&self, ctx: &dyn EvalContext) -> Option<Val> {
        (self.expr)(ctx)
    }
}

impl Op {
    /// Creates a new constant operator for the given `value`.
    fn constant<T>(value: T) -> Self
    where
        T: Into<ConstVal>,
    {
        Self::Const(ConstOp {
            value: value.into(),
        })
    }

    /// Creates a new `global.get` operator with the given index.
    fn global_get(index: u32) -> Self {
        Self::GlobalGet(GlobalGetOp { index })
    }

    /// Creates a new `func.ref` operator with the given index.
    fn func_ref(index: u32) -> Self {
        Self::FuncRef(FuncRefOp { index })
    }

    /// Creates a new expression operator for the given `expr`.
    fn expr<T>(expr: T) -> Self
    where
        T: Fn(&dyn EvalContext) -> Option<Val> + Send + Sync + 'static,
    {
        Self::Expr(ExprOp {
            expr: Box::new(expr),
        })
    }
}

/// A constant initializer value.
///
/// # Note
///
/// - In contrast to [`Val`] this type does not allow for [`Store`](crate::Store)
///   or [`Instance`](crate::Instance) related values such as non-`null` reference values.
/// - This type is meant to be used in Wasm initializer expressions only.
#[derive(Debug, Copy, Clone)]
enum ConstVal {
    /// A Wasm `i32` value.
    I32(i32),
    /// A Wasm `i64` value.
    I64(i64),
    /// A Wasm `f32` value.
    F32(F32),
    /// A Wasm `f64` value.
    F64(F64),
    /// A Wasm `v128` value.
    #[cfg(feature = "simd")]
    V128(V128),
    /// A Wasm reference type `null` value.
    Null(RefType),
}

impl From<i32> for ConstVal {
    #[inline]
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<i64> for ConstVal {
    #[inline]
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<f32> for ConstVal {
    #[inline]
    fn from(value: f32) -> Self {
        Self::F32(F32::from_float(value))
    }
}

impl From<f64> for ConstVal {
    #[inline]
    fn from(value: f64) -> Self {
        Self::F64(F64::from_float(value))
    }
}

impl From<F32> for ConstVal {
    #[inline]
    fn from(value: F32) -> Self {
        Self::F32(value)
    }
}

impl From<F64> for ConstVal {
    #[inline]
    fn from(value: F64) -> Self {
        Self::F64(value)
    }
}

#[cfg(feature = "simd")]
impl From<V128> for ConstVal {
    #[inline]
    fn from(value: V128) -> Self {
        Self::V128(value)
    }
}

/// Allows to unwrap `self` as type `T`.
///
/// Returns `None` if conversion from `self` to `T` is invalid.
trait UnwrapAs<T> {
    fn unwrap_as(self) -> Option<T>;
}

impl UnwrapAs<i32> for ConstVal {
    #[inline]
    fn unwrap_as(self) -> Option<i32> {
        match self {
            Self::I32(value) => Some(value),
            _ => None,
        }
    }
}

impl UnwrapAs<i64> for ConstVal {
    #[inline]
    fn unwrap_as(self) -> Option<i64> {
        match self {
            Self::I64(value) => Some(value),
            _ => None,
        }
    }
}

impl UnwrapAs<f32> for ConstVal {
    #[inline]
    fn unwrap_as(self) -> Option<f32> {
        match self {
            Self::F32(value) => Some(value.to_float()),
            _ => None,
        }
    }
}

impl UnwrapAs<f64> for ConstVal {
    #[inline]
    fn unwrap_as(self) -> Option<f64> {
        match self {
            Self::F64(value) => Some(value.to_float()),
            _ => None,
        }
    }
}

impl UnwrapAs<F32> for ConstVal {
    #[inline]
    fn unwrap_as(self) -> Option<F32> {
        match self {
            Self::F32(value) => Some(value),
            _ => None,
        }
    }
}

impl UnwrapAs<F64> for ConstVal {
    #[inline]
    fn unwrap_as(self) -> Option<F64> {
        match self {
            Self::F64(value) => Some(value),
            _ => None,
        }
    }
}

#[cfg(feature = "simd")]
impl UnwrapAs<V128> for ConstVal {
    #[inline]
    fn unwrap_as(self) -> Option<V128> {
        match self {
            Self::V128(value) => Some(value),
            _ => None,
        }
    }
}

impl UnwrapAs<Nullable<Func>> for ConstVal {
    #[inline]
    fn unwrap_as(self) -> Option<Nullable<Func>> {
        match self {
            Self::Null(RefType::Func) => Some(Nullable::Null),
            _ => None,
        }
    }
}

impl UnwrapAs<Nullable<ExternRef>> for ConstVal {
    #[inline]
    fn unwrap_as(self) -> Option<Nullable<ExternRef>> {
        match self {
            Self::Null(RefType::Extern) => Some(Nullable::Null),
            _ => None,
        }
    }
}

impl From<ConstVal> for Val {
    #[inline]
    fn from(value: ConstVal) -> Self {
        match value {
            ConstVal::I32(value) => value.into(),
            ConstVal::I64(value) => value.into(),
            ConstVal::F32(value) => value.into(),
            ConstVal::F64(value) => value.into(),
            #[cfg(feature = "simd")]
            ConstVal::V128(value) => value.into(),
            ConstVal::Null(RefType::Func) => Self::FuncRef(Nullable::Null),
            ConstVal::Null(RefType::Extern) => Self::ExternRef(Nullable::Null),
        }
    }
}

impl Val {
    /// Returns the underlying [`ConstVal`] of `self` or `None`.
    ///
    /// Returns `None` if `self` is a non-null reference value.
    fn as_const_or_none(&self) -> Option<ConstVal> {
        let value = match *self {
            Self::I32(value) => value.into(),
            Self::I64(value) => value.into(),
            Self::F32(value) => value.to_float().into(),
            Self::F64(value) => value.to_float().into(),
            #[cfg(feature = "simd")]
            Self::V128(value) => value.into(),
            Self::FuncRef(Nullable::Null) => ConstVal::Null(RefType::Func),
            Self::ExternRef(Nullable::Null) => ConstVal::Null(RefType::Extern),
            _ => return None,
        };
        Some(value)
    }
}

impl ConstVal {
    /// Creates a new `null` reference type [`ConstVal`].
    #[inline]
    pub fn null(ty: RefType) -> Self {
        Self::Null(ty)
    }
}

impl Eval for Op {
    fn eval(&self, ctx: &dyn EvalContext) -> Option<Val> {
        match self {
            Self::Const(op) => op.eval(ctx),
            Self::GlobalGet(op) => op.eval(ctx),
            Self::FuncRef(op) => op.eval(ctx),
            Self::Expr(op) => op.eval(ctx),
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
    fn eval(&self, ctx: &dyn EvalContext) -> Option<Val> {
        self.op.eval(ctx)
    }
}

macro_rules! def_expr {
    ($lhs:ident, $rhs:ident, $expr:expr) => {{
        Op::expr(move |ctx: &dyn EvalContext| -> Option<Val> {
            let lhs = $lhs.eval(ctx)?.as_const_or_none()?.unwrap_as()?;
            let rhs = $rhs.eval(ctx)?.as_const_or_none()?.unwrap_as()?;
            Some(ConstVal::from($expr(lhs, rhs)).into())
        })
    }};
}

/// Stack to translate [`ConstExpr`].
#[derive(Debug, Default)]
pub struct ConstExprStack {
    /// The top-most [`Op`] on the stack.
    ///
    /// # Note
    ///
    /// This is an optimization so that the [`ConstExprStack`] does not
    /// require heap allocations for the common case where only a single
    /// stack slot is needed.
    // TODO: turn into `Op` to signal that `ConstExprStack` cannot be empty
    top: Option<Op>,
    /// The remaining ops on the stack.
    ops: Vec<Op>,
}

impl ConstExprStack {
    /// Returns `true` if [`ConstExprStack`] is empty.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Pushes an [`Op`] to the [`ConstExprStack`].
    pub fn push(&mut self, op: Op) {
        let old_top = self.top.replace(op);
        if let Some(old_top) = old_top {
            self.ops.push(old_top);
        }
    }

    /// Pops the top-most [`Op`] from the [`ConstExprStack`] if any.
    pub fn pop(&mut self) -> Option<Op> {
        let new_top = self.ops.pop();
        mem::replace(&mut self.top, new_top)
    }

    /// Pops the 2 top-most [`Op`]s from the [`ConstExprStack`] if any.
    pub fn pop2(&mut self) -> Option<(Op, Op)> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        Some((lhs, rhs))
    }
}

impl ConstExpr {
    /// Creates a new [`ConstExpr`] from the given Wasm [`ConstExpr`].
    ///
    /// # Note
    ///
    /// The constructor assumes that Wasm validation already succeeded
    /// on the input Wasm [`ConstExpr`].
    pub fn new(expr: wasmparser::ConstExpr<'_>) -> Self {
        use wasmparser::Operator as WasmOp;

        /// Convenience function to create the various expression operators.
        fn expr_op<Lhs, Rhs, T>(stack: &mut ConstExprStack, expr: fn(Lhs, Rhs) -> T) -> Op
        where
            Lhs: 'static,
            Rhs: 'static,
            T: 'static,
            ConstVal: UnwrapAs<Lhs> + UnwrapAs<Rhs> + From<T>,
        {
            let (lhs, rhs) = stack
                .pop2()
                .expect("must have 2 operators on the stack due to Wasm validation");
            match (lhs, rhs) {
                (Op::Const(lhs), Op::Const(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Const(lhs), Op::GlobalGet(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Const(lhs), Op::FuncRef(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Const(lhs), Op::Expr(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::GlobalGet(lhs), Op::Const(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::GlobalGet(lhs), Op::GlobalGet(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::GlobalGet(lhs), Op::FuncRef(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::GlobalGet(lhs), Op::Expr(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::FuncRef(lhs), Op::Const(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::FuncRef(lhs), Op::GlobalGet(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::FuncRef(lhs), Op::FuncRef(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::FuncRef(lhs), Op::Expr(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Expr(lhs), Op::Const(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Expr(lhs), Op::GlobalGet(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Expr(lhs), Op::FuncRef(rhs)) => def_expr!(lhs, rhs, expr),
                (Op::Expr(lhs), Op::Expr(rhs)) => def_expr!(lhs, rhs, expr),
            }
        }

        let mut reader = expr.get_operators_reader();
        let mut stack = ConstExprStack::default();
        loop {
            let wasm_op = reader
                .read()
                .unwrap_or_else(|error| panic!("invalid const expression operator: {error}"));
            let op = match wasm_op {
                WasmOp::I32Const { value } => Op::constant(value),
                WasmOp::I64Const { value } => Op::constant(value),
                WasmOp::F32Const { value } => Op::constant(f32::from(value)),
                WasmOp::F64Const { value } => Op::constant(f64::from(value)),
                #[cfg(feature = "simd")]
                WasmOp::V128Const { value } => Op::constant(V128::from(value.i128() as u128)),
                WasmOp::GlobalGet { global_index } => Op::global_get(global_index),
                WasmOp::RefNull { hty } => {
                    let value = match hty {
                        wasmparser::HeapType::Abstract {
                            shared: false,
                            ty: AbstractHeapType::Func,
                        } => ConstVal::null(RefType::Func),
                        wasmparser::HeapType::Abstract {
                            shared: false,
                            ty: AbstractHeapType::Extern,
                        } => ConstVal::null(RefType::Extern),
                        invalid => {
                            panic!("invalid heap type for `ref.null`: {invalid:?}")
                        }
                    };
                    Op::constant(value)
                }
                WasmOp::RefFunc { function_index } => Op::func_ref(function_index),
                WasmOp::I32Add => expr_op(&mut stack, wasm::i32_add),
                WasmOp::I32Sub => expr_op(&mut stack, wasm::i32_sub),
                WasmOp::I32Mul => expr_op(&mut stack, wasm::i32_mul),
                WasmOp::I64Add => expr_op(&mut stack, wasm::i64_add),
                WasmOp::I64Sub => expr_op(&mut stack, wasm::i64_sub),
                WasmOp::I64Mul => expr_op(&mut stack, wasm::i64_mul),
                WasmOp::End => break,
                op => panic!("unexpected Wasm const expression operator: {op:?}"),
            };
            stack.push(op);
        }
        reader
            .ensure_end()
            .expect("Wasm validation requires const expressions to have an `end`");
        let op = stack
            .pop()
            .expect("must contain the root const expression at this point");
        debug_assert!(stack.is_empty());
        Self { op }
    }

    /// Create a new `ref.func x` [`ConstExpr`].
    ///
    /// # Note
    ///
    /// Required for setting up table elements.
    pub fn new_funcref(function_index: u32) -> Self {
        Self {
            op: Op::FuncRef(FuncRefOp {
                index: function_index,
            }),
        }
    }

    /// Returns `Some(index)` if the [`ConstExpr`] is a `funcref(index)`.
    ///
    /// Otherwise returns `None`.
    pub fn funcref(&self) -> Option<FuncIdx> {
        if let Op::FuncRef(op) = &self.op {
            return Some(FuncIdx::from(op.index));
        }
        None
    }
}
