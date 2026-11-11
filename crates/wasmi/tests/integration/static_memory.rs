use core::ptr::addr_of_mut;
use wasmi::{Engine, Instance, Memory, MemoryType, Module, Store};

/// A Wasm module that imports a Wasm memory and exports a function that grows it.
const WASM: &str = r#"
    (module
        (import "host" "mem" (memory 1))
        (func (export "grow") (param $delta i32) (result i32)
            (memory.grow (local.get $delta))
        )
    )
"#;

#[test]
fn memory_grow_beyond_static_buffer() -> Result<(), wasmi::Error> {
    static mut BUFFER: [u8; 2 * 65536] = [0; 2 * 65536];
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let buffer = unsafe { &mut *addr_of_mut!(BUFFER) };
    let mem = Memory::new_static(&mut store, MemoryType::new(1, None), buffer)?;
    let module = Module::new(&engine, WASM)?;
    let instance = Instance::new(&mut store, &module, &[mem.into()])?;
    let grow = instance.get_typed_func::<i32, i32>(&mut store, "grow")?;
    assert_eq!(grow.call(&mut store, 2)?, -1);
    assert_eq!(grow.call(&mut store, 1)?, 1);
    assert_eq!(grow.call(&mut store, 1)?, -1);
    Ok(())
}
