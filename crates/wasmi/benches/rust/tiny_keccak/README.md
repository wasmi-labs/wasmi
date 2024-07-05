## Benchmark - Tiny Keccak

Rust based `tiny_keccak` benchmark test case.

Build with

```
cargo build \
    --package wasmi_benches_tiny_keccak \
    --profile wasm \
    --target wasm32-unknown-unknown
```

Post-optimize with `wasm-opt` using:

```
wasm-opt \
    -O3 \
    target/wasm32-unknown-unknown/wasm/wasmi_benches_tiny_keccak.wasm \
    -o target/wasm32-unknown-unknown/wasm/wasmi_benches_tiny_keccak.opt.wasm
```

Finally move the post-optimized Wasm binary into the proper place:

```
cp \
    target/wasm32-unknown-unknown/wasm/wasmi_benches_tiny_keccak.opt.wasm \
    crates/wasmi/benches/rust/tiny_keccak.wasm
```

Benchmark with:

```
cargo bench --bench benches execute/tiny_keccak
```
