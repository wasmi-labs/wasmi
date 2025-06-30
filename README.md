
| Continuous Integration |     Test Coverage    |  Documentation   |      Crates.io       |
|:----------------------:|:--------------------:|:----------------:|:--------------------:|
| [![ci][1]][2]          | [![codecov][3]][4]   | [![docs][5]][6] | [![crates][7]][8]  |

[1]: https://github.com/wasmi-labs/wasmi/actions/workflows/rust.yml/badge.svg
[2]: https://github.com/wasmi-labs/wasmi/actions?query=workflow%3A%22Rust+-+Continuous+Integration%22+branch%3Amaster
[3]: https://codecov.io/gh/wasmi-labs/wasmi/branch/main/graph/badge.svg
[4]: https://codecov.io/gh/wasmi-labs/wasmi/branch/main
[5]: https://docs.rs/wasmi/badge.svg
[6]: https://docs.rs/wasmi
[7]: https://img.shields.io/crates/v/wasmi.svg
[8]: https://crates.io/crates/wasmi

[license-mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-apache-badge]: https://img.shields.io/badge/license-APACHE-orange.svg

# Wasmi - WebAssembly (Wasm) Interpreter

<p align="center">
  <img src="./resources/wasmi-logo.png" width="100" height="100">
</p>

Wasmi is an efficient and lightweight WebAssembly interpreter with a focus on constrained and embedded systems.

## Security Audits

Wasmi is suitable for safety critical use cases and has been audited several times already.

| Wasmi Version(s) | Auditor | Contractor | Report |
|--:|:--|:--|:--|
| `0.36.0`-`0.38.0` | [Runtime Verification Inc.] | [Stellar Development Foundation] | [PDF](./resources/audit-2024-11-27.pdf) |
| `0.31.0` | [SRLabs] | [Parity Technologies] | [PDF](./resources/audit-2023-12-20.pdf) |

[Wasmtime]: https://github.com/bytecodealliance/wasmtime
[SRLabs]: https://www.srlabs.de/
[Runtime Verification Inc.]: https://runtimeverification.com/
[Stellar Development Foundation]: https://stellar.org/foundation
[Parity Technologies]: https://www.parity.io/

## Distinct Features

