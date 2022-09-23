# Changelog

All notable changes to this project will be documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Additionally we have an `Internal` section for changes that are of interest to developers.

## [0.17.0] - UNRELEASED

### Added

- Added `Memory::data_and_store_mut` API inspired by Wasmtime's API. (https://github.com/paritytech/wasmi/pull/462)

### Changed

- Updated `wasmparser-nostd` dependency from `0.90.0` to `0.91.0`.
    - This improved performance of Wasm module compilation by ~10%.
- Updated `wasmi_core` from `0.3.0` to `0.4.0`.
- Optimized execution of several Wasm float to int conversion instructions. (https://github.com/paritytech/wasmi/pull/439)
    - We measured a performance improvement of 6000% or in other words those
      instructions are now 60 times faster than before.
    - This allowed us to remove the big `num-rational` dependency from `wasmi_core`
      for some nice speed-ups in compilation time of `wasmi` itself.
- Optimized `global.get` and `global.set` Wasm instruction execution. (https://github.com/paritytech/wasmi/pull/427)
    - This improved performance of those instructions by up to 17%.
- Optimized Wasm value stack emulation. (https://github.com/paritytech/wasmi/pull/459)
    - This improved performance of compute intense workloads by up to 23%.

### Internal

- Added automated continuous benchmarking to `wasmi`. (https://github.com/paritytech/wasmi/pull/422)
    - This allows us to have a more consistent overview over the performance of `wasmi`.
- Updated `criterion` benchmarking framework to version `0.4.0`.
- Reuse allocations during Wasm validation and translation:
     - Wasm validation and translation combined. (https://github.com/paritytech/wasmi/pull/462)
     - Wasm `br_table` translations. (https://github.com/paritytech/wasmi/pull/440)
- Enabled more useful `clippy` lints for `wasmi` and `wasmi_core`. (https://github.com/paritytech/wasmi/pull/438)
- Reorganized the `wasmi` workspace. (https://github.com/paritytech/wasmi/pull/466)

## [0.16.0] - 2022-08-30

### Changed

- Update `wasmparser-nostd` dependency from version `0.83.0` -> `0.90.0`.
  [**Link:**](https://github.com/paritytech/wasmi/commit/e9b0463817e277cd9daccca7e66e52e4fd147d8e)
    - This significantly improved `wasmi`'s Wasm parsing, validation and
      Wasm to `wasmi` bytecode translation performance.

### Internal

- Transition to the new `wasmparser::VisitOperator` API.
  [**Link**](https://github.com/paritytech/wasmi/commit/225c8224729661ea091e650e3278c4980bd1d405)
    - This again significantly improved `wasmi`'s Wasm parsing, validation and
      Wasm to `wasmi` bytecode translation performance by avoiding many
      unnecessary unpredictable branches in the process.

## [0.15.0] - 2022-08-22

### Fixed

- Fixed bugs found during fuzzing the translation phase of `wasmi`.
  [**Link**](https://github.com/paritytech/wasmi/commit/43d7037745a266ece2baccd9e78f7d983dacbb93)
- Fix `Read` trait implementation for `no_std` compilations.
  [**Link**](https://github.com/paritytech/wasmi/commit/baab359de955240fbb9c89ebbc369d7a6e6d8569)

### Changed

- Update to `wasmi_core` version `0.3.0`.
- Changed API of `wasmi::Config` in order to better reflect the API of
  `wasmtime::Config`.
- Refactor `Trap` type to be of pointer size which resulted in significant
  performance wins across the board especially for call intense work loads.
  [**Link**](https://github.com/paritytech/wasmi/commit/4a5d113a11a0f0020491c2cc08dd195a184256f0)

### Removed

- Removed support for virtual memory based Wasm linear memory.
  We decided to remove support since benchmarks showed that our current
  implementation actually regresses performance compared to our naive
  `Vec` based implementation.
  [**Link**](https://github.com/paritytech/wasmi/commit/10f8780a49b8cc8d8719e2b74089bf6848b8f982)

### Internal

- The `wasmi::Engine` now caches the bytes of the default linear memory for
  performance wins in `memory.store` and `memory.load` intense work loads.
  [**Link**](https://github.com/paritytech/wasmi/commit/c0df344e970bcdd4c6ce25f64265c854a1239220)
- The `wasmi` engine internals have been reorganized and modernised to improve
  performance on function call intense work loads. This resulted in performance
  improvements across the board.
  [**Link**](https://github.com/paritytech/wasmi/commit/d789570b51effb3a0c397c2d4ea1dc03c5d76918)
- The Wasm to `wasmi` bytecode translation now properly reuses heap allocations
  across function translation units which improved translation performance by
  roughly 10%.
  [**Link**](https://github.com/paritytech/wasmi/commit/71a913fc508841b3b7f799c8e4406e1e48feb046)
- Optimized the `wasmi` engine Wasm value stack implementation for significant
  performance wins across the board.
  [**Link**](https://github.com/paritytech/wasmi/commit/3886d9190e89d44a701ad5cbbda0c7457feba510)
- Shrunk size of some internal identifier types for minor performance wins.
  [**Link**](https://github.com/paritytech/wasmi/commit/3d544b82a5089ae4331024b1e6762dcb48a02898)
- Added initial naive fuzz testing for Wasm parsing, validation and Wasm to
  `wasmi` bytecode translation.
  [**Link**](https://github.com/paritytech/wasmi/commit/4d1f2ad6cbf07e61656185101bbd0bd5a941335f)

## [0.14.0] - 2022-07-26

### Added

- Added support for the following Wasm proposals:

    - [Import and export of mutable globals](https://github.com/WebAssembly/mutable-global)
    - [Non-trapping float-to-int conversions](https://github.com/WebAssembly/nontrapping-float-to-int-conversions)
    - [Sign-extension operators](https://github.com/WebAssembly/sign-extension-ops)
    - [Multi-value](https://github.com/WebAssembly/multi-value)

  We plan to support more Wasm proposals in the future.

### Changed

- Wasmi has been entirely redesigned and reimplemented.
  This work resulted in an entirely new API that is heavily inspired by
  the [Wasmtime API](https://docs.rs/wasmtime/0.39.1/wasmtime/),
  a brand new Wasm execution engine that performs roughly 30-40%
  better than the previous engine according to our benchmarks,
  the support of many Wasm proposals and Wasm parsing and validation
  using the battle tested [`wasmparser`](https://crates.io/crates/wasmparser)
  crate by the BytecodeAlliance.

  The new `wasmi` design allows to reuse the Wasm execution engine
  resources instead of spinning up a new Wasm execution engine for every
  function call.

  **Note:** If you plan to use `wasmi` it is of critical importance
  to compile `wasmi` using the following Cargo `profile` settings:

  ```toml
  [profile.release]
  lto = "fat"
  codegen-units = 1
  ```

  If you do not use these profile settings you might risk regressing
  performance of `wasmi` by up to 400%. You can read more about this
  issue [here](https://github.com/paritytech/wasmi/issues/339).

### Removed

- Removed support for resuming function execution.
  We may consider to add this feature back into the new engine.
  If you are a user of `wasmi` and want this feature please feel
  free to [open an issue](https://github.com/paritytech/wasmi/issues)
  and provide us with your use case.

## [0.13.2] - 2022-09-20

### Fixed

- Support allocating 4GB of memory (https://github.com/paritytech/wasmi/pull/452)

## [0.13.1] - 2022-09-20

**Note:** Yanked because of missing `wasmi_core` bump.

## [0.13.0] - 2022-07-25

**Note:** This is the last major release of the legacy `wasmi` engine.
          Future releases are using the new Wasm execution engines
          that are currently in development.
          We may consider to publish new major versions of this Wasm engine
          as `wasmi-legacy` crate.

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
