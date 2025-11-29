use self::engine::AvgBytesPerFunctionLimit;
use super::*;
use crate::{error::ErrorKind, Config, Engine, Error, Module};

/// Parses and returns the Wasm module `wasm` with the given [`EnforcedLimits`] `limits`.
fn parse_with(wasm: &str, limits: EnforcedLimits) -> Result<Module, Error> {
    let mut config = Config::default();
    config.enforced_limits(limits);
    let engine = Engine::new(&config);
    Module::new(&engine, wasm)
}

#[test]
fn max_globals_ok() {
    let wasm = "
        (module
            (global i32 (i32.const 1))
            (global i32 (i32.const 2))
        )
    ";
    parse_with(
        wasm,
        EnforcedLimits {
            max_globals: Some(2),
            ..EnforcedLimits::default()
        },
    )
    .unwrap();
}

#[test]
fn max_globals_err() {
    let wasm = "
        (module
            (global i32 (i32.const 1))
            (global i32 (i32.const 2))
            (global i32 (i32.const 3))
        )
    ";
    let limits = EnforcedLimits {
        max_globals: Some(2),
        ..EnforcedLimits::default()
    };
    assert!(matches!(
        parse_with(wasm, limits).unwrap_err().kind(),
        ErrorKind::Limits(EnforcedLimitsError::TooManyGlobals { limit: 2 }),
    ))
}

#[test]
fn max_functions_ok() {
    let wasm = "
        (module
            (func)
            (func)
        )
    ";
    parse_with(
        wasm,
        EnforcedLimits {
            max_functions: Some(2),
            ..EnforcedLimits::default()
        },
    )
    .unwrap();
}

#[test]
fn max_functions_err() {
    let wasm = "
        (module
            (func)
            (func)
            (func)
        )
    ";
    let limits = EnforcedLimits {
        max_functions: Some(2),
        ..EnforcedLimits::default()
    };
    assert!(matches!(
        parse_with(wasm, limits).unwrap_err().kind(),
        ErrorKind::Limits(EnforcedLimitsError::TooManyFunctions { limit: 2 }),
    ))
}

#[test]
fn max_tables_ok() {
    let wasm = "
        (module
            (table 0 funcref)
            (table 0 funcref)
        )
    ";
    parse_with(
        wasm,
        EnforcedLimits {
            max_tables: Some(2),
            ..EnforcedLimits::default()
        },
    )
    .unwrap();
}

#[test]
fn max_tables_err() {
    let wasm = "
        (module
            (table 0 funcref)
            (table 0 funcref)
            (table 0 funcref)
        )
    ";
    let limits = EnforcedLimits {
        max_tables: Some(2),
        ..EnforcedLimits::default()
    };
    assert!(matches!(
        parse_with(wasm, limits).unwrap_err().kind(),
        ErrorKind::Limits(EnforcedLimitsError::TooManyTables { limit: 2 }),
    ))
}

#[test]
fn max_memories_ok() {
    let wasm = "
        (module
            (memory 0)
            (memory 0)
        )
    ";
    parse_with(
        wasm,
        EnforcedLimits {
            max_memories: Some(2),
            ..EnforcedLimits::default()
        },
    )
    .unwrap();
}

#[test]
fn max_memories_err() {
    let wasm = "
        (module
            (memory 0)
            (memory 0)
            (memory 0)
        )
    ";
    let limits = EnforcedLimits {
        max_memories: Some(2),
        ..EnforcedLimits::default()
    };
    assert!(matches!(
        parse_with(wasm, limits).unwrap_err().kind(),
        ErrorKind::Limits(EnforcedLimitsError::TooManyMemories { limit: 2 }),
    ))
}

#[test]
fn max_element_segments_ok() {
    let wasm = "
        (module
            (table $t 0 funcref)
            (func $f)
            (elem (table $t) (i32.const 0) funcref (ref.func $f) (ref.null func))
            (elem (table $t) (i32.const 1) funcref (ref.func $f) (ref.null func))
        )
    ";
    parse_with(
        wasm,
        EnforcedLimits {
            max_element_segments: Some(2),
            ..EnforcedLimits::default()
        },
    )
    .unwrap();
}

#[test]
fn max_element_segments_err() {
    let wasm = "
        (module
            (table $t 0 funcref)
            (func $f)
            (elem (table $t) (i32.const 0) funcref (ref.func $f) (ref.null func))
            (elem (table $t) (i32.const 1) funcref (ref.func $f) (ref.null func))
            (elem (table $t) (i32.const 2) funcref (ref.func $f) (ref.null func))
        )
    ";
    let limits = EnforcedLimits {
        max_element_segments: Some(2),
        ..EnforcedLimits::default()
    };
    assert!(matches!(
        parse_with(wasm, limits).unwrap_err().kind(),
        ErrorKind::Limits(EnforcedLimitsError::TooManyElementSegments { limit: 2 }),
    ))
}

