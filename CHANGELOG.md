# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Additionally we have an `Internal` section for changes that are of interest to developers.

## [0.13.0] - 2022-07-25

**Note:** This is going to be the last release with the legacy `wasmi` engine.
          Future releases are going to use the new Wasm execution engines
          that are currently in development.
          We may consider to publish the legacy `wasmi` engine as `wasmi-legacy`
          crate.

### Changed

- Update dependency: `wasmi-validation v0.4.2 -> v0.5.0`

## [0.12.0] - 2022-07-24

### Changed

- `wasmi` now depends on the [`wasmi_core`](https://crates.io/crates/wasmi_core) crate.
- Deprecated `RuntimeValue::decode_{f32,f64}` methods.
    - **Reason**: These methods expose details about the `F32` and `F64` types.
                  The `RuntimeValue` type provides `from_bits` methods for similar purposes.
    - **Replacement:** Replace those deprecated methods with `F{32,64}::from_bits().into()` respectively.
- Refactor traps in `wasmi`: [PR](https://github.com/paritytech/wasmi/commit/cd59462bc946a52a7e3e4db491ac6675e3a2f53f)
    - This change also renames `TrapKind` to `TrapCode`.
    - The `wasmi` crate now properly reuses the `TrapCode` definitions from the `wasmi_core` crate.
- Updated dependency:
    - `parity-wasm v0.42 -> v0.45`
    - `memory_units v0.3.0 -> v0.4.0`

### Internal

- Rename `RuntimeValue` to `Value` internally.
- Now uses `wat` crate dependency instead of `wabt` for reading `.wat` files in tests.
- Updated dev-dependencies:
    - `assert_matches: v1.1 -> v1.5`
    - `rand 0.4.2 -> 0.8.2`
- Fix some `clippy` warnings.

## [0.11.0] - 2022-01-06

### Fixed

- Make `wasmi` traps more conformant with the Wasm specification. (https://github.com/paritytech/wasmi/pull/300)
- Fixed a bug in `{f32, f64}_copysign` implementations. (https://github.com/paritytech/wasmi/pull/293)
- Fixed a bug in `{f32, f64}_{min, max}` implementations. (https://github.com/paritytech/wasmi/pull/295)

### Changed

- Optimized Wasm to host calls. (https://github.com/paritytech/wasmi/pull/291)
    - In some artificial benchmarks we saw improvements of up to 42%!
- Introduce a more efficient `LittleEndianConvert` trait. (https://github.com/paritytech/wasmi/pull/290)

### Internal

- Refactor and clean up benchmarking code and added more benchmarks.
    - https://github.com/paritytech/wasmi/pull/299
    - https://github.com/paritytech/wasmi/pull/298
- Apply some clippy suggestions with respect ot `#[must_use]`. (https://github.com/paritytech/wasmi/pull/288)
- Improve Rust code formatting of imports.
- Improve debug impl of `ValueStack` so that only the live parts are printed.

## [0.10.0] - 2021-12-14

### Added

- Support for virtual memory usage on Windows 64-bit platforms.
    - Technically we now support the same set of platforms as the `region` crate does:
      https://github.com/darfink/region-rs#platforms

### Changed

- The `wasmi` and `wasmi-validation` crates now both use Rust edition 2021.
- The `README` now better teaches how to test and benchmark the crate.
- Updated `num-rational` from version `0.2.2` -> `0.4.0`.

### Deprecated

- Deprecated `MemoryInstance::get` method.
    - Users are recommended to use `MemoryInstance::get_value` or `MemoryInstance::get_into`
      methods instead.

### Removed

- Removed support for virtual memory on 32-bit platforms.
    - Note that the existing support was supposedly not more efficient than the `Vec`
      based fallback implementation anyways due to technical design.
- Removed the `core` crate feature that previously has been required for `no_std` builds.
    - Now users only have to specify `--no-default-features` for a `no_std` build.

### Internal

- Fully deploy GitHub Actions CI and remove deprecated Travis based CI. Added CI jobs for:
    - Testing on Linux, MacOS and Windows
    - Checking docs and dead links in docs.
    - Audit crate dependencies for vulnerabilities.
    - Check Wasm builds.
    - File test coverage reports to codecov.io.

## [0.9.1] - 2021-09-23

### Changed

- Added possibility to forward `reduced_stack_buffers` crate feature to `parity-wasm` crate.

### Internal

- Added a default `rustfmt.toml` configuration file.
- Fixed some warnings associated to Rust edition 2021.
    - Note: The crate itself remains in Rust edition 2018.

## [0.9.0] - 2021-05-27

### Changed

- Updated `parity-wasm` from verion `0.41` to `0.42`.
- Bumped `wasmi-validation` from version `0.3.1` to `0.4.0`.
