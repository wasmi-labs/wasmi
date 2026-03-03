# The Wasmi CLI Application

A lightweight **WebAssembly interpreter CLI** built on top of the [Wasmi crate].

This application provides a command-line interface for executing WebAssembly modules (`.wasm`) and WebAssembly script files (`.wast`). It supports WASI, configurable compilation strategies and fuel metering.

For details about the Wasmi project itself (architecture, design goals, embedding API, `no_std` support, etc.), see the [Wasmi repository] or the [Wasmi blog].

[Wasmi crate]: https://crates.io/crates/wasmi
[Wasmi repository]: https://github.com/wasmi-labs/wasmi
[Wasmi blog]: https://wasmi-labs.github.io/blog

## Overview

`wasmi` is intended for:

- Deterministic and efficient WebAssembly execution
- Embedded and constrained environments
- WASI-compatible execution

## Build & Install

Build from source:

```bash
cargo build --release
```

The resulting binary will be located at:

```bash
target/release/wasmi
```

Install from crates.io:

```bash
cargo install wasmi_cli
```

This installs the `wasmi` binary.

## Usage

```bash
wasmi [OPTIONS] [ARGS]...
wasmi <COMMAND>
```

### Commands

| Command | Description |
|----------|-------------|
| `run`   | Executes a WebAssembly module |
| `wast`  | Executes a WebAssembly Script (`.wast`) file |
| `help`  | Prints help information |

If no command is provided, `run` is assumed if the `run` crate feature is enabled.

## Running a WebAssembly Module

Execute the `foo.wasm` Wasm module.

```bash
wasmi foo.wasm
```

Execute the `foo.wasm` Wasm module and provide `a b c` as WASI arguments.

```bash
wasmi foo.wasm a b c
```

Execute the function `bar` from the `foo.wasm` Wasm module and provide `a b c` as parameters.

```bash
wasmi foo.wasm --invoke bar a b c
```

## WASI Integration

### Pre-open Directories

Preopens a host directory for guest access.

```bash
wasmi foo.wasm --dir ./data
```

### Environment Variables

Adds an environment variable visible to the guest.

```bash
wasmi foo.wasm --env FOO=bar
```

### TCP Listening Socket

Provides a listening socket to the module for WASI socket operations.

```bash
wasmi foo.wasm --tcplisten 127.0.0.1:8080
```

## Compilation Modes

Control Wasmi’s [compilation mode](https://docs.rs/wasmi/latest/wasmi/enum.CompilationMode.html):

```bash
wasmi foo.wasm --compilation-mode <MODE>
```

Where `<MODE>` is any of `eager`, `lazy-translation` or `lazy`.

## Fuel Metering

Enable deterministic execution fuel limits to make sure any execution halts.

```bash
wasmi foo.wasm --fuel 1000
```

## Executing `.wast` Files

```bash
wasmi wast foo.wast
```

Executes the `foo.wast` Wasm script.

This is primarily intended for specification testing,
conformance validation and regression testing of the Wasmi interpreter.

## Crate Features

| Feature | Default | Description |
|---|---|---|
| `run` | ✅ | Enables execution of WebAssembly modules (`.wasm`). This is the primary functionality most users require. Can be disabled via `--no-default-features` to build a reduced binary. |
| `wat` | ✅ | Allows the `run` command to accept WebAssembly Text (`.wat`) modules in addition to `.wasm`. |
| `wast` | ✅ | Enables execution of WebAssembly Script (`.wast`) files. Primarily intended for specification testing and interpreter debugging. Enabled by default for CLI parity with other WebAssembly runtimes. |
| `wasi` | ✅ | Enables WASI support when executing modules via `run`, including environment variables, preopened directories, and socket support. |
| `simd` | ❌ | Enables support for WebAssembly SIMD proposal for both module execution and script testing. Disabled by default due to significant bloat. |
| `portable-dispatch` | ❌ | Allows to compile Wasmi universally at the cost of execution performance. Use `--profile bench` to counteract performance regressions to some extend. |
| `indirect-dispatch` | ❌ | Uses a slightly more compact IR encoding at the cost of execution performance. |

## License

Licensed under either of

  * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
  * MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
