## Benchmark - Regex Redux

Rust based `regex_redux` benchmark test case.

Build with

```
cargo build \
    --package wasmi_benches_regex_redux \
    --profile wasm \
    --target wasm32-unknown-unknown
```

Post-optimize with `wasm-opt` using:

```
wasm-opt \
    -O3 \
    target/wasm32-unknown-unknown/wasm/wasmi_benches_regex_redux.wasm \
    -o target/wasm32-unknown-unknown/wasm/wasmi_benches_regex_redux.opt.wasm
```

Finally move the post-optimized Wasm binary into the proper place:

```
cp \
    target/wasm32-unknown-unknown/wasm/wasmi_benches_regex_redux.opt.wasm \
    crates/wasmi/benches/rust/regex_redux.wasm
```

Benchmark with:

```
cargo bench --bench benches execute/regex_redux
```
