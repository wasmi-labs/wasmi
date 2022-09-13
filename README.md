
| Continuous Integration |     Test Coverage    |  Documentation   |      Crates.io       |
|:----------------------:|:--------------------:|:----------------:|:--------------------:|
| [![ci][1]][2]          | [![codecov][5]][6]   | [![docs][9]][10] | [![crates][11]][12]  |

[1]: https://github.com/paritytech/wasmi/workflows/Rust%20-%20Continuous%20Integration/badge.svg?branch=master
[2]: https://github.com/paritytech/wasmi/actions?query=workflow%3A%22Rust+-+Continuous+Integration%22+branch%3Amaster
[5]: https://codecov.io/gh/paritytech/wasmi/branch/master/graph/badge.svg
[6]: https://codecov.io/gh/paritytech/wasmi/branch/master
[9]: https://docs.rs/wasmi/badge.svg
[10]: https://docs.rs/wasmi
[11]: https://img.shields.io/crates/v/wasmi.svg
[12]: https://crates.io/crates/wasmi

[license-mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-apache-badge]: https://img.shields.io/badge/license-APACHE-orange.svg

# `wasmi`- WebAssembly (Wasm) Interpreter

`wasmi` was conceived as a component of [parity-ethereum](https://github.com/paritytech/parity-ethereum) (ethereum-like contracts in wasm) and [substrate](https://github.com/paritytech/substrate). These projects are related to blockchain and require a high degree of correctness. The project is not trying to be involved in any implementation of any work-in-progress Wasm proposals. Instead the project tries to be as close as possible to the specification, therefore avoiding features that are not directly supported by the specification.

With all that said `wasmi` should be a good option for initial prototyping and there shouldn't be a problem migrating from `wasmi` to another specification compliant execution engine later on.

## Distinct Features

The following list states some of the distinct features of `wasmi`.

- Primarily concerned about
    - correct and deterministic WebAssembly execution.
    - WebAssembly specification compliance.
- Can itself be compiled to and executed via WebAssembly.
- Low-overhead and cross-platform WebAssembly runtime.
- Loosely mirrors the [Wasmtime API](https://docs.rs/wasmtime/0.39.1/wasmtime/)
  to act as a drop-in solution.

## WebAssembly Proposals

The new `wasmi_v1` engine supports a variety of WebAssembly proposals and will support even more of them in the future.

| WebAssembly Proposal | Status | Comment |
|:--|:--:|:--|
| [`mutable-global`] | ✅ | |
| [`saturating-float-to-int`] | ✅ | |
| [`sign-extension`] | ✅ | |
| [`multi-value`] | ✅ | |
| [`reference-types`] | ⌛ | Planned but not yet implemented. Low priority. |
| [`bulk-memory`] | ⌛ | Planned but not yet implemented. Low priority. |
| [`simd`] | ❌ | No support is planned for `wasmi`. |
| [`tail-calls`] | ⌛ | Not yet part of the Wasm standard but support in `wasmi` is planned. Low priority. |

[`mutable-global`]: https://github.com/WebAssembly/mutable-global
[`saturating-float-to-int`]: https://github.com/WebAssembly/nontrapping-float-to-int-conversions
[`sign-extension`]: https://github.com/WebAssembly/sign-extension-ops
[`multi-value`]: https://github.com/WebAssembly/multi-value
[`reference-types`]: https://github.com/WebAssembly/reference-types
[`bulk-memory`]: https://github.com/WebAssembly/bulk-memory-operations
[`simd` ]: https://github.com/webassembly/simd
[`tail-calls`]: https://github.com/WebAssembly/tail-call

## Building

Clone `wasmi` from our official repository and then build using the standard `cargo` procedure:

```
git clone https://github.com/paritytech/wasmi.git
cd wasmi
cargo build
```

## Testing

In order to test `wasmi` you need to initialize and update the Git submodules using:

```
git submodule update --init --recursive
```

Alternatively you can provide `--recursive` flag to `git clone` command while cloning the repository:

```
git clone https://github.com/paritytech/wasmi.git ---recursive
```

After Git submodules have been initialized and updated you can test using:

```
cargo test --workspace
```

## Development

Before pushing a PR to our repository we would like you to execute the
`ci.sh` script that can be found in the repository's root folder.

## Supported Platforms

Supported platforms are primarily Linux, MacOS, Windows and WebAssembly.

Use the following command in order to produce a WebAssembly build:

```
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

## Benchmarks

In order to benchmark `wasmi` use the following command:

```
cargo bench
```

**Note:** Benchmarks can be filtered by `compile_and_validate`,
`instantiate` and `execute` flags given to `cargo bench`.
For example `cargo bench execute` will only execute the benchmark
tests that test the performance of WebAssembly execution.

## License

`wasmi` is primarily distributed under the terms of both the MIT
license and the APACHE license (Version 2.0), at your choice.

See `LICENSE-APACHE` and `LICENSE-MIT` for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `wasmi` by you, as defined in the APACHE 2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
