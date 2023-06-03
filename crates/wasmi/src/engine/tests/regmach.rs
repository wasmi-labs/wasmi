//! Tests for the register-machine `wasmi` engine translation implementation.

#![allow(unused_imports)] // TODO: remove

use crate::{
    engine::{bytecode2::Instruction, CompiledFunc, DedupFuncType},
    Config,
    Engine,
    Module,
};

/// Asserts that the given `wasm` bytes yield functions with expected instructions.
///
/// Uses the given [`Config`] to configure the [`Engine`] that the tests are run on.
///
/// # Note
///
/// This enables the register machine bytecode translation.
///
/// # Panics
///
/// If any of the yielded functions consists of instruction different from the
/// expected instructions for that function.
fn assert_func_bodies_with_config<E, T>(_wasm_bytes: impl AsRef<[u8]>, _expected: E)
where
    E: IntoIterator<Item = T>,
    T: IntoIterator<Item = Instruction>,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    // let wasm_bytes = wasm_bytes.as_ref();
    // let module = create_module(config, wasm_bytes);
    // let engine = module.engine();
    // for ((func_type, func_body), expected) in module.internal_funcs().zip(expected) {
    //     assert_func_body(engine, func_type, func_body, expected);
    // }
    todo!()
}

/// Asserts that the given `func_body` consists of the expected instructions.
///
/// # Note
///
/// This enables the register machine bytecode translation.
///
/// # Panics
///
/// If there is an instruction mismatch between the actual instructions in
/// `func_body` and the `expected_instructions`.
fn assert_func_body<E>(
    _engine: &Engine,
    _func_type: DedupFuncType,
    _func_body: CompiledFunc,
    _expected_instructions: E,
) where
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    // let expected_instructions = expected_instructions.into_iter();
    // let len_expected = expected_instructions.len();
    // for (index, actual, expected) in
    //     expected_instructions
    //         .into_iter()
    //         .enumerate()
    //         .map(|(index, expected)| {
    //             (
    //                 index,
    //                 engine.resolve_instr(func_body, index).unwrap_or_else(|| {
    //                     panic!("encountered missing instruction at position {index}")
    //                 }),
    //                 expected,
    //             )
    //         })
    // {
    //     assert_eq!(
    //         actual,
    //         expected,
    //         "encountered instruction mismatch for {:?} at position {index}",
    //         engine.resolve_func_type(&func_type, Clone::clone),
    //     );
    // }
    // if let Some(unexpected) = engine.resolve_instr(func_body, len_expected) {
    //     panic!("encountered unexpected instruction at position {len_expected}: {unexpected:?}",);
    // }
    todo!()
}
