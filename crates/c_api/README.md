# Wasmi C-API

## Usage in a C Project

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