#[test]
fn max_data_segments_ok() {
    let wasm = "
        (module
            (memory $m 0)
            (data (memory $m) (i32.const 0) \"abc\")
            (data (memory $m) (i32.const 1) \"abc\")
        )
    ";
    parse_with(
        wasm,
        EnforcedLimits {
            max_data_segments: Some(2),
            ..EnforcedLimits::default()
        },
    )
    .unwrap();
}

#[test]
fn max_data_segments_err() {
    let wasm = "
        (module
            (memory $m 0)
            (data (memory $m) (i32.const 0) \"abc\")
            (data (memory $m) (i32.const 1) \"abc\")
            (data (memory $m) (i32.const 2) \"abc\")
        )
    ";
    let limits = EnforcedLimits {
        max_data_segments: Some(2),
        ..EnforcedLimits::default()
    };
    assert!(matches!(
        parse_with(wasm, limits).unwrap_err().kind(),
        ErrorKind::Limits(EnforcedLimitsError::TooManyDataSegments { limit: 2 }),
    ))
}

#[test]
fn max_params_func_ok() {
    let wasm = "
        (module
            (func (param i32 i32))
        )
    ";
    parse_with(
        wasm,
        EnforcedLimits {
            max_params: Some(2),
            ..EnforcedLimits::default()
        },
    )
    .unwrap();
}

#[test]
fn max_params_func_err() {
    let wasm = "
        (module
            (func (param i32 i32 i32))
        )
    ";
    let limits = EnforcedLimits {
        max_params: Some(2),
        ..EnforcedLimits::default()
    };
    assert!(matches!(
        parse_with(wasm, limits).unwrap_err().kind(),
        ErrorKind::Limits(EnforcedLimitsError::TooManyParameters { limit: 2 }),
    ))
}

#[test]
fn max_params_control_ok() {
    let wasm = "
        (module
            (func (param i32)
                (local.get 0)
                (local.get 0)
                (block (param i32 i32)
                    (drop)
                    (drop)
                )
            )
        )
    ";
    parse_with(
        wasm,
        EnforcedLimits {
            max_params: Some(2),
            ..EnforcedLimits::default()
        },
    )
    .unwrap();
}

#[test]
fn max_params_control_err() {
    let wasm = "
        (module
            (func (param i32)
                (local.get 0)
                (local.get 0)
                (block (param i32 i32 i32)
                    (drop)
                    (drop)
                    (drop)
                )
            )
        )
    ";
    let limits = EnforcedLimits {
        max_params: Some(2),
        ..EnforcedLimits::default()
    };
    assert!(matches!(
        parse_with(wasm, limits).unwrap_err().kind(),
        ErrorKind::Limits(EnforcedLimitsError::TooManyParameters { limit: 2 }),
    ))
}

#[test]
fn max_results_func_ok() {
    let wasm = "
        (module
            (func (result i32 i32)
                (i32.const 1)
                (i32.const 2)
            )
        )
    ";
    parse_with(
        wasm,
        EnforcedLimits {
            max_results: Some(2),
            ..EnforcedLimits::default()
        },
    )
    .unwrap();
}

#[test]
fn max_results_func_err() {
    let wasm = "
        (module
            (func (result i32 i32 i32)
                (i32.const 1)
                (i32.const 2)
                (i32.const 3)
            )
        )
    ";
    let limits = EnforcedLimits {
        max_results: Some(2),
        ..EnforcedLimits::default()
    };
    assert!(matches!(
        parse_with(wasm, limits).unwrap_err().kind(),
        ErrorKind::Limits(EnforcedLimitsError::TooManyResults { limit: 2 }),
    ))
}

#[test]
fn max_results_control_ok() {
    let wasm = "
        (module
            (func
                (block (result i32 i32)
                    (i32.const 1)
                    (i32.const 2)
                )
                (drop)
                (drop)
            )
        )
    ";
    parse_with(
        wasm,
        EnforcedLimits {
            max_results: Some(2),
            ..EnforcedLimits::default()
        },
    )
    .unwrap();
}

#[test]
fn max_results_control_err() {
    let wasm = "
        (module
            (func
                (block (result i32 i32 i32)
                    (i32.const 1)
                    (i32.const 2)
                    (i32.const 3)
                )
                (drop)
                (drop)
                (drop)
            )
        )
    ";
    let limits = EnforcedLimits {
        max_results: Some(2),
        ..EnforcedLimits::default()
    };
    assert!(matches!(
        parse_with(wasm, limits).unwrap_err().kind(),
        ErrorKind::Limits(EnforcedLimitsError::TooManyResults { limit: 2 }),
    ))
}

#[test]
fn min_avg_code_bytes_ok() {
    let wasm = "
        (module
            (func
                (nop)
                (nop)
                (nop)
            )
            (func
                (nop)
                (nop)
                (nop)
            )
        )
    ";
    parse_with(
        wasm,
        EnforcedLimits {
            min_avg_bytes_per_function: Some(AvgBytesPerFunctionLimit {
                req_funcs_bytes: 0,
                min_avg_bytes_per_function: 6,
            }),
            ..EnforcedLimits::default()
        },
    )
    .unwrap();
}

#[test]
fn min_avg_code_bytes_err() {
    let wasm = "
        (module
            (func
                (nop)
                (nop)
            )
            (func
                (nop)
                (nop)
            )
        )
    ";
    let limits = EnforcedLimits {
        min_avg_bytes_per_function: Some(AvgBytesPerFunctionLimit {
            req_funcs_bytes: 0,
            min_avg_bytes_per_function: 6,
        }),
        ..EnforcedLimits::default()
    };
    std::println!("{:?}", parse_with(wasm, limits).unwrap_err());
    assert!(matches!(
        parse_with(wasm, limits).unwrap_err().kind(),
        ErrorKind::Limits(EnforcedLimitsError::MinAvgBytesPerFunction { limit: 6, avg: 5 }),
    ))
}

#[test]
fn min_avg_code_bytes_ok_threshold() {
    let wasm = "
        (module
            (func
                (nop)
                (nop)
            )
            (func
                (nop)
                (nop)
            )
        )
    ";
    let limits = EnforcedLimits {
        min_avg_bytes_per_function: Some(AvgBytesPerFunctionLimit {
            req_funcs_bytes: 12,
            min_avg_bytes_per_function: 6,
        }),
        ..EnforcedLimits::default()
    };
    parse_with(wasm, limits).unwrap();
}
