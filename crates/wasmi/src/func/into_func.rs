use super::{
    super::engine::{FuncFinished, FuncParams, FuncResults},
    TrampolineEntity,
};
use crate::{
    core::{Trap, ValueType, F32, F64},
    foreach_tuple::for_each_tuple,
    Caller,
    ExternRef,
    FuncRef,
    FuncType,
};
use core::{array, iter::FusedIterator};
use wasmi_core::{DecodeUntypedSlice, EncodeUntypedSlice, UntypedValue};

/// Closures and functions that can be used as host functions.
pub trait IntoFunc<T, Params, Results>: Send + Sync + 'static {
    /// The parameters of the host function.
    #[doc(hidden)]
    type Params: WasmTypeList;
    /// The results of the host function.
    #[doc(hidden)]
    type Results: WasmTypeList;

    /// Converts the function into its `wasmi` signature and its trampoline.
    #[doc(hidden)]
    fn into_func(self) -> (FuncType, TrampolineEntity<T>);
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
            R: WasmRet,
        {
            type Params = ($($tuple,)*);
            type Results = <R as WasmRet>::Ok;

            #[allow(non_snake_case)]
            fn into_func(self) -> (FuncType, TrampolineEntity<T>) {
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
            R: WasmRet,
        {
            type Params = ($($tuple,)*);
            type Results = <R as WasmRet>::Ok;

            #[allow(non_snake_case)]
            fn into_func(self) -> (FuncType, TrampolineEntity<T>) {
                let signature = FuncType::new(
                    <Self::Params as WasmTypeList>::types(),
                    <Self::Results as WasmTypeList>::types(),
                );
                let trampoline = TrampolineEntity::new(
                    move |caller: Caller<T>, params_results: FuncParams| -> Result<FuncFinished, Trap> {
                        let (($($tuple,)*), func_results): (Self::Params, FuncResults) = params_results.decode_params();
                        let results: Self::Results =
                            (self)(caller, $($tuple),*).into_fallible()?;
                        Ok(func_results.encode_results(results))
                    },
                );
                (signature, trampoline)
            }
        }
    };
}
for_each_tuple!(impl_into_func);

/// Types and type sequences that can be used as return values of host functions.
pub trait WasmRet {
    #[doc(hidden)]
    type Ok: WasmTypeList;

    #[doc(hidden)]
    fn into_fallible(self) -> Result<<Self as WasmRet>::Ok, Trap>;
}

impl<T1> WasmRet for T1
where
    T1: WasmType,
{
    type Ok = T1;

    #[inline]
    fn into_fallible(self) -> Result<Self::Ok, Trap> {
        Ok(self)
    }
}

impl<T1> WasmRet for Result<T1, Trap>
where
    T1: WasmType,
{
    type Ok = T1;

    #[inline]
    fn into_fallible(self) -> Result<<Self as WasmRet>::Ok, Trap> {
        self
    }
}

macro_rules! impl_wasm_return_type {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> WasmRet for ($($tuple,)*)
        where
            $(
                $tuple: WasmType
            ),*
        {
            type Ok = ($($tuple,)*);

            #[inline]
            fn into_fallible(self) -> Result<Self::Ok, Trap> {
                Ok(self)
            }
        }

        impl<$($tuple),*> WasmRet for Result<($($tuple,)*), Trap>
        where
            $(
                $tuple: WasmType
            ),*
        {
            type Ok = ($($tuple,)*);

            #[inline]
            fn into_fallible(self) -> Result<<Self as WasmRet>::Ok, Trap> {
                self
            }
        }
    };
}
for_each_tuple!(impl_wasm_return_type);

/// Types that can be used as parameters or results of host functions.
pub trait WasmType: From<UntypedValue> + Into<UntypedValue> + Send {
    /// Returns the value type of the Wasm type.
    #[doc(hidden)]
    fn ty() -> ValueType;
}

