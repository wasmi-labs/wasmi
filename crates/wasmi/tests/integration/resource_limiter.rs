//! Tests to check if wasmi's ResourceLimiter works as intended.
use wasmi::{
    core::TrapCode,
    Config,
    Engine,
    Error,
    Linker,
    Module,
    Store,
    StoreLimits,
    StoreLimitsBuilder,
    TypedFunc,
};

/// Setup [`Engine`] and [`Store`] for resource limiting.
fn test_setup(limits: StoreLimits) -> (Store<StoreLimits>, Linker<StoreLimits>) {
    let config = Config::default();
    let engine = Engine::new(&config);
    let mut store = Store::new(&engine, limits);
    store.limiter(|limits| limits);
    let linker = Linker::new(&engine);
    (store, linker)
}

/// Compiles the `wasm` encoded bytes into a [`Module`].
///
/// # Panics
///
/// If an error occurred upon module compilation, validation or translation.
fn create_module(store: &Store<StoreLimits>, bytes: &[u8]) -> Result<Module, Error> {
    Module::new(store.engine(), bytes)
}

struct Test {
    store: Store<StoreLimits>,
    memory_grow: TypedFunc<i32, i32>,
    memory_size: TypedFunc<(), i32>,
    table_grow: TypedFunc<i32, i32>,
    table_size: TypedFunc<(), i32>,
}

impl Test {
    fn new(mem_pages: i32, table_elts: i32, limits: StoreLimits) -> Result<Self, Error> {
        let wasm = format!(
            r#"
            (module
                (memory {mem_pages})
                (table {table_elts} funcref)
                (func (export "memory_grow") (param $pages i32) (result i32) (memory.grow (local.get $pages)))
                (func (export "memory_size") (result i32) (memory.size))
                (func (export "table_grow") (param $elts i32) (result i32) (table.grow (ref.func 0) (local.get $elts)))
                (func (export "table_size") (result i32) (table.size))
            )
            "#
        );
        let (mut store, linker) = test_setup(limits);
        let module = create_module(&store, wasm.as_bytes())?;
        let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;
        let memory_grow = instance.get_func(&store, "memory_grow").unwrap();
        let memory_size = instance.get_func(&store, "memory_size").unwrap();
        let table_grow = instance.get_func(&store, "table_grow").unwrap();
        let table_size = instance.get_func(&store, "table_size").unwrap();
        let memory_grow = memory_grow.typed::<i32, i32>(&store)?;
        let memory_size = memory_size.typed::<(), i32>(&store)?;
        let table_grow = table_grow.typed::<i32, i32>(&store)?;
        let table_size = table_size.typed::<(), i32>(&store)?;
        Ok(Self {
            store,
            memory_grow,
            memory_size,
            table_grow,
            table_size,
        })
    }
}

#[test]
fn test_big_memory_fails_to_instantiate() {
    let loose_limits = StoreLimitsBuilder::new().memory_size(3 * (1 << 16)).build();
    let tight_limits = StoreLimitsBuilder::new().memory_size(2 * (1 << 16)).build();
    assert!(Test::new(3, 0, loose_limits).is_ok());
    assert!(Test::new(3, 0, tight_limits).is_err());
}

#[test]
fn test_big_table_fails_to_instantiate() {
    let loose_limits = StoreLimitsBuilder::new().table_elements(100).build();
    let tight_limits = StoreLimitsBuilder::new().table_elements(99).build();
    assert!(Test::new(0, 100, loose_limits).is_ok());
    assert!(Test::new(0, 100, tight_limits).is_err());
}

#[test]
fn test_memory_count_limit() {
    let limits = StoreLimitsBuilder::new().memories(0).build();
    assert!(Test::new(0, 0, limits).is_err());
}

#[test]
fn test_instance_count_limit() {
    let limits = StoreLimitsBuilder::new().instances(0).build();
    assert!(Test::new(0, 0, limits).is_err());
}

#[test]
fn test_tables_count_limit() {
    let limits = StoreLimitsBuilder::new().tables(0).build();
    assert!(Test::new(0, 0, limits).is_err());
}

