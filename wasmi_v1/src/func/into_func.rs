use super::{
    super::engine::{FuncParams, FuncResults, WasmType as InternalWasmType},
    HostFuncTrampoline,
};
use crate::{
    engine::{ReadParams, WriteResults},
    Caller,
    SignatureEntity,
};
use alloc::sync::Arc;
use core::array;
use wasmi_core::{FromValue, Trap, Value, ValueType, F32, F64};

pub trait IntoFunc<T, Params, Results>: Send + Sync + 'static {
    type Params: WasmTypeList;
    type Results: WasmTypeList;

    #[doc(hidden)]
    fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>);
}

impl<T, F, R> IntoFunc<T, (), R> for F
where
    F: Fn() -> R,
    F: Send + Sync + 'static,
    R: WasmReturnType,
{
    type Params = ();
    type Results = <R as WasmReturnType>::Ok;

    fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
        IntoFunc::into_func(move |_: Caller<'_, T>| self())
    }
}

impl<T, F, P1, R> IntoFunc<T, P1, R> for F
where
    F: Fn(P1) -> R,
    F: Send + Sync + 'static,
    P1: WasmType,
    R: WasmReturnType,
{
    type Params = P1;
    type Results = <R as WasmReturnType>::Ok;

    fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
        IntoFunc::into_func(move |_: Caller<'_, T>, p1: P1| self(p1))
    }
}

impl<T, F, P1, P2, R> IntoFunc<T, (P1, P2), R> for F
where
    F: Fn(P1, P2) -> R,
    F: Send + Sync + 'static,
    P1: WasmType,
    P2: WasmType,
    R: WasmReturnType,
{
    type Params = (P1, P2);
    type Results = <R as WasmReturnType>::Ok;

    fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
        IntoFunc::into_func(move |_: Caller<'_, T>, p1: P1, p2: P2| self(p1, p2))
    }
}

impl<T, F, P1, P2, P3, R> IntoFunc<T, (P1, P2, P3), R> for F
where
    F: Fn(P1, P2, P3) -> R,
    F: Send + Sync + 'static,
    P1: WasmType,
    P2: WasmType,
    P3: WasmType,
    R: WasmReturnType,
{
    type Params = (P1, P2, P3);
    type Results = <R as WasmReturnType>::Ok;

    fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
        IntoFunc::into_func(move |_: Caller<'_, T>, p1: P1, p2: P2, p3: P3| self(p1, p2, p3))
    }
}

impl<T, F, R> IntoFunc<T, Caller<'_, T>, R> for F
where
    F: Fn(Caller<T>) -> R,
    F: Send + Sync + 'static,
    R: WasmReturnType,
{
    type Params = ();
    type Results = <R as WasmReturnType>::Ok;

    fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
        let signature = SignatureEntity::new(
            <Self::Params as WasmTypeList>::value_types(),
            <Self::Results as WasmTypeList>::value_types(),
        );
        let trampoline = HostFuncTrampoline::new(
            move |caller: Caller<T>, params_results: FuncParams| -> Result<FuncResults, Trap> {
                let _params: Self::Params = params_results.read_params();
                let results: Self::Results = self(caller).into_fallible()?;
                Ok(params_results.write_results(results))
            },
        );
        (signature, trampoline)
    }
}

impl<T, F, P1, R> IntoFunc<T, (Caller<'_, T>, P1), R> for F
where
    F: Fn(Caller<T>, P1) -> R,
    F: Send + Sync + 'static,
    P1: WasmType,
    R: WasmReturnType,
{
    type Params = P1;
    type Results = <R as WasmReturnType>::Ok;

    fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
        let signature = SignatureEntity::new(
            <Self::Params as WasmTypeList>::value_types(),
            <Self::Results as WasmTypeList>::value_types(),
        );
        let trampoline = HostFuncTrampoline::new(
            move |caller: Caller<T>, params_results: FuncParams| -> Result<FuncResults, Trap> {
                let params: Self::Params = params_results.read_params();
                let results: Self::Results = (caller, params).apply_ref(&self).into_fallible()?;
                Ok(params_results.write_results(results))
            },
        );
        (signature, trampoline)
    }
}

