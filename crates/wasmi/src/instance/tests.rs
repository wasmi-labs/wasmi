use super::*;
use crate::{
    Caller,
    Engine,
    ExternRef,
    Func,
    MemoryType,
    Mutability,
    Nullable,
    Ref,
    RefType,
    Store,
    TableType,
    TrapCode,
    Val,
    error::ErrorKind,
    module::InstantiationError,
};

#[test]
fn instantiate_no_imports() {
    let wasm = r#"
        (module
            (func (export "f") (param i32 i32) (result i32)
                (i32.add (local.get 0) (local.get 1))
            )
        )
    "#;
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).unwrap();
    assert!(instance.get_func(&store, "f").is_some());
}

#[test]
fn instantiate_with_start() {
    let wasm = r#"
        (module
            (func $f)
            (start $f)
        )
    "#;
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let mut store = Store::new(&engine, ());
    let _instance = Instance::new(&mut store, &module, &[]).unwrap();
}

#[test]
fn instantiate_with_trapping_start() {
    let wasm = r#"
        (module
            (func $f
                (unreachable)
            )
            (start $f)
        )
    "#;
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let mut store = Store::new(&engine, ());
    let error = Instance::new(&mut store, &module, &[]).unwrap_err();
    assert_eq!(error.as_trap_code(), Some(TrapCode::UnreachableCodeReached));
}

#[test]
fn instantiate_with_imports_and_start() {
    let wasm = r#"
        (module
            (import "env" "f" (func $f (param i32)))
            (import "env" "t" (table $t 0 funcref))
            (import "env" "m" (memory $m 0))
            (import "env" "g" (global $g (mut i32)))

            (elem declare func $f)

            (func $main
                (global.set $g (i32.const 1))
                (i32.store8 $m (i32.const 0) (i32.const 1))
                (table.set $t (i32.const 0) (ref.func $f))
                (call $f (i32.const 1))
            )
            (start $main)
        )
    "#;
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let data: i32 = 0;
    let mut store = Store::new(&engine, data);
    let g = Global::new(&mut store, Val::I32(0), Mutability::Var);
    let m = Memory::new(&mut store, MemoryType::new(1, None)).unwrap();
    let t = Table::new(
        &mut store,
        TableType::new(RefType::Func, 1, None),
        Ref::from(<Nullable<Func>>::Null),
    )
    .unwrap();
    let f = Func::wrap(&mut store, |mut caller: Caller<i32>, a: i32| {
        let data = caller.data_mut();
        *data = a;
    });
    let externals = [
        Extern::from(f),
        Extern::from(t),
        Extern::from(m),
        Extern::from(g),
    ]
    .map(Extern::from);
    let _instance = Instance::new(&mut store, &module, &externals).unwrap();
    assert_eq!(g.get(&store).i32(), Some(1));
    assert_eq!(m.data(&store)[0], 0x01_u8);
    assert!(!t.get(&store, 0).unwrap().unwrap_func().is_null());
    assert_eq!(store.data(), &1);
}

#[test]
fn instantiate_with_invalid_global_import() {
    let wasm = r#"
        (module
            (import "env" "g" (global $g (mut i32)))
            (func $main
                (global.set $g (i32.const 1))
            )
        )
    "#;
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let mut store = Store::new(&engine, ());
    let g = Global::new(&mut store, Val::I64(0), Mutability::Var);
    let externals = [Extern::from(g)].map(Extern::from);
    let error = Instance::new(&mut store, &module, &externals).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Instantiation(InstantiationError::GlobalTypeMismatch { .. })
    ));
}

#[test]
fn instantiate_with_invalid_memory_import() {
    let wasm = r#"
        (module
            (import "env" "m" (memory $m 2))
            (func
                (i32.store8 $m (i32.const 0) (i32.const 1))
            )
        )
    "#;
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let mut store = Store::new(&engine, ());
    let m = Memory::new(&mut store, MemoryType::new(0, Some(1))).unwrap();
    let externals = [Extern::from(m)].map(Extern::from);
    let error = Instance::new(&mut store, &module, &externals).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Instantiation(InstantiationError::MemoryTypeMismatch { .. })
    ));
}

#[test]
fn instantiate_with_invalid_table_import() {
    let wasm = r#"
        (module
            (import "env" "t" (table $t 0 funcref))
            (elem declare func $f)
            (func $f
                (table.set $t (i32.const 0) (ref.func $f))
            )
        )
    "#;
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let mut store = Store::new(&engine, ());
    let t = Table::new(
        &mut store,
        TableType::new(RefType::Extern, 1, None),
        Ref::from(<Nullable<ExternRef>>::Null),
    )
    .unwrap();
    let externals = [Extern::from(t)].map(Extern::from);
    let error = Instance::new(&mut store, &module, &externals).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Instantiation(InstantiationError::TableTypeMismatch { .. })
    ));
}

#[test]
fn instantiate_with_invalid_func_import() {
    let wasm = r#"
        (module
            (import "env" "f" (func $f (param i32)))
            (elem declare func $f)
            (func
                (call $f (i32.const 1))
            )
        )
    "#;
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let data: i64 = 0;
    let mut store = Store::new(&engine, data);
    let f = Func::wrap(&mut store, |mut caller: Caller<i64>, a: i64| {
        let data = caller.data_mut();
        *data = a;
    });
    let externals = [Extern::from(f)].map(Extern::from);
    let error = Instance::new(&mut store, &module, &externals).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Instantiation(InstantiationError::FuncTypeMismatch { .. })
    ));
}
