use super::TrampolineEntity;
use crate::{
    core::{DecodeUntypedSlice, EncodeUntypedSlice, UntypedVal},
    engine::{InOutParams, InOutResults, LoadFromCellsByValue, StoreToCells},
    Caller,
    Error,
    ExternRef,
    F32,
    F64,
    Func,
    FuncType,
    Nullable,
    ValType,
    core::{DecodeUntypedSlice, EncodeUntypedSlice, UntypedVal},
};
use core::{array, iter::FusedIterator};

#[cfg(feature = "simd")]
use crate::V128;

/// Closures and functions that can be used as host functions.
pub trait IntoFunc<T, Params, Results>: Send + Sync + 'static {
    /// The parameters of the host function.
    #[doc(hidden)]
    type Params: WasmTyList;
    /// The results of the host function.
    #[doc(hidden)]
    type Results: WasmTyList;

    /// Converts the function into its Wasmi signature and its trampoline.
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
                $tuple: WasmTy,
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
                $tuple: WasmTy,
            )*
            R: WasmRet,
        {
            type Params = ($($tuple,)*);
            type Results = <R as WasmRet>::Ok;

            #[allow(non_snake_case)]
            fn into_func(self) -> (FuncType, TrampolineEntity<T>) {
                let func_type = FuncType::new(
                    <Self::Params as WasmTyList>::types(),
                    <Self::Results as WasmTyList>::types(),
                );
                let trampoline = TrampolineEntity::new(
                    move |caller: Caller<T>, inout: InOutParams| -> Result<InOutResults, Error> {
                        let ($($tuple,)*) = inout.decode_params().unwrap(); // TODO: replace `unwrap`
                        let results: Self::Results =
                            (self)(caller, $($tuple),*).into_fallible()?;
                        let inout = inout.encode_results(&results).unwrap(); // TODO: replace `unwrap`
                        Ok(inout)
                    },
                );
                (func_type, trampoline)
            }
        }
    };
}
for_each_tuple!(impl_into_func);

/// Types and type sequences that can be used as return values of host functions.
pub trait WasmRet {
    #[doc(hidden)]
    type Ok: WasmTyList;

    #[doc(hidden)]
    fn into_fallible(self) -> Result<<Self as WasmRet>::Ok, Error>;
}

impl<T1> WasmRet for T1
where
    T1: WasmTy,
{
    type Ok = T1;

    #[inline]
    fn into_fallible(self) -> Result<Self::Ok, Error> {
        Ok(self)
    }
}

impl<T1> WasmRet for Result<T1, Error>
where
    T1: WasmTy,
{
    type Ok = T1;

    #[inline]
    fn into_fallible(self) -> Result<<Self as WasmRet>::Ok, Error> {
        self
    }
}

macro_rules! impl_wasm_return_type {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> WasmRet for ($($tuple,)*)
        where
            $(
                $tuple: WasmTy
            ),*
        {
            type Ok = ($($tuple,)*);

            #[inline]
            fn into_fallible(self) -> Result<Self::Ok, Error> {
                Ok(self)
            }
        }

        impl<$($tuple),*> WasmRet for Result<($($tuple,)*), Error>
        where
            $(
                $tuple: WasmTy
            ),*
        {
            type Ok = ($($tuple,)*);

            #[inline]
            fn into_fallible(self) -> Result<<Self as WasmRet>::Ok, Error> {
                self
            }
        }
    };
}
for_each_tuple!(impl_wasm_return_type);

/// Types that can be used as parameters or results of host functions.
pub trait WasmTy:
    From<UntypedVal> + Into<UntypedVal> + Send + LoadFromCellsByValue + StoreToCells
{
    /// Returns the value type of the Wasm type.
    #[doc(hidden)]
    fn ty() -> ValType;
}

