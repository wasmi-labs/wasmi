use std::env;
use std::process;

fn main() {
    println!("cargo:rerun-if-changed=./wasm-kernel/");

    // The CARGO environment variable provides a path to the executable that
    // runs this build process.
    let cargo_bin = env::var("CARGO").expect("CARGO env variable should be defined");

    // Build a release version of wasm-kernel. The code in the output wasm binary
    // will be used in benchmarks.
    let output = process::Command::new(cargo_bin)
        .arg("build")
        .arg("--target=wasm32-unknown-unknown")
        .arg("--release")
        .arg("--manifest-path=./wasm-kernel/Cargo.toml")
        .arg("--verbose")
        .output()
        .expect("failed to execute `cargo`");

    if !output.status.success() {
        let msg = format!(
            "status: {status}\nstdout: {stdout}\nstderr: {stderr}\n",
            status = output.status,
            stdout = String::from_utf8_lossy(&output.stdout),
            stderr = String::from_utf8_lossy(&output.stderr),
        );
        panic!("{}", msg);
    }
}
