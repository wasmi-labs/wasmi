use cargo_metadata::{camino::Utf8PathBuf, Error as MetaError, Metadata, Package};
use std::{
    error::Error,
    fs,
    io::{stdout, Write},
    process::Command,
};
use wasm_opt::OptimizationOptions;

fn build(target_dir: &Utf8PathBuf, package: &Package) -> Result<(), Box<dyn Error>> {
    let name = &package.name;
    let Some(pretty_name) = package.name.strip_prefix("wasmi_benches_") else {
        // Do not build packages that do not start with `wasmi_benches_`
        // since they are no benchmarks by convention of this repository.
        return Ok(());
    };
    println!("{pretty_name}:");
    print!("    building .. ");
    stdout().flush().unwrap();
    Command::new("cargo")
        .args([
            "build",
            "--package",
            &format!("wasmi_benches_{pretty_name}"),
            "--profile",
            "wasm",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .output()?;
    print!("done\n    optimizing .. ");
    stdout().flush().unwrap();
    let path_prefix = format!("{target_dir}/wasm32-unknown-unknown/wasm");
    let in_file = format!("{path_prefix}/{name}.wasm");
    let out_file = format!("{path_prefix}/{name}.opt.wasm");
    OptimizationOptions::new_opt_level_3()
        .run(&in_file, &out_file)
        .unwrap();
    print!("done\n    finalizing .. ");
    stdout().flush().unwrap();
    fs::copy(
        &out_file,
        &format!("crates/wasmi/benches/rust/{pretty_name}/out.wasm"),
    )?;
    println!("done");
    Ok(())
}

fn metadata() -> Result<Metadata, MetaError> {
    cargo_metadata::MetadataCommand::new().no_deps().exec()
}

fn main() -> Result<(), Box<dyn Error>> {
    let meta = metadata()?;
    for package in &meta.packages {
        build(&meta.target_directory, package)?;
    }
    Ok(())
}
