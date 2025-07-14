use std::{fs, io::Write, path::PathBuf};

use anyhow::{anyhow, Result};
use wasmi::{
    serialization::{deserialize_module, serialize_module},
    Engine,
};

use crate::args::SerializeArgs;

pub(super) fn serialize(args: SerializeArgs) -> Result<()> {
    let required_features = args.to_required_features();
    println!("req feautures: {required_features:?}");
    let engine = wasmi::Engine::default();

    let wasm_file = args.module;
    let wasm =
        fs::read(&wasm_file).map_err(|_| anyhow!("failed to read Wasm file {wasm_file:?}"))?;
    let module = wasmi::Module::new(&engine, wasm).map_err(|error| {
        anyhow!("failed to parse and validate Wasm module {wasm_file:?}: {error}")
    })?;

    let bytes = serialize_module(&module, &required_features)
        .map_err(|error| anyhow!("failed to serialize Wasm module {wasm_file:?}: {error}"))?;

    let output = args
        .output
        .unwrap_or_else(|| wasm_file.with_extension("wasm.ser"));

    let other_engine = Engine::default();
    let _deser_module =
        deserialize_module(&other_engine, &bytes).expect("failed to deserialize back");

    if output == PathBuf::from("-") {
        std::io::stdout().write_all(&bytes)?;
    } else {
        fs::write(&output, &bytes)?;
    }

    let read_bytes = fs::read(&output).expect("failed to read bytes");
    let other_engine = Engine::default();
    let _deser_module =
        deserialize_module(&other_engine, &read_bytes).expect("failed to deserialize back");

    println!("deser from read okay");

    println!("Serialized Wasm module to {output:?}");

    Ok(())
}
