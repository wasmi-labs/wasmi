use wasmi::{CompilationMode, Config, Engine, Module};

/// One function of type `[] -> []` whose body is
/// [0 locals, `end`, `nop`]: an operator after the final `end`.
const WASM: [u8; 25] = [
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // magic + version
    0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // type section: 1 type, [] -> []
    0x03, 0x02, 0x01, 0x00, // function section: 1 function, type 0
    0x0a, 0x05, 0x01, 0x03, 0x00, 0x0b, 0x01, // code section: locals=0, end, nop
];

#[test]
fn operators_after_function_end_fails() {
    for fuel in [false, true] {
        let mut config = Config::default();
        config.consume_fuel(fuel);
        config.compilation_mode(CompilationMode::Eager);
        let engine = Engine::new(&config);
        let expected = Module::validate(&engine, &WASM).unwrap_err().to_string();
        let actual = Module::new(&engine, &WASM[..]).unwrap_err().to_string();
        assert_eq!(actual, expected);
    }
}