macro_rules! impl_wasm_type {
    ( $(
        $( #[$attr:meta] )*
        type $rust_type:ty = $wasmi_type:ident );* $(;)?
    ) => {
        $(
            $( #[$attr] )*
            impl WasmTy for $rust_type {
                #[inline]
                fn ty() -> ValType {
                    ValType::$wasmi_type
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
    type f32 = F32;
    type f64 = F64;
    #[cfg(feature = "simd")]
    type V128 = V128;
    type Nullable<Func> = FuncRef;
    type Nullable<ExternRef> = ExternRef;
}

/// A list of [`WasmTy`] types.
///
/// # Note
///
/// This is a convenience trait that allows to:
///
/// - Read host function parameters from a region of the value stack.
/// - Write host function results into a region of the value stack.
/// - Iterate over the value types of the Wasm type sequence
///     - This is useful to construct host function signatures.
pub trait WasmTyList:
    DecodeUntypedSlice + EncodeUntypedSlice + LoadFromCellsByValue + StoreToCells + Sized + Send
{
    /// The number of Wasm types in the list.
    #[doc(hidden)]
    const LEN: usize;

    /// The [`ValType`] sequence as array.
    #[doc(hidden)]
    type Types: IntoIterator<IntoIter = Self::TypesIter, Item = ValType>
        + AsRef<[ValType]>
        + AsMut<[ValType]>
        + Copy
        + Clone;

    /// The iterator type of the sequence of [`ValType`].
    #[doc(hidden)]
    type TypesIter: ExactSizeIterator<Item = ValType> + DoubleEndedIterator + FusedIterator;

    /// The [`UntypedVal`] sequence as array.
    #[doc(hidden)]
    type Values: IntoIterator<IntoIter = Self::ValuesIter, Item = UntypedVal>
        + AsRef<[UntypedVal]>
        + AsMut<[UntypedVal]>
        + Copy
        + Clone;

    /// The iterator type of the sequence of [`Val`].
    ///
    /// [`Val`]: [`crate::core::Value`]
    #[doc(hidden)]
    type ValuesIter: ExactSizeIterator<Item = UntypedVal> + DoubleEndedIterator + FusedIterator;

    /// Returns an array representing the [`ValType`] sequence of `Self`.
    #[doc(hidden)]
    fn types() -> Self::Types;

    /// Returns an array representing the [`UntypedVal`] sequence of `self`.
    #[doc(hidden)]
    fn values(self) -> Self::Values;

    /// Consumes the [`UntypedVal`] iterator and creates `Self` if possible.
    ///
    /// Returns `None` if construction of `Self` is impossible.
    #[doc(hidden)]
    fn from_values(values: &[UntypedVal]) -> Option<Self>;
}

impl<T1> WasmTyList for T1
where
    T1: WasmTy,
{
    const LEN: usize = 1;

    type Types = [ValType; 1];
    type TypesIter = array::IntoIter<ValType, 1>;
    type Values = [UntypedVal; 1];
    type ValuesIter = array::IntoIter<UntypedVal, 1>;

    #[inline]
    fn types() -> Self::Types {
        [<T1 as WasmTy>::ty()]
    }

    #[inline]
    fn values(self) -> Self::Values {
        [<T1 as Into<UntypedVal>>::into(self)]
    }

    #[inline]
    fn from_values(values: &[UntypedVal]) -> Option<Self> {
        if let [value] = *values {
            return Some(value.into());
        }
        None
    }
}

macro_rules! impl_wasm_type_list {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> WasmTyList for ($($tuple,)*)
        where
            $(
                $tuple: WasmTy
            ),*
        {
            const LEN: usize = $n;

            type Types = [ValType; $n];
            type TypesIter = array::IntoIter<ValType, $n>;
            type Values = [UntypedVal; $n];
            type ValuesIter = array::IntoIter<UntypedVal, $n>;

            #[inline]
            fn types() -> Self::Types {
                [$(
                    <$tuple as WasmTy>::ty()
                ),*]
            }

            #[inline]
            #[allow(non_snake_case)]
            fn values(self) -> Self::Values {
                let ($($tuple,)*) = self;
                [$(
                    <$tuple as Into<UntypedVal>>::into($tuple)
                ),*]
            }

            #[inline]
            #[allow(non_snake_case)]
            fn from_values(values: &[UntypedVal]) -> Option<Self> {
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
    use std::string::String;

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
        assert!(!implements_wasm_results!(String));
        assert!(!implements_wasm_results!(Option<i32>));
        assert!(implements_wasm_results!(()));
        assert!(implements_wasm_results!(i32));
        assert!(implements_wasm_results!((i32,)));
        assert!(implements_wasm_results!((i32, u32, i64, u64, F32, F64)));
        assert!(implements_wasm_results!(Result<(), Error>));
        assert!(implements_wasm_results!(Result<i32, Error>));
        assert!(implements_wasm_results!(Result<(i32,), Error>));
        assert!(implements_wasm_results!(Result<(i32, u32, i64, u64, F32, F64), Error>));
    }
}
