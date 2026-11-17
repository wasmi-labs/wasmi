//! Regression tests for `br_table` with branch parameters in temporary stack slots where all
//! targets share the same height but the branch values sit above another operand.
//!
//! In this case no per-target copies are encoded, so the branch values must be copied to the
//! slots the targets expect. Otherwise the targets read stale stack cells.

use wasmi::{Engine, Instance, Module, Store};

const WASM: &str = r#"
    (module
        (func (export "slot_index") (param $index i32) (result i64 i64)
            (block (result i64 i64)
                (i32.const 7)
                (i64.const 0x1111)
                (i64.const 0x2222)
                (br_table 0 1 0 (local.get $index))
            )
        )
        (func (export "reg_index") (param $index i32) (result i64 i64)
            (block (result i64 i64)
                (i32.const 7)
                (i64.const 0x1111)
                (i64.const 0x2222)
                (br_table 0 1 0 (i32.add (local.get $index) (i32.const 0)))
            )
        )
    )
"#;

#[test]
fn br_table_equal_heights_copies_branch_params() -> Result<(), wasmi::Error> {
    for func in ["slot_index", "reg_index"] {
        for index in [0, 1, 2] {
            let engine = Engine::default();
            let mut store = Store::new(&engine, ());
            let module = Module::new(&engine, WASM)?;
            let instance = Instance::new(&mut store, &module, &[])?;
            let func = instance.get_typed_func::<i32, (i64, i64)>(&mut store, func)?;
            assert_eq!(func.call(&mut store, index)?, (0x1111, 0x2222));
        }
    }
    Ok(())
}
