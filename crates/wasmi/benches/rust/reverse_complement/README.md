## Benchmark - Reverse Complement

Rust based `reverse_complement` benchmark test case.

Build with

```
cargo build \
    --package wasmi_benches_reverse_complement \
    --profile wasm \
    --target wasm32-unknown-unknown
```

Post-optimize with `wasm-opt` using:

```
wasm-opt \
    -O3 \
    target/wasm32-unknown-unknown/wasm/wasmi_benches_reverse_complement.wasm \
    -o target/wasm32-unknown-unknown/wasm/wasmi_benches_reverse_complement.opt.wasm
```

Finally move the post-optimized Wasm binary into the proper place:

```
cp \
    target/wasm32-unknown-unknown/wasm/wasmi_benches_reverse_complement.opt.wasm \
    crates/wasmi/benches/rust/reverse_complement.wasm
```

Benchmark with:

```
cargo bench --bench benches execute/reverse_complement
```
