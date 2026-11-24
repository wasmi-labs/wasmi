//! Regression tests for `v128.store{8,16}_lane` with an offset or memory that rules out the
//! `mem0` + 16-bit offset encoding. Their decoder used to read the offset as `u64` while the
//! encoder wrote `Offset`, which is only 32-bit without the `memory64` crate feature.

use wasmi::{Engine, Instance, Module, Store};

const WASM: &str = r#"
    (module
        (memory $m0 2)
        (memory $m1 1)
        (export "m0" (memory $m0))
        (export "m1" (memory $m1))
        (func (export "store8_offset") (param $ptr i32) (param $v v128) (result i32)
            (v128.store8_lane $m0 offset=65536 3 (local.get $ptr) (local.get $v))
            (i32.add (local.get $ptr) (i32.const 1))
        )
        (func (export "store16_offset") (param $ptr i32) (param $v v128) (result i32)
            (v128.store16_lane $m0 offset=65536 5 (local.get $ptr) (local.get $v))
            (i32.add (local.get $ptr) (i32.const 2))
        )
        (func (export "store8_mem1") (param $ptr i32) (param $v v128) (result i32)
            (v128.store8_lane $m1 offset=4 7 (i32.add (local.get $ptr) (local.get $ptr)) (local.get $v))
            (i32.add (local.get $ptr) (i32.const 3))
        )
        (func (export "store16_mem1") (param $ptr i32) (param $v v128) (result i32)
            (v128.store16_lane $m1 offset=8 6 (i32.add (local.get $ptr) (local.get $ptr)) (local.get $v))
            (i32.add (local.get $ptr) (i32.const 4))
        )
    )
"#;

fn v128(bytes: [u8; 16]) -> wasmi::V128 {
    wasmi::V128::from(u128::from_le_bytes(bytes))
}

#[test]
fn v128_store_lane_with_offset_or_memory() -> Result<(), wasmi::Error> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = Module::new(&engine, WASM)?;
    let instance = Instance::new(&mut store, &module, &[])?;
    let m0 = instance.get_memory(&store, "m0").unwrap();
    let m1 = instance.get_memory(&store, "m1").unwrap();
    let call = |store: &mut Store<()>, name: &str, ptr: i32, v: [u8; 16]| {
        instance
            .get_typed_func::<(i32, wasmi::V128), i32>(&*store, name)?
            .call(store, (ptr, v128(v)))
    };
    let bytes: [u8; 16] = core::array::from_fn(|i| 0x10 + i as u8);

    assert_eq!(call(&mut store, "store8_offset", 1, bytes)?, 2);
    assert_eq!(m0.data(&store)[65537], 0x13);

    assert_eq!(call(&mut store, "store16_offset", 10, bytes)?, 12);
    assert_eq!(&m0.data(&store)[65546..65548], &[0x1A, 0x1B]);

    assert_eq!(call(&mut store, "store8_mem1", 2, bytes)?, 5);
    assert_eq!(m1.data(&store)[8], 0x17);

    assert_eq!(call(&mut store, "store16_mem1", 20, bytes)?, 24);
    assert_eq!(&m1.data(&store)[48..50], &[0x1C, 0x1D]);
    Ok(())
}
