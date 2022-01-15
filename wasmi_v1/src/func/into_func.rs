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

macro_rules! impl_into_func {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<T, F, $($tuple,)* R> IntoFunc<T, ($($tuple,)*), R> for F
        where
            F: Fn($($tuple),*) -> R,
            F: Send + Sync + 'static,
            $(
                $tuple: WasmType,
            )*
            R: WasmReturnType,
        {
            type Params = ($($tuple,)*);
            type Results = <R as WasmReturnType>::Ok;

            #[allow(non_snake_case)]
            fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
                IntoFunc::into_func(
                    move |
                        _: Caller<'_, T>,
                        $(
                            $tuple: $tuple,
                        )*
                    | {
                        (self)($($tuple),*)
                    }
                )
            }
        }

        impl<T, F, $($tuple,)* R> IntoFunc<T, (Caller<'_, T>, $($tuple),*), R> for F
        where
            F: Fn(Caller<T>, $($tuple),*) -> R,
            F: Send + Sync + 'static,
            $(
                $tuple: WasmType,
            )*
            R: WasmReturnType,
        {
            type Params = ($($tuple,)*);
            type Results = <R as WasmReturnType>::Ok;

            #[allow(non_snake_case)]
            fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
                let signature = SignatureEntity::new(
                    <Self::Params as WasmTypeList>::value_types(),
                    <Self::Results as WasmTypeList>::value_types(),
                );
                let trampoline = HostFuncTrampoline::new(
                    move |caller: Caller<T>, params_results: FuncParams| -> Result<FuncResults, Trap> {
                        let ($($tuple,)*): Self::Params = params_results.read_params();
                        let results: Self::Results =
                            (self)(caller, $($tuple),*).into_fallible()?;
                        Ok(params_results.write_results(results))
                    },
                );
                (signature, trampoline)
            }
        }
    };
}
for_each_tuple!(impl_into_func);

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

impl WasmType for u32 {
    fn value_type() -> ValueType {
        ValueType::I32
    }
}

impl WasmType for u64 {
    fn value_type() -> ValueType {
        ValueType::I64
    }
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
