# Wasmi Migration Guide: v1 to v2

This document lists all changes that require action from users upgrading from Wasmi `1.x` to Wasmi `2.0`.

For the highlights of the v2 release refer to the [changelog](../CHANGELOG.md).

## Table of Contents

- [Reference Types: `Ref` and `RefType`](#reference-types-ref-and-reftype)
- [`ResourceLimiter` API](#resourcelimiter-api)
- [Fuel Metering](#fuel-metering)
- [The `Handle` Trait](#the-handle-trait)
- [Crate Features](#crate-features)
- [WASI](#wasi)
- [CLI](#cli)
- [Rust Edition](#rust-edition)

## Reference Types: `Ref` and `RefType`

Wasmi v2 introduces the `Ref` and `RefType` types and makes `wasmi::Table` operate on them
instead of `Val` and `ValType`. This mirrors Wasmtime's API and shrinks table elements from
64-bit (or even 128-bit with the `simd` crate feature enabled) to flat 32-bit references,
reducing the memory consumption of `wasmi::Table` instances by a factor of 2-4x.

### `Ref<T>` was renamed to `Nullable<T>`

The generic nullable-reference wrapper that v1 called `Ref<T>` is now called `Nullable<T>`.
The name `Ref` still exists in v2 but refers to an entirely different type, so this rename
does not produce compile errors at all use sites and requires attention:

```rust
// v1
let func_ref: wasmi::Ref<wasmi::Func> = wasmi::Ref::Val(func);
let null: wasmi::Ref<wasmi::Func> = wasmi::Ref::Null;

// v2
let func_ref: wasmi::Nullable<wasmi::Func> = wasmi::Nullable::Val(func);
let null: wasmi::Nullable<wasmi::Func> = wasmi::Nullable::Null;
```

Consequently the `Val::FuncRef` and `Val::ExternRef` variants now hold `Nullable<Func>` and
`Nullable<ExternRef>` respectively.

### The new `Ref` type

`Ref` is now a non-generic `enum` over all Wasm reference kinds, and `RefType` is its type:

```rust
pub enum Ref {
    Func(Nullable<Func>),
    Extern(Nullable<ExternRef>),
}

pub enum RefType {
    Func,
    Extern,
}
```

Useful constructors and queries: `Ref::null(ty)`, `Ref::default_for_ty(ty)`, `Ref::ty()`,
`Ref::is_null()`, `Ref::is_non_null()`, `Ref::as_func()`, `Ref::as_extern()`.
`Ref` also has `From` impls for `Nullable<Func>` and `Nullable<ExternRef>`.

### `TableType` takes a `RefType`

```rust
// v1
let ty = TableType::new(ValType::FuncRef, 10, Some(100));

// v2
let ty = TableType::new(RefType::Func, 10, Some(100));
```

The same applies to `TableType::new64`.

### `wasmi::Table` operates on `Ref`

`Table::new`, `Table::get`, `Table::set`, `Table::grow` and `Table::fill` now take or return
`Ref` instead of `Val`:

```rust
// v1
let table = Table::new(&mut store, ty, Val::FuncRef(Ref::Null))?;
let value: Option<Val> = table.get(&store, 0);
table.set(&mut store, 0, Val::FuncRef(Ref::Val(func)))?;

// v2
let table = Table::new(&mut store, ty, Ref::null(RefType::Func))?;
let value: Option<Ref> = table.get(&store, 0);
table.set(&mut store, 0, Ref::Func(Nullable::Val(func)))?;
```

Passing a `Ref` whose `RefType` does not match the table's element type is still an error,
but it is now expressed in terms of `RefType` rather than `ValType`.

## `ResourceLimiter` API

The `ResourceLimiter` API has been cleaned up.

`memory_grow_failed` and `table_grow_failed` now receive the concrete error type of the failed
operation and return a `Result` so that a limiter can turn a failed growth into a trap:

```rust
// v1
fn memory_grow_failed(&mut self, error: &LimiterError) {}
fn table_grow_failed(&mut self, error: &LimiterError) {}

// v2
fn memory_grow_failed(&mut self, error: &MemoryError) -> Result<(), LimiterError> { Ok(()) }
fn table_grow_failed(&mut self, error: &TableError) -> Result<(), LimiterError> { Ok(()) }
```

`MemoryError` and `TableError` are available from `wasmi::errors`.

`LimiterError` is now a single variant `enum` making its use much simpler:

```rust
// v2
pub enum LimiterError {
    ResourceLimiterDeniedAllocation,
}
```

Furthermore, `StoreLimits` now properly enforces panics if `trap_on_grow_failure` was enabled.
Users relying on the previous (silently ignoring) behavior have to adjust their
`StoreLimitsBuilder::trap_on_grow_failure` configuration accordingly.

## Fuel Metering

Wasmi v2 replaces the old fuel metering that was tied to Wasmi's internal bytecode with a
**stable** fuel metering that is tied to the input Wasm bytecode instead. While less precise,
fuel costs now stay relatively stable across Wasmi versions and use the same technique that is
also used in Wasmtime.

**Absolute fuel numbers therefore differ from Wasmi `1.x`.** Applications that persist, compare
or hard-code fuel amounts across a Wasmi upgrade must re-calibrate them.

Fuel costs are now configurable:

- `Config::operator_cost` selects how much fuel a single Wasm operator costs.
- `Config::fuel_cost` customizes the dynamic fuel costs via `CustomFuelCosts`, e.g. the fuel
  required per translated or validated byte and the amount of bytes copied per unit of fuel by
  `memory.{grow,copy,fill,init}` and `table.{grow,copy,fill,init}`.

## The `Handle` Trait

Wasmi's internal `Handle` trait no longer has a `From` super-trait. This fixes an issue for
users with conflicting `From` impls for Wasmi handles such as `Func`, `Global`, `Memory` and
`Table`. No action is required unless you relied on those blanket `From` conversions.

## Crate Features

Wasmi v2 has a significantly extended set of crate features, several of them enabled by default:

```toml
default = ["stable", "std", "wat", "validate", "memory64", "auto-dispatch"]
```

Builds using `--no-default-features` have to opt back into what they require. The features that
are new in v2 and most relevant when upgrading:

| Feature | Default | Purpose |
|:--|:--|:--|
| `validate` | ✅ | Wasm validation support. Disabling saves ~200-300kB of binary artifact size but requires full control over the Wasm inputs. |
| `memory64` | ✅ | Support for the Wasm `memory64` proposal. |
| `auto-dispatch` | ✅ | Use tail-call based operator dispatch only on targets known to support it, otherwise fall back to portable dispatch. |
| `stable` | ✅ | Restrict Wasmi to stable Rust. Suppresses `unstable`. |
| `unstable` | ❌ | Enables nightly-only `rustc` features, concretely Rust's `become` keyword to enforce tail calls in operator dispatch and the `rust-preserve-none` ABI for execution handlers. |
| `portable-dispatch` | ❌ | Force the portable (loop-based) operator dispatch. Takes precedence over `auto-dispatch`. |
| `indirect-dispatch` | ❌ | Use less encoding space for Wasmi's internal IR at the cost of execution performance. |
| `deterministic` | ❌ | Wasm deterministic profile support. |
| `debug` | ❌ | Richer `Debug` output for Wasmi's internal operators. Off by default to reduce binary artifact size. |
| `libm` | ❌ | Enforce `libm` usage for floating point operations. |

The `wasmi_c_api_impl` and `wasmi_c_api` crates now forward most of these features to `wasmi`.
This also fixes a bug where the `std` feature was not forwarded.

## WASI

`wasmi_wasi::add_to_externals` is a new and more efficient alternative to `add_to_linker` and
should be preferred where possible. It fills a `Vec<wasmi::Extern>` for a concrete `Module`
instead of populating a `Linker`:

```rust
wasmi_wasi::add_to_externals(&mut store, &module, &mut externals, |ctx| &mut ctx.wasi)?;
let instance = Instance::new(&mut store, &module, &externals)?;
```

## CLI

The Wasmi CLI command has been renamed from `wasmi_cli` to just `wasmi` and, similar to
Wasmtime's CLI, now requires a sub-command:

```console
# v1
wasmi_cli <WASM_FILE> --invoke <FUNC_NAME> [<FUNC_ARGS>]

# v2
wasmi run <WASM_FILE> --invoke <FUNC_NAME> [<FUNC_ARGS>]
wasmi wast <WAST_FILE>
```

Additionally, `v128` argument types are now rejected in order to mirror Wasmtime CLI's behavior.

The `wasmi_cli` crate gained the `wasi`, `wast`, `run`, `portable-dispatch` and
`indirect-dispatch` crate features. The first three are enabled by default.

## Rust Edition

Wasmi now uses Rust edition 2024 in the entire workspace. This does not affect users since
Wasmi's MSRV did not change.
