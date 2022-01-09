use super::{super::SignatureEntity, Caller, HostFuncTrampoline, RuntimeValue, ValueType};
use crate::{
    nan_preserving_float::{F32, F64},
    FromRuntimeValue,
    Trap,
    TrapCode,
};
use alloc::sync::Arc;

pub trait IntoFunc<T, Params, Results>: Send + Sync + 'static {
    #[doc(hidden)]
    fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>);
}

macro_rules! for_each_function_signature {
    ($mac:ident) => {
        $mac!( 0 );
        $mac!( 1 T1);
        $mac!( 2 T1 T2);
        $mac!( 3 T1 T2 T3);
        $mac!( 4 T1 T2 T3 T4);
        $mac!( 5 T1 T2 T3 T4 T5);
        $mac!( 6 T1 T2 T3 T4 T5 T6);
        $mac!( 7 T1 T2 T3 T4 T5 T6 T7);
        $mac!( 8 T1 T2 T3 T4 T5 T6 T7 T8);
        $mac!( 9 T1 T2 T3 T4 T5 T6 T7 T8 T9);
        $mac!(10 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10);
        $mac!(11 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11);
        $mac!(12 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12);
        $mac!(13 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13);
        $mac!(14 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14);
        $mac!(15 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15);
        $mac!(16 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16);
    }
}

macro_rules! impl_into_func {
    ($n:literal $($args:ident)*) => {
        // Implement for functions without a leading `&Caller` parameter,
        // delegating to the implementation below which does have the leading
        // `Caller` parameter.
        #[allow(non_snake_case)]
        impl<T, F, $($args,)* R> IntoFunc<T, ($($args,)*), R> for F
        where
            F: Fn($($args),*) -> R,
            F: Send + Sync + 'static,
            $(
                $args: WasmType,
            )*
            R: WasmReturnType,
        {
            fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
                let f = move |_: Caller<'_, T>, $($args:$args),*| {
                    self($($args),*)
                };
                f.into_func()
            }
        }

        #[allow(non_snake_case)]
        impl<T, F, $($args,)* R> IntoFunc<T, (Caller<'_, T>, $($args,)*), R> for F
        where
            F: Fn(Caller<T>, $($args),*) -> R,
            F: Send + Sync + 'static,
            $(
                $args: WasmType,
            )*
            R: WasmReturnType,
        {
            fn into_func(self) -> (SignatureEntity, HostFuncTrampoline<T>) {
                let inputs = [$(<$args as WasmType>::value_type()),*];
                let signature = R::signature(inputs);
                let len_inputs = signature.inputs().len();
                let len_outputs = signature.outputs().len();
                #[rustfmt::skip]
                #[allow(unused_mut, unused_variables)]
                let trampoline = move |
                    caller: Caller<T>,
                    inputs: &[RuntimeValue],
                    outputs: &mut [RuntimeValue],
                | -> Result<(), Trap> {
                    if inputs.len() != len_inputs || outputs.len() != len_outputs {
                        return Err(Trap::from(TrapCode::UnexpectedSignature))
                    }
                    let mut inputs_iter = inputs.iter();
                    let ( $($args,)* ) = <($($args,)*) as ReadInputs>::read_inputs(inputs)?;
                    let result = (self)(
                        caller,
                        $(
                            $args
                        ),*
                    ).into_fallible()?;
                    result.write_outputs(outputs)?;
                    Ok(())
                };
                // We are using an `Arc` instead of a `Box` here to make `clone` cheap.
                // We currently need cloning to prevent `unsafe` Rust usage when calling
                // a Wasm or host function.
                (signature, HostFuncTrampoline { closure: Arc::new(trampoline) })
            }
        }
    }
}
for_each_function_signature!(impl_into_func);

pub trait WriteOutputs {
    fn write_outputs(self, outputs: &mut [RuntimeValue]) -> Result<(), Trap>;
}

impl<T1> WriteOutputs for T1
where
    T1: Into<RuntimeValue>,
{
    fn write_outputs(self, outputs: &mut [RuntimeValue]) -> Result<(), Trap> {
        if outputs.len() != 1 {
            return Err(Trap::from(TrapCode::UnexpectedSignature));
        }
        outputs[0] = self.into();
        Ok(())
    }
}

macro_rules! impl_write_outputs {
    ($n:literal $($args:ident)*) => {
        #[allow(non_snake_case)]
        impl<$($args),*> WriteOutputs for ($($args,)*)
        where
            $(
                $args: Into<RuntimeValue>
            ),*
        {
            #[allow(unused_mut, unused_variables)]
            fn write_outputs(self, outputs: &mut [RuntimeValue]) -> Result<(), Trap> {
                if outputs.len() != $n {
                    return Err(Trap::from(TrapCode::UnexpectedSignature));
                }
                let ($($args,)*) = self;
                let mut i = 0;
                $({
                    outputs[i] = $args.into();
                })*
                Ok(())
            }
        }
    }
}
for_each_function_signature!(impl_write_outputs);