macro_rules! impl_wasm_type {
    ( $( type $rust_type:ty = $wasmi_type:ident );* $(;)? ) => {
        $(
            impl WasmType for $rust_type {
                #[inline]
                fn ty() -> ValueType {
                    ValueType::$wasmi_type
                }
            }
        )*
    };
}
impl_wasm_type! {
    type u32 = I32;
    type u64 = I64;
    type i32 = I32;
    type i64 = I64;
    type F32 = F32;
    type F64 = F64;
    type FuncRef = FuncRef;
    type ExternRef = ExternRef;
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
pub trait WasmTypeList: DecodeUntypedSlice + EncodeUntypedSlice + Sized + Send {
    /// The number of Wasm types in the list.
    #[doc(hidden)]
    const LEN: usize;

    /// The [`ValueType`] sequence as array.
    #[doc(hidden)]
    type Types: IntoIterator<IntoIter = Self::TypesIter, Item = ValueType>
        + AsRef<[ValueType]>
        + AsMut<[ValueType]>
        + Copy
        + Clone;

    /// The iterator type of the sequence of [`ValueType`].
    #[doc(hidden)]
    type TypesIter: ExactSizeIterator<Item = ValueType> + DoubleEndedIterator + FusedIterator;

    /// The [`UntypedValue`] sequence as array.
    #[doc(hidden)]
    type Values: IntoIterator<IntoIter = Self::ValuesIter, Item = UntypedValue>
        + AsRef<[UntypedValue]>
        + AsMut<[UntypedValue]>
        + Copy
        + Clone;

    /// The iterator type of the sequence of [`Value`].
    ///
    /// [`Value`]: [`crate::core::Value`]
    #[doc(hidden)]
    type ValuesIter: ExactSizeIterator<Item = UntypedValue> + DoubleEndedIterator + FusedIterator;

    /// Returns an array representing the [`ValueType`] sequence of `Self`.
    #[doc(hidden)]
    fn types() -> Self::Types;

    /// Returns an array representing the [`UntypedValue`] sequence of `self`.
    #[doc(hidden)]
    fn values(self) -> Self::Values;

    /// Consumes the [`UntypedValue`] iterator and creates `Self` if possible.
    ///
    /// Returns `None` if construction of `Self` is impossible.
    #[doc(hidden)]
    fn from_values(values: &[UntypedValue]) -> Option<Self>;
}

impl<T1> WasmTypeList for T1
where
    T1: WasmType,
{
    const LEN: usize = 1;

    type Types = [ValueType; 1];
    type TypesIter = array::IntoIter<ValueType, 1>;
    type Values = [UntypedValue; 1];
    type ValuesIter = array::IntoIter<UntypedValue, 1>;

    #[inline]
    fn types() -> Self::Types {
        [<T1 as WasmType>::ty()]
    }

    #[inline]
    fn values(self) -> Self::Values {
        [<T1 as Into<UntypedValue>>::into(self)]
    }

    #[inline]
    fn from_values(values: &[UntypedValue]) -> Option<Self> {
        if let [value] = *values {
            return Some(value.into());
        }
        None
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
            const LEN: usize = $n;

            type Types = [ValueType; $n];
            type TypesIter = array::IntoIter<ValueType, $n>;
            type Values = [UntypedValue; $n];
            type ValuesIter = array::IntoIter<UntypedValue, $n>;

            #[inline]
            fn types() -> Self::Types {
                [$(
                    <$tuple as WasmType>::ty()
                ),*]
            }

            #[inline]
            #[allow(non_snake_case)]
            fn values(self) -> Self::Values {
                let ($($tuple,)*) = self;
                [$(
                    <$tuple as Into<UntypedValue>>::into($tuple)
                ),*]
            }

            #[inline]
            #[allow(non_snake_case)]
            fn from_values(values: &[UntypedValue]) -> Option<Self> {
                if let [$($tuple),*] = *values {
                    return Some(
                        ( $( Into::into($tuple), )* )
                    )
                }
                None
            }
        }
    };
}
for_each_tuple!(impl_wasm_type_list);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{F32, F64};

    /// Utility struct helper for the `implements_wasm_results` macro.
    pub struct ImplementsWasmRet<T> {
        marker: core::marker::PhantomData<fn() -> T>,
    }
    /// Utility trait for the fallback case of the `implements_wasm_results` macro.
    pub trait ImplementsWasmRetFallback {
        const VALUE: bool = false;
    }
    impl<T> ImplementsWasmRetFallback for ImplementsWasmRet<T> {}
    /// Utility trait impl for the `true` case of the `implements_wasm_results` macro.
    impl<T> ImplementsWasmRet<T>
    where
        T: WasmRet,
    {
        // We need to allow for dead code at this point because
        // the Rust compiler thinks this function is unused even
        // though it acts as the specialized case for detection.
        #[allow(dead_code)]
        pub const VALUE: bool = true;
    }
    /// Returns `true` if the given type `T` implements the `WasmRet` trait.
    #[macro_export]
    #[doc(hidden)]
    macro_rules! implements_wasm_results {
        ( $T:ty $(,)? ) => {{
            #[allow(unused_imports)]
            use ImplementsWasmRetFallback as _;
            ImplementsWasmRet::<$T>::VALUE
        }};
    }

    #[test]
    fn into_func_trait_impls() {
        assert!(implements_wasm_results!(()));
        assert!(implements_wasm_results!(i32));
        assert!(implements_wasm_results!((i32,)));
        assert!(implements_wasm_results!((i32, u32, i64, u64, F32, F64)));
        assert!(implements_wasm_results!(Result<(), Trap>));
        assert!(implements_wasm_results!(Result<i32, Trap>));
        assert!(implements_wasm_results!(Result<(i32,), Trap>));
        assert!(implements_wasm_results!(Result<(i32, u32, i64, u64, F32, F64), Trap>));
    }
}
