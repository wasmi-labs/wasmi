use assert_cmd::Command;
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
    assert.failure().code(1);
}

/// UTILS

/// gets the path to a wasm binary given it's name
fn get_bin_path(name: &str) -> PathBuf {
    let mut path = PathBuf::new();
    path.push("tests");
    path.push("wats");
    path.push(format!("{name}.wat"));
    path
}

fn get_cmd() -> assert_cmd::Command {
    Command::cargo_bin("wasmi_cli").expect("could not create wasmi_cli command")
}