impl<T, F, P1, P2, R> IntoFunc<T, (Caller<'_, T>, P1, P2), R> for F
where
    F: Fn(Caller<T>, P1, P2) -> R,
    F: Send + Sync + 'static,
    P1: WasmType,
    P2: WasmType,
    R: WasmReturnType,
{
    type Params = (P1, P2);
    type Results = <R as WasmReturnType>::Ok;

    fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
        let signature = SignatureEntity::new(
            <Self::Params as WasmTypeList>::value_types(),
            <Self::Results as WasmTypeList>::value_types(),
        );
        let trampoline = HostFuncTrampoline::new(
            move |caller: Caller<T>, params_results: FuncParams| -> Result<FuncResults, Trap> {
                let (p1, p2): Self::Params = params_results.read_params();
                let results: Self::Results = (caller, p1, p2).apply_ref(&self).into_fallible()?;
                Ok(params_results.write_results(results))
            },
        );
        (signature, trampoline)
    }
}

impl<T, F, P1, P2, P3, R> IntoFunc<T, (Caller<'_, T>, P1, P2, P3), R> for F
where
    F: Fn(Caller<T>, P1, P2, P3) -> R,
    F: Send + Sync + 'static,
    P1: WasmType,
    P2: WasmType,
    P3: WasmType,
    R: WasmReturnType,
{
    type Params = (P1, P2, P3);
    type Results = <R as WasmReturnType>::Ok;

    fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
        let signature = SignatureEntity::new(
            <Self::Params as WasmTypeList>::value_types(),
            <Self::Results as WasmTypeList>::value_types(),
        );
        let trampoline = HostFuncTrampoline::new(
            move |caller: Caller<T>, params_results: FuncParams| -> Result<FuncResults, Trap> {
                let (p1, p2, p3): Self::Params = params_results.read_params();
                let results: Self::Results =
                    (caller, p1, p2, p3).apply_ref(&self).into_fallible()?;
                Ok(params_results.write_results(results))
            },
        );
        (signature, trampoline)
    }
}

impl<T> HostFuncTrampoline<T> {
    pub fn new<F>(trampoline: F) -> Self
    where
        F: Fn(Caller<T>, FuncParams) -> Result<FuncResults, Trap>,
        F: Send + Sync + 'static,
    {
        Self {
            closure: Arc::new(trampoline),
        }
    }
}

pub trait WasmReturnType {
    type Ok: WasmTypeList;

    fn into_fallible(self) -> Result<<Self as WasmReturnType>::Ok, Trap>;
}

impl WasmReturnType for () {
    type Ok = ();

    fn into_fallible(self) -> Result<Self::Ok, Trap> {
        Ok(self)
    }
}

impl<T1> WasmReturnType for T1
where
    T1: WasmType,
{
    type Ok = T1;

    fn into_fallible(self) -> Result<Self::Ok, Trap> {
        Ok(self)
    }
}

impl<T1> WasmReturnType for Result<T1, Trap>
where
    T1: WasmType,
{
    type Ok = T1;

    fn into_fallible(self) -> Result<<Self as WasmReturnType>::Ok, Trap> {
        self
    }
}

pub trait WasmType: FromValue + Into<Value> + InternalWasmType {
    /// Returns the value type of the Wasm type.
    fn value_type() -> ValueType;
}

impl WasmType for i32 {
    fn value_type() -> ValueType {
        ValueType::I32
    }
}

impl WasmType for i64 {
    fn value_type() -> ValueType {
        ValueType::I64
    }
}

impl WasmType for F32 {
    fn value_type() -> ValueType {
        ValueType::F32
    }
}

impl WasmType for F64 {
    fn value_type() -> ValueType {
        ValueType::F64
    }
}

/// A list of [`WasmType`] types.
///
/// # Note
///
/// This is a convenience trait that allows to:
///
/// - Read host function parameters from a region of the value stack.
/// - Write host function results into a region of the value stack.
/// - Iterate over the value types of the Wasm type sequence
///     - This is useful to construct host function signatures.
pub trait WasmTypeList: ReadParams + WriteResults {
    /// The [`ValueType`] sequence iterator type.
    type Iter: IntoIterator<Item = ValueType> + ExactSizeIterator + DoubleEndedIterator;

