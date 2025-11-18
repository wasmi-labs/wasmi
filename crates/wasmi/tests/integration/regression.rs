use std::mem::take;
use wasmi::*;

#[test]
fn regression_call_indirect_call_to_null() -> Result<(), wasmi::Error> {
    let wasm = include_bytes!("./wasm/regression.wasm");
    let config = test_config();
    let engine = Engine::new(&config);
    let module = Module::new(&engine, wasm)?;

    #[derive(Default)]
    struct State {
        input: Vec<u8>,
        output: Vec<u8>,
    }
    let mut store = Store::new(&engine, State::default());

    let mut linker = <Linker<State>>::new(&engine);
    linker.func_wrap(
        "typst_env",
        "wasm_minimal_protocol_write_args_to_buffer",
        |mut caller: Caller<'_, State>, param: u32| {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            let input = take(&mut caller.data_mut().input);
            memory.write(caller, param as usize, &input).unwrap();
        },
    )?;
    linker.func_wrap(
        "typst_env",
        "wasm_minimal_protocol_send_result_to_host",
        |mut caller: Caller<'_, State>, ptr: u32, len: u32| {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            let mut output = vec![0; len as usize];
            memory.read(&mut caller, ptr as usize, &mut output).unwrap();
            caller.data_mut().output = output;
        },
    )?;
    let instance = linker.instantiate_and_start(&mut store, &module)?;

    instance
        .get_typed_func::<(), u32>(&store, "typsitter_init")?
        .call(&mut store, ())?;
    store.data_mut().input = String::from(r#"print hi"#).into_bytes();
    let len = store.data().input.len();
    instance
        .get_typed_func::<(u32,), u32>(&store, "typsitter_highlight")?
        .call(&mut store, (len as u32,))?;
    dbg!(&store.data_mut().output);

    Ok(())
}

fn test_config() -> Config {
    let mut config = Config::default();
    // Disable all features irrelevant for this test:
    // config.wasm_bulk_memory(false);
    config.wasm_custom_page_sizes(false);
    config.wasm_extended_const(false);
    config.wasm_memory64(false);
    config.wasm_multi_memory(false);
    config.wasm_multi_value(false);
    config.wasm_mutable_global(false);
    config.wasm_reference_types(false);
    // config.wasm_saturating_float_to_int(false);
    // config.wasm_sign_extension(false);
    config.wasm_tail_call(false);
    config.wasm_wide_arithmetic(false);
    config
}
