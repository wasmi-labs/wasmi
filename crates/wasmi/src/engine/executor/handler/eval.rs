use crate::{core::wasm, ir::Sign, TrapCode};
use core::{
    num::NonZero,
    ops::{Div, Rem},
};

pub fn wasmi_i32_div_ssi(lhs: i32, rhs: NonZero<i32>) -> Result<i32, TrapCode> {
    wasm::i32_div_s(lhs, rhs.get())
}

pub fn wasmi_i64_div_ssi(lhs: i64, rhs: NonZero<i64>) -> Result<i64, TrapCode> {
    wasm::i64_div_s(lhs, rhs.get())
}

pub fn wasmi_u32_div_ssi(lhs: u32, rhs: NonZero<u32>) -> u32 {
    <u32 as Div<NonZero<u32>>>::div(lhs, rhs)
}

pub fn wasmi_u64_div_ssi(lhs: u64, rhs: NonZero<u64>) -> u64 {
    <u64 as Div<NonZero<u64>>>::div(lhs, rhs)
}

pub fn wasmi_i32_rem_ssi(lhs: i32, rhs: NonZero<i32>) -> Result<i32, TrapCode> {
    wasm::i32_rem_s(lhs, rhs.get())
}

pub fn wasmi_i64_rem_ssi(lhs: i64, rhs: NonZero<i64>) -> Result<i64, TrapCode> {
    wasm::i64_rem_s(lhs, rhs.get())
}

pub fn wasmi_u32_rem_ssi(lhs: u32, rhs: NonZero<u32>) -> u32 {
    <u32 as Rem<NonZero<u32>>>::rem(lhs, rhs)
}

pub fn wasmi_u64_rem_ssi(lhs: u64, rhs: NonZero<u64>) -> u64 {
    <u64 as Rem<NonZero<u64>>>::rem(lhs, rhs)
}

pub fn wasmi_i32_shl_ssi(lhs: i32, rhs: u8) -> i32 {
    wasm::i32_shl(lhs, i32::from(rhs))
}

pub fn wasmi_i32_shr_ssi(lhs: i32, rhs: u8) -> i32 {
    wasm::i32_shr_s(lhs, i32::from(rhs))
}

pub fn wasmi_u32_shr_ssi(lhs: u32, rhs: u8) -> u32 {
    wasm::i32_shr_u(lhs, u32::from(rhs))
}

pub fn wasmi_i32_rotl_ssi(lhs: i32, rhs: u8) -> i32 {
    wasm::i32_rotl(lhs, i32::from(rhs))
}

pub fn wasmi_i32_rotr_ssi(lhs: i32, rhs: u8) -> i32 {
    wasm::i32_rotr(lhs, i32::from(rhs))
}

pub fn wasmi_i64_shl_ssi(lhs: i64, rhs: u8) -> i64 {
    wasm::i64_shl(lhs, i64::from(rhs))
}

pub fn wasmi_i64_shr_ssi(lhs: i64, rhs: u8) -> i64 {
    wasm::i64_shr_s(lhs, i64::from(rhs))
}

pub fn wasmi_u64_shr_ssi(lhs: u64, rhs: u8) -> u64 {
    wasm::i64_shr_u(lhs, u64::from(rhs))
}

pub fn wasmi_i64_rotl_ssi(lhs: i64, rhs: u8) -> i64 {
    wasm::i64_rotl(lhs, i64::from(rhs))
}

pub fn wasmi_i64_rotr_ssi(lhs: i64, rhs: u8) -> i64 {
    wasm::i64_rotr(lhs, i64::from(rhs))
}

pub fn wasmi_i32_and(lhs: i32, rhs: i32) -> bool {
    (lhs & rhs) != 0
}

pub fn wasmi_i32_not_and(lhs: i32, rhs: i32) -> bool {
    !wasmi_i32_and(lhs, rhs)
}

pub fn wasmi_i32_or(lhs: i32, rhs: i32) -> bool {
    (rhs != 0) || (lhs != 0)
}

pub fn wasmi_i32_not_or(lhs: i32, rhs: i32) -> bool {
    !wasmi_i32_or(lhs, rhs)
}

pub fn wasmi_i64_and(lhs: i64, rhs: i64) -> bool {
    (rhs != 0) && (lhs != 0)
}

pub fn wasmi_i64_not_and(lhs: i64, rhs: i64) -> bool {
    !wasmi_i64_and(lhs, rhs)
}

pub fn wasmi_i64_or(lhs: i64, rhs: i64) -> bool {
    (rhs != 0) || (lhs != 0)
}

pub fn wasmi_i64_not_or(lhs: i64, rhs: i64) -> bool {
    !wasmi_i64_or(lhs, rhs)
}

pub fn wasmi_f32_copysign_ssi(lhs: f32, rhs: Sign<f32>) -> f32 {
    wasm::f32_copysign(lhs, f32::from(rhs))
}

pub fn wasmi_f64_copysign_ssi(lhs: f64, rhs: Sign<f64>) -> f64 {
    wasm::f64_copysign(lhs, f64::from(rhs))
}

pub fn wasmi_f32_not_le(lhs: f32, rhs: f32) -> bool {
    !wasm::f32_le(lhs, rhs)
}

pub fn wasmi_f64_not_le(lhs: f64, rhs: f64) -> bool {
    !wasm::f64_le(lhs, rhs)
}

pub fn wasmi_f32_not_lt(lhs: f32, rhs: f32) -> bool {
    !wasm::f32_lt(lhs, rhs)
}

pub fn wasmi_f64_not_lt(lhs: f64, rhs: f64) -> bool {
    !wasm::f64_lt(lhs, rhs)
}
