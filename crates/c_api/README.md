# Wasmi C-API

## Usage in a C Project

If you have a C project with which you want to use Wasmi, you can interface with Wasmi's C API:

### Prerequisites

- [CMake](https://cmake.org/)
- [A Rust Toolchain](https://www.rust-lang.org/tools/install)

From the root of the Wasmi repository, run the following commands:

```shell
cmake -S crates/c_api -B target/c_api --install-prefix "$(pwd)/artifacts" &&
cmake --build target/c_api --target install
```

These commands will produce the following files:

- `artifacts/lib/libwasmi.{a,lib}`:
    The static Wasmi library.
- `artifacts/lib/libwasmi.{so,dylib,dll}`:
    The dynamic (or shared) Wasmi library.
- `artifacts/include/**.h`:
    The header files for interfacing with Wasmi from C or C++.

### Build Features

By default Wasmi's operator dispatch relies on LLVM tail-calling, which is only
guaranteed in optimized builds. Unoptimized **Debug** builds
(`-DCMAKE_BUILD_TYPE=Debug`) can therefore cause stackoverflows. To avoid this,
the CMake build **automatically enables the `portable-dispatch` feature for Debug
builds**, which selects a portable (loop-based) dispatch scheme that does not rely on
tail calls. Release builds keep the faster tail-call dispatch.

You can enable `portable-dispatch` (or any other Cargo feature) for any build by
passing additional cargo flags via `WASMI_USER_CARGO_BUILD_OPTIONS`:

```shell
cmake -S crates/c_api -B target/c_api \
    -DWASMI_USER_CARGO_BUILD_OPTIONS="--features portable-dispatch"
```

## Usage in a Rust Project

If you have a Rust crate that uses a C or C++ library that uses Wasmi, you can link to the Wasmi C API as follows:

1. Add a dependency on the `wasmi_c_api_impl` crate to your `Cargo.toml`. Note that package name differs from the library name.

```toml
[dependencies]
wasmi_c_api = { version = "0.35.0", package = "wasmi_c_api_impl" }
```

2. In your `build.rs` file, when compiling your C/C++ source code, add the C `wasmi_c_api` headers to the include path:

```rust
fn main() {
    let mut cfg = cc::Build::new();
    // Add the Wasmi and standard Wasm C-API headers to the include path.
    cfg
        .include(std::env::var("DEP_WASMI_C_API_INCLUDE").unwrap());
        .include(std::env::var("DEP_WASMI_C_API_WASM_INCLUDE").unwrap());
    // Compile your C code.
    cfg
        .file("src/your_c_code.c")
        .compile("your_library");
}
```

## `no_std` Support

The `wasmi_c_api_impl` crate supports `no_std` by disabling its default features.

For `no_std` builds, users may have to define their own `#[global_allocator]` and `#[panic_handler]`
for the resulting final `staticlib` or `cdylib`.

For targets without a pre-built `core`/`alloc`, build the standard library from
source with nightly `-Z build-std`. Include `std` in the list (the host
proc-macro dependency needs it) even though the artifact is `no_std`:

```shell
cargo +nightly build --no-default-features -Z build-std=core,alloc,std --target <target>
```