#[test]
fn test_memory_does_not_grow_on_limited_growth() -> Result<(), Error> {
    let limits = StoreLimitsBuilder::new().memory_size(3 * (1 << 16)).build();
    let mut test = Test::new(2, 0, limits)?;
    // By default the policy of a memory.grow failure is just for the instruction
    // to return -1 and not-grow the underlying memory. We also have the option to
    // trap on failure, which is exercised by the next test below.

    // Check memory size is what we expect.
    assert_eq!(test.memory_size.call(&mut test.store, ())?, 2);
    // First memory.grow doesn't hit the limit, so succeeds, returns previous size.
    assert_eq!(test.memory_grow.call(&mut test.store, 1)?, 2);
    // Check memory size is what we expect.
    assert_eq!(test.memory_size.call(&mut test.store, ())?, 3);
    // Second call goes past the limit, so fails to grow the memory, but returns Ok(-1)
    assert_eq!(test.memory_grow.call(&mut test.store, 1)?, -1);
    // Check memory size is what we expect.
    assert_eq!(test.memory_size.call(&mut test.store, ())?, 3);
    Ok(())
}

#[test]
fn test_memory_traps_on_limited_growth() -> Result<(), Error> {
    let limits = StoreLimitsBuilder::new()
        .memory_size(3 * (1 << 16))
        .trap_on_grow_failure(true)
        .build();
    let mut test = Test::new(2, 0, limits)?;
    // Check memory size is what we expect.
    assert_eq!(test.memory_size.call(&mut test.store, ())?, 2);
    // First memory.grow doesn't hit the limit, so succeeds, returns previous size.
    assert_eq!(test.memory_grow.call(&mut test.store, 1)?, 2);
    // Check memory size is what we expect.
    assert_eq!(test.memory_size.call(&mut test.store, ())?, 3);
    // Second call goes past the limit, so fails to grow the memory, and we've configured it to trap.
    assert!(matches!(
        test.memory_grow
            .call(&mut test.store, 1)
            .unwrap_err()
            .as_trap_code(),
        Some(TrapCode::GrowthOperationLimited)
    ));
    // Check memory size is what we expect.
    assert_eq!(test.memory_size.call(&mut test.store, ())?, 0x3);
    Ok(())
}

#[test]
fn test_table_does_not_grow_on_limited_growth() -> Result<(), Error> {
    let limits = StoreLimitsBuilder::new().table_elements(100).build();
    let mut test = Test::new(0, 99, limits)?;
    // By default the policy of a table.grow failure is just for the instruction
    // to return -1 and not-grow the underlying table. We also have the option to
    // trap on failure, which is exercised by the next test below.

    // Check table size is what we expect.
    assert_eq!(test.table_size.call(&mut test.store, ())?, 99);
    // First table.grow doesn't hit the limit, so succeeds, returns previous size.
    assert_eq!(test.table_grow.call(&mut test.store, 1)?, 99);
    // Check table size is what we expect.
    assert_eq!(test.table_size.call(&mut test.store, ())?, 100);
    // Second call goes past the limit, so fails to grow the table, but returns Ok(-1)
    assert_eq!(test.table_grow.call(&mut test.store, 1)?, -1);
    // Check table size is what we expect.
    assert_eq!(test.table_size.call(&mut test.store, ())?, 100);
    Ok(())
}

#[test]
fn test_table_traps_on_limited_growth() -> Result<(), Error> {
    let limits = StoreLimitsBuilder::new()
        .table_elements(100)
        .trap_on_grow_failure(true)
        .build();
    let mut test = Test::new(0, 99, limits)?;
    // Check table size is what we expect.
    assert_eq!(test.table_size.call(&mut test.store, ())?, 99);
    // First table.grow doesn't hit the limit, so succeeds, returns previous size.
    assert_eq!(test.table_grow.call(&mut test.store, 1)?, 99);
    // Check table size is what we expect.
    assert_eq!(test.table_size.call(&mut test.store, ())?, 100);
    // Second call goes past the limit, so fails to grow the table, and we've configured it to trap.
    assert!(matches!(
        test.table_grow
            .call(&mut test.store, 1)
            .unwrap_err()
            .as_trap_code(),
        Some(TrapCode::GrowthOperationLimited)
    ));
    // Check table size is what we expect.
    assert_eq!(test.table_size.call(&mut test.store, ())?, 100);
    Ok(())
}
