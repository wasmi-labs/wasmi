use super::{
    super::engine::{FuncParams, FuncResults, WasmType as InternalWasmType},
    HostFuncTrampoline,
};
use crate::{
    engine::{ReadParams, WriteResults},
    foreach_tuple::for_each_tuple,
    Caller,
    SignatureEntity,
};
use core::array;
use wasmi_core::{FromValue, Trap, Value, ValueType, F32, F64};

pub trait IntoFunc<T, Params, Results>: Send + Sync + 'static {
    #[doc(hidden)]
    type Params: WasmTypeList;
    #[doc(hidden)]
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

pub trait WasmReturnType {
    type Ok: WasmTypeList;

    fn into_fallible(self) -> Result<<Self as WasmReturnType>::Ok, Trap>;
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

macro_rules! impl_wasm_return_type {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> WasmReturnType for ($($tuple,)*)
        where
            $(
                $tuple: WasmType
            ),*
        {
            type Ok = ($($tuple,)*);

            fn into_fallible(self) -> Result<Self::Ok, Trap> {
                Ok(self)
            }
        }

        impl<$($tuple),*> WasmReturnType for Result<($($tuple,)*), Trap>
        where
            $(
                $tuple: WasmType
            ),*
        {
            type Ok = ($($tuple,)*);

            fn into_fallible(self) -> Result<<Self as WasmReturnType>::Ok, Trap> {
                self
            }
        }
    };
}
for_each_tuple!(impl_wasm_return_type);

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

impl<T1> WasmTypeList for T1
where
    T1: WasmType,
{
    type Iter = array::IntoIter<ValueType, 1>;

    fn value_types() -> Self::Iter {
        [<T1 as WasmType>::value_type()].into_iter()
    }
}

macro_rules! impl_wasm_type_list {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> WasmTypeList for ($($tuple,)*)
        where
            $(
                $tuple: WasmType
            ),*
        {
            type Iter = array::IntoIter<ValueType, $n>;

            fn value_types() -> Self::Iter {
                [$(
                    <$tuple as WasmType>::value_type()
                ),*].into_iter()
            }
        }
    };
}
for_each_tuple!(impl_wasm_type_list);

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

macro_rules! impl_apply_func {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<F, R, $($tuple),*> ApplyFunc<F, R> for ($($tuple,)*)
        where
            F: Fn($($tuple),*) -> R,
        {
            #[allow(non_snake_case)]
            fn apply_ref(self, f: F) -> R {
                let ($($tuple,)*) = self;
                f($($tuple),*)
            }
        }
    };
}
for_each_tuple!(impl_apply_func);
