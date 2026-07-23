//! Regression tests for `memory.copy` and `table.copy` between two _distinct_ instance
//! indices that resolve to the _same_ store entity (the same memory/table imported under
//! two names). The executor must branch on the resolved entity, not the operand index, or
//! it would form aliasing `&mut`/`&` references to a single entity (undefined behavior).

use wasmi::{Engine, Instance, Memory, MemoryType, Module, Ref, RefType, Store, Table, TableType};

/// Imports the same memory under two names and copies within it via `memory.copy $dst $src`.
const COPY_MEMORY_WASM: &str = r#"
    (module
        (import "host" "mem0" (memory $mem0 1))
        (import "host" "mem1" (memory $mem1 1))
        (func (export "copy") (param $dst i32) (param $src i32) (param $len i32)
            (memory.copy $mem1 $mem0 (local.get $dst) (local.get $src) (local.get $len))
        )
    )
"#;

#[test]
fn memory_copy_between_aliased_imports() -> Result<(), wasmi::Error> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    // The same memory is provided for both imports, so `$mem0` and `$mem1` alias it.
    let mem = Memory::new(&mut store, MemoryType::new(1, None))?;
    let module = Module::new(&engine, COPY_MEMORY_WASM)?;
    let instance = Instance::new(&mut store, &module, &[mem.into(), mem.into()])?;
    let copy = instance.get_typed_func::<(u32, u32, u32), ()>(&mut store, "copy")?;

    mem.data_mut(&mut store)[0] = 100;
    assert_eq!(mem.data(&store)[4], 0);
    // Copy the byte from offset 0 to offset 4; both indices target the same memory.
    copy.call(&mut store, (4, 0, 1))?;
    assert_eq!(mem.data(&store)[4], 100);
    Ok(())
}

/// Imports the same table under two names and copies within it via `table.copy $dst $src`.
const COPY_TABLE_WASM: &str = r#"
    (module
        (import "host" "t0" (table $t0 2 funcref))
        (import "host" "t1" (table $t1 2 funcref))
        (type $ft (func (result i32)))
        (func $f (type $ft) (i32.const 42))
        (elem (table $t0) (i32.const 0) func $f)
        (func (export "copy") (param $dst i32) (param $src i32) (param $len i32)
            (table.copy $t1 $t0 (local.get $dst) (local.get $src) (local.get $len))
        )
        (func (export "call") (param $i i32) (result i32)
            (call_indirect $t0 (type $ft) (local.get $i))
        )
    )
"#;

#[test]
fn table_copy_between_aliased_imports() -> Result<(), wasmi::Error> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    // The same table is provided for both imports, so `$t0` and `$t1` alias it.
    let table = Table::new(
        &mut store,
        TableType::new(RefType::Func, 2, None),
        Ref::null(RefType::Func),
    )?;
    let module = Module::new(&engine, COPY_TABLE_WASM)?;
    let instance = Instance::new(&mut store, &module, &[table.into(), table.into()])?;
    let copy = instance.get_typed_func::<(u32, u32, u32), ()>(&mut store, "copy")?;
    let call = instance.get_typed_func::<u32, i32>(&mut store, "call")?;

    // The active element segment initialized `table[0]` with `$f`; slot 1 is still null.
    assert_eq!(call.call(&mut store, 0)?, 42);
    // Copy slot 0 into slot 1; both operand indices target the same table.
    copy.call(&mut store, (1, 0, 1))?;
    assert_eq!(call.call(&mut store, 1)?, 42);
    Ok(())
}