    /// Returns an iterator over the [`ValueType`] sequence representing `Self`.
    fn value_types() -> Self::Iter;
}

impl WasmTypeList for () {
    type Iter = array::IntoIter<ValueType, 0>;

    fn value_types() -> Self::Iter {
        [].into_iter()
    }
}

impl<T1> WasmTypeList for T1
where
    T1: WasmType,
{
    type Iter = array::IntoIter<ValueType, 1>;

    fn value_types() -> Self::Iter {
        [<T1 as WasmType>::value_type()].into_iter()
    }
}

impl<T1> WasmTypeList for (T1,)
where
    T1: WasmType,
{
    type Iter = array::IntoIter<ValueType, 1>;

    fn value_types() -> Self::Iter {
        [<T1 as WasmType>::value_type()].into_iter()
    }
}

impl<T1, T2> WasmTypeList for (T1, T2)
where
    T1: WasmType,
    T2: WasmType,
{
    type Iter = array::IntoIter<ValueType, 2>;

    fn value_types() -> Self::Iter {
        [
            <T1 as WasmType>::value_type(),
            <T2 as WasmType>::value_type(),
        ]
        .into_iter()
    }
}

impl<T1, T2, T3> WasmTypeList for (T1, T2, T3)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
{
    type Iter = array::IntoIter<ValueType, 3>;

    fn value_types() -> Self::Iter {
        [
            <T1 as WasmType>::value_type(),
            <T2 as WasmType>::value_type(),
            <T3 as WasmType>::value_type(),
        ]
        .into_iter()
    }
}

impl<T1, T2, T3, T4> WasmTypeList for (T1, T2, T3, T4)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
{
    type Iter = array::IntoIter<ValueType, 4>;

    fn value_types() -> Self::Iter {
        [
            <T1 as WasmType>::value_type(),
            <T2 as WasmType>::value_type(),
            <T3 as WasmType>::value_type(),
            <T4 as WasmType>::value_type(),
        ]
        .into_iter()
    }
}

impl<T1, T2, T3, T4, T5> WasmTypeList for (T1, T2, T3, T4, T5)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
{
    type Iter = array::IntoIter<ValueType, 5>;

    fn value_types() -> Self::Iter {
        [
            <T1 as WasmType>::value_type(),
            <T2 as WasmType>::value_type(),
            <T3 as WasmType>::value_type(),
            <T4 as WasmType>::value_type(),
            <T5 as WasmType>::value_type(),
        ]
        .into_iter()
    }
}

impl<T1, T2, T3, T4, T5, T6> WasmTypeList for (T1, T2, T3, T4, T5, T6)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
{
    type Iter = array::IntoIter<ValueType, 6>;

    fn value_types() -> Self::Iter {
        [
            <T1 as WasmType>::value_type(),
            <T2 as WasmType>::value_type(),
            <T3 as WasmType>::value_type(),
            <T4 as WasmType>::value_type(),
            <T5 as WasmType>::value_type(),
            <T6 as WasmType>::value_type(),
        ]
        .into_iter()
    }
}

