
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

`wasmi` was conceived as a component of [parity-ethereum](https://github.com/paritytech/parity-ethereum) (ethereum-like contracts in wasm) and [substrate](https://github.com/paritytech/substrate). These projects are related to blockchain and require a high degree of correctness. The project is not trying to be be involved in any implementation of any of work-in-progress Wasm proposals. Instead the project tries to be as close as possible to the specification, therefore avoiding features that are not directly supported by the specification.

With all that said `wasmi` should be a good option for initial prototyping and there shouldn't be a problem migrating from `wasmi` to another specification compliant execution engine later on.

# Developer Notes

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
cargo test
```

It is recommended to further specify `--release` since compiling and testing without optimizations
usually is slower compared to compiling and testing with optimizations.

## Platforms

Supported platforms are primarily Linux, MacOS, Windows and WebAssembly.

Use the following command in order to produce a WebAssembly build:

```
cargo build --no-default-features --target wasm32-unknown-unknown
```

## Features

On 64-bit platforms we further provide cross-platform suppport for virtual memory usage.
For this build `wasmi` using:

```
cargo build --features virtual_memory
```

## Benchmarks

In order to benchmark `wasmi` use the following command:

```
cargo bench --manifest-path benches/Cargo.toml
```

# License

`wasmi` is primarily distributed under the terms of both the MIT
license and the APACHE license (Version 2.0), at your choice.

See `LICENSE-APACHE` and `LICENSE-MIT` for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `wasmi` by you, as defined in the APACHE 2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
