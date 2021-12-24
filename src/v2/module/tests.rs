use super::*;
use crate::v2::{
    engine::{bytecode::Instruction, DropKeep},
    Engine,
};

fn wat2wasm(wat: &str) -> Vec<u8> {
    wabt::wat2wasm(wat).unwrap()
}

fn compile(bytes: impl AsRef<[u8]>) -> (Engine, Vec<FuncBody>) {
    let engine = Engine::default();
    let module = Module::new(&engine, bytes).unwrap();
    (engine, module.func_bodies)
}

fn assert_func_body(engine: &Engine, func_body: FuncBody, expected: &[Instruction]) {
    // let resolved = engine.resolve_func_body(func_body);
    for (index, actual, expected) in expected
        .iter()
        .enumerate()
        .map(|(index, actual)| (index, actual, engine.resolve_inst(func_body, index)))
    {
        assert_eq!(
            actual, &expected,
            "encountered instruction mismatch at position {}",
            index
        );
    }
}

#[test]
fn implicit_return_no_value() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call")
			)
		)
	"#,
    );
    let (engine, func_bodies) = compile(&wasm);
    assert_eq!(func_bodies.len(), 1);
    assert_func_body(
        &engine,
        func_bodies[0],
        &[Instruction::Return(DropKeep::new(0, 0))],
    );
}
