use super::state::{DoneReason, Sp, VmState};
use crate::{
    core::UntypedVal,
    ir::{Sign, Slot},
    TrapCode,
};
use core::num::NonZero;

pub trait GetValue<T> {
    fn get_value(src: Self, sp: Sp) -> T;
}

macro_rules! impl_get_value {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl GetValue<$ty> for $ty {
                #[inline(always)]
                fn get_value(src: Self, _sp: Sp) -> $ty {
                    src
                }
            }
        )*
    };
}
impl_get_value!(
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
    f32,
    f64,
    NonZero<i32>,
    NonZero<i64>,
    NonZero<u32>,
    NonZero<u64>,
    Sign<f32>,
    Sign<f64>,
);

impl<T> GetValue<T> for Slot
where
    T: From<UntypedVal>,
{
    fn get_value(src: Self, sp: Sp) -> T {
        sp.get::<T>(src)
    }
}

pub fn get_value<T, L>(src: T, sp: Sp) -> L
where
    T: GetValue<L>,
{
    <T as GetValue<L>>::get_value(src, sp)
}

pub trait SetValue<T> {
    fn set_value(src: Self, value: T, sp: Sp);
}

impl<T> SetValue<T> for Slot
where
    T: Into<UntypedVal>,
{
    fn set_value(src: Self, value: T, sp: Sp) {
        sp.set::<T>(src, value)
    }
}

pub fn set_value<T, V>(sp: Sp, src: T, value: V)
where
    T: SetValue<V>,
{
    <T as SetValue<V>>::set_value(src, value, sp)
}

pub trait UnwrapResult {
    type Item;

    fn unwrap_result(self, state: &mut VmState) -> Option<Self::Item>;
}

impl<T> UnwrapResult for Result<T, TrapCode> {
    type Item = T;

    fn unwrap_result(self, state: &mut VmState) -> Option<Self::Item> {
        match self {
            Ok(item) => Some(item),
            Err(trap_code) => {
                state.done_reason = DoneReason::Trap(trap_code);
                None
            }
        }
    }
}

macro_rules! impl_unwrap_result {
    ($($ty:ty),* $(,)?) => {
        $(
            impl UnwrapResult for $ty {
                type Item = Self;

                #[inline(always)]
                fn unwrap_result(self, _state: &mut VmState) -> Option<Self::Item> {
                    Some(self)
                }
            }
        )*
    };
}
impl_unwrap_result!(bool, i32, i64, u32, u64, f32, f64);

macro_rules! unwrap_result {
    ($value:expr, $state:expr) => {{
        match <_ as $crate::engine::executor::handler::utils::UnwrapResult>::unwrap_result(
            $value, $state,
        ) {
            Some(value) => value,
            None => return Done::default(),
        }
    }};
}
