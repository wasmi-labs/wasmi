use crate::{
    Error,
    engine::{WasmOperator, translator::FuncTranslator},
};
use wasmparser::VisitOperator;

macro_rules! impl_visit_operator_for_fuel_metering {
    ( @mvp $($rest:tt)* ) => {
        impl_visit_operator_for_fuel_metering!(@@supported $($rest)*);
    };
    ( @sign_extension $($rest:tt)* ) => {
        impl_visit_operator_for_fuel_metering!(@@supported $($rest)*);
    };
    ( @saturating_float_to_int $($rest:tt)* ) => {
        impl_visit_operator_for_fuel_metering!(@@supported $($rest)*);
    };
    ( @bulk_memory $($rest:tt)* ) => {
        impl_visit_operator_for_fuel_metering!(@@supported $($rest)*);
    };
    ( @reference_types $($rest:tt)* ) => {
        impl_visit_operator_for_fuel_metering!(@@supported $($rest)*);
    };
    ( @tail_call $($rest:tt)* ) => {
        impl_visit_operator_for_fuel_metering!(@@supported $($rest)*);
    };
    ( @wide_arithmetic $($rest:tt)* ) => {
        impl_visit_operator_for_fuel_metering!(@@supported $($rest)*);
    };
    ( @simd $($rest:tt)* ) => {
        impl_visit_operator_for_fuel_metering!(@@supported $($rest)*);
    };
    ( @relaxed_simd $($rest:tt)* ) => {
        impl_visit_operator_for_fuel_metering!(@@supported $($rest)*);
    };
    ( @@supported $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $_ann:tt $($rest:tt)* ) => {
        // Implementation for supported Wasm operators.
        #[inline]
        fn $visit(&mut self $($(, $arg: $argty)*)?) -> Self::Output {
            // Apply fuel metering and forward to the underlying func translator's visit implementation.
            self.0.bump_fuel_for_operator(WasmOperator::$op)?;
            self.0.$visit( $( $($arg),* )? )
        }
        impl_visit_operator_for_fuel_metering!($($rest)*);
    };
    ( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $_ann:tt $($rest:tt)* ) => {
        // Fallback implementation for unsupported Wasm operators.
        #[inline]
        fn $visit(&mut self $($(, $arg: $argty)*)?) -> Self::Output {
            // Simply forward to the underlying func translator's visit implementation.
            self.0.$visit( $( $($arg),* )? )
        }
        impl_visit_operator_for_fuel_metering!($($rest)*);
    };
    () => {};
}

/// Wrapper around [`FuncTranslator`] to handle fuel metering.
#[derive(Debug)]
pub struct FuelMeteringFuncTranslator(FuncTranslator);

impl From<FuncTranslator> for FuelMeteringFuncTranslator {
    fn from(translator: FuncTranslator) -> Self {
        Self(translator)
    }
}

impl<'a> VisitOperator<'a> for FuelMeteringFuncTranslator {
    type Output = Result<(), Error>;

    #[cfg(feature = "simd")]
    fn simd_visitor(
        &mut self,
    ) -> Option<&mut dyn wasmparser::VisitSimdOperator<'a, Output = Self::Output>> {
        Some(self)
    }

    wasmparser::for_each_visit_operator!(impl_visit_operator_for_fuel_metering);
}

#[cfg(feature = "simd")]
impl<'a> wasmparser::VisitSimdOperator<'a> for FuelMeteringFuncTranslator {
    wasmparser::for_each_visit_simd_operator!(impl_visit_operator_for_fuel_metering);
}