pub trait ReadInputs: Sized {
    fn read_inputs(inputs: &[RuntimeValue]) -> Result<Self, Trap>;
}

impl<T1> ReadInputs for T1
where
    T1: FromRuntimeValue,
{
    fn read_inputs(inputs: &[RuntimeValue]) -> Result<Self, Trap> {
        if inputs.len() != 1 {
            return Err(Trap::from(TrapCode::UnexpectedSignature));
        }
        RuntimeValue::try_into::<T1>(inputs[0])
            .ok_or_else(|| Trap::from(TrapCode::UnexpectedSignature))
    }
}

macro_rules! impl_read_inputs {
    ($n:literal $($args:ident)*) => {
        impl<$($args),*> ReadInputs for ($($args,)*)
        where
            $(
                $args: FromRuntimeValue
            ),*
        {
            #[allow(unused_mut, unused_variables)]
            fn read_inputs(inputs: &[RuntimeValue]) -> Result<Self, Trap> {
                if inputs.len() != $n {
                    return Err(Trap::from(TrapCode::UnexpectedSignature))
                }
                let mut inputs = inputs.iter();
                Ok((
                    $(
                        inputs
                            .next()
                            .copied()
                            .map(RuntimeValue::try_into::<$args>)
                            .flatten()
                            .ok_or(Trap::from(TrapCode::UnexpectedSignature))?,
                    )*
                ))
            }
        }
    }
}
for_each_function_signature!(impl_read_inputs);

pub trait WasmType: FromRuntimeValue + Into<RuntimeValue> {
    /// The underlying ABI type.
    type Abi: Copy;

    /// Returns the value type of the Wasm type.
    fn value_type() -> ValueType;
}

impl WasmType for i32 {
    type Abi = Self;

    fn value_type() -> ValueType {
        ValueType::I32
    }
}

impl WasmType for i64 {
    type Abi = Self;

    fn value_type() -> ValueType {
        ValueType::I64
    }
}

impl WasmType for F32 {
    type Abi = Self;

    fn value_type() -> ValueType {
        ValueType::F32
    }
}

impl WasmType for F64 {
    type Abi = Self;

    fn value_type() -> ValueType {
        ValueType::F64
    }
}

pub trait WasmReturnType {
    type Abi: WriteOutputs;

    fn signature<I>(inputs: I) -> SignatureEntity
    where
        I: IntoIterator<Item = ValueType>,
        I::IntoIter: ExactSizeIterator;

    fn into_fallible(self) -> Result<Self::Abi, Trap>;
}

impl<T1> WasmReturnType for T1
where
    T1: WasmType,
{
    type Abi = T1;

    fn signature<I>(inputs: I) -> SignatureEntity
    where
        I: IntoIterator<Item = ValueType>,
        I::IntoIter: ExactSizeIterator,
    {
        <Result<Self::Abi, Trap>>::signature(inputs)
    }

    fn into_fallible(self) -> Result<Self::Abi, Trap> {
        Ok(self)
    }
}

impl<T1> WasmReturnType for Result<T1, Trap>
where
    T1: WasmType,
{
    type Abi = T1;

    fn signature<I>(inputs: I) -> SignatureEntity
    where
        I: IntoIterator<Item = ValueType>,
        I::IntoIter: ExactSizeIterator,
    {
        SignatureEntity::new(inputs, [<T1 as WasmType>::value_type()])
    }

    fn into_fallible(self) -> Result<Self::Abi, Trap> {
        self
    }
}

macro_rules! impl_wasm_return_type {
    ($n:literal $($args:ident)*) => {
        impl<$($args),*> WasmReturnType for ($($args,)*)
        where
            $(
                $args: WasmType,
            )*
        {
            type Abi = ($($args,)*);

            fn signature<I>(inputs: I) -> SignatureEntity
            where
                I: IntoIterator<Item = ValueType>,
                I::IntoIter: ExactSizeIterator,
            {
                <Result<Self::Abi, Trap>>::signature(inputs)
            }

            fn into_fallible(self) -> Result<Self::Abi, Trap> {
                Ok(self)
            }
        }

        impl<$($args),*> WasmReturnType for Result<($($args,)*), Trap>
        where
            $(
                $args: WasmType,
            )*
        {
            type Abi = ($($args,)*);

            fn signature<I>(inputs: I) -> SignatureEntity
            where
                I: IntoIterator<Item = ValueType>,
                I::IntoIter: ExactSizeIterator,
            {
                SignatureEntity::new(inputs, [
                    $(
                        <$args as WasmType>::value_type(),
                    )*
                ])
            }

            fn into_fallible(self) -> Result<Self::Abi, Trap> {
                self
            }
        }
    };
}
for_each_function_signature!(impl_wasm_return_type);
