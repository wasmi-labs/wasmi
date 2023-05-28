# Changelog

All notable changes to this project will be documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Additionally we have an `Internal` section for changes that are of interest to developers.

Dates in this file are formattes as `YYYY-MM-DD`.

## [`0.30.0`] - 2023-05-28

### Changed

- Optimized `wasmi` bytecode memory consumption. (https://github.com/paritytech/wasmi/pull/718)
  - This reduced the memory consumption of `wasmi` bytecode by organizing the instructions
    into so-called instruction words, effectively reducing the amount of bytes required per
    `wasmi` instruction 16 bytes to 8 bytes.
    There was an experiment with 4 bytes but experiments confirmed that 8 bytes per instruction
    word was the sweetspot for `wasmi` execution and translation performance.
  - This did not affect execution performance too much but we saw performance improvements
    for translation from Wasm to `wasmi` bytecode by roughly 15-20%.
- Optimized `call` and `return_call` for Wasm module internal calls. (https://github.com/paritytech/wasmi/pull/724)
  - `wasmi` bytecode now differentiates between calls to Wasm module internal functions
    and imported functions which allows the `wasmi` bytecode executor to perform the common
    internal calls more efficiently.
  - This led to an execution performance improvement across the board but especially for
    call intense workloads of up to 30% in some test cases.

## [`0.29.0`] - 2023-03-20

### Added

- Added support for `extended-const` Wasm proposal. (https://github.com/paritytech/wasmi/pull/707)
- Added fuel consumption modes. (https://github.com/paritytech/wasmi/pull/706)
  - This allows eager and lazy fuel consumption modes to be used which
    mainly affects bulk operations such as `table.copy` and `memory.grow`.
    Eager fuel consumption always consumes fuel before a bulk operation for the
    total amount independent of success or failure of the operation whereras
    lazy fuel consumption only consumes fuel for successful executions.

### Changed

- Normalize fuel costs of all instructions. (https://github.com/paritytech/wasmi/pull/705)
  - With this change most instructions cost roughly 1 fuel upon execution.
    This is more similar to how Wasmtime deals with fuel metered instruction costs.
    Before this change `wasmi` tried to have fuel costs that more closely mirror
    the computation intensity of the respective instruction according to benchmarks.

## [`0.28.0`] - 2023-03-01

### Added

- Added support for the `tail-call` Wasm proposal. (https://github.com/paritytech/wasmi/pull/683)
- Added support for `Linker` defined host functions. (https://github.com/paritytech/wasmi/pull/692)
  - Apparently this PR introduced some performance wins for the Wasm target according to our tests.
    This information shall be taken with a grain of salt since we are not sure why those performance
    improvement occured since the PR's functionality is orthogonal to Wasm engine performance.
  - Required precursor refactoring PR: https://github.com/paritytech/wasmi/pull/681

[`tail-call`]: https://github.com/WebAssembly/tail-call

### Changed

- The `wasmi_wasi` crate now more closely mirrors the `wasmtime_wasi` crate API. (https://github.com/paritytech/wasmi/pull/700)

### Internal

- Refactor the `wasmi` Wasm engine to handle Wasm calls and returns in its core. [(#694)]
  - This improved performance of Wasm function calls significantly at the cost of host function call performance.
  - Also this seemed to have impacts Wasm target performance quite positively, too.
- The `Store` now handles Wasm functions and host functions separately. (https://github.com/paritytech/wasmi/pull/686)
  - This allows to store Wasm functions into the `StoreInner` type which was an important
    step towards the major refactoring in [(#694)]
  - It was expected that host function call performance would degrade by this PR but our tests
    actually showed that the opposite was true and Wasm target performance was improved overall.
- Introduce `ValueStackPtr` abstraction for the `wasmi` engine core. (https://github.com/paritytech/wasmi/pull/688)
  - This change significantly improved performance especially on the Wasm target according to our tests.
- Optimize `memory.{load,store}` when reading or writing single bytes. (https://github.com/paritytech/wasmi/pull/689)
  - The performance wins were more modest than we hoped but still measurable.
- Use `StoreContextMut<T>` instead of `impl AsContextMut` in the `wasmi` engine core. (https://github.com/paritytech/wasmi/pull/685)
  - This is a simple refactoring with the goal to make the Rust compiler have a simpler job at
    optimizing certain functions in the engine's inner workings since `StoreContextMut` provides
    more information to the compiler.

[(#694)]: https://github.com/paritytech/wasmi/pull/694

## [`0.27.0`] - 2023-02-14

### Added

- Added support for fuel metering in the `wasmi` CLI. (https://github.com/paritytech/wasmi/pull/679)
  - Users can now specify an amount of fuel via `--fuel N` to commit for the execution.
    Upon success the `wasmi` CLI will display the total amount of consumed and remaining fuel.

### Fixed

- Fixed a bug that `wasmi` CLI did not preserve the WASI exit status. (https://github.com/paritytech/wasmi/pull/677)
  - Thanks to [YAMAMOTO Takashi @yamt](https://github.com/yamt) for reporting the issue.
- The `wasmi` CLI now properly displays exported functions if `--invoke x` was provided and `x` was not found. (https://github.com/paritytech/wasmi/pull/678)
- Applied minor fixes to `Config` docs. (https://github.com/paritytech/wasmi/pull/673)

### Changed

- Defer charging fuel for costly bulk `memory` and bulk `table` operations. (https://github.com/paritytech/wasmi/pull/676)
  - Note that the check to assert that enough fuel is provided for these costly
    operation is still happening before the actual computation and only the charging
    is deferred to after a successful run. The reason behind this is that all the affected
    operations fail fast and therefore should not cost lots of fuel in case of failure.

## [`0.26.1`] - 2023-02-13

### Fixed

- Fixed a bug where resuming a resumable function from a host function with more outputs than
  inputs could lead to incorrect behavior or runtime panics. (https://github.com/paritytech/wasmi/pull/671)
    - Thanks to [Pierre Krieger (tomaka)](https://github.com/tomaka) for reporting and crafting an initial minimal test case.

## [`0.26.0`] - 2023-02-11

### Added

- `wasmi` CLI: Add WASI support. (https://github.com/paritytech/wasmi/pull/597)
  - Big shoutout to [Onigbinde Oluwamuyiwa Elijah](https://github.com/OLUWAMUYIWA) for contributing this to `wasmi`!
- Add built-in support for fuel metering. (https://github.com/paritytech/wasmi/pull/653)
  - This allows to control the runtime of Wasm executions in a deterministic fasion
    effectively avoiding the halting problem by charging for executed instructions.
    Not using the feature will not affect the execution efficiency of `wasmi` for users.
- Add `Pages::checked_sub` method. (https://github.com/paritytech/wasmi/pull/660)
- Add `Func::new` constructor. (https://github.com/paritytech/wasmi/pull/662)
  - This allows to create `Func` instances from closures without statically known types.

### Changed

- Update to `wasmparser-nostd` version `0.100.1`. (https://github.com/paritytech/wasmi/pull/666)

### Internal

- Clean up and reorganization of the `wasmi_cli` crate. (https://github.com/paritytech/wasmi/pull/655)
- Refactoring of internal host call API. (https://github.com/paritytech/wasmi/pull/664)

## [`0.25.0`] - 2023-02-04

### Added

- Added `Config::floats` option to enable or disable Wasm float operators during Wasm validation.
- `Trap::downcast_mut` and `Trap::downcast` methods. (https://github.com/paritytech/wasmi/pull/650)
  - This helps users to downcast into `T: HostError`.
- Added `WasmType` impls for `FuncRef` and `ExternRef` types. (https://github.com/paritytech/wasmi/pull/642)
  - This allows `FuncRef` and `ExternRef` instances to be used in `TypedFunc` parameters and results.

### Removed

- Removed from `From` impls from `wasmparser-nostd` types to `wasmi` types.
  - For example `From<wasmparser::FuncType> for wasmi::FuncType` got removed.

### Changed

- Update the `wasmparser-nostd` dependency from version `0.91.0` to `0.99.0`. (https://github.com/paritytech/wasmi/pull/640)
- The `Trap` type is no longer `Clone`. (https://github.com/paritytech/wasmi/pull/650)

### Internal

- Resolved plenty of technical debt and improved structure of the `wasmi` crate.
  - PRs: https://github.com/paritytech/wasmi/pull/648, https://github.com/paritytech/wasmi/pull/647, https://github.com/paritytech/wasmi/pull/646, https://github.com/paritytech/wasmi/pull/645, https://github.com/paritytech/wasmi/pull/644, https://github.com/paritytech/wasmi/pull/641

## [`0.24.0`] - 2023-01-31

### Added

- Added support for the [`bulk-memory`] Wasm proposal. (https://github.com/paritytech/wasmi/pull/628)
- Added support for the [`reference-types`] Wasm proposal. (https://github.com/paritytech/wasmi/pull/635)
- Added `ValueType::{is_ref, is_num`} methods. (https://github.com/paritytech/wasmi/pull/635)
- Added `Value::{i32, i64, f32, f64, externref, funcref}` accessor methods to `Value`.

[`bulk-memory`]: https://github.com/WebAssembly/bulk-memory-operations
[`reference-types`]: https://github.com/WebAssembly/reference-types

### Fixed

- Fix a bug with `Table` and `Memory` imports not respecting the current size. (https://github.com/paritytech/wasmi/pull/635)
  - This sometimes led to the problem that valid `Table` and `Memory` imports
    could incorrectly be rejected for having an invalid size for the subtype check.
  - This has been fixed as part of the [`reference-types`] Wasm proposal implementation.

### Changed

- Use more references in places to provide the compiler with more optimization opportunities. (https://github.com/paritytech/wasmi/pull/634)
  - This led to a speed-up across the board for Wasm targets of about 15-20%.
- Move the `Value` type from `wasmi_core` to `wasmi`. (https://github.com/paritytech/wasmi/pull/636)
  - This change was necessary in order to support the [`reference-types`] Wasm proposal.
- There has been some consequences from implementing the [`reference-types`] Wasm proposal which are listed below:
  - The `Value` type no longer implements `Copy` and `PartialEq`.
  - The `From<&Value> for UntypedValue` impl has been removed.
  - Remove some `From` impls for `Value`.
  - Moved some `Display` impls for types like `FuncType` and `Value` to the `wasmi_cli` crate.
  - Remove the `try_into` API from the `Value` type.
    - Users should use the new accessor methods as in the Wasmtime API.

### Internal

- Update `wast` dependency from version `0.44` to `0.52`. (https://github.com/paritytech/wasmi/pull/632)
- Update the Wasm spec testsuite to the most recent commit: `3a04b2cf9`
- Improve error reporting for the internal Wasm spec testsuite runner.
  - It will now show proper span information in many more cases.

## [`0.23.0`] - 2023-01-19

> **Note:** This is the Wasmtime API Compatibility update.

### Added

- Add `Module::get_export` method. (https://github.com/paritytech/wasmi/pull/617)

### Changed

- Removed `ModuleError` export from crate root. (https://github.com/paritytech/wasmi/pull/618)
  - Now `ModuleError` is exported from `crate::errors` just like all the other error types.
- Refactor and cleanup traits underlying to `IntoFunc`. (https://github.com/paritytech/wasmi/pull/620)
  - This is only the first step in moving closer to the Wasmtime API traits.
- Mirror Wasmtime API more closely. (https://github.com/paritytech/wasmi/pull/615, https://github.com/paritytech/wasmi/pull/616)
  - Renamed `Caller::host_data` method to `Caller::data`.
  - Renamed `Caller::host_data_mut` method to `Caller::data_mut`.
  - Add `Extern::ty` method and the `ExternType` type.
  - Rename `ExportItem` to `ExportType`:
    - Rename the `ExportItem::kind` method to `ty` and return `ExternType` instead of `ExportItemKind`.
    - Remove the no longer used `ExportItemKind` entirely.
  - The `ExportsIter` now yields items of the new type `Export` instead of pairs of `(&str, Extern)`.
  - Rename `ModuleImport` to `ImportType`.
    - Rename `ImportType::item_type` to `ty`.
    - Rename `ImportType::field` to `name`.
    - Properly forward `&str` lifetimes in `ImportType::{module, name}`.
    - Replace `ModuleImportType` by `ExternType`.
  - Add new convenience methods to `Instance`:
    - `Instance::get_func`
    - `Instance::get_typed_func`
    - `Instance::get_global`
    - `Instance::get_table`
    - `Instance::get_memory`
  - Rename getters for querying types of runtime objects:
    - `Func::func_type` => `Func::ty`
    - `Global::global_type` => `Global::ty`
    - `Table::table_type` => `Table::ty`
    - `Memory::memory_type` => `Memory::ty`
    - `Value::value_type` => `Value::ty`
  - Remove `Global::value_type` getter.
    - Use `global.ty().content()` instead.
  - Remove `Global::is_mutable` getter.
    - Use `global.ty().mutability().is_mut()` instead.
  - Rename `Mutability::Mutable` to `Var`.
  - Add `Mutability::is_mut` getter.
    - While this API is not included in Wasmtime it is a useful convenience method.
  - Rename `TableType::initial` method to `minimum`.
  - Rename `Table::len` method to `size`.
  - `Table` and `TableType` now operate on `u32` instead of `usize` just like in Wasmtime.
    - This affects `Table::{new, size, set, get, grow}` methods and `TableType::{new, minimum, maximum}` methods and their users.

## [`0.22.0`] - 2023-01-16

### Added

- Add missing `TypedFunc::call_resumable` API. (https://github.com/paritytech/wasmi/pull/605)
  - So far resumable calls were only available for the `Func` type.
    However, there was no technical reason why it was not implemented
    for `TypedFunc` so this mirrored API now exists.
  - This also cleans up rough edges with the `Func::call_resumable` API.

### Changed

- Clean up the `wasmi_core` crate API. (https://github.com/paritytech/wasmi/pull/607, https://github.com/paritytech/wasmi/pull/608, https://github.com/paritytech/wasmi/pull/609)
  - This removes plenty of traits from the public interface of the crate
    which greatly simplifies the API surface for users.
  - The `UntypedValue` type gained some new methods to replace functionality
    that was provided in parts by the removed traits.
- The `wasmi` crate now follows the Wasmtime API a bit more closely. (https://github.com/paritytech/wasmi/pull/613)
  - `StoreContext` new methods:
    - `fn engine(&self) -> &Engine`
    - `fn data(&self) -> &T` 
  - `StoreContextMut` new methods:
    - `fn engine(&self) -> &Engine`
    - `fn data(&self) -> &T` 
    - `fn data_mut(&mut self) -> &mut T`
  - Renamed `Store::state` method to `Store::data`.
  - Renamed `Store::state_mut` method to `Store::data_mut`.
  - Renamed `Store::into_state` method to `Store::into_data`.
### Internal

- The `Store` and `Engine` types are better decoupled from their generic parts. (https://github.com/paritytech/wasmi/pull/610, https://github.com/paritytech/wasmi/pull/611)
  - This might reduce binary bloat and may have positive effects on the performance.
    In fact we measured significant performance improvements on the Wasm target.

## [`0.21.0`] - 2023-01-04

### Added

- Add support for resumable function calls. (https://github.com/paritytech/wasmi/pull/598)
  - This feature allows to resume a function call upon encountering a host trap.
- Add support for concurrently running function executions using a single `wasmi` engine.
  - This feature also allows to call Wasm functions from host functions. (https://github.com/paritytech/wasmi/pull/590)
- Add initial naive WASI support for `wasmi` using the new `wasmi_wasi` crate. (https://github.com/paritytech/wasmi/pull/557)
  - Special thanks to [Onigbinde Oluwamuyiwa Elijah](https://github.com/OLUWAMUYIWA) for carrying the WASI support efforts!
  - Also thanks to [Yuyi Wang](https://github.com/Berrysoft) for testing and improving initial WASI support. (https://github.com/paritytech/wasmi/pull/592, https://github.com/paritytech/wasmi/pull/571, https://github.com/paritytech/wasmi/pull/568)
  - **Note:** There is ongoing work to integrate WASI support in `wasmi_cli` so that the `wasmi` CLI will then
              be able to execute arbitrary `wasm-wasi` files out of the box in the future.
- Add `Module::imports` that allows to query Wasm module imports. (https://github.com/paritytech/wasmi/pull/573, https://github.com/paritytech/wasmi/pull/583)

### Fixed

- Fix a bug that imported linear memories and tables were initialized twice upon instantiation. (https://github.com/paritytech/wasmi/pull/593)
- The `wasmi` CLI now properly hints for file path arguments. (https://github.com/paritytech/wasmi/pull/596)

### Changed

- The `wasmi::Trap` type is now more similar to Wasmtime's `Trap` type. (https://github.com/paritytech/wasmi/pull/559)
- The `wasmi::Store` type is now `Send` and `Sync` as intended. (https://github.com/paritytech/wasmi/pull/566)
- The `wasmi` CLI now prints exported functions names if the function name CLI argument is missing. (https://github.com/paritytech/wasmi/pull/579)
- Improve feedback when running a Wasm module without exported function using `wasmi` CLI. (https://github.com/paritytech/wasmi/pull/584)

## [`0.20.0`] - 2022-11-04

### Added

- Contribution documentation about fuzz testing. (https://github.com/paritytech/wasmi/pull/529)

### Removed

- Removed some deprecated functions in the `wasmi_core` crate. (https://github.com/paritytech/wasmi/pull/545)

### Fixed

- Fixed a critical performance regression introduced in Rust 1.65. (https://github.com/paritytech/wasmi/pull/518)
  - While the PR's main job was to clean up some code it was found out that it
    also fixes a critical performance regression introduced in Rust 1.65.
  - You can read more about this performance regression [in this thread](https://github.com/rust-lang/rust/issues/102952).

### Changed

- Fixed handling of edge cases with respect to Wasm linear memory. (https://github.com/paritytech/wasmi/pull/449)
  - This allows for `wasmi` to properly setup and use linear memory instances of up to 4GB.
- Optimize and improve Wasm instantiation. (https://github.com/paritytech/wasmi/pull/531)
- Optimize `global.get` of immutable non-imported globals. (https://github.com/paritytech/wasmi/pull/533)
  - Also added a benchmark test for this. (https://github.com/paritytech/wasmi/pull/532)

### Internal

- Implemented miscellaneous improvements to our CI system.
  - https://github.com/paritytech/wasmi/pull/539 (and more)
- Miscellaneous clean ups in `wasmi_core` and `wasmi`'s executor.
  - https://github.com/paritytech/wasmi/pull/542 https://github.com/paritytech/wasmi/pull/541
  https://github.com/paritytech/wasmi/pull/508 https://github.com/paritytech/wasmi/pull/543

## [`0.19.0`] - 2022-10-20

### Fixed

- Fixed a potential undefined behavior as reported by the `miri` tool
  with respect to its experimental stacked borrows. (https://github.com/paritytech/wasmi/pull/524)

### Changed

- Optimized Wasm to `wasmi` translation phase by removing unnecessary Wasm
  validation type checks. (https://github.com/paritytech/wasmi/pull/527)
    - Speedups were in the range of 15%.
- `Linker::instantiate` now takes `&self` instead of `&mut self`. (https://github.com/paritytech/wasmi/pull/512)
  - This allows users to easily predefine a linker and reused its definitions
    as shared resource.
- Fixed a bug were `Caller::new` was public. (https://github.com/paritytech/wasmi/pull/514)
  - It is now a private method as it was meant to be.
- Optimized `TypedFunc::call` at slight cost of `Func::call`. (https://github.com/paritytech/wasmi/pull/522)
  - For many parameters and return values the measured improvements are in the range of 25%.
    Note that this is only significant for a large amount of host to Wasm calls of small functions.

### Internal

- Added new benchmarks and cleaned up benchmarking code in general.
  - https://github.com/paritytech/wasmi/pull/525
  https://github.com/paritytech/wasmi/pull/526
  https://github.com/paritytech/wasmi/pull/521
- Add `miri` testing to `wasmi` CI (https://github.com/paritytech/wasmi/pull/523)

## [`0.18.1`] - 2022-10-13

### Changed

- Optimize for common cases for branch and return instructions.
  (https://github.com/paritytech/wasmi/pull/493)
    - This led to up to 10% performance improvement according to our benchmarks
      in some cases.
- Removed extraneous `S: impl AsContext` generic parameter from `Func::typed` method.
- Make `IntoFunc`, `WasmType` and `WasmRet` traits publicly available.
- Add missing impl for `WasmRet` for `Result<T, Trap> where T: WasmType`.
    - Without this impl it was impossible to provide closures to `Func::wrap`
      that returned `Result<T, Trap>` where `T: WasmType`, only `Result<(), Trap>`
      or `Result<(T,), Trap>` was possible before.

### Internal

- Added `wasmi_arena` crate which defines all internally used arena data structures.
  (https://github.com/paritytech/wasmi/pull/502)
- Update to `clap 4.0` in `wasmi_cli`. (https://github.com/paritytech/wasmi/pull/498)
- Many more improvements to our internal benchmarking CI.
  (https://github.com/paritytech/wasmi/pull/494, https://github.com/paritytech/wasmi/pull/501,
  https://github.com/paritytech/wasmi/pull/506, https://github.com/paritytech/wasmi/pull/509)

## [`0.18.0`] - 2022-10-02

### Added

- Added Contibution Guidelines and Code of Conduct to the repository. (https://github.com/paritytech/wasmi/pull/485)

### Changed

- Optimized instruction dispatch in the `wasmi` interpreter.
  (https://github.com/paritytech/wasmi/pull/478, https://github.com/paritytech/wasmi/pull/482)
  - This yielded combined speed-ups of ~20% across the board.
  - As a side effect we also refactored the way we compute branching offsets
    at Wasm module compilation time which improved performance of Wasm module
    compilation by roughly 5%.

### Internal

- Our CI now also benchmarks `wasmi` when ran inside Wasmtime as Wasm.
  (https://github.com/paritytech/wasmi/pull/483, https://github.com/paritytech/wasmi/pull/487)
  - This allows us to optimize `wasmi` towards Wasm performance more easily in the future.

## [`0.17.0`] - 2022-09-23

### Added

- Added `Memory::data_and_store_mut` API inspired by Wasmtime's API. (https://github.com/paritytech/wasmi/pull/448)

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

## [`0.16.0`] - 2022-08-30

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

## [`0.15.0`] - 2022-08-22

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

## [`0.14.0`] - 2022-07-26

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

## [`0.13.2`] - 2022-09-20

### Fixed

- Support allocating 4GB of memory (https://github.com/paritytech/wasmi/pull/452)

## [`0.13.1`] - 2022-09-20

**Note:** Yanked because of missing `wasmi_core` bump.

## [`0.13.0`] - 2022-07-25

**Note:** This is the last major release of the legacy `wasmi` engine.
          Future releases are using the new Wasm execution engines
          that are currently in development.
          We may consider to publish new major versions of this Wasm engine
          as `wasmi-legacy` crate.

### Changed

- Update dependency: `wasmi-validation v0.4.2 -> v0.5.0`

## [`0.12.0`] - 2022-07-24

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

## [`0.11.0`] - 2022-01-06

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

## [`0.10.0`] - 2021-12-14

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

## [`0.9.1`] - 2021-09-23

### Changed

- Added possibility to forward `reduced_stack_buffers` crate feature to `parity-wasm` crate.

### Internal

- Added a default `rustfmt.toml` configuration file.
- Fixed some warnings associated to Rust edition 2021.
    - Note: The crate itself remains in Rust edition 2018.

## [`0.9.0`] - 2021-05-27

### Changed

- Updated `parity-wasm` from verion `0.41` to `0.42`.
- Bumped `wasmi-validation` from version `0.3.1` to `0.4.0`.