- Simple, correct and deterministic execution of WebAssembly.
- Low-overhead and cross-platform WebAssembly runtime for embedded environments.
- JIT bomb resisting translation.
- Loosely mirrors the [Wasmtime API](https://docs.rs/wasmtime/).
- 100% WebAssembly spec testsuite compliance.
- Built-in support for fuel metering.
- Supports the official [Wasm C-API](https://github.com/WebAssembly/wasm-c-api).

## Usage

Refer to the [Wasmi usage guide](./docs/usage.md) to learn how properly to use [Wasmi](https://crates.io/crates/wasmi).

## WebAssembly Features

| | WebAssembly Proposal | | | | WebAssembly Proposal | |
|:-:|:--|:--|:-:|:--|:--|:--|
| ✅ | [`mutable-global`] | ≥ `0.14.0` | | ✅ | [`custom-page-sizes`] | [≥ `0.41.0`][(#1197)] |
| ✅ | [`saturating-float-to-int`] | ≥ `0.14.0` | | ✅ | [`memory64`] | [≥ `0.41.0`][(#1357)] |
| ✅ | [`sign-extension`] | ≥ `0.14.0` | | ✅ | [`wide-arithmetic`] | [≥ `0.42.0`][(#1369)] |
| ✅ | [`multi-value`] | ≥ `0.14.0` | | ✅ | [`simd`] | [≥ `0.43.0`][(#1364)] |
| ✅ | [`bulk-memory`] | [≥ `0.24.0`][(#628)] | | ✅ | [`relaxed-simd`] | [≥ `0.44.0`][(#1443)] |
| ✅ | [`reference-types`] | [≥ `0.24.0`][(#635)] | | 📅 | [`function-references`] | [Tracking Issue][(#774)] |
| ✅ | [`tail-calls`] | [≥ `0.28.0`][(#683)] | | 📅 | [`gc`] | [Tracking Issue][(#775)] |
| ✅ | [`extended-const`] | [≥ `0.29.0`][(#707)] | | 📅 | [`threads`] | [Tracking Issue][(#777)] |
| ✅ | [`multi-memory`] | [≥ `0.37.0`][(#1191)] | | 📅 | [`exception-handling`] | [Tracking Issue][(#1037)] |

| | Embeddings | |
|:-:|:--|:--|
| ✅ | [WASI] | WASI (`wasip1`) support via the [`wasmi_wasi` crate]. |
| ✅ | [C-API] | Official Wasm C-API support via the [`wasmi_c_api_impl` crate]. |

[`mutable-global`]: https://github.com/WebAssembly/mutable-global
[`saturating-float-to-int`]: https://github.com/WebAssembly/nontrapping-float-to-int-conversions
[`sign-extension`]: https://github.com/WebAssembly/sign-extension-ops
[`multi-value`]: https://github.com/WebAssembly/multi-value
[`reference-types`]: https://github.com/WebAssembly/reference-types
[`bulk-memory`]: https://github.com/WebAssembly/bulk-memory-operations
[`simd` ]: https://github.com/webassembly/simd
[`tail-calls`]: https://github.com/WebAssembly/tail-call
[`extended-const`]: https://github.com/WebAssembly/extended-const
[`function-references`]: https://github.com/WebAssembly/function-references
[`gc`]: https://github.com/WebAssembly/gc
[`multi-memory`]: https://github.com/WebAssembly/multi-memory
[`threads`]: https://github.com/WebAssembly/threads
[`relaxed-simd`]: https://github.com/WebAssembly/relaxed-simd
[`exception-handling`]: https://github.com/WebAssembly/exception-handling
[`custom-page-sizes`]: https://github.com/WebAssembly/custom-page-sizes
[`memory64`]: https://github.com/WebAssembly/memory64
[`wide-arithmetic`]: https://github.com/WebAssembly/wide-arithmetic

[WASI]: https://github.com/WebAssembly/WASI
[C-API]: https://github.com/WebAssembly/wasm-c-api
[`wasmi_wasi` crate]: ./crates/wasi
[`wasmi_c_api_impl` crate]: ./crates/c_api

[(#363)]: https://github.com/wasmi-labs/wasmi/issues/363
[(#364)]: https://github.com/wasmi-labs/wasmi/issues/364
[(#496)]: https://github.com/wasmi-labs/wasmi/issues/496
[(#628)]: https://github.com/wasmi-labs/wasmi/pull/628
[(#635)]: https://github.com/wasmi-labs/wasmi/pull/635
[(#638)]: https://github.com/wasmi-labs/wasmi/pull/638
[(#683)]: https://github.com/wasmi-labs/wasmi/pull/683
[(#707)]: https://github.com/wasmi-labs/wasmi/pull/707
[(#774)]: https://github.com/wasmi-labs/wasmi/pull/774
[(#775)]: https://github.com/wasmi-labs/wasmi/pull/775
[(#776)]: https://github.com/wasmi-labs/wasmi/pull/776
[(#777)]: https://github.com/wasmi-labs/wasmi/pull/777
[(#1037)]: https://github.com/wasmi-labs/wasmi/issues/1137
[(#1197)]: https://github.com/wasmi-labs/wasmi/issues/1197
[(#1191)]: https://github.com/wasmi-labs/wasmi/issues/1191
[(#1357)]: https://github.com/wasmi-labs/wasmi/issues/1357
[(#1364)]: https://github.com/wasmi-labs/wasmi/issues/1364
[(#1369)]: https://github.com/wasmi-labs/wasmi/issues/1369
[(#1443)]: https://github.com/wasmi-labs/wasmi/pull/1443

## Crate Features

| Feature | Crates | Description |
|:-:|:--|:--|
| `std` | `wasmi`<br>`wasmi_core`<br>`wasmi_ir`<br>`wasmi_collections` | Enables usage of Rust's standard library. This may have some performance advantages when enabled. Disabling this feature makes Wasmi compile on platforms that do not provide Rust's standard library such as many embedded platforms. <br><br> Enabled by default. |
| `wat` | `wasmi` | Enables support to parse Wat encoded Wasm modules. <br><br> Enabled by default. |
| `simd` | `wasmi`<br>`wasmi_core`<br>`wasmi_ir`<br>`wasmi_cli` | Enables support for the Wasm `simd` and `relaxed-simd` proposals. Note that this may introduce execution overhead and increased memory consumption for Wasm executions that do not need Wasm `simd` functionality. <br><br> Disabled by default. |
| `hash-collections` | `wasmi`<br>`wasmi_collections` | Enables use of hash-map based collections in Wasmi internals. This might yield performance improvements in some use cases. <br><br> Disabled by default. |
| `prefer-btree-collections` | `wasmi`<br>`wasmi_collections` | Enforces use of btree-map based collections in Wasmi internals. This may yield performance improvements and memory consumption decreases in some use cases. Also it enables Wasmi to run on platforms that have no random source. <br><br> Disabled by default. |
| `extra-checks` | `wasmi` | Enables extra runtime checks in the Wasmi executor. Expected execution overhead is ~20%. Enable this if your focus is on safety. Disable this for maximum execution performance. <br><br> Disabled by default. |

## Development

### Build & Test

Clone the Wasmi repository and build using `cargo`:

```console
git clone https://github.com/wasmi-labs/wasmi.git --recursive
cd wasmi
cargo build
cargo test
```

### Benchmarks

In order to benchmark Wasmi use the following command:

```console
cargo bench
```

Use `translate`, `instantiate`, `execute` or `overhead` filters to only run benchmarks that test performance of Wasm translation, instantiation, execution or miscellaneous overhead respectively, e.g. `cargo bench execute`.

## Supported Platforms

Wasmi supports a wide variety of architectures and platforms.

- Fore more details see this [list of supported platforms for Rust](https://doc.rust-lang.org/stable/rustc/platform-support.html).
- **Note:** Wasmi can be used in `no_std` embedded environments, thus not requiring the standard library (`std`).
- Only some platforms are checked in CI and guaranteed to be fully working by the Wasmi maintainers.

## License

Licensed under either of

  * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
  * MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