impl<T1, T2, T3, T4, T5, T6, T7> WasmTypeList for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
    T7: WasmType,
{
    type Iter = array::IntoIter<ValueType, 7>;

    fn value_types() -> Self::Iter {
        [
            <T1 as WasmType>::value_type(),
            <T2 as WasmType>::value_type(),
            <T3 as WasmType>::value_type(),
            <T4 as WasmType>::value_type(),
            <T5 as WasmType>::value_type(),
            <T6 as WasmType>::value_type(),
            <T7 as WasmType>::value_type(),
        ]
        .into_iter()
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> WasmTypeList for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
    T7: WasmType,
    T8: WasmType,
{
    type Iter = array::IntoIter<ValueType, 8>;

    fn value_types() -> Self::Iter {
        [
            <T1 as WasmType>::value_type(),
            <T2 as WasmType>::value_type(),
            <T3 as WasmType>::value_type(),
            <T4 as WasmType>::value_type(),
            <T5 as WasmType>::value_type(),
            <T6 as WasmType>::value_type(),
            <T7 as WasmType>::value_type(),
            <T8 as WasmType>::value_type(),
        ]
        .into_iter()
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9> WasmTypeList for (T1, T2, T3, T4, T5, T6, T7, T8, T9)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
    T7: WasmType,
    T8: WasmType,
    T9: WasmType,
{
    type Iter = array::IntoIter<ValueType, 9>;

    fn value_types() -> Self::Iter {
        [
            <T1 as WasmType>::value_type(),
            <T2 as WasmType>::value_type(),
            <T3 as WasmType>::value_type(),
            <T4 as WasmType>::value_type(),
            <T5 as WasmType>::value_type(),
            <T6 as WasmType>::value_type(),
            <T7 as WasmType>::value_type(),
            <T8 as WasmType>::value_type(),
            <T9 as WasmType>::value_type(),
        ]
        .into_iter()
    }
}

/// Tuple types that can be applied to a function taking matching parameters.
///
/// # Note
///
/// This is a convenience type to clean up some generic code.
pub trait ApplyFunc<F, R> {
    /// Applies `f` given `self` as parameters.
    ///
    /// # Note
    ///
    /// `Self` usually is a tuple type `(T1, T2, ..)` and `f` is
    /// a function that takes parameters of the same order and structure
    /// as `Self`.
    fn apply_ref(self, f: F) -> R;
}

impl<F, R> ApplyFunc<F, R> for ()
where
    F: Fn() -> R,
{
    fn apply_ref(self, f: F) -> R {
        f()
    }
}

impl<T1, F, R> ApplyFunc<F, R> for (T1,)
where
    F: Fn(T1) -> R,
{
    fn apply_ref(self, f: F) -> R {
        f(self.0)
    }
}

impl<T1, T2, F, R> ApplyFunc<F, R> for (T1, T2)
where
    F: Fn(T1, T2) -> R,
{
    fn apply_ref(self, f: F) -> R {
        f(self.0, self.1)
    }
}

impl<T1, T2, T3, F, R> ApplyFunc<F, R> for (T1, T2, T3)
where
    F: Fn(T1, T2, T3) -> R,
{
    fn apply_ref(self, f: F) -> R {
        f(self.0, self.1, self.2)
    }
}

impl<T1, T2, T3, T4, F, R> ApplyFunc<F, R> for (T1, T2, T3, T4)
where
    F: Fn(T1, T2, T3, T4) -> R,
{
    fn apply_ref(self, f: F) -> R {
        f(self.0, self.1, self.2, self.3)
    }
}

impl<T1, T2, T3, T4, T5, F, R> ApplyFunc<F, R> for (T1, T2, T3, T4, T5)
where
    F: Fn(T1, T2, T3, T4, T5) -> R,
{
    fn apply_ref(self, f: F) -> R {
        f(self.0, self.1, self.2, self.3, self.4)
    }
}

impl<T1, T2, T3, T4, T5, T6, F, R> ApplyFunc<F, R> for (T1, T2, T3, T4, T5, T6)
where
    F: Fn(T1, T2, T3, T4, T5, T6) -> R,
{
    fn apply_ref(self, f: F) -> R {
        f(self.0, self.1, self.2, self.3, self.4, self.5)
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, F, R> ApplyFunc<F, R> for (T1, T2, T3, T4, T5, T6, T7)
where
    F: Fn(T1, T2, T3, T4, T5, T6, T7) -> R,
{
    fn apply_ref(self, f: F) -> R {
        f(self.0, self.1, self.2, self.3, self.4, self.5, self.6)
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, F, R> ApplyFunc<F, R> for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    F: Fn(T1, T2, T3, T4, T5, T6, T7, T8) -> R,
{
    fn apply_ref(self, f: F) -> R {
        f(
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7,
        )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, F, R> ApplyFunc<F, R>
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9)
where
    F: Fn(T1, T2, T3, T4, T5, T6, T7, T8, T9) -> R,
{
    fn apply_ref(self, f: F) -> R {
        f(
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8,
        )
    }
}
