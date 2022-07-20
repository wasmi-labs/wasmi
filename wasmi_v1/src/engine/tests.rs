use super::{bytecode::Global, *};
use crate::{
    engine::{ExecInstruction, ExecProvider, ExecProviderSlice, ExecRegister, Offset},
    module::{FuncIdx, FuncTypeIdx},
    Engine,
    Module,
};
use core::{
    fmt::Display,
    ops::{Shl, Shr},
};
use wasmi_core::{
    ArithmeticOps,
    ExtendInto,
    Float,
    Integer,
    SignExtendFrom,
    TrapCode,
    TruncateSaturateInto,
    TryTruncateInto,
    UntypedValue,
    WrapInto,
    F32,
    F64,
};

/// Allows to create a `1` instance for a type.
pub trait One {
    /// Returns a value of `Self` that equals or represents `1` (one).
    fn one() -> Self;
}

macro_rules! impl_one_for {
    ( $( type $ty:ty = $value:literal );* $(;)? ) => {
        $(
            impl One for $ty {
                fn one() -> Self {
                    $value
                }
            }
        )*
    };
}

impl_one_for! {
    type i32 = 1_i32;
    type i64 = 1_i64;
    type f32 = 1.0_f32;
    type f64 = 1.0_f64;
}

/// Implemented by Wasm compatible types to print them into `.wat` sources.
pub trait WasmTypeName {
    /// The Wasm name of `Self`.
    const NAME: &'static str;
}

macro_rules! impl_wasm_type_name {
    ( $( type $ty:ty = $name:literal );* $(;)? ) => {
        $(
            impl WasmTypeName for $ty {
                const NAME: &'static str = $name;
            }
        )*
    };
}

impl_wasm_type_name! {
    type i32 = "i32";
    type u32 = "i32";
    type i64 = "i64";
    type u64 = "i64";
    type f32 = "f32";
    type f64 = "f64";
    type F32 = "f32";
    type F64 = "f64";
    type bool = "i32";
}

/// Creates a closure taking 3 parameters and constructing a `wasmi` instruction.
macro_rules! make_op {
    ( $name:ident ) => {{
        |result, lhs, rhs| ExecInstruction::$name { result, lhs, rhs }
    }};
}

/// Creates a closure taking 2 parameters and constructing a `wasmi` instruction.
macro_rules! make_op2 {
    ( $name:ident ) => {{
        |result, input| ExecInstruction::$name { result, input }
    }};
}

/// Creates a closure for constructing a `wasmi` load instruction.
macro_rules! load_op {
    ( $name:ident ) => {{
        |result, ptr, offset| ExecInstruction::$name {
            result,
            ptr,
            offset,
        }
    }};
}

/// Creates a closure for constructing a `wasmi` store instruction.
macro_rules! store_op {
    ( $name:ident ) => {{
        |ptr, offset, value| ExecInstruction::$name { ptr, offset, value }
    }};
}

/// Converts the `wat` string source into `wasm` encoded byte.
fn wat2wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).unwrap()
}

/// Compiles the `wasm` encoded bytes into a [`Module`].
///
/// # Panics
///
/// If an error occurred upon module compilation, validation or translation.
fn create_module(bytes: &[u8]) -> Module {
    let engine = Engine::default();
    Module::new(&engine, bytes).unwrap()
}

/// Asserts that the given `func_body` consists of the expected instructions.
///
/// # Panics
///
/// If there is an instruction mismatch between the actual instructions in
/// `func_body` and the `expected_instructions`.
fn assert_func_body<E>(
    engine: &Engine,
    func_type: DedupFuncType,
    func_body: FuncBody,
    expected_instructions: E,
) where
    E: IntoIterator<Item = ExecInstruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let expected_instructions = expected_instructions.into_iter();
    let len_expected = expected_instructions.len();
    for (index, actual, expected) in
        expected_instructions
            .into_iter()
            .enumerate()
            .map(|(index, expected)| {
                (
                    index,
                    engine.resolve_inst(func_body, index).unwrap_or_else(|| {
                        panic!("encountered missing instruction at position {}", index)
                    }),
                    expected,
                )
            })
    {
        assert_eq!(
            actual,
            expected,
            "encountered instruction mismatch for {} at position {}",
            engine.resolve_func_type(func_type, Clone::clone),
            index
        );
    }
    if let Some(unexpected) = engine.resolve_inst(func_body, len_expected) {
        panic!(
            "encountered unexpected instruction at position {}: {:?}",
            len_expected, unexpected,
        );
    }
}

fn assert_func_bodies_for_module<E, T>(module: &Module, expected: E)
where
    E: IntoIterator<Item = T>,
    T: IntoIterator<Item = ExecInstruction>,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let engine = module.engine();
    for ((func_type, func_body), expected) in module.internal_funcs().zip(expected) {
        assert_func_body(engine, func_type, func_body, expected);
    }
}

/// Asserts that the given `wasm` bytes yield functions with expected instructions.
///
/// # Panics
///
/// If any of the yielded functions consists of instruction different from the
/// expected instructions for that function.
fn assert_func_bodies<E, T>(wasm_bytes: impl AsRef<[u8]>, expected: E)
where
    E: IntoIterator<Item = T>,
    T: IntoIterator<Item = ExecInstruction>,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let wasm_bytes = wasm_bytes.as_ref();
    let module = create_module(wasm_bytes);
    let engine = module.engine();
    for ((func_type, func_body), expected) in module.internal_funcs().zip(expected) {
        assert_func_body(engine, func_type, func_body, expected);
    }
}

/// This test has a function that only has a single `unreachable` instruction.
/// We expect to see exactly that as the only emitted instruction.
#[test]
fn unreachable() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                unreachable
            )
        )
    "#,
    );
    let expected = [ExecInstruction::Trap {
        trap_code: TrapCode::Unreachable,
    }];
    assert_func_bodies(&wasm, [expected]);
}

/// This test has 2 consecutive `unreachable` instructions of which we expect
/// just one to be emitted since the `wasmi` translator sees that the other
/// one is unreachable.
#[test]
fn unreachable_double() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                unreachable
                unreachable
            )
        )
    "#,
    );
    let expected = [ExecInstruction::Trap {
        trap_code: TrapCode::Unreachable,
    }];
    assert_func_bodies(&wasm, [expected]);
}

