# Wasmi Usage Guide

This document briefly explains how to use Wasmi as a Rust dependency and get the most out of it performance-wise.

Refer to the [Wasmi crate docs](https://docs.rs/wasmi) to learn how to use the [Wasmi crate](https://crates.io/crates/wasmi) as library. Reading the API docs provide a good overview of Wasmi's potential and possibilities.

## Usage: As CLI Installation

Install the newest Wasmi CLI version using:

```
cargo install wasmi_cli
```

Then run `wasm32-unknown-unknown` or `wasm32-wasi` Wasm binaries via:

```console
wasmi_cli <WASM_FILE> --invoke <FUNC_NAME> [<FUNC_ARGS>]
```

Where

- `<WASM_FILE>` is the path to your WebAssembly binary
- `<FUNC_NAME>` is the name of the _exported_ function that you want to invoke.
- `[<FUNC_ARGS>]` is the list of parameters with which to invoke the _exported_ function specified as `FUNC_NAME`.

## Usage: As Rust Dependency

### Cargo

Incorporate Wasmi in your Rust application in the usual way by adding it to your projects `Cargo.toml` file:

```toml
[dependencies]
wasmi = "0.32"
```

Alternatively use `cargo add wasmi` which automatically uses the most recent version of a new dependency that is applicable.

### Embedded Environments

If you want to use Wasmi in an embedded environment that happens to _not_ support Rust's `std` facilities you can simply disable Wasmi's default features.

```toml
[dependencies]
wasmi = { version = "0.32", default-features = false }
```

This disables Wasmi's `std` feature which is enabled by default and thus makes Wasmi usable in `no_std` Rust environments.

### Optimizations

Wasmi heavily depends on proper Rust and LLVM optimizations. The difference between a `debug` Wasmi build and a properly optimized one can be 100x.

Use the following profile to compile your application that uses Wasmi:

```toml
[profile.production]
inherits = "release"
lto = "fat"
codegen-units = 1
```

Then build your application using:

```shell
cargo build --profile production
```

For Rust CLI applications you have to overwrite the `release` profile instead to take effect upon installation via `cargo install`:

```toml
[profile.release]
lto = "fat"
codegen-units = 1
```

Read more about Cargo profiles [here](https://doc.rust-lang.org/cargo/reference/profiles.html).

### Footgun: Profile Overwrites

Before Wasmi v0.32 it was possible to apply certain optimization just to Wasmi via [Cargo profile overwrites](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides):

```toml
[profile.release.package.wasmi]
lto = "fat"
codegen-units = 1
```

However, since Wasmi v0.32 this is no longer easily possible.

The reasons for this is technical: Wasmi's executor is generic over the generic type `Store<T>`. This causes Rust and LLVM to compile Wasmi's executor not while compiling Wasmi itself but upon compiling the crate that uses Wasmi. Thus, Cargo profile overwrites must be applied to Wasmi users as well to take effect.

One way to achieve this is to isolate Wasmi usage into its own crate and apply the optimization required by Wasmi on that crate instead:

- `myapp`: Your root crate application that originally depended on Wasmi.
- `myapp-wasmi`: A new crate with the only purpose to isolate Wasmi usage. It is critical that the Wasmi API exposed by this crate is itself non-generic.

```toml
[profile.release.package.myapp-wasmi]
lto = "fat"
codegen-units = 1
```

## WebAssembly Optimizations

WebAssembly runtimes are fast because they usually are fed with pre-optimized Wasm binaries.  
This is especially true for Wasm runtimes that have no sophisticated optimizations built-in such as Wasmi.

In order to reap the most out of your WebAssembly experience make sure to always apply proper optimizations on your programs that are being compiled to WebAssembly before executing them via Wasmi.

For compiling a Rust application to WebAssembly make sure to use the following profile:

```toml
[profile.release]
lto = "fat"
codegen-units = 1
panic = "abort"
```

After compilation via the Rust compiler it is recommended to apply [Binaryen]'s `wasm-opt` on the resulting Wasm binary as a post-optimization routine.

[Binaryen]: https://github.com/WebAssembly/binaryen
