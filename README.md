
| Continuous Integration |     Test Coverage    |  Documentation   |      Crates.io       |
|:----------------------:|:--------------------:|:----------------:|:--------------------:|
| [![ci][1]][2]          | [![codecov][3]][4]   | [![docs][5]][6] | [![crates][7]][8]  |

[1]: https://github.com/wasmi-labs/wasmi/workflows/Rust%20-%20Continuous%20Integration/badge.svg?branch=master
[2]: https://github.com/wasmi-labs/wasmi/actions?query=workflow%3A%22Rust+-+Continuous+Integration%22+branch%3Amaster
[3]: https://codecov.io/gh/wasmi-labs/wasmi/branch/master/graph/badge.svg
[4]: https://codecov.io/gh/wasmi-labs/wasmi/branch/master
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

Version `0.31.0` has been [audited by SRLabs].

[Wasmtime]: https://github.com/bytecodealliance/wasmtime
[audited by SRLabs]: ./resources/security-audit-2023-12-20.pdf

## Announcement: Transfer of Ownership

As of 2024-02-01, the original owner and maintainer of the Wasmi project, Parity Technologies, has officially transferred ownership of the project to me, Robin Freyler. Read more about this transfer [here](NEWS.md#announcement-transfer-of-ownership).

## Distinct Features

The following list states some of the distinct features of Wasmi.

- Simple, correct and deterministic execution of WebAssembly.
- Low-overhead and cross-platform WebAssembly runtime for embedded environments.
- JIT bomb resisting translation.
- Loosely mirrors the [Wasmtime API](https://docs.rs/wasmtime/).
- 100% WebAssembly spec testsuite compliance.
- Built-in support for fuel metering.

## WebAssembly Proposals

The new Wasmi engine supports a variety of WebAssembly proposals and will support even more of them in the future.

| WebAssembly Proposal | Status | Comment |
|:--|:--:|:--|
| [`mutable-global`] | ✅ | Since version `0.14.0`. |
| [`saturating-float-to-int`] | ✅ | Since version `0.14.0`. |
| [`sign-extension`] | ✅ | Since version `0.14.0`. |
| [`multi-value`] | ✅ | Since version `0.14.0`. |
| [`bulk-memory`] | ✅ | Since version `0.24.0`. [(#628)] |
| [`reference-types`] | ✅ | Since version `0.24.0`. [(#635)] |
| [`simd`] | ❌ | Unlikely to be supported. |
| [`tail-calls`] | ✅ | Since version `0.28.0`. [(#683)] |
| [`extended-const`] | ✅ | Since version `0.29.0`. [(#707)] |
| [`function-references`] | 📅 | Planned but not yet implemented. [(#774)] |
| [`gc`] | 📅 | Planned but not yet implemented. [(#775)] |
| [`multi-memory`] | 📅 | Planned but not yet implemented. [(#776)] |
| [`threads`] | 📅 | Planned but not yet implemented. [(#777)] |
| [`relaxed-simd`] | ❌ | Unlikely to be supported since `simd` is unlikely to be supported. |
| [`component-model`] | 📅 | Planned but not yet implemented. [(#897)] |
| | |
| [WASI] | 👨‍🔬 | Experimental support via the [`wasmi_wasi` crate] or the Wasmi CLI application. |

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
[`component-model`]: https://github.com/WebAssembly/component-model

[WASI]: https://github.com/WebAssembly/WASI
[`wasmi_wasi` crate]: ./crates/wasi

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
[(#897)]: https://github.com/wasmi-labs/wasmi/pull/897

## Usage

### As CLI Application

Install the newest Wasmi CLI version:
```console
cargo install wasmi_cli
```
Run `wasm32-unknown-unknown` or `wasm32-wasi` Wasm binaries:
```console
wasmi_cli <WASM_FILE> --invoke <FUNC_NAME> [<FUNC_ARGS>]*
```

### As Rust Library

Refer to the [Wasmi usage guide](./docs/usage.md) to learn how properly to use the [Wasmi crate](https://crates.io/crates/wasmi) as library. 

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

We maintain a timeline for benchmarks of every commit to `master` that [can be viewed here](https://wasmi-labs.github.io/wasmi/benchmarks/).

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