/// The `br` instructions ends its enclosing block.
/// We expect to see just a single `br` instruction branching to its next
/// instructions which is a `return` instruction.
#[test]
fn br_simple() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                block
                    br 0
                end
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let results = engine.alloc_provider_slice([]);
    let expected = [
        ExecInstruction::Br {
            target: Target::from_inner(1),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br` ends function execution immediately.
/// We expect to see just a single `return` instruction since the translation
/// process is in an unreachable state when encountering the `end` instruction.
#[test]
fn br_as_return_unnested() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                br 0
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let results = engine.alloc_provider_slice([]);
    let expected = [ExecInstruction::Return { results }];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br` ends function execution immediately but is nested in a `block`.
/// Therefore we expect to see 2 `return` instructions.
/// - 1 for the `br` instruction
/// - 2 for the `end` instruction since the translation process does not
///   notice that all paths to it are unreachable
#[test]
fn br_as_return_nested() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                block
                    br 1
                end
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let results = engine.alloc_provider_slice([]);
    let expected = [
        ExecInstruction::Return { results },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br` instruction targets its enclosing loop header so it is jumping back.
/// This modules an infinite loop but we are not interested in actual code
/// semantics for this unit test.
/// We expect to see a `br` instruction that targets itself since it is the first
/// instruction of the loop body, followed by a `return` instruction for the
/// `end` instruction of the `loop` in order to end function execution.
/// The `wasmi` translator cannot detect (yet) that the `end` is unreachable
/// in this case.
#[test]
fn br_to_loop_header() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                loop
                    br 0
                end
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let results = engine.alloc_provider_slice([]);
    let loop_header = Target::from_inner(0);
    let expected = [
        ExecInstruction::Br {
            target: loop_header,
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br_if` instructions ends its enclosing block.
/// We expect to see just a single `br` instruction branching to its next
/// instructions which is a `return` instruction.
#[test]
fn br_if_simple() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32)
                block
                    local.get 0
                    br_if 0
                end
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let condition = ExecRegister::from_inner(0);
    let results = engine.alloc_provider_slice([]);
    let expected = [
        ExecInstruction::BrNez {
            target: Target::from_inner(1),
            condition,
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br_if` ends function execution immediately.
/// We expect to see just 2 `return` instructions since the translation
/// process is not always in an unreachable state when encountering the
/// `end` instruction.
#[test]
fn br_if_as_return_unnested() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32)
                local.get 0
                br_if 0
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let condition = ExecRegister::from_inner(0);
    let results = engine.alloc_provider_slice([]);
    let expected = [
        ExecInstruction::ReturnNez { results, condition },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br_if` ends function execution immediately but is nested in a `block`.
/// We expect to see 3 `return[nez]` instructions.
/// - 1 for the `br_if` instruction
/// - 1 for the `br` instruction that follows the `br_if`.
/// - 1 for the `end` instruction since the translation process does not
///   notice that all paths to it are unreachable
#[test]
fn br_if_as_return_nested() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32)
                block
                    local.get 0
                    br_if 1
                    br 1
                end
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let condition = ExecRegister::from_inner(0);
    let results = engine.alloc_provider_slice([]);
    let expected = [
        ExecInstruction::ReturnNez { results, condition },
        ExecInstruction::Return { results },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The test `.wat` features a `loop` within a `block`.
/// In the loop the `br_if` continues to the loop header and is followed
/// by a `br` that ends the loop.
/// Obviously this models an infinite loop but this is just a test.
#[test]
fn br_if_to_loop_header() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32)
                block
                    loop
                        local.get 0
                        br_if 0
                        br 1
                    end
                end
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let condition = ExecRegister::from_inner(0);
    let results = engine.alloc_provider_slice([]);
    let loop_header = Target::from_inner(0);
    let loop_end = Target::from_inner(2);
    let expected = [
        ExecInstruction::BrNez {
            target: loop_header,
            condition,
        },
        ExecInstruction::Br { target: loop_end },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br_if` is equivalent to a `br` instruction since its condition
/// is always `true`.
/// Therefore we expect to see a single `return` instruction only since
/// unreachable code detection prevents emitting of the `end` return
/// instruction.
#[test]
fn br_if_const_true() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                i32.const 1
                br_if 0
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let results = engine.alloc_provider_slice([]);
    let expected = [ExecInstruction::Return { results }];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br_if` is like a `br` instruction since its condition is always `true`.
/// Therefore we jump to the final `result` instruction that returns `10` instead
/// of returning `20` in the block since we know that this part of the code
/// is unreachable.
#[test]
fn br_if_const_true_return() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (result i32)
                block
                    i32.const 1
                    br_if 0
                    i32.const 20
                    return
                end
                i32.const 10
                return
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let result = engine.alloc_const(10_i32);
    let results = engine.alloc_provider_slice([result.into()]);
    let block_end = Target::from_inner(1);
    let expected = [
        ExecInstruction::Br { target: block_end },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br_if` is equivalent to a `nop` (do nothing) instruction since
/// its condition is always `false`.
/// We expect to see a single `return` instruction  the `end` instruction
/// will emit a `return` instruction.
#[test]
fn br_if_const_false() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                i32.const 0
                br_if 0
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let results = engine.alloc_provider_slice([]);
    let expected = [ExecInstruction::Return { results }];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br_if` is just a `nop` since its condition is always `false`.
/// Therefore we are returning `20` always.
/// Still we expect a `return` instruction afterwards since the translation
/// cannot guarantee that this part of the code is unreachable (yet).
#[test]
fn br_if_const_false_return() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (result i32)
                block
                    i32.const 0
                    br_if 0
                    i32.const 20
                    return
                end
                i32.const 10
                return
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let const_10 = engine.alloc_const(10_i32);
    let const_20 = engine.alloc_const(20_i32);
    let results_10 = engine.alloc_provider_slice([const_10.into()]);
    let results_20 = engine.alloc_provider_slice([const_20.into()]);
    let expected = [
        ExecInstruction::Return {
            results: results_20,
        },
        ExecInstruction::Return {
            results: results_10,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br_table` sets different constant values to the global variable
/// via different branches taken.
/// The default branch does not modify the global variable and instead
/// immediately bails out of the function execution.
#[test]
fn br_table_simple() {
    let wasm = wat2wasm(
        r#"
        (module
            (global $result (mut i32) (i32.const 0))
            (func (export "call") (param i32)
                block
                    block
                        block
                            local.get 0
                            br_table 0 1 2 3
                        end
                        i32.const 10
                        global.set $result
                        return
                    end
                    i32.const 20
                    global.set $result
                    return
                end
                i32.const 30
                global.set $result
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let c10 = ExecProvider::from_immediate(engine.alloc_const(10_i32));
    let c20 = ExecProvider::from_immediate(engine.alloc_const(20_i32));
    let c30 = ExecProvider::from_immediate(engine.alloc_const(30_i32));
    let reg0 = ExecRegister::from_inner(0);
    let results = engine.alloc_provider_slice([]);
    let global = Global::from(0);
    #[rustfmt::skip]
    let expected = [
        /* 0 */ ExecInstruction::BrTable {
            case: reg0,
            len_targets: 4, // note: amount is including the default target
        },
        /* 1 case 0       */ ExecInstruction::BrMulti {
            target: Target::from_inner(5),
            results: ExecRegisterSlice::empty(),
            returned: ExecProviderSlice::empty(),
        },
        /* 2 case 1       */ ExecInstruction::BrMulti {
            target: Target::from_inner(7),
            results: ExecRegisterSlice::empty(),
            returned: ExecProviderSlice::empty(),
        },
        /* 3 case 2       */ ExecInstruction::BrMulti {
            target: Target::from_inner(9),
            results: ExecRegisterSlice::empty(),
            returned: ExecProviderSlice::empty(),
        },
        /* 4 case default */ ExecInstruction::Return { results },
        // branch for case 0
        /* 5 */ ExecInstruction::GlobalSet {
            global,
            value: c10,
        },
        /* 6 */ ExecInstruction::Return { results },
        // branch for case 1
        /* 7 */ ExecInstruction::GlobalSet {
            global,
            value: c20,
        },
        /* 8 */ ExecInstruction::Return { results },
        // branch for case 2
        /* 9 */ ExecInstruction::GlobalSet {
            global,
            value: c30,
        },
        /* 10 */ ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br_table` add different constant offsets to the function parameter
/// and returns the result of the computation.
#[test]
fn br_table_return() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                block
                    block
                        block
                            block
                                local.get 0
                                br_table 0 1 2 3
                            end
                            local.get 0
                            i32.const 10
                            i32.add
                            return
                        end
                        local.get 0
                        i32.const 20
                        i32.add
                        return
                    end
                    local.get 0
                    i32.const 30
                    i32.add
                    return
                end
                local.get 0
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let c10 = ExecProvider::from_immediate(engine.alloc_const(10_i32));
    let c20 = ExecProvider::from_immediate(engine.alloc_const(20_i32));
    let c30 = ExecProvider::from_immediate(engine.alloc_const(30_i32));
    let reg0 = ExecRegister::from_inner(0);
    let reg1 = ExecRegister::from_inner(1);
    let results_reg1 = engine.alloc_provider_slice([reg1.into()]);
    let results_reg0 = engine.alloc_provider_slice([reg0.into()]);
    #[rustfmt::skip]
    let expected = [
        /* 0 */ ExecInstruction::BrTable {
            case: reg0,
            len_targets: 4, // note: amount is including the default target
        },
        /* 1 case 0       */ ExecInstruction::BrMulti {
            target: Target::from_inner(5),
            results: ExecRegisterSlice::empty(),
            returned: ExecProviderSlice::empty(),
        },
        /* 2 case 1       */ ExecInstruction::BrMulti {
            target: Target::from_inner(7),
            results: ExecRegisterSlice::empty(),
            returned: ExecProviderSlice::empty(),
        },
        /* 3 case 2       */ ExecInstruction::BrMulti {
            target: Target::from_inner(9),
            results: ExecRegisterSlice::empty(),
            returned: ExecProviderSlice::empty(),
        },
        /* 4 default case */ ExecInstruction::BrMulti {
            target: Target::from_inner(11),
            results: ExecRegisterSlice::empty(),
            returned: ExecProviderSlice::empty(),
        },
        // branch for case 0
        /* 5 */ ExecInstruction::I32Add {
            result: reg1,
            lhs: reg0,
            rhs: c10,
        },
        /* 6 */ ExecInstruction::Return { results: results_reg1 },
        // branch for case 1
        /* 7 */ ExecInstruction::I32Add {
            result: reg1,
            lhs: reg0,
            rhs: c20,
        },
        /* 8 */ ExecInstruction::Return { results: results_reg1 },
        // branch for case 2
        /* 9 */ ExecInstruction::I32Add {
            result: reg1,
            lhs: reg0,
            rhs: c30,
        },
        /* 10 */ ExecInstruction::Return { results: results_reg1 },
        // end of function implicit return
        /* 11 */ ExecInstruction::Return { results: results_reg0 },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `br_table` has a constant value case and therefore the `wasmi`
/// translation reduces the entire `br_table` down to the constant known
/// branch.
#[test]
fn br_table_const_case() {
    fn test(const_case: i32, expect_branch: fn(&Engine) -> ExecInstruction) {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (global $result (mut i32) (i32.const 0))
                (func (export "call")
                    block
                        block
                            block
                                i32.const {const_case}
                                br_table 0 1 2 3
                            end
                            i32.const 10
                            global.set $result
                            return
                        end
                        i32.const 20
                        global.set $result
                        return
                    end
                    i32.const 30
                    global.set $result
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let c10 = ExecProvider::from_immediate(engine.alloc_const(10_i32));
        let c20 = ExecProvider::from_immediate(engine.alloc_const(20_i32));
        let c30 = ExecProvider::from_immediate(engine.alloc_const(30_i32));
        let results = engine.alloc_provider_slice([]);
        let global = Global::from(0);
        #[rustfmt::skip]
        let expected = [
            /* 0 */ expect_branch(engine),
            // branch for case 0
            /* 1 */ ExecInstruction::GlobalSet {
                global,
                value: c10,
            },
            /* 2 */ ExecInstruction::Return { results },
            // branch for case 1
            /* 3 */ ExecInstruction::GlobalSet {
                global,
                value: c20,
            },
            /* 4 */ ExecInstruction::Return { results },
            // branch for case 2
            /* 5 */ ExecInstruction::GlobalSet {
                global,
                value: c30,
            },
            /* 6 */ ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test(0, |_engine| ExecInstruction::BrMulti {
        target: Target::from_inner(1),
        results: ExecRegisterSlice::empty(),
        returned: ExecProviderSlice::empty(),
    });
    test(1, |_engine| ExecInstruction::BrMulti {
        target: Target::from_inner(3),
        results: ExecRegisterSlice::empty(),
        returned: ExecProviderSlice::empty(),
    });
    test(2, |_engine| ExecInstruction::BrMulti {
        target: Target::from_inner(5),
        results: ExecRegisterSlice::empty(),
        returned: ExecProviderSlice::empty(),
    });
    test(3, |engine| {
        let results = engine.alloc_provider_slice([]);
        ExecInstruction::Return { results }
    });
}

#[test]
fn add_10_assign() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                local.get 0
                i32.const 10
                i32.add
                local.tee 0
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let c10 = engine.alloc_const(10_i32).into();
    let local_0 = ExecRegister::from_inner(0);
    let results = engine.alloc_provider_slice([local_0.into()]);
    let expected = [
        ExecInstruction::I32Add {
            result: local_0,
            lhs: local_0,
            rhs: c10,
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// Tests compilation of a no-op function.
#[test]
fn implicit_return_no_value() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
            )
        )
    "#,
    );
    let expected = [ExecInstruction::Return {
        results: ExecProviderSlice::empty(),
    }];
    assert_func_bodies(&wasm, [expected]);
}

/// Tests compilation of all commutative binary Wasm instructions.
///
/// # Note
///
/// This test specializes on cases where both inputs are register inputs
/// (e.g. `local.get 0`).
/// This is the most trivial case to cover and simply checks that the
/// correct instruction with the correct operands is resulting.
///
/// This includes the following Wasm instructions:
///
/// - `{i32, i64, f32, f64}.eq`
/// - `{i32, i64, f32, f64}.ne`
/// - `{i32, i64, f32, f64}.add`
/// - `{i32, i64, f32, f64}.sub`
/// - `{i32, i64, f32, f64}.mul`
/// - `{i32, i64}.div_s`
/// - `{i32, i64}.div_u`
/// - `{i32, i64}.rem_s`
/// - `{i32, i64}.rem_u`
/// - `{i32, i64}.shl`
/// - `{i32, i64}.shr_s`
/// - `{i32, i64}.shr_u`
/// - `{i32, i64}.rotl`
/// - `{i32, i64}.rotr`
/// - `{i32, i64}.and`
/// - `{i32, i64}.or`
/// - `{i32, i64}.xor`
/// - `{f32, f64}.div`
/// - `{f32, f64}.rem`
/// - `{f32, f64}.min`
/// - `{f32, f64}.max`
/// - `{f32, f64}.copysign`
#[test]
fn binary_simple() {
    fn test_register_register<T, F, R>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + Into<UntypedValue>,
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction,
        R: WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (param {input_type}) (result {output_type})
                    local.get 0
                    local.get 1
                    {input_type}.{wasm_op}
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let lhs = ExecRegister::from_inner(0);
        let rhs = ExecRegister::from_inner(1);
        let result = ExecRegister::from_inner(2);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let expected = [
            make_op(result, lhs, rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    fn test_register_const<T, F, R>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + Into<UntypedValue> + One,
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction,
        R: WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <R as WasmTypeName>::NAME;
        let one = T::one();
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {output_type})
                    local.get 0
                    {input_type}.const {one}
                    {input_type}.{wasm_op}
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let lhs = ExecRegister::from_inner(0);
        let rhs = ExecProvider::from_immediate(engine.alloc_const(one));
        let result = ExecRegister::from_inner(1);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let expected = [
            make_op(result, lhs, rhs),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    fn run_test_bin<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + Into<UntypedValue> + One,
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction + Copy,
    {
        test_register_register::<T, F, T>(wasm_op, make_op);
        test_register_const::<T, F, T>(wasm_op, make_op);
    }

    fn run_test_cmp<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + Into<UntypedValue> + One,
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction + Copy,
    {
        test_register_register::<T, F, bool>(wasm_op, make_op);
        test_register_const::<T, F, bool>(wasm_op, make_op);
    }

    run_test_cmp::<i32, _>("eq", make_op!(I32Eq));
    run_test_cmp::<i64, _>("eq", make_op!(I64Eq));
    run_test_cmp::<i32, _>("ne", make_op!(I32Ne));
    run_test_cmp::<i64, _>("ne", make_op!(I64Ne));

    run_test_bin::<i32, _>("add", make_op!(I32Add));
    run_test_bin::<i64, _>("add", make_op!(I64Add));
    run_test_bin::<i32, _>("sub", make_op!(I32Sub));
    run_test_bin::<i64, _>("sub", make_op!(I64Sub));
    run_test_bin::<i32, _>("mul", make_op!(I32Mul));
    run_test_bin::<i64, _>("mul", make_op!(I64Mul));
    run_test_bin::<i32, _>("div_s", make_op!(I32DivS));
    run_test_bin::<i64, _>("div_s", make_op!(I64DivS));
    run_test_bin::<i32, _>("div_u", make_op!(I32DivU));
    run_test_bin::<i64, _>("div_u", make_op!(I64DivU));
    run_test_bin::<i32, _>("rem_s", make_op!(I32RemS));
    run_test_bin::<i64, _>("rem_s", make_op!(I64RemS));
    run_test_bin::<i32, _>("rem_u", make_op!(I32RemU));
    run_test_bin::<i64, _>("rem_u", make_op!(I64RemU));
    run_test_bin::<i32, _>("shl", make_op!(I32Shl));
    run_test_bin::<i64, _>("shl", make_op!(I64Shl));
    run_test_bin::<i32, _>("shr_s", make_op!(I32ShrS));
    run_test_bin::<i64, _>("shr_s", make_op!(I64ShrS));
    run_test_bin::<i32, _>("shr_u", make_op!(I32ShrU));
    run_test_bin::<i64, _>("shr_u", make_op!(I64ShrU));
    run_test_bin::<i32, _>("rotl", make_op!(I32Rotl));
    run_test_bin::<i64, _>("rotr", make_op!(I64Rotr));
    run_test_bin::<i32, _>("and", make_op!(I32And));
    run_test_bin::<i64, _>("and", make_op!(I64And));
    run_test_bin::<i32, _>("or", make_op!(I32Or));
    run_test_bin::<i64, _>("or", make_op!(I64Or));
    run_test_bin::<i32, _>("xor", make_op!(I32Xor));
    run_test_bin::<i64, _>("xor", make_op!(I64Xor));

    run_test_cmp::<f32, _>("eq", make_op!(F32Eq));
    run_test_cmp::<f64, _>("eq", make_op!(F64Eq));
    run_test_cmp::<f32, _>("ne", make_op!(F32Ne));
    run_test_cmp::<f64, _>("ne", make_op!(F64Ne));

    run_test_bin::<f32, _>("add", make_op!(F32Add));
    run_test_bin::<f64, _>("add", make_op!(F64Add));
    run_test_bin::<f32, _>("sub", make_op!(F32Sub));
    run_test_bin::<f64, _>("sub", make_op!(F64Sub));
    run_test_bin::<f32, _>("mul", make_op!(F32Mul));
    run_test_bin::<f64, _>("mul", make_op!(F64Mul));
    run_test_bin::<f32, _>("div", make_op!(F32Div));
    run_test_bin::<f64, _>("div", make_op!(F64Div));
    run_test_bin::<f32, _>("min", make_op!(F32Min));
    run_test_bin::<f64, _>("min", make_op!(F64Min));
    run_test_bin::<f32, _>("max", make_op!(F32Max));
    run_test_bin::<f64, _>("max", make_op!(F64Max));
    run_test_bin::<f32, _>("copysign", make_op!(F32Copysign));
    run_test_bin::<f64, _>("copysign", make_op!(F64Copysign));
}

/// Tests compilation of all commutative binary Wasm instructions.
///
/// # Note
///
/// This test specializes on cases where one of the inputs is a constant value
/// (e.g. `i32.const 1`) and the other a register input (e.g. `local.get 0`).
/// In this case the `wasmi` compiler unfortunately has to insert an artificial
/// `copy` instruction in between in order to be able to properly represent
/// the underlying instruction. This is due to the fact that due to performance
/// reasons the `lhs` operand of an instruction can only be a register and
/// never an immediate value unlike the right-hand side operand. Fortunately
/// having an immediate value as the left-hand operand is quite uncommon.
///
/// This includes the following Wasm instructions:
///
/// - `{i32, i64, f32, f64}.sub`
/// - `{i32, i64}.div_s`
/// - `{i32, i64}.div_u`
/// - `{i32, i64}.rem_s`
/// - `{i32, i64}.rem_u`
/// - `{i32, i64}.shl`
/// - `{i32, i64}.shr_s`
/// - `{i32, i64}.shr_u`
/// - `{i32, i64}.rotl`
/// - `{i32, i64}.rotr`
/// - `{f32, f64}.div`
/// - `{f32, f64}.copysign`
#[test]
fn binary_const_register() {
    fn test_const_register<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + One + Into<UntypedValue>,
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {output_type})
                    {input_type}.const 1
                    local.get 0
                    {input_type}.{wasm_op}
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let input = engine.alloc_const(T::one());
        let rhs = ExecRegister::from_inner(0);
        let result = ExecRegister::from_inner(1);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let expected = [
            ExecInstruction::CopyImm { result, input },
            make_op(result, result, rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test_const_register::<i32, _>("sub", make_op!(I32Sub));
    test_const_register::<i64, _>("sub", make_op!(I64Sub));
    test_const_register::<i32, _>("div_s", make_op!(I32DivS));
    test_const_register::<i64, _>("div_s", make_op!(I64DivS));
    test_const_register::<i32, _>("div_u", make_op!(I32DivU));
    test_const_register::<i64, _>("div_u", make_op!(I64DivU));
    test_const_register::<i32, _>("rem_s", make_op!(I32RemS));
    test_const_register::<i64, _>("rem_s", make_op!(I64RemS));
    test_const_register::<i32, _>("rem_u", make_op!(I32RemU));
    test_const_register::<i64, _>("rem_u", make_op!(I64RemU));
    test_const_register::<i32, _>("shl", make_op!(I32Shl));
    test_const_register::<i64, _>("shl", make_op!(I64Shl));
    test_const_register::<i32, _>("shr_s", make_op!(I32ShrS));
    test_const_register::<i64, _>("shr_s", make_op!(I64ShrS));
    test_const_register::<i32, _>("shr_u", make_op!(I32ShrU));
    test_const_register::<i64, _>("shr_u", make_op!(I64ShrU));
    test_const_register::<i32, _>("rotl", make_op!(I32Rotl));
    test_const_register::<i64, _>("rotl", make_op!(I64Rotl));
    test_const_register::<i32, _>("rotr", make_op!(I32Rotr));
    test_const_register::<i64, _>("rotr", make_op!(I64Rotr));
    test_const_register::<f32, _>("sub", make_op!(F32Sub));
    test_const_register::<f64, _>("sub", make_op!(F64Sub));
    test_const_register::<f32, _>("div", make_op!(F32Div));
    test_const_register::<f64, _>("div", make_op!(F64Div));
    test_const_register::<f32, _>("copysign", make_op!(F32Copysign));
    test_const_register::<f64, _>("copysign", make_op!(F64Copysign));
}

/// Tests compilation of all commutative binary Wasm instructions.
///
/// # Note
///
/// This test specializes on cases where one of the inputs is a constant value
/// (e.g. `i32.const 1`) and the other a register input (e.g. `local.get 0`).
/// In this case the `wasmi` compiler may swap the order of operands in order
/// to represents the `wasmi` bytecode in a more compact form.
///
/// This includes the following Wasm instructions:
///
/// - `{i32, i64, f32, f64}.eq`
/// - `{i32, i64, f32, f64}.ne`
/// - `{i32, i64, f32, f64}.add`
/// - `{i32, i64, f32, f64}.mul`
/// - `{i32, i64}.and`
/// - `{i32, i64}.or`
/// - `{i32, i64}.xor`
/// - `{f32, f64}.min`
/// - `{f32, f64}.max`
#[test]
fn binary_const_register_commutative() {
    fn test_const_register<T, F, R>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + One + Into<UntypedValue>,
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction,
        R: WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {output_type})
                    {input_type}.const 1
                    local.get 0
                    {input_type}.{wasm_op}
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let rhs = engine.alloc_const(T::one());
        let result = ExecRegister::from_inner(1);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let expected = [
            make_op(
                ExecRegister::from_inner(1),
                ExecRegister::from_inner(0),
                rhs.into(),
            ),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    fn run_test_bin<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + Into<UntypedValue> + WasmTypeName + One,
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction + Copy,
    {
        test_const_register::<T, F, T>(wasm_op, make_op);
    }

    fn run_test_cmp<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + Into<UntypedValue> + WasmTypeName + One,
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction + Copy,
    {
        test_const_register::<T, F, bool>(wasm_op, make_op);
    }

    run_test_cmp::<i32, _>("eq", make_op!(I32Eq));
    run_test_cmp::<i64, _>("eq", make_op!(I64Eq));
    run_test_cmp::<i32, _>("ne", make_op!(I32Ne));
    run_test_cmp::<i64, _>("ne", make_op!(I64Ne));

    run_test_bin::<i32, _>("add", make_op!(I32Add));
    run_test_bin::<i64, _>("add", make_op!(I64Add));
    run_test_bin::<i32, _>("mul", make_op!(I32Mul));
    run_test_bin::<i64, _>("mul", make_op!(I64Mul));
    run_test_bin::<i32, _>("and", make_op!(I32And));
    run_test_bin::<i64, _>("and", make_op!(I64And));
    run_test_bin::<i32, _>("or", make_op!(I32Or));
    run_test_bin::<i64, _>("or", make_op!(I64Or));
    run_test_bin::<i32, _>("xor", make_op!(I32Xor));
    run_test_bin::<i64, _>("xor", make_op!(I64Xor));

    run_test_cmp::<f32, _>("eq", make_op!(F32Eq));
    run_test_cmp::<f64, _>("eq", make_op!(F64Eq));
    run_test_cmp::<f32, _>("ne", make_op!(F32Ne));
    run_test_cmp::<f64, _>("ne", make_op!(F64Ne));

    run_test_bin::<f32, _>("add", make_op!(F32Add));
    run_test_bin::<f64, _>("add", make_op!(F64Add));
    run_test_bin::<f32, _>("mul", make_op!(F32Mul));
    run_test_bin::<f64, _>("mul", make_op!(F64Mul));
    run_test_bin::<f32, _>("min", make_op!(F32Min));
    run_test_bin::<f32, _>("min", make_op!(F32Min));
    run_test_bin::<f64, _>("max", make_op!(F64Max));
    run_test_bin::<f64, _>("max", make_op!(F64Max));
}

/// The expected outcome of a fallible constant evaluation.
#[derive(Debug, Copy, Clone)]
pub enum Outcome {
    /// The instruction evaluation resulted in a proper value.
    Eval,
    /// The instruction evaluation resulted in a trap.
    Trap,
}

/// Tests compilation of all fallible binary Wasm instructions.
///
/// # Note
///
/// This test specializes on cases where both inputs are constant values.
/// In this case the `wasmi` compiler will directly evaluate the results.
///
/// This includes the following Wasm instructions:
///
/// - `{i32, i64}.div_s`
/// - `{i32, i64}.div_u`
/// - `{i32, i64}.rem_s`
/// - `{i32, i64}.rem_u`
#[test]
fn binary_const_const_fallible() {
    fn test_const_const<T, E>(wasm_op: &str, outcome: Outcome, lhs: T, rhs: T, exec_op: E)
    where
        T: Display + WasmTypeName + Into<UntypedValue>,
        E: FnOnce(T, T) -> Result<T, TrapCode>,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (result {output_type})
                    {input_type}.const {lhs}
                    {input_type}.const {rhs}
                    {input_type}.{wasm_op}
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let expected = match exec_op(lhs, rhs) {
            Ok(result) => {
                assert!(matches!(outcome, Outcome::Eval));
                let result = engine.alloc_const(result.into());
                let results = engine.alloc_provider_slice([ExecProvider::from(result)]);
                [ExecInstruction::Return { results }]
            }
            Err(trap_code) => {
                assert!(matches!(outcome, Outcome::Trap));
                [ExecInstruction::Trap { trap_code }]
            }
        };
        assert_func_bodies(&wasm, [expected]);
    }

    test_const_const::<i32, _>("div_s", Outcome::Eval, 1, 2, |lhs, rhs| lhs.div(rhs));
    test_const_const::<i32, _>("div_s", Outcome::Trap, 1, 0, |lhs, rhs| lhs.div(rhs));
    test_const_const::<i64, _>("div_s", Outcome::Eval, 1, 2, |lhs, rhs| lhs.div(rhs));
    test_const_const::<i64, _>("div_s", Outcome::Trap, 1, 0, |lhs, rhs| lhs.div(rhs));

    test_const_const::<u32, _>("div_u", Outcome::Eval, 1, 2, |lhs, rhs| lhs.div(rhs));
    test_const_const::<u32, _>("div_u", Outcome::Trap, 1, 0, |lhs, rhs| lhs.div(rhs));
    test_const_const::<u64, _>("div_u", Outcome::Eval, 1, 2, |lhs, rhs| lhs.div(rhs));
    test_const_const::<u64, _>("div_u", Outcome::Trap, 1, 0, |lhs, rhs| lhs.div(rhs));

    test_const_const::<i32, _>("rem_s", Outcome::Eval, 1, 2, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<i32, _>("rem_s", Outcome::Trap, 1, 0, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<i64, _>("rem_s", Outcome::Eval, 1, 2, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<i64, _>("rem_s", Outcome::Trap, 1, 0, |lhs, rhs| lhs.rem(rhs));

    test_const_const::<u32, _>("rem_u", Outcome::Eval, 1, 2, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<u32, _>("rem_u", Outcome::Trap, 1, 0, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<u64, _>("rem_u", Outcome::Eval, 1, 2, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<u64, _>("rem_u", Outcome::Trap, 1, 0, |lhs, rhs| lhs.rem(rhs));
}

/// Tests compilation of all infallible binary Wasm instructions.
///
/// # Note
///
/// This test specializes on cases where both inputs are constant values.
/// In this case the `wasmi` compiler will directly evaluate the results.
///
/// This includes the following Wasm instructions:
///
/// - `{i32, i64, f32, f64}.eq`
/// - `{i32, i64, f32, f64}.ne`
/// - `{i32, i64, f32, f64}.add`
/// - `{i32, i64, f32, f64}.sub`
/// - `{i32, i64, f32, f64}.mul`
/// - `{i32, i64}.shl`
/// - `{i32, i64}.shr_s`
/// - `{i32, i64}.shr_u`
/// - `{i32, i64}.rotl`
/// - `{i32, i64}.rotr`
/// - `{i32, i64}.and`
/// - `{i32, i64}.or`
/// - `{i32, i64}.xor`
/// - `{f32, f64}.min`
/// - `{f32, f64}.max`
/// - `{i32, i64}.copysign`
#[test]
fn binary_const_const_infallible() {
    fn run_test<T, E, R>(wasm_op: &str, lhs: T, rhs: T, exec_op: E)
    where
        T: Display + WasmTypeName,
        E: FnOnce(T, T) -> R,
        R: Into<UntypedValue> + WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (result {output_type})
                    {input_type}.const {lhs}
                    {input_type}.const {rhs}
                    {input_type}.{wasm_op}
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = engine.alloc_const(exec_op(lhs, rhs).into());
        let results = engine.alloc_provider_slice([ExecProvider::from(result)]);
        let expected = [ExecInstruction::Return { results }];
        assert_func_bodies(&wasm, [expected]);
    }

    fn run_test_bin<T, E>(wasm_op: &str, lhs: T, rhs: T, exec_op: E)
    where
        T: Display + Into<UntypedValue> + WasmTypeName,
        E: FnOnce(T, T) -> T,
    {
        run_test::<T, E, T>(wasm_op, lhs, rhs, exec_op)
    }

    fn run_test_cmp<T, E>(wasm_op: &str, lhs: T, rhs: T, exec_op: E)
    where
        T: Display + WasmTypeName,
        E: FnOnce(T, T) -> bool,
    {
        run_test::<T, E, bool>(wasm_op, lhs, rhs, exec_op)
    }

    run_test_cmp::<i32, _>("eq", 1, 2, |lhs, rhs| lhs == rhs);
    run_test_cmp::<i64, _>("eq", 1, 2, |lhs, rhs| lhs == rhs);
    run_test_cmp::<i32, _>("ne", 1, 2, |lhs, rhs| lhs != rhs);
    run_test_cmp::<i64, _>("ne", 1, 2, |lhs, rhs| lhs != rhs);

    run_test_bin::<i32, _>("add", 1, 2, |lhs, rhs| lhs.wrapping_add(rhs));
    run_test_bin::<i64, _>("add", 1, 2, |lhs, rhs| lhs.wrapping_add(rhs));
    run_test_bin::<i32, _>("sub", 1, 2, |lhs, rhs| lhs.wrapping_sub(rhs));
    run_test_bin::<i64, _>("sub", 1, 2, |lhs, rhs| lhs.wrapping_sub(rhs));
    run_test_bin::<i32, _>("mul", 1, 2, |lhs, rhs| lhs.wrapping_mul(rhs));
    run_test_bin::<i64, _>("mul", 1, 2, |lhs, rhs| lhs.wrapping_mul(rhs));
    run_test_bin::<i32, _>("shl", 1, 2, |lhs, rhs| lhs.shl(rhs & 0x1F));
    run_test_bin::<i64, _>("shl", 1, 2, |lhs, rhs| lhs.shl(rhs & 0x3F));
    run_test_bin::<i32, _>("shr_s", 1, 2, |lhs, rhs| lhs.shr(rhs & 0x1F));
    run_test_bin::<i64, _>("shr_s", 1, 2, |lhs, rhs| lhs.shr(rhs & 0x3F));
    run_test_bin::<u32, _>("shr_u", 1, 2, |lhs, rhs| lhs.shr(rhs & 0x1F));
    run_test_bin::<u64, _>("shr_u", 1, 2, |lhs, rhs| lhs.shr(rhs & 0x3F));
    run_test_bin::<i32, _>("rotl", 1, 2, |lhs, rhs| lhs.rotl(rhs));
    run_test_bin::<i64, _>("rotl", 1, 2, |lhs, rhs| lhs.rotl(rhs));
    run_test_bin::<i32, _>("rotr", 1, 2, |lhs, rhs| lhs.rotr(rhs));
    run_test_bin::<i64, _>("rotr", 1, 2, |lhs, rhs| lhs.rotr(rhs));
    run_test_bin::<i32, _>("and", 1, 2, |lhs, rhs| lhs & rhs);
    run_test_bin::<i64, _>("and", 1, 2, |lhs, rhs| lhs & rhs);
    run_test_bin::<i32, _>("or", 1, 2, |lhs, rhs| lhs | rhs);
    run_test_bin::<i64, _>("or", 1, 2, |lhs, rhs| lhs | rhs);
    run_test_bin::<i32, _>("xor", 1, 2, |lhs, rhs| lhs ^ rhs);
    run_test_bin::<i64, _>("xor", 1, 2, |lhs, rhs| lhs ^ rhs);

    run_test_cmp::<f32, _>("eq", 1.0, 2.0, |lhs, rhs| F32::from(lhs) == F32::from(rhs));
    run_test_cmp::<f64, _>("eq", 1.0, 2.0, |lhs, rhs| F64::from(lhs) == F64::from(rhs));
    run_test_cmp::<f32, _>("ne", 1.0, 2.0, |lhs, rhs| F32::from(lhs) != F32::from(rhs));
    run_test_cmp::<f64, _>("ne", 1.0, 2.0, |lhs, rhs| F64::from(lhs) != F64::from(rhs));

    run_test_bin::<f32, _>("add", 1.0, 2.0, |lhs, rhs| {
        (F32::from(lhs) + F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("add", 1.0, 2.0, |lhs, rhs| {
        (F64::from(lhs) + F64::from(rhs)).into()
    });
    run_test_bin::<f32, _>("sub", 1.0, 2.0, |lhs, rhs| {
        (F32::from(lhs) - F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("sub", 1.0, 2.0, |lhs, rhs| {
        (F64::from(lhs) - F64::from(rhs)).into()
    });
    run_test_bin::<f32, _>("mul", 1.0, 2.0, |lhs, rhs| {
        (F32::from(lhs) * F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("mul", 1.0, 2.0, |lhs, rhs| {
        (F64::from(lhs) * F64::from(rhs)).into()
    });
    run_test_bin::<f32, _>("div", 1.0, 2.0, |lhs, rhs| {
        (F32::from(lhs) / F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("div", 1.0, 2.0, |lhs, rhs| {
        (F64::from(lhs) / F64::from(rhs)).into()
    });
    run_test_bin::<f32, _>("min", 1.0, 2.0, |lhs, rhs| {
        F32::from(lhs).min(F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("min", 1.0, 2.0, |lhs, rhs| {
        F64::from(lhs).min(F64::from(rhs)).into()
    });
    run_test_bin::<f32, _>("max", 1.0, 2.0, |lhs, rhs| {
        F32::from(lhs).max(F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("max", 1.0, 2.0, |lhs, rhs| {
        F64::from(lhs).max(F64::from(rhs)).into()
    });
    run_test_bin::<f32, _>("copysign", 1.0, 2.0, |lhs, rhs| {
        F32::from(lhs).copysign(F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("copysign", 1.0, 2.0, |lhs, rhs| {
        F64::from(lhs).copysign(F64::from(rhs)).into()
    });
}

/// Tests translation of Wasm `{i32,i64}.eqz` functions.
///
/// # Note
///
/// This tests asserts correct compilation of register inputs.
#[test]
fn cmp_zero_register() {
    fn run_test<T, F>(ty: &str, make_op: F)
    where
        T: Default + Into<UntypedValue>,
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {ty}) (result i32)
                    local.get 0
                    {ty}.eqz
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let rhs = engine.alloc_const(T::default());
        let result = ExecRegister::from_inner(1);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let expected = [
            make_op(result, ExecRegister::from_inner(0), rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies(&wasm, [expected]);
    }

    run_test::<i32, _>("i32", make_op!(I32Eq));
    run_test::<i64, _>("i64", make_op!(I64Eq));
}

/// Tests translation of Wasm `{i32,i64}.eqz` functions.
///
/// # Note
///
/// This tests asserts compile time evaluation of constant value inputs.
#[test]
fn cmp_zero_const() {
    fn run_test<T, F>(ty: &str, value: T, exec_op: F)
    where
        T: Default + Display + Into<UntypedValue>,
        F: FnOnce(T) -> bool,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (result i32)
                    {ty}.const {value}
                    {ty}.eqz
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = engine.alloc_const(exec_op(value));
        let results = engine.alloc_provider_slice([ExecProvider::from(result)]);
        let expected = [ExecInstruction::Return { results }];
        assert_func_bodies(&wasm, [expected]);
    }

    run_test("i32", 1, |input: i32| input == 0);
    run_test("i64", 1, |input: i64| input == 0);
}

/// Tests translation of all Wasm comparison functions.
///
/// # Note
///
/// In this test all Wasm functions have 2 registers (`local.get`) as inputs.
/// This is one of the simple cases to cover.
#[test]
fn cmp_registers() {
    fn run_test<F>(ty: &str, wasm_op: &str, make_op: F)
    where
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {ty}) (param {ty}) (result i32)
                    local.get 0
                    local.get 1
                    {ty}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = ExecRegister::from_inner(2);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let expected = [
            make_op(
                result,
                ExecRegister::from_inner(0),
                ExecRegister::from_inner(1).into(),
            ),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies(&wasm, [expected]);
    }
    run_test("i32", "lt_s", make_op!(I32LtS));
    run_test("i32", "lt_u", make_op!(I32LtU));
    run_test("i32", "gt_s", make_op!(I32GtS));
    run_test("i32", "gt_u", make_op!(I32GtU));
    run_test("i64", "lt_s", make_op!(I64LtS));
    run_test("i64", "lt_u", make_op!(I64LtU));
    run_test("i64", "gt_s", make_op!(I64GtS));
    run_test("i64", "gt_u", make_op!(I64GtU));

    run_test("f32", "lt", make_op!(F32Lt));
    run_test("f32", "le", make_op!(F32Le));
    run_test("f32", "gt", make_op!(F32Gt));
    run_test("f32", "ge", make_op!(F32Ge));

    run_test("f64", "lt", make_op!(F64Lt));
    run_test("f64", "le", make_op!(F64Le));
    run_test("f64", "gt", make_op!(F64Gt));
    run_test("f64", "ge", make_op!(F64Ge));
}

/// Tests translation of all Wasm comparison functions.
///
/// # Note
///
/// In this test all Wasm functions have 1 register (`local.get`)
/// and a constant value (`i32.const`) as inputs.
///
/// This is one of the simple cases to cover.
#[test]
fn cmp_register_and_const() {
    fn run_test<T, F>(ty: &str, wasm_op: &str, value: T, make_op: F)
    where
        T: Display + Into<UntypedValue>,
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {ty}) (result i32)
                    local.get 0
                    {ty}.const {value}
                    {ty}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = ExecRegister::from_inner(1);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let rhs = engine.alloc_const(value);
        let expected = [
            make_op(result, ExecRegister::from_inner(0), rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies(&wasm, [expected]);
    }
    run_test("i32", "lt_s", 1_i32, make_op!(I32LtS));
    run_test("i32", "lt_u", 1_i32, make_op!(I32LtU));
    run_test("i32", "gt_s", 1_i32, make_op!(I32GtS));
    run_test("i32", "gt_u", 1_i32, make_op!(I32GtU));
    run_test("i64", "lt_s", 1_i32, make_op!(I64LtS));
    run_test("i64", "lt_u", 1_i32, make_op!(I64LtU));
    run_test("i64", "gt_s", 1_i32, make_op!(I64GtS));
    run_test("i64", "gt_u", 1_i32, make_op!(I64GtU));

    run_test("f32", "lt", 1.0_f32, make_op!(F32Lt));
    run_test("f32", "le", 1.0_f32, make_op!(F32Le));
    run_test("f32", "gt", 1.0_f32, make_op!(F32Gt));
    run_test("f32", "ge", 1.0_f32, make_op!(F32Ge));

    run_test("f64", "lt", 1.0_f64, make_op!(F64Lt));
    run_test("f64", "le", 1.0_f64, make_op!(F64Le));
    run_test("f64", "gt", 1.0_f64, make_op!(F64Gt));
    run_test("f64", "ge", 1.0_f64, make_op!(F64Ge));
}

/// Tests translation of all Wasm comparison functions.
///
/// # Note
///
/// In this test all Wasm functions have 1 register (`local.get`)
/// and a constant value (`i32.const`) as inputs.
///
/// This is generally non-trivial to handle but comparison functions
/// make it easy to swap the operands by switching to the reversed comparison
/// instruction, e.g. switching from `less-than` to `greater-than`.
#[test]
fn cmp_const_and_register() {
    fn run_test<T, F>(ty: &str, wasm_op: &str, value: T, make_op: F)
    where
        T: Display + Into<UntypedValue>,
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {ty}) (result i32)
                    {ty}.const {value}
                    local.get 0
                    {ty}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = ExecRegister::from_inner(1);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let rhs = engine.alloc_const(value);
        let expected = [
            make_op(result, ExecRegister::from_inner(0), rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies(&wasm, [expected]);
    }
    run_test("i32", "lt_s", 1_i32, make_op!(I32GtS));
    run_test("i32", "lt_u", 1_i32, make_op!(I32GtU));
    run_test("i32", "gt_s", 1_i32, make_op!(I32LtS));
    run_test("i32", "gt_u", 1_i32, make_op!(I32LtU));
    run_test("i64", "lt_s", 1_i32, make_op!(I64GtS));
    run_test("i64", "lt_u", 1_i32, make_op!(I64GtU));
    run_test("i64", "gt_s", 1_i32, make_op!(I64LtS));
    run_test("i64", "gt_u", 1_i32, make_op!(I64LtU));

    run_test("f32", "lt", 1.0_f32, make_op!(F32Gt));
    run_test("f32", "le", 1.0_f32, make_op!(F32Ge));
    run_test("f32", "gt", 1.0_f32, make_op!(F32Lt));
    run_test("f32", "ge", 1.0_f32, make_op!(F32Le));

    run_test("f64", "lt", 1.0_f64, make_op!(F64Gt));
    run_test("f64", "le", 1.0_f64, make_op!(F64Ge));
    run_test("f64", "gt", 1.0_f64, make_op!(F64Lt));
    run_test("f64", "ge", 1.0_f64, make_op!(F64Le));
}

/// Tests translation of all Wasm comparison functions.
///
/// # Note
///
/// In this test all Wasm functions have 2 constant values (`i32.const`) as inputs.
///
/// In this case we can simply apply const folding to resolve the instruction
/// entirely.
#[test]
fn cmp_const_and_const() {
    fn run_test<T, E>(ty: &str, wasm_op: &str, lhs: T, rhs: T, exec_op: E)
    where
        T: Display + Into<UntypedValue> + PartialOrd,
        E: FnOnce(T, T) -> bool,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {ty}) (result i32)
                    {ty}.const {lhs}
                    {ty}.const {rhs}
                    {ty}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = engine.alloc_const(exec_op(lhs, rhs) as i32);
        let results = engine.alloc_provider_slice([ExecProvider::from(result)]);
        let expected = [ExecInstruction::Return { results }];
        assert_func_bodies(&wasm, [expected]);
    }
    run_test("i32", "lt_s", 1_i32, 2_i32, |l, r| l < r);
    run_test("i32", "lt_u", 1_i32, 2_i32, |l, r| l < r);
    run_test("i32", "gt_s", 1_i32, 2_i32, |l, r| l > r);
    run_test("i32", "gt_u", 1_i32, 2_i32, |l, r| l > r);
    run_test("i64", "lt_s", 1_i64, 2_i64, |l, r| l < r);
    run_test("i64", "lt_u", 1_i64, 2_i64, |l, r| l < r);
    run_test("i64", "gt_s", 1_i64, 2_i64, |l, r| l > r);
    run_test("i64", "gt_u", 1_i64, 2_i64, |l, r| l > r);

    run_test("f32", "lt", 1.0_f32, 2.0_f32, |l, r| l < r);
    run_test("f32", "le", 1.0_f32, 2.0_f32, |l, r| l <= r);
    run_test("f32", "gt", 1.0_f32, 2.0_f32, |l, r| l > r);
    run_test("f32", "ge", 1.0_f32, 2.0_f32, |l, r| l >= r);

    run_test("f64", "lt", 1.0_f64, 2.0_f64, |l, r| l < r);
    run_test("f64", "le", 1.0_f64, 2.0_f64, |l, r| l <= r);
    run_test("f64", "gt", 1.0_f64, 2.0_f64, |l, r| l > r);
    run_test("f64", "ge", 1.0_f64, 2.0_f64, |l, r| l >= r);
}

/// Tests translation of all unary Wasm instructions.
///
/// # Note
///
/// In this test all Wasm functions have a register input (e.g. via `local.get`).
///
/// This tests the following Wasm instructions:
///
/// - `{i32, i64}.clz`
/// - `{i32, i64}.ctz`
/// - `{i32, i64}.popcnt`
/// - `{i32, i64}.extend_8s`
/// - `{i32, i64}.extend_16s`
/// - `i64.extend_32s`
/// - `{f32, f64}.abs`
/// - `{f32, f64}.neg`
/// - `{f32, f64}.ceil`
/// - `{f32, f64}.floor`
/// - `{f32, f64}.trunc`
/// - `{f32, f64}.nearest`
/// - `{f32, f64}.sqrt`
#[test]
fn unary_register() {
    fn test<T, R, F>(wasm_op: &str, make_op: F)
    where
        T: WasmTypeName,
        R: WasmTypeName,
        F: FnOnce(ExecRegister, ExecRegister) -> ExecInstruction,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let result_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {result_type})
                    local.get 0
                    {result_type}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = ExecRegister::from_inner(1);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let input = ExecRegister::from_inner(0);
        let expected = [make_op(result, input), ExecInstruction::Return { results }];
        assert_func_bodies(&wasm, [expected]);
    }

    fn test_unary<T, F>(wasm_op: &str, make_op: F)
    where
        T: WasmTypeName,
        F: FnOnce(ExecRegister, ExecRegister) -> ExecInstruction,
    {
        test::<T, T, F>(wasm_op, make_op)
    }

    test_unary::<i32, _>("clz", make_op2!(I32Clz));
    test_unary::<i64, _>("clz", make_op2!(I64Clz));
    test_unary::<i32, _>("ctz", make_op2!(I32Ctz));
    test_unary::<i64, _>("ctz", make_op2!(I64Ctz));
    test_unary::<i32, _>("popcnt", make_op2!(I32Popcnt));
    test_unary::<i64, _>("popcnt", make_op2!(I64Popcnt));
    test_unary::<i32, _>("extend8_s", make_op2!(I32Extend8S));
    test_unary::<i64, _>("extend8_s", make_op2!(I64Extend8S));
    test_unary::<i32, _>("extend16_s", make_op2!(I32Extend16S));
    test_unary::<i64, _>("extend16_s", make_op2!(I64Extend16S));
    test_unary::<i64, _>("extend32_s", make_op2!(I64Extend32S));
    test_unary::<f32, _>("abs", make_op2!(F32Abs));
    test_unary::<f64, _>("abs", make_op2!(F64Abs));
    test_unary::<f32, _>("neg", make_op2!(F32Neg));
    test_unary::<f64, _>("neg", make_op2!(F64Neg));
    test_unary::<f32, _>("ceil", make_op2!(F32Ceil));
    test_unary::<f64, _>("ceil", make_op2!(F64Ceil));
    test_unary::<f32, _>("floor", make_op2!(F32Floor));
    test_unary::<f64, _>("floor", make_op2!(F64Floor));
    test_unary::<f32, _>("trunc", make_op2!(F32Trunc));
    test_unary::<f64, _>("trunc", make_op2!(F64Trunc));
    test_unary::<f32, _>("nearest", make_op2!(F32Nearest));
    test_unary::<f64, _>("nearest", make_op2!(F64Nearest));
    test_unary::<f32, _>("sqrt", make_op2!(F32Sqrt));
    test_unary::<f64, _>("sqrt", make_op2!(F64Sqrt));

    test::<i64, i32, _>("wrap_i64", make_op2!(I32WrapI64));
    test::<F32, i32, _>("trunc_f32_s", make_op2!(I32TruncSF32));
    test::<F32, u32, _>("trunc_f32_u", make_op2!(I32TruncUF32));
    test::<F64, i32, _>("trunc_f64_s", make_op2!(I32TruncSF64));
    test::<F64, u32, _>("trunc_f64_u", make_op2!(I32TruncUF64));
    test::<i32, i64, _>("extend_i32_s", make_op2!(I64ExtendSI32));
    test::<u32, i64, _>("extend_i32_u", make_op2!(I64ExtendUI32));
    test::<F32, i64, _>("trunc_f32_s", make_op2!(I64TruncSF32));
    test::<F32, u64, _>("trunc_f32_u", make_op2!(I64TruncUF32));
    test::<F64, i64, _>("trunc_f64_s", make_op2!(I64TruncSF64));
    test::<F64, u64, _>("trunc_f64_u", make_op2!(I64TruncUF64));
    test::<i32, F32, _>("convert_i32_s", make_op2!(F32ConvertSI32));
    test::<u32, F32, _>("convert_i32_u", make_op2!(F32ConvertUI32));
    test::<i64, F32, _>("convert_i64_s", make_op2!(F32ConvertSI64));
    test::<u64, F32, _>("convert_i64_u", make_op2!(F32ConvertUI64));
    test::<F64, F32, _>("demote_f64", make_op2!(F32DemoteF64));
    test::<i32, F64, _>("convert_i32_s", make_op2!(F64ConvertSI32));
    test::<u32, F64, _>("convert_i32_u", make_op2!(F64ConvertUI32));
    test::<i64, F64, _>("convert_i64_s", make_op2!(F64ConvertSI64));
    test::<u64, F64, _>("convert_i64_u", make_op2!(F64ConvertUI64));
    test::<F32, F64, _>("promote_f32", make_op2!(F64PromoteF32));
    test::<F32, i32, _>("trunc_sat_f32_s", make_op2!(I32TruncSatF32S));
    test::<F32, u32, _>("trunc_sat_f32_u", make_op2!(I32TruncSatF32U));
    test::<F64, i32, _>("trunc_sat_f64_s", make_op2!(I32TruncSatF64S));
    test::<F64, u32, _>("trunc_sat_f64_u", make_op2!(I32TruncSatF64U));
    test::<F32, i64, _>("trunc_sat_f32_s", make_op2!(I64TruncSatF32S));
    test::<F32, u64, _>("trunc_sat_f32_u", make_op2!(I64TruncSatF32U));
    test::<F64, i64, _>("trunc_sat_f64_s", make_op2!(I64TruncSatF64S));
    test::<F64, u64, _>("trunc_sat_f64_u", make_op2!(I64TruncSatF64U));
}

/// Tests translation of all unary Wasm instructions.
///
/// # Note
///
/// In this test all Wasm functions have a constant input (e.g. via `i32.const`).
///
/// This tests the following unary Wasm instructions:
///
/// - `{i32, i64}.clz`
/// - `{i32, i64}.ctz`
/// - `{i32, i64}.popcnt`
/// - `{i32, i64}.extend_8s`
/// - `{i32, i64}.extend_16s`
/// - `i64.extend_32s`
/// - `{f32, f64}.abs`
/// - `{f32, f64}.neg`
/// - `{f32, f64}.ceil`
/// - `{f32, f64}.floor`
/// - `{f32, f64}.trunc`
/// - `{f32, f64}.nearest`
/// - `{f32, f64}.sqrt`
///
/// And also this tests the following Wasm conversion instructions:
///
/// - `i32.wrap_i64`
/// - `i64.extend_i32_s`
/// - `i64.extend_i32_u`
/// - `{f32, f64}.convert_i32_s`
/// - `{f32, f64}.convert_i32_u`
/// - `{f32, f64}.convert_i64_s`
/// - `{f32, f64}.convert_i64_u`
/// - `f32.demote_f64`
/// - `f64.promote_f32`
/// - `{i32, i64}.trunc_sat_f32_s`
/// - `{i32, i64}.trunc_sat_f32_u`
/// - `{i32, i64}.trunc_sat_f64_s`
/// - `{i32, i64}.trunc_sat_f64_u`
#[test]
fn unary_const_infallible() {
    fn test<T, R, F>(wasm_op: &str, input: T, exec_op: F)
    where
        T: Display + WasmTypeName + Into<UntypedValue>,
        F: FnOnce(T) -> R,
        R: Into<UntypedValue> + WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let result_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {result_type})
                    {input_type}.const {input}
                    {result_type}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = engine.alloc_const(exec_op(input));
        let results = engine.alloc_provider_slice([ExecProvider::from_immediate(result)]);
        let expected = [ExecInstruction::Return { results }];
        assert_func_bodies(&wasm, [expected]);
    }

    fn test_unary<T, F>(wasm_op: &str, input: T, exec_op: F)
    where
        T: Display + WasmTypeName + Into<UntypedValue>,
        F: FnOnce(T) -> T,
    {
        test::<T, T, F>(wasm_op, input, exec_op)
    }

    test_unary("clz", 1, <i32 as Integer<i32>>::leading_zeros);
    test_unary("clz", 1, <i64 as Integer<i64>>::leading_zeros);
    test_unary("ctz", 1, <i32 as Integer<i32>>::trailing_zeros);
    test_unary("ctz", 1, <i64 as Integer<i64>>::trailing_zeros);
    test_unary("popcnt", 1, <i32 as Integer<i32>>::count_ones);
    test_unary("popcnt", 1, <i64 as Integer<i64>>::count_ones);
    test_unary(
        "extend8_s",
        1,
        <i32 as SignExtendFrom<i8>>::sign_extend_from,
    );
    test_unary(
        "extend16_s",
        1,
        <i32 as SignExtendFrom<i16>>::sign_extend_from,
    );
    test_unary(
        "extend8_s",
        1,
        <i64 as SignExtendFrom<i8>>::sign_extend_from,
    );
    test_unary(
        "extend16_s",
        1,
        <i64 as SignExtendFrom<i16>>::sign_extend_from,
    );
    test_unary(
        "extend16_s",
        1,
        <i64 as SignExtendFrom<i32>>::sign_extend_from,
    );
    test("abs", 1.0, |input| F32::from(input).abs());
    test("abs", 1.0, |input| F64::from(input).abs());
    test("neg", 1.0, |input| -F32::from(input));
    test("neg", 1.0, |input| -F64::from(input));
    test("ceil", 1.0, |input| F32::from(input).ceil());
    test("ceil", 1.0, |input| F64::from(input).ceil());
    test("floor", 1.0, |input| F32::from(input).floor());
    test("floor", 1.0, |input| F64::from(input).floor());
    test("trunc", 1.0, |input| F32::from(input).trunc());
    test("trunc", 1.0, |input| F64::from(input).trunc());
    test("nearest", 1.0, |input| F32::from(input).nearest());
    test("nearest", 1.0, |input| F64::from(input).nearest());
    test("sqrt", 1.0, |input| F32::from(input).sqrt());
    test("sqrt", 1.0, |input| F64::from(input).sqrt());

    test::<i64, i32, _>("wrap_i64", 1, <i64 as WrapInto<i32>>::wrap_into);

    test::<i32, i64, _>("extend_i32_s", 1, <i32 as ExtendInto<i64>>::extend_into);
    test::<u32, i64, _>("extend_i32_u", 1, <u32 as ExtendInto<i64>>::extend_into);

    test::<i32, F32, _>("convert_i32_s", 1, <i32 as ExtendInto<F32>>::extend_into);
    test::<u32, F32, _>("convert_i32_u", 1, <u32 as ExtendInto<F32>>::extend_into);
    test::<i64, F32, _>("convert_i64_s", 1, <i64 as WrapInto<F32>>::wrap_into);
    test::<u64, F32, _>("convert_i64_u", 1, <u64 as WrapInto<F32>>::wrap_into);
    test::<f64, f32, _>("demote_f64", 1.0, |input| input.wrap_into());
    test::<i32, F64, _>("convert_i32_s", 1, <i32 as ExtendInto<F64>>::extend_into);
    test::<u32, F64, _>("convert_i32_u", 1, <u32 as ExtendInto<F64>>::extend_into);
    test::<i64, F64, _>("convert_i64_s", 1, <i64 as ExtendInto<F64>>::extend_into);
    test::<u64, F64, _>("convert_i64_u", 1, <u64 as ExtendInto<F64>>::extend_into);
    test::<f32, f64, _>("promote_f32", 1.0, |input| input.extend_into());
    test::<f32, i32, _>(
        "trunc_sat_f32_s",
        1.0,
        <f32 as TruncateSaturateInto<i32>>::truncate_saturate_into,
    );
    test::<f32, u32, _>(
        "trunc_sat_f32_u",
        1.0,
        <f32 as TruncateSaturateInto<u32>>::truncate_saturate_into,
    );
    test::<f64, i32, _>(
        "trunc_sat_f64_s",
        1.0,
        <f64 as TruncateSaturateInto<i32>>::truncate_saturate_into,
    );
    test::<f64, u32, _>(
        "trunc_sat_f64_u",
        1.0,
        <f64 as TruncateSaturateInto<u32>>::truncate_saturate_into,
    );
    test::<f32, i64, _>(
        "trunc_sat_f32_s",
        1.0,
        <f32 as TruncateSaturateInto<i64>>::truncate_saturate_into,
    );
    test::<f32, u64, _>(
        "trunc_sat_f32_u",
        1.0,
        <f32 as TruncateSaturateInto<u64>>::truncate_saturate_into,
    );
    test::<f64, i64, _>(
        "trunc_sat_f64_s",
        1.0,
        <f64 as TruncateSaturateInto<i64>>::truncate_saturate_into,
    );
    test::<f64, u64, _>(
        "trunc_sat_f64_u",
        1.0,
        <f64 as TruncateSaturateInto<u64>>::truncate_saturate_into,
    );
}

#[test]
fn unary_const_fallible() {
    fn test<T, R, F>(wasm_op: &str, outcome: Outcome, input: T, exec_op: F)
    where
        T: Display + WasmTypeName + Into<UntypedValue>,
        F: FnOnce(T) -> Result<R, TrapCode>,
        R: Into<UntypedValue> + WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let result_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {result_type})
                    {input_type}.const {input}
                    {result_type}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let expected = match exec_op(input) {
            Ok(result) => {
                assert!(matches!(outcome, Outcome::Eval));
                let result = engine.alloc_const(result);
                let results = engine.alloc_provider_slice([ExecProvider::from_immediate(result)]);
                [ExecInstruction::Return { results }]
            }
            Err(trap_code) => {
                assert!(matches!(outcome, Outcome::Trap));
                [ExecInstruction::Trap { trap_code }]
            }
        };
        assert_func_bodies(&wasm, [expected]);
    }

    fn test_f32<R, F>(wasm_op: &str, outcome: Outcome, input: f32, exec_op: F)
    where
        F: FnOnce(F32) -> Result<R, TrapCode>,
        R: Into<UntypedValue> + WasmTypeName,
    {
        test::<f32, R, _>(wasm_op, outcome, input, |input| exec_op(F32::from(input)))
    }

    fn test_f64<R, F>(wasm_op: &str, outcome: Outcome, input: f64, exec_op: F)
    where
        F: FnOnce(F64) -> Result<R, TrapCode>,
        R: Into<UntypedValue> + WasmTypeName,
    {
        test::<f64, R, _>(wasm_op, outcome, input, |input| exec_op(F64::from(input)))
    }

    test_f32::<i32, _>(
        "trunc_f32_s",
        Outcome::Eval,
        1.0,
        <F32 as TryTruncateInto<i32, TrapCode>>::try_truncate_into,
    );
    test_f32::<u32, _>(
        "trunc_f32_u",
        Outcome::Eval,
        1.0,
        <F32 as TryTruncateInto<u32, TrapCode>>::try_truncate_into,
    );
    test_f64::<i32, _>(
        "trunc_f64_s",
        Outcome::Eval,
        1.0,
        <F64 as TryTruncateInto<i32, TrapCode>>::try_truncate_into,
    );
    test_f64::<u32, _>(
        "trunc_f64_u",
        Outcome::Eval,
        1.0,
        <F64 as TryTruncateInto<u32, TrapCode>>::try_truncate_into,
    );
    test_f32::<i64, _>(
        "trunc_f32_s",
        Outcome::Eval,
        1.0,
        <F32 as TryTruncateInto<i64, TrapCode>>::try_truncate_into,
    );
    test_f32::<u64, _>(
        "trunc_f32_u",
        Outcome::Eval,
        1.0,
        <F32 as TryTruncateInto<u64, TrapCode>>::try_truncate_into,
    );
    test_f64::<i64, _>(
        "trunc_f64_s",
        Outcome::Eval,
        1.0,
        <F64 as TryTruncateInto<i64, TrapCode>>::try_truncate_into,
    );
    test_f64::<u64, _>(
        "trunc_f64_u",
        Outcome::Eval,
        1.0,
        <F64 as TryTruncateInto<u64, TrapCode>>::try_truncate_into,
    );

    test_f32::<i32, _>(
        "trunc_f32_s",
        Outcome::Trap,
        f32::MAX,
        <F32 as TryTruncateInto<i32, TrapCode>>::try_truncate_into,
    );
    test_f32::<u32, _>(
        "trunc_f32_u",
        Outcome::Trap,
        f32::MAX,
        <F32 as TryTruncateInto<u32, TrapCode>>::try_truncate_into,
    );
    test_f64::<i32, _>(
        "trunc_f64_s",
        Outcome::Trap,
        f64::MAX,
        <F64 as TryTruncateInto<i32, TrapCode>>::try_truncate_into,
    );
    test_f64::<u32, _>(
        "trunc_f64_u",
        Outcome::Trap,
        f64::MAX,
        <F64 as TryTruncateInto<u32, TrapCode>>::try_truncate_into,
    );
    test_f32::<i64, _>(
        "trunc_f32_s",
        Outcome::Trap,
        f32::MAX,
        <F32 as TryTruncateInto<i64, TrapCode>>::try_truncate_into,
    );
    test_f32::<u64, _>(
        "trunc_f32_u",
        Outcome::Trap,
        f32::MAX,
        <F32 as TryTruncateInto<u64, TrapCode>>::try_truncate_into,
    );
    test_f64::<i64, _>(
        "trunc_f64_s",
        Outcome::Trap,
        f64::MAX,
        <F64 as TryTruncateInto<i64, TrapCode>>::try_truncate_into,
    );
    test_f64::<u64, _>(
        "trunc_f64_u",
        Outcome::Trap,
        f64::MAX,
        <F64 as TryTruncateInto<u64, TrapCode>>::try_truncate_into,
    );
}

#[test]
fn load_from_register() {
    fn test<T, F>(load_op: &str, offset: u32, make_op: F)
    where
        T: Display + WasmTypeName + Into<UntypedValue>,
        F: FnOnce(ExecRegister, ExecRegister, Offset) -> ExecInstruction,
    {
        let load_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (memory 1)
                (func (export "call") (param i32) (result {load_type})
                    local.get 0
                    {load_type}.{load_op} 0 offset={offset}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let ptr = ExecRegister::from_inner(0);
        let result = ExecRegister::from_inner(1);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let expected = [
            make_op(result, ptr, offset.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test::<i32, _>("load", 42, load_op!(I32Load));
    test::<i64, _>("load", 42, load_op!(I64Load));
    test::<f32, _>("load", 42, load_op!(F32Load));
    test::<f64, _>("load", 42, load_op!(F64Load));

    test::<i32, _>("load8_s", 42, load_op!(I32Load8S));
    test::<i32, _>("load16_s", 42, load_op!(I32Load16S));
    test::<i64, _>("load8_s", 42, load_op!(I64Load8S));
    test::<i64, _>("load16_s", 42, load_op!(I64Load16S));
    test::<i64, _>("load32_s", 42, load_op!(I64Load32S));

    test::<i32, _>("load8_u", 42, load_op!(I32Load8U));
    test::<i32, _>("load16_u", 42, load_op!(I32Load16U));
    test::<i64, _>("load8_u", 42, load_op!(I64Load8U));
    test::<i64, _>("load16_u", 42, load_op!(I64Load16U));
    test::<i64, _>("load32_u", 42, load_op!(I64Load32U));
}

#[test]
fn load_from_const() {
    fn test<T, F>(load_op: &str, offset: u32, make_op: F)
    where
        T: Display + WasmTypeName + Into<UntypedValue>,
        F: FnOnce(ExecRegister, ExecRegister, Offset) -> ExecInstruction,
    {
        let load_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (memory 1)
                (func (export "call") (result {load_type})
                    i32.const 100
                    {load_type}.{load_op} 0 offset={offset}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let const_ptr = engine.alloc_const(100);
        let result = ExecRegister::from_inner(0);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let expected = [
            ExecInstruction::CopyImm {
                result,
                input: const_ptr,
            },
            make_op(result, result, offset.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test::<i32, _>("load", 42, load_op!(I32Load));
    test::<i64, _>("load", 42, load_op!(I64Load));
    test::<f32, _>("load", 42, load_op!(F32Load));
    test::<f64, _>("load", 42, load_op!(F64Load));

    test::<i32, _>("load8_s", 42, load_op!(I32Load8S));
    test::<i32, _>("load16_s", 42, load_op!(I32Load16S));
    test::<i64, _>("load8_s", 42, load_op!(I64Load8S));
    test::<i64, _>("load16_s", 42, load_op!(I64Load16S));
    test::<i64, _>("load32_s", 42, load_op!(I64Load32S));

    test::<i32, _>("load8_u", 42, load_op!(I32Load8U));
    test::<i32, _>("load16_u", 42, load_op!(I32Load16U));
    test::<i64, _>("load8_u", 42, load_op!(I64Load8U));
    test::<i64, _>("load16_u", 42, load_op!(I64Load16U));
    test::<i64, _>("load32_u", 42, load_op!(I64Load32U));
}

#[test]
fn store_to_register() {
    fn test<T, F>(store_op: &str, offset: u32, make_op: F)
    where
        T: Display + WasmTypeName + Into<UntypedValue>,
        F: FnOnce(ExecRegister, Offset, ExecProvider) -> ExecInstruction,
    {
        let store_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (memory 1)
                (func (export "call") (param i32) (param {store_type})
                    local.get 0
                    local.get 1
                    {store_type}.{store_op} 0 offset={offset}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let ptr = ExecRegister::from_inner(0);
        let value = ExecRegister::from_inner(1);
        let results = engine.alloc_provider_slice([]);
        let expected = [
            make_op(ptr, offset.into(), value.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test::<i32, _>("store", 42, store_op!(I32Store));
    test::<i64, _>("store", 42, store_op!(I64Store));
    test::<f32, _>("store", 42, store_op!(F32Store));
    test::<f64, _>("store", 42, store_op!(F64Store));

    test::<i32, _>("store8", 42, store_op!(I32Store8));
    test::<i32, _>("store16", 42, store_op!(I32Store16));
    test::<i64, _>("store8", 42, store_op!(I64Store8));
    test::<i64, _>("store16", 42, store_op!(I64Store16));
    test::<i64, _>("store32", 42, store_op!(I64Store32));
}

#[test]
fn store_to_const() {
    fn test<T, F>(store_op: &str, offset: u32, make_op: F)
    where
        T: Display + WasmTypeName + Into<UntypedValue>,
        F: FnOnce(ExecRegister, Offset, ExecProvider) -> ExecInstruction,
    {
        let store_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (memory 1)
                (func (export "call") (param {store_type})
                    i32.const 100 ;; ptr
                    local.get 0   ;; stored value
                    {store_type}.{store_op} 0 offset={offset}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let const_ptr = engine.alloc_const(100);
        let value = ExecRegister::from_inner(0);
        let temp = ExecRegister::from_inner(1);
        let results = engine.alloc_provider_slice([]);
        let expected = [
            ExecInstruction::CopyImm {
                result: temp,
                input: const_ptr,
            },
            make_op(temp, offset.into(), value.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test::<i32, _>("store", 42, store_op!(I32Store));
    test::<i64, _>("store", 42, store_op!(I64Store));
    test::<f32, _>("store", 42, store_op!(F32Store));
    test::<f64, _>("store", 42, store_op!(F64Store));

    test::<i32, _>("store8", 42, store_op!(I32Store8));
    test::<i32, _>("store16", 42, store_op!(I32Store16));
    test::<i64, _>("store8", 42, store_op!(I64Store8));
    test::<i64, _>("store16", 42, store_op!(I64Store16));
    test::<i64, _>("store32", 42, store_op!(I64Store32));
}

#[test]
fn global_get() {
    use super::bytecode::Global as GlobalIndex;

    fn test<T>(global_index: u32, init_value: T)
    where
        T: WasmTypeName + Display,
    {
        let wasm_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
                (module
                    (global (mut {wasm_type}) ({wasm_type}.const {init_value}))
                    (func (export "call") (result {wasm_type})
                        global.get {global_index}
                    )
                )
            "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = ExecRegister::from_inner(0);
        let results = engine.alloc_provider_slice([ExecProvider::from_register(result)]);
        let expected = [
            ExecInstruction::GlobalGet {
                result,
                global: GlobalIndex::from(global_index),
            },
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test::<i32>(0, 42);
    test::<i64>(0, 42);
    test::<f32>(0, 42.0);
    test::<f64>(0, 42.0);
}

#[test]
fn global_set_register() {
    use super::bytecode::Global as GlobalIndex;

    fn test<T>(global_index: u32, init_value: T)
    where
        T: WasmTypeName + Display,
    {
        let wasm_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
                (module
                    (global (mut {wasm_type}) ({wasm_type}.const {init_value}))
                    (func (export "call") (param {wasm_type})
                        local.get 0
                        global.set {global_index}
                    )
                )
            "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let value = ExecRegister::from_inner(0);
        let results = engine.alloc_provider_slice([]);
        let expected = [
            ExecInstruction::GlobalSet {
                global: GlobalIndex::from(global_index),
                value: value.into(),
            },
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test::<i32>(0, 42);
    test::<i64>(0, 42);
    test::<f32>(0, 42.0);
    test::<f64>(0, 42.0);
}

#[test]
fn global_set_const() {
    use super::bytecode::Global as GlobalIndex;

    fn test<T>(global_index: u32, init_value: T, new_value: T)
    where
        T: WasmTypeName + Display + Into<UntypedValue>,
    {
        let wasm_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
                (module
                    (global (mut {wasm_type}) ({wasm_type}.const {init_value}))
                    (func (export "call")
                        {wasm_type}.const {new_value}
                        global.set {global_index}
                    )
                )
            "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let value = ExecProvider::from_immediate(engine.alloc_const(new_value));
        let results = engine.alloc_provider_slice([]);
        let expected = [
            ExecInstruction::GlobalSet {
                global: GlobalIndex::from(global_index),
                value: value.into(),
            },
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test::<i32>(0, 42, 77);
    test::<i64>(0, 42, 77);
    test::<f32>(0, 42.0, 77.0);
    test::<f64>(0, 42.0, 77.0);
}

#[test]
fn memory_size() {
    let wasm = wat2wasm(
        r#"
            (module
                (memory 1)
                (func (export "call") (result i32)
                    memory.size
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let result = ExecRegister::from_inner(0);
    let results = engine.alloc_provider_slice([result.into()]);
    let expected = [
        ExecInstruction::MemorySize { result },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn memory_grow_register() {
    let wasm = wat2wasm(
        r#"
            (module
                (memory 1)
                (func (export "call") (param i32) (result i32)
                    local.get 0
                    memory.grow
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let amount = ExecRegister::from_inner(0).into();
    let result = ExecRegister::from_inner(1);
    let results = engine.alloc_provider_slice([result.into()]);
    let expected = [
        ExecInstruction::MemoryGrow { result, amount },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn memory_grow_const() {
    let amount = 1;
    let wasm = wat2wasm(&format!(
        r#"
            (module
                (memory 1)
                (func (export "call") (result i32)
                    i32.const {amount}
                    memory.grow
                )
            )
        "#
    ));
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let amount = engine.alloc_const(amount).into();
    let result = ExecRegister::from_inner(0);
    let results = engine.alloc_provider_slice([result.into()]);
    let expected = [
        ExecInstruction::MemoryGrow { result, amount },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn drop() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (param i64) (param f32) (param f64)
                local.get 0
                local.get 1
                local.get 2
                local.get 3
                drop
                drop
                drop
                drop
            )
        )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let results = engine.alloc_provider_slice([]);
    let expected = [ExecInstruction::Return { results }];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn select_register() {
    let wasm = wat2wasm(&format!(
        r#"
            (module
                (memory 1)
                (func (export "call") (param $condition i32) (param $if_true f64) (param $if_false f64) (result f64)
                    local.get $if_true
                    local.get $if_false
                    local.get $condition
                    select
                )
            )
        "#
    ));
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let condition = ExecRegister::from_inner(0);
    let if_true = ExecRegister::from_inner(1).into();
    let if_false = ExecRegister::from_inner(2).into();
    let result = ExecRegister::from_inner(3);
    let results = engine.alloc_provider_slice([result.into()]);
    let expected = [
        ExecInstruction::Select {
            result,
            condition,
            if_true,
            if_false,
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn select_const() {
    fn test(condition: bool) {
        let condition_flag = condition as i32;
        let wasm = wat2wasm(&format!(
            r#"
                (module
                    (memory 1)
                    (func (export "call") (param $if_true f64) (param $if_false f64) (result f64)
                        local.get $if_true
                        local.get $if_false
                        i32.const {condition_flag}
                        select
                    )
                )
            "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let if_true = ExecRegister::from_inner(0).into();
        let if_false = ExecRegister::from_inner(1).into();
        let input = if condition { if_true } else { if_false };
        let result = ExecRegister::from_inner(2);
        let results = engine.alloc_provider_slice([result.into()]);
        let expected = [
            ExecInstruction::Copy { result, input },
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test(true);
    test(false);
}

#[test]
fn local_set_copy() {
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    local.get 0
                    local.set 1
                    local.get 1
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    let results = engine.alloc_provider_slice([local_1.into()]);
    let expected = [
        ExecInstruction::Copy {
            result: local_1,
            input: local_0.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn local_set_override() {
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                    local.set 0
                    local.get 0
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    let results = engine.alloc_provider_slice([local_0.into()]);
    let expected = [
        ExecInstruction::I32Add {
            result: local_0,
            lhs: local_0.into(),
            rhs: local_1.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn local_set_override_and_copy() {
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                    local.set 0
                    local.get 0
                    local.set 1 ;; this must not override `i32.add` result again
                    local.get 1
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    let results = engine.alloc_provider_slice([local_1.into()]);
    let expected = [
        ExecInstruction::I32Add {
            result: local_0,
            lhs: local_0.into(),
            rhs: local_1.into(),
        },
        ExecInstruction::Copy {
            result: local_1,
            input: local_0.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn local_set_preserve_single() {
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    local.get 0 ;; preserved
                    local.get 0
                    local.get 1
                    i32.add
                    local.set 0
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    // Note: we skip Register(2) since we do not currently
    //       perform proper dead register elimination.
    //       Register(2) is temporarily allocated during
    //       compilation for `i32.add` before changing its
    //       result register via `local.set 0`.
    let preserve = ExecRegister::from_inner(3);
    let results = engine.alloc_provider_slice([preserve.into()]);
    let expected = [
        ExecInstruction::I32Add {
            result: local_0,
            lhs: local_0.into(),
            rhs: local_1.into(),
        },
        ExecInstruction::Copy {
            result: preserve,
            input: local_0.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn local_set_preserve_multiple() {
    // This test pushes both local variables to the Wasm stack
    // then sets their values both to zero and afterwards uses their
    // previous values to compute their sum which is finally returned
    // to the caller.
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    ;; sets both parameters to 0 and returns the sum
                    ;; of their previous values afterwards
                    local.get 0 ;; preserved
                    local.get 1 ;; preserved
                    i32.const 0
                    i32.const 0
                    local.set 0
                    local.set 1
                    i32.add
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    let zero = engine.alloc_const(0_i32);
    let result = ExecRegister::from_inner(2);
    let preserve_0 = ExecRegister::from_inner(3);
    let preserve_1 = ExecRegister::from_inner(4);
    let results = engine.alloc_provider_slice([result.into()]);
    let expected = [
        ExecInstruction::Copy {
            result: preserve_0,
            input: local_0.into(),
        },
        ExecInstruction::CopyImm {
            result: local_0,
            input: zero,
        },
        ExecInstruction::Copy {
            result: preserve_1,
            input: local_1.into(),
        },
        ExecInstruction::CopyImm {
            result: local_1,
            input: zero,
        },
        ExecInstruction::I32Add {
            result,
            lhs: preserve_0.into(),
            rhs: preserve_1.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn local_set_preserve_multi_phase() {
    // The `.wasm` for this test pushes 2 `local.get 0` to the stack
    // but modifies the 2nd `local.get 0` to be equal to `local.get 1`
    // so that the `i32.add` afterwards effectively computes the sum
    // of both local variables.
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    local.get 0 ;; preserved
                    local.get 1
                    local.set 0
                    local.get 0 ;; preserved
                    i32.const 0
                    local.set 0
                    i32.add
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    let zero = engine.alloc_const(0_i32);
    let result = ExecRegister::from_inner(2);
    let preserve_0 = ExecRegister::from_inner(3);
    let preserve_1 = ExecRegister::from_inner(4);
    let results = engine.alloc_provider_slice([result.into()]);
    let expected = [
        ExecInstruction::Copy {
            result: preserve_0,
            input: local_0.into(),
        },
        ExecInstruction::Copy {
            result: local_0,
            input: local_1.into(),
        },
        ExecInstruction::Copy {
            result: preserve_1,
            input: local_0.into(),
        },
        ExecInstruction::CopyImm {
            result: local_0,
            input: zero,
        },
        ExecInstruction::I32Add {
            result,
            lhs: preserve_0.into(),
            rhs: preserve_1.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// This tests that a `local.set` instruction that follows a `call` instruction
/// with exactly one result will alter the result of the `call` instruction
/// instead of inserting a `copy` instruction.
///
/// This is kinda special since `call` and `call_indirect` instructions may
/// return multiple results unlike most Wasm instructions.
#[test]
fn local_set_after_call() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "field" (func $imported_func (param i32) (result i32)))
                (func (export "call") (param i32) (result i32)
                    local.get 0
                    call $imported_func
                    local.set 0
                    local.get 0
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let call_result = ExecRegister::from_inner(0);
    let call_results = ExecRegisterSlice::new(call_result, 1);
    let param = ExecRegister::from_inner(0);
    let params = engine.alloc_provider_slice([param.into()]);
    let return_result = call_result;
    let return_results = engine.alloc_provider_slice([return_result.into()]);
    let expected = [
        ExecInstruction::Call {
            func_idx: FuncIdx::from_u32(0),
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// This tests that a `local.set` instruction that follows a `call_indirect`
/// instruction with exactly one result will alter the result of the `call`
/// instruction instead of inserting a `copy` instruction.
///
/// This is kinda special since `call` and `call_indirect` instructions may
/// return multiple results unlike most Wasm instructions.
#[test]
fn local_set_after_call_indirect() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "table" (table $t 1 funcref))
                (func (export "call") (param i32) (result i32)
                    local.get 0
                    i32.const 1
                    call_indirect (param i32) (result i32)
                    local.set 0
                    local.get 0
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let index = ExecProvider::from_immediate(engine.alloc_const(1_i32));
    let call_result = ExecRegister::from_inner(0);
    let call_results = ExecRegisterSlice::new(call_result, 1);
    let param = ExecRegister::from_inner(0);
    let params = engine.alloc_provider_slice([param.into()]);
    let return_result = call_result;
    let return_results = engine.alloc_provider_slice([return_result.into()]);
    let expected = [
        ExecInstruction::CallIndirect {
            func_type_idx: FuncTypeIdx::from_u32(0),
            index,
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn local_tee_copy() {
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    local.get 0
                    local.tee 1
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    let results = engine.alloc_provider_slice([local_1.into()]);
    let expected = [
        ExecInstruction::Copy {
            result: local_1,
            input: local_0.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn local_tee_override() {
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                    local.tee 0
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    let results = engine.alloc_provider_slice([local_0.into()]);
    let expected = [
        ExecInstruction::I32Add {
            result: local_0,
            lhs: local_0.into(),
            rhs: local_1.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn local_tee_override_and_copy() {
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                    local.tee 0
                    local.tee 1 ;; this must not override `i32.add` result again
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    let results = engine.alloc_provider_slice([local_1.into()]);
    let expected = [
        ExecInstruction::I32Add {
            result: local_0,
            lhs: local_0.into(),
            rhs: local_1.into(),
        },
        ExecInstruction::Copy {
            result: local_1,
            input: local_0.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn local_tee_preserve_single() {
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    local.get 0 ;; preserved
                    local.get 0
                    local.get 1
                    i32.add
                    local.tee 0
                    drop
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    // Note: we skip Register(2) since we do not currently
    //       perform proper dead register elimination.
    //       Register(2) is temporarily allocated during
    //       compilation for `i32.add` before changing its
    //       result register via `local.set 0`.
    let preserve = ExecRegister::from_inner(3);
    let results = engine.alloc_provider_slice([preserve.into()]);
    let expected = [
        ExecInstruction::I32Add {
            result: local_0,
            lhs: local_0.into(),
            rhs: local_1.into(),
        },
        ExecInstruction::Copy {
            result: preserve,
            input: local_0.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn local_tee_preserve_multiple() {
    // This test pushes both local variables to the Wasm stack
    // then sets their values both to zero and afterwards uses their
    // previous values to compute their sum which is finally returned
    // to the caller.
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    ;; sets both parameters to 0 and returns the sum
                    ;; of their previous values afterwards
                    local.get 0 ;; preserved
                    local.get 1 ;; preserved
                    i32.const 0
                    local.tee 0
                    local.tee 1
                    drop
                    i32.add
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    let zero = engine.alloc_const(0_i32);
    let result = ExecRegister::from_inner(2);
    let preserve_0 = ExecRegister::from_inner(3);
    let preserve_1 = ExecRegister::from_inner(4);
    let results = engine.alloc_provider_slice([result.into()]);
    let expected = [
        ExecInstruction::Copy {
            result: preserve_0,
            input: local_0.into(),
        },
        ExecInstruction::CopyImm {
            result: local_0,
            input: zero,
        },
        ExecInstruction::Copy {
            result: preserve_1,
            input: local_1.into(),
        },
        ExecInstruction::Copy {
            result: local_1,
            input: local_0.into(),
        },
        ExecInstruction::I32Add {
            result,
            lhs: preserve_0.into(),
            rhs: preserve_1.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn local_tee_preserve_multi_phase() {
    // The `.wasm` for this test pushes 2 `local.get 0` to the stack
    // but modifies the 2nd `local.get 0` to be equal to `local.get 1`
    // so that the `i32.add` afterwards effectively computes the sum
    // of both local variables.
    let wasm = wat2wasm(
        r#"
            (module
                (func (export "call") (param i32) (param i32) (result i32)
                    local.get 0 ;; preserved
                    local.get 1
                    local.tee 0 ;; preserved
                    i32.const 0
                    local.tee 0
                    drop
                    i32.add
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let local_0 = ExecRegister::from_inner(0);
    let local_1 = ExecRegister::from_inner(1);
    let zero = engine.alloc_const(0_i32);
    let result = ExecRegister::from_inner(2);
    let preserve_0 = ExecRegister::from_inner(3);
    let preserve_1 = ExecRegister::from_inner(4);
    let results = engine.alloc_provider_slice([result.into()]);
    let expected = [
        ExecInstruction::Copy {
            result: preserve_0,
            input: local_0.into(),
        },
        ExecInstruction::Copy {
            result: local_0,
            input: local_1.into(),
        },
        ExecInstruction::Copy {
            result: preserve_1,
            input: local_0.into(),
        },
        ExecInstruction::CopyImm {
            result: local_0,
            input: zero,
        },
        ExecInstruction::I32Add {
            result,
            lhs: preserve_0.into(),
            rhs: preserve_1.into(),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// This tests that a `local.set` instruction that follows a `call` instruction
/// with exactly one result will alter the result of the `call` instruction
/// instead of inserting a `copy` instruction.
///
/// This is kinda special since `call` and `call_indirect` instructions may
/// return multiple results unlike most Wasm instructions.
#[test]
fn local_tee_after_call() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "field" (func $imported_func (param i32) (result i32)))
                (func (export "call") (param i32) (result i32)
                    local.get 0
                    call $imported_func
                    local.tee 0
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let call_result = ExecRegister::from_inner(0);
    let call_results = ExecRegisterSlice::new(call_result, 1);
    let param = ExecRegister::from_inner(0);
    let params = engine.alloc_provider_slice([param.into()]);
    let return_result = call_result;
    let return_results = engine.alloc_provider_slice([return_result.into()]);
    let expected = [
        ExecInstruction::Call {
            func_idx: FuncIdx::from_u32(0),
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// This tests that a `local.set` instruction that follows a `call_indirect`
/// instruction with exactly one result will alter the result of the `call`
/// instruction instead of inserting a `copy` instruction.
///
/// This is kinda special since `call` and `call_indirect` instructions may
/// return multiple results unlike most Wasm instructions.
#[test]
fn local_tee_after_call_indirect() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "table" (table $t 1 funcref))
                (func (export "call") (param i32) (result i32)
                    local.get 0
                    i32.const 1
                    call_indirect (param i32) (result i32)
                    local.tee 0
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let index = ExecProvider::from_immediate(engine.alloc_const(1_i32));
    let call_result = ExecRegister::from_inner(0);
    let call_results = ExecRegisterSlice::new(call_result, 1);
    let param = ExecRegister::from_inner(0);
    let params = engine.alloc_provider_slice([param.into()]);
    let return_result = call_result;
    let return_results = engine.alloc_provider_slice([return_result.into()]);
    let expected = [
        ExecInstruction::CallIndirect {
            func_type_idx: FuncTypeIdx::from_u32(0),
            index,
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn call_0_params_0_results() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "field" (func $imported_func))
                (func (export "call")
                    call $imported_func
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let call_results = ExecRegisterSlice::empty();
    let params = engine.alloc_provider_slice([]);
    let return_results = engine.alloc_provider_slice([]);
    let expected = [
        ExecInstruction::Call {
            func_idx: FuncIdx::from_u32(0),
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn call_1_params_1_results() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "field" (func $imported_func (param i32) (result i32)))
                (func (export "call") (param i32) (result i32)
                    local.get 0
                    call $imported_func
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let call_result = ExecRegister::from_inner(1);
    let call_results = ExecRegisterSlice::new(call_result, 1);
    let param = ExecRegister::from_inner(0);
    let params = engine.alloc_provider_slice([param.into()]);
    let return_result = call_result;
    let return_results = engine.alloc_provider_slice([return_result.into()]);
    let expected = [
        ExecInstruction::Call {
            func_idx: FuncIdx::from_u32(0),
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn call_2_params_1_results() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "field" (func $imported_func (param i32) (param f32) (result i32)))
                (func (export "call") (param i32) (param f32) (result i32)
                    local.get 0
                    local.get 1
                    call $imported_func
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let call_result = ExecRegister::from_inner(2);
    let call_results = ExecRegisterSlice::new(call_result, 1);
    let param_0 = ExecRegister::from_inner(0);
    let param_1 = ExecRegister::from_inner(1);
    let params = engine.alloc_provider_slice([param_0.into(), param_1.into()]);
    let return_result = call_result;
    let return_results = engine.alloc_provider_slice([return_result.into()]);
    let expected = [
        ExecInstruction::Call {
            func_idx: FuncIdx::from_u32(0),
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn call_1_params_2_results() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "field" (func $imported_func (param i32) (result i32) (result f32)))
                (func (export "call") (param i32) (result i32) (result f32)
                    local.get 0
                    call $imported_func
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let call_result = ExecRegister::from_inner(1);
    let call_results = ExecRegisterSlice::new(call_result, 2);
    let param = ExecRegister::from_inner(0);
    let params = engine.alloc_provider_slice([param.into()]);
    let return_result_0 = call_result;
    let return_result_1 = ExecRegister::from_inner(2);
    let return_results =
        engine.alloc_provider_slice([return_result_0.into(), return_result_1.into()]);
    let expected = [
        ExecInstruction::Call {
            func_idx: FuncIdx::from_u32(0),
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn call_2_params_2_results() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "field" (func $imported_func (param i32) (param f32) (result i32) (result f32)))
                (func (export "call") (param i32) (param f32) (result i32) (result f32)
                    local.get 0
                    local.get 1
                    call $imported_func
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let call_result = ExecRegister::from_inner(2);
    let call_results = ExecRegisterSlice::new(call_result, 2);
    let param_0 = ExecRegister::from_inner(0);
    let param_1 = ExecRegister::from_inner(1);
    let params = engine.alloc_provider_slice([param_0.into(), param_1.into()]);
    let return_result_0 = call_result;
    let return_result_1 = ExecRegister::from_inner(3);
    let return_results =
        engine.alloc_provider_slice([return_result_0.into(), return_result_1.into()]);
    let expected = [
        ExecInstruction::Call {
            func_idx: FuncIdx::from_u32(0),
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn call_indirect_0_params_0_results() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "table" (table 1 funcref))
                (func (export "call")
                    i32.const 1 ;; table index
                    call_indirect
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let index = ExecProvider::from_immediate(engine.alloc_const(1_i32));
    let call_results = ExecRegisterSlice::empty();
    let params = engine.alloc_provider_slice([]);
    let return_results = engine.alloc_provider_slice([]);
    let expected = [
        ExecInstruction::CallIndirect {
            func_type_idx: FuncTypeIdx::from_u32(0),
            index,
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn call_indirect_1_params_1_results() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "table" (table 1 funcref))
                (func (export "call") (param i32) (result i32)
                    local.get 0 ;; first param
                    i32.const 1 ;; table index
                    call_indirect (param i32) (result i32)
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let index = ExecProvider::from_immediate(engine.alloc_const(1_i32));
    let call_result = ExecRegister::from_inner(1);
    let call_results = ExecRegisterSlice::new(call_result, 1);
    let param = ExecRegister::from_inner(0);
    let params = engine.alloc_provider_slice([param.into()]);
    let return_result = call_result;
    let return_results = engine.alloc_provider_slice([return_result.into()]);
    let expected = [
        ExecInstruction::CallIndirect {
            func_type_idx: FuncTypeIdx::from_u32(0),
            index,
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn call_indirect_2_params_1_results() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "table" (table 1 funcref))
                (func (export "call") (param i32) (param f32) (result i32)
                    local.get 0 ;; 1st param
                    local.get 1 ;; 2nd param
                    i32.const 1 ;; table index
                    call_indirect (param i32) (param f32) (result i32)
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let index = ExecProvider::from_immediate(engine.alloc_const(1_i32));
    let call_result = ExecRegister::from_inner(2);
    let call_results = ExecRegisterSlice::new(call_result, 1);
    let param_0 = ExecRegister::from_inner(0);
    let param_1 = ExecRegister::from_inner(1);
    let params = engine.alloc_provider_slice([param_0.into(), param_1.into()]);
    let return_result = call_result;
    let return_results = engine.alloc_provider_slice([return_result.into()]);
    let expected = [
        ExecInstruction::CallIndirect {
            func_type_idx: FuncTypeIdx::from_u32(0),
            index,
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn call_indirect_1_params_2_results() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "table" (table 1 funcref))
                (func (export "call") (param i32) (result i32) (result f32)
                    local.get 0 ;; 1st param
                    i32.const 1 ;; table index
                    call_indirect (param i32) (result i32) (result f32)
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let index = ExecProvider::from_immediate(engine.alloc_const(1_i32));
    let call_result = ExecRegister::from_inner(1);
    let call_results = ExecRegisterSlice::new(call_result, 2);
    let param = ExecRegister::from_inner(0);
    let params = engine.alloc_provider_slice([param.into()]);
    let return_result_0 = call_result;
    let return_result_1 = ExecRegister::from_inner(2);
    let return_results =
        engine.alloc_provider_slice([return_result_0.into(), return_result_1.into()]);
    let expected = [
        ExecInstruction::CallIndirect {
            func_type_idx: FuncTypeIdx::from_u32(0),
            index,
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

#[test]
fn call_indirect_2_params_2_results() {
    let wasm = wat2wasm(
        r#"
            (module
                (import "module" "table" (table 1 funcref))
                (func (export "call") (param i32) (param f32) (result i32) (result f32)
                    local.get 0 ;; 1st param
                    local.get 1 ;; 2nd param
                    i32.const 1 ;; table index
                    call_indirect (param i32) (param f32) (result i32) (result f32)
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let index = ExecProvider::from_immediate(engine.alloc_const(1_i32));
    let call_result = ExecRegister::from_inner(2);
    let call_results = ExecRegisterSlice::new(call_result, 2);
    let param_0 = ExecRegister::from_inner(0);
    let param_1 = ExecRegister::from_inner(1);
    let params = engine.alloc_provider_slice([param_0.into(), param_1.into()]);
    let return_result_0 = call_result;
    let return_result_1 = ExecRegister::from_inner(3);
    let return_results =
        engine.alloc_provider_slice([return_result_0.into(), return_result_1.into()]);
    let expected = [
        ExecInstruction::CallIndirect {
            func_type_idx: FuncTypeIdx::from_u32(0),
            index,
            results: call_results,
            params,
        },
        ExecInstruction::Return {
            results: return_results,
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `if` and `else` blocks can be constant folded since the condition
/// is a constant value.
/// This test demonstrates a simple case where it is easy for the translation
/// process to perform this flattening.
#[test]
fn if_simple() {
    let wasm = wat2wasm(&format!(
        r#"
            (module
                (import "module" "table" (table 1 funcref))
                (func (export "call") (param i32) (param i32) (param i32) (result i32)
                    (local $result i32)
                    local.get 0
                    if
                        local.get 1
                        local.get 2
                        i32.add
                        local.set $result
                    else
                        local.get 1
                        local.get 2
                        i32.mul
                        local.set $result
                    end
                    local.get $result
                )
            )
        "#
    ));
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let result = ExecRegister::from_inner(3);
    let results = engine.alloc_provider_slice([result.into()]);
    let else_label = Target::from_inner(3);
    let end_label = Target::from_inner(4);
    let reg0 = ExecRegister::from_inner(0);
    let reg1 = ExecRegister::from_inner(1);
    let reg2 = ExecRegister::from_inner(2);
    let reg3 = ExecRegister::from_inner(3);
    let expected = [
        /* 0 */
        ExecInstruction::BrEqz {
            target: else_label,
            condition: reg0,
        },
        /* 1 */
        ExecInstruction::I32Add {
            result: reg3,
            lhs: reg1,
            rhs: reg2.into(),
        },
        /* 2 */
        ExecInstruction::Br { target: end_label },
        /* 3 */
        ExecInstruction::I32Mul {
            result: reg3,
            lhs: reg1,
            rhs: reg2.into(),
        },
        /* 4 */ ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// The `if` and `else` blocks can be constant folded since the condition
/// is a constant value.
/// This test demonstrates a simple case where it is easy for the translation
/// process to perform this flattening.
#[test]
fn if_const_simple() {
    fn test(condition: bool, expected_op: fn() -> ExecInstruction) {
        let condition = condition as i32;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (import "module" "table" (table 1 funcref))
                (func (export "call") (param i32) (param i32) (result i32)
                    (local $result i32)
                    i32.const {condition}
                    if
                        local.get 0
                        local.get 1
                        i32.add
                        local.set $result
                    else
                        local.get 0
                        local.get 1
                        i32.mul
                        local.set $result
                    end
                    local.get $result
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = ExecRegister::from_inner(2);
        let results = engine.alloc_provider_slice([result.into()]);
        let expected = [expected_op(), ExecInstruction::Return { results }];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test(true, || ExecInstruction::I32Add {
        result: ExecRegister::from_inner(2),
        lhs: ExecRegister::from_inner(0),
        rhs: ExecRegister::from_inner(1).into(),
    });
    test(false, || ExecInstruction::I32Mul {
        result: ExecRegister::from_inner(2),
        lhs: ExecRegister::from_inner(0),
        rhs: ExecRegister::from_inner(1).into(),
    });
}

/// The `if` and `else` blocks can be constant folded since the condition
/// is a constant value.
/// This test demonstrates a case where there is a block nested in the `if`
/// `then` and `else` sub blocks which make it slightly harder for the
/// unreachability checker of the translation process to do the right job.
#[test]
fn if_const_nested() {
    fn test(condition: bool, expected_op: fn() -> ExecInstruction) {
        let condition = condition as i32;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (import "module" "table" (table 1 funcref))
                (func (export "call") (param i32) (param i32) (result i32)
                    (local $result i32)
                    i32.const {condition}
                    if
                        local.get 0
                        block
                            nop
                        end
                        local.get 1
                        i32.add
                        local.set $result
                    else
                        local.get 0
                        block
                            nop
                        end
                        local.get 1
                        i32.mul
                        local.set $result
                    end
                    local.get $result
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = ExecRegister::from_inner(2);
        let results = engine.alloc_provider_slice([result.into()]);
        let expected = [expected_op(), ExecInstruction::Return { results }];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test(true, || ExecInstruction::I32Add {
        result: ExecRegister::from_inner(2),
        lhs: ExecRegister::from_inner(0),
        rhs: ExecRegister::from_inner(1).into(),
    });
    test(false, || ExecInstruction::I32Mul {
        result: ExecRegister::from_inner(2),
        lhs: ExecRegister::from_inner(0),
        rhs: ExecRegister::from_inner(1).into(),
    });
}

#[test]
fn regression_const_lhs() {
    let wasm = wat2wasm(
        r#"
            ;; Regression test to verify correct binary expression with a constant `lhs`
            ;; in a case where an incorrect `copy` instruction was emitted.
            ;;
            ;; Spec Test Suite: float_exprs.wast | f32.no_approximate_reciprocal_sqrt
            (module
                (func (export "func") (param $input f32) (result f32)
                    (f32.div
                        (f32.const 1.0)
                        (f32.sqrt (local.get $input))
                    )
                )
            )
        "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let const_one = engine.alloc_const(1.0_f32);
    let v0 = ExecRegister::from_inner(0);
    let v1 = ExecRegister::from_inner(1);
    let v2 = ExecRegister::from_inner(2);
    let results = engine.alloc_provider_slice([v1.into()]);
    let expected = [
        ExecInstruction::F32Sqrt {
            result: v1,
            input: v0,
        },
        ExecInstruction::CopyImm {
            result: v2,
            input: const_one,
        },
        ExecInstruction::F32Div {
            result: v1,
            lhs: v2,
            rhs: ExecProvider::from_register(v1),
        },
        ExecInstruction::Return { results },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}
