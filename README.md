
| Continuous Integration |     Test Coverage    |  Documentation   |      Crates.io       |
|:----------------------:|:--------------------:|:----------------:|:--------------------:|
| [![ci][1]][2]          | [![codecov][3]][4]   | [![docs][5]][6] | [![crates][7]][8]  |

[1]: https://github.com/paritytech/wasmi/workflows/Rust%20-%20Continuous%20Integration/badge.svg?branch=master
[2]: https://github.com/paritytech/wasmi/actions?query=workflow%3A%22Rust+-+Continuous+Integration%22+branch%3Amaster
[3]: https://codecov.io/gh/paritytech/wasmi/branch/master/graph/badge.svg
[4]: https://codecov.io/gh/paritytech/wasmi/branch/master
[5]: https://docs.rs/wasmi/badge.svg
[6]: https://docs.rs/wasmi
[7]: https://img.shields.io/crates/v/wasmi.svg
[8]: https://crates.io/crates/wasmi

[license-mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-apache-badge]: https://img.shields.io/badge/license-APACHE-orange.svg

# `wasmi`- WebAssembly (Wasm) Interpreter

`wasmi` is an efficient WebAssembly interpreter with low-overhead and support
for embedded environment such as WebAssembly itself.

At Parity we are using `wasmi` in [Substrate](https://github.com/paritytech/substrate)
as the execution engine for our WebAssembly based smart contracts.
Furthermore we run `wasmi` within the Substrate runtime which is a WebAssembly
environment itself and driven via [Wasmtime] at the time of this writing.
As such `wasmi`'s implementation requires a high degree of correctness and
Wasm specification conformance.

Since `wasmi` is relatively lightweight compared to other Wasm virtual machines
such as Wasmtime it is also a decent option for initial prototyping.

[Wasmtime]: https://github.com/bytecodealliance/wasmtime

## Distinct Features

The following list states some of the distinct features of `wasmi`.

- Focus on simple, correct and deterministic WebAssembly execution.
- Can itself run inside of WebAssembly.
- Low-overhead and cross-platform WebAssembly runtime.
- Loosely mirrors the [Wasmtime API](https://docs.rs/wasmtime/).
- Resumable function calls.
- Built-in support for fuel metering.
- 100% official WebAssembly spec testsuite compliance.

## WebAssembly Proposals

The new `wasmi` engine supports a variety of WebAssembly proposals and will support even more of them in the future.

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
| | |
| [WASI] | 🟡 | Experimental support via the [`wasmi_wasi` crate] or the `wasmi` CLI application. |

[`mutable-global`]: https://github.com/WebAssembly/mutable-global
[`saturating-float-to-int`]: https://github.com/WebAssembly/nontrapping-float-to-int-conversions
[`sign-extension`]: https://github.com/WebAssembly/sign-extension-ops
[`multi-value`]: https://github.com/WebAssembly/multi-value
[`reference-types`]: https://github.com/WebAssembly/reference-types
[`bulk-memory`]: https://github.com/WebAssembly/bulk-memory-operations
[`simd` ]: https://github.com/webassembly/simd
[`tail-calls`]: https://github.com/WebAssembly/tail-call
[`extended-const`]: https://github.com/WebAssembly/extended-const

[WASI]: https://github.com/WebAssembly/WASI
[`wasmi_wasi` crate]: ./crates/wasi

[(#363)]: https://github.com/paritytech/wasmi/issues/363
[(#364)]: https://github.com/paritytech/wasmi/issues/364
[(#496)]: https://github.com/paritytech/wasmi/issues/496
[(#628)]: https://github.com/paritytech/wasmi/pull/628
[(#635)]: https://github.com/paritytech/wasmi/pull/635
[(#638)]: https://github.com/paritytech/wasmi/pull/638
[(#683)]: https://github.com/paritytech/wasmi/pull/683
[(#707)]: https://github.com/paritytech/wasmi/pull/707

## Usage

### As CLI Application

Install the newest `wasmi` CLI version via:
```console
cargo install wasmi_cli
```
Then run arbitrary `wasm32-unknown-unknown` Wasm blobs via:
```console
wasmi_cli <WASM_FILE> <FUNC_NAME> [<FUNC_ARGS>]*
```

### As Rust Library

Any Rust crate can depend on the [`wasmi` crate](https://crates.io/crates/wasmi)
in order to integrate a WebAssembly intepreter into their stack.

Refer to the [`wasmi` crate docs](https://docs.rs/wasmi) to learn how to use the `wasmi` crate as library.

## Development

### Building

Clone `wasmi` from our official repository and then build using the standard `cargo` procedure:

```console
git clone https://github.com/paritytech/wasmi.git
cd wasmi
cargo build
```

### Testing

In order to test `wasmi` you need to initialize and update the Git submodules using:

```console
git submodule update --init --recursive
```

Alternatively you can provide `--recursive` flag to `git clone` command while cloning the repository:

```console
git clone https://github.com/paritytech/wasmi.git --recursive
```

After Git submodules have been initialized and updated you can test using:

```console
cargo test --workspace
```

### Benchmarks

In order to benchmark `wasmi` use the following command:

```console
cargo bench
```

You can filter which set of benchmarks to run:
- `cargo bench translate`
  - Only runs benchmarks concerned with WebAssembly module translation.

- `cargo bench instantiate`
  - Only runs benchmarks concerned with WebAssembly module instantiation.

- `cargo bench execute`
  - Only runs benchmarks concerned with executing WebAssembly functions.

## Supported Platforms

Supported platforms are primarily Linux, MacOS, Windows and WebAssembly.  
Other platforms might be working but are not guaranteed to be so by the `wasmi` maintainers.

Use the following command in order to produce a WebAssembly build:

```console
cargo build --no-default-features --target wasm32-unknown-unknown
```

## Production Builds

In order to reap the most performance out of `wasmi` we highly recommended
to compile the `wasmi` crate using the following Cargo `profile`:

```toml
[profile.release]
lto = "fat"
codegen-units = 1
```

When compiling for the WebAssembly target we highly recommend to post-optimize
`wasmi` using [Binaryen]'s `wasm-opt` tool since our experiments displayed a
80-100% performance improvements when executed under Wasmtime and also
slightly smaller Wasm binaries.

[Binaryen]: https://github.com/WebAssembly/binaryen

## License

`wasmi` is primarily distributed under the terms of both the MIT
license and the APACHE license (Version 2.0), at your choice.

See `LICENSE-APACHE` and `LICENSE-MIT` for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `wasmi` by you, as defined in the APACHE 2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
