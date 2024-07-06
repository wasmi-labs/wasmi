# Rust based Benchmarks

## Building

Build with

```
cargo run --package build_benches
```

This builds the benchmark `.wasm` files for all the Rust based benchmarks
such as `tiny_keccak`, `reverse_complement` and `regex_redux` with proper
optimizations and stores the results in their respective directories.

## Usage

Use this script whenever a benchmark has changed or when a new Rust, LLVM or `wasm-opt`
version has been released.
