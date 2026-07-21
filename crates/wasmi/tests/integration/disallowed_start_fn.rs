use wasmi::{Config, Engine, Module, errors::ErrorKind};

#[test]
fn disallowed_start_fn() {
    let wasm = r#"
        (module
            (start $f)
            (func $f)
        )
    "#;
    let mut config = Config::default();
    config.allow_start_fn(false);
    let engine = Engine::new(&config);
    let module_or_err = Module::new(&engine, wasm.as_bytes());
    match module_or_err {
        Ok(module) => panic!("expected error but found: {module:?}"),
        Err(err) => {
            assert!(matches!(err.kind(), ErrorKind::Translation(_)));
            assert_eq!(
                err.to_string(),
                "configuration disallows start functions but found one".to_string()
            )
        }
    }
}
