# Wasmi Development

## Checkout & Build

Clone the Wasmi repository and build using `cargo`:

```console
git clone https://github.com/wasmi-labs/wasmi.git --recursive
cd wasmi
cargo build
```

## Testing

Wasmi has two major test suites that are accessed via the `wasmi` and `wasmi_wast` crates respectively.

- `cargo test -p wasmi`: unit tests and tons of detailed translation unit tests
- `cargo test -p wasmi_wast`: integration tests including the official Wasm spec testsuite

As usual both testsuites can be run at the same time via `cargo test`.

## Benchmarks

In order to benchmark Wasmi use the following command:

```console
cargo bench --bench benches -p wasmi
```

Benchmarks in Wasmi are structured in the following way:

- `translate`: test cases that primarily benchmark the translation performance
    - `checked`: validate + translate eagerly
    - `unchecked`: eagerly translate but not validate
    - `fueled`: same as `checked` but with fuel metering enabled
    - `lazy`: validate + translate lazily
    - `lazy-translation`: validate eagerly but translate lazily
- `instantiate`: test cases that primarily benchmark the Wasm module instantiation performance
- `execute`: test cases that primarily benchmark Wasmi's execution performance
    - `call`: call based testcases
    - `tiny_keccak`, `regex_redux`, `reverse_complement`: compiled from optimized Rust sources
    - `fibonacci`: variety of fibonacci tests (recusion, tail-recursion and iteration)
    - `memory`: test cases benchmarking memory accesses (load, store, bulk-ops)
    - many more ..
- `overhead`: test cases that benchmark Wasmi call performance overhead

Example: to benchmark all fibonacci test cases use the following command:

```
`cargo bench --bench benches -p wasmi execute/fibonacci`.
```

## Fuzzing

Wasmi has some built-in fuzzers that are even used in Google's OSSFuzz.

- `translate`: optimized to find crashes and bugs during Wasmi translation
- `execute`: optimized to find crashes and bugs during Wasmi execution
- `differential`: finds mismatches between different Wasm runtimes
    - Wasm runtimes compared against trunc Wasmi are Wasmtime and an old Wasmi v0.31

## Publishing a Release

Publishing new Wasmi versions requires to publish new versions for all crates in the Wasmi workspace.

In order to successfully publish one needs to publish the following Wasmi crates in the following order:

- `wasmi_core`
- `wasmi_collections`
- `wasmi_ir`
- `wasmi`
- `wasmi_wast`
- `wasmi_wasi`
- `wasmi_c_api_macros`
- `wasmi_c_api_impl`
- `wasmi_cli` (*)

(*) Before publishing `wasmi_cli` one needs to comment-out the `profile.release` information
in its `Cargo.toml`. This is required due to a bug in Cargo: https://github.com/rust-lang/cargo/issues/8264
This step can and should be dropped once this Cargo bug has been fixed.
