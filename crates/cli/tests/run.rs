use assert_cmd::cargo;
use std::path::PathBuf;

#[test]
fn test_simple_print() {
    let mut cmd = get_cmd();
    let assert = cmd.arg(get_bin_path("simple_print")).assert();
    let output = assert.get_output();
    let stdout = &output.stdout;
    assert!(contains_slice(stdout, b"Hello World"));
    if !(contains_slice(stdout, b"Hello World\n")) {
        eprint!("UNEQUAL: {}", std::str::from_utf8(stdout).unwrap());
    }
}

fn contains_slice<T>(slice: &[T], other: &[T]) -> bool
where
    T: Eq,
{
    if other.is_empty() {
        return true;
    }
    slice.windows(other.len()).any(|window| window == other)
}

#[test]
fn test_proc_exit() {
    let mut cmd = get_cmd();
    let assert = cmd.arg(get_bin_path("proc_exit")).assert();
    assert!(assert.get_output().stdout.is_empty());
    assert.failure().code(1);
}

#[test]
fn test_verbose() {
    let mut cmd = get_cmd();
    let assert = cmd.arg(get_bin_path("proc_exit")).arg("--verbose").assert();
    let stdout = &assert.get_output().stdout;
    assert!(contains_slice(stdout, b"proc_exit.wat\")::()"));
}

/// gets the path to a wasm binary given it's name
fn get_bin_path(name: &str) -> PathBuf {
    let mut path = PathBuf::new();
    path.push("tests");
    path.push("wats");
    path.push(format!("{name}.wat"));
    path
}

fn get_cmd() -> assert_cmd::Command {
    cargo::cargo_bin_cmd!("wasmi")
}
