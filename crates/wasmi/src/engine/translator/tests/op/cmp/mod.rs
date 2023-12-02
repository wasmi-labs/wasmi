//! Translation tests for all Wasm comparison instructions.
//!
//! These include the following Wasm instructions:
//!
//! `{i32, i64, f32, f64}.{eq, ne}`
//! `{i32, i64}.{lt_s, lt_u, gt_s, gt_u, le_s, le_u, ge_s, ge_u}`
//! `{f32, f64}.{lt, gt, le, ge}`
//! `{i32, i64}.eqz`
//!
//! # Note
//!
//! Technically `{i32, i64}.eqz` are unary instructions but we still
//! include them here since in `wasmi` bytecode these are represented by
//! more generic comparison instructions.

use super::*;

mod f32_eq;
mod f32_ne;
mod f64_eq;
mod f64_ne;
mod i32_eq;
mod i32_ne;
mod i64_eq;
mod i64_ne;

mod f32_ge;
mod f32_gt;
mod f32_le;
mod f32_lt;
mod f64_ge;
mod f64_gt;
mod f64_le;
mod f64_lt;

mod i32_ge_s;
mod i32_ge_u;
mod i32_gt_s;
mod i32_gt_u;
mod i32_le_s;
mod i32_le_u;
mod i32_lt_s;
mod i32_lt_u;

mod i64_ge_s;
mod i64_ge_u;
mod i64_gt_s;
mod i64_gt_u;
mod i64_le_s;
mod i64_le_u;
mod i64_lt_s;
mod i64_lt_u;
