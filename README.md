[![crates.io link](https://img.shields.io/crates/v/wasmi.svg)](https://crates.io/crates/wasmi)
[![Build Status](https://travis-ci.org/paritytech/wasmi.svg?branch=master)](https://travis-ci.org/paritytech/wasmi)

# `wasmi`

WASM interpreter (previously lived in [parity-wasm](https://github.com/paritytech/parity-wasm))

Primary purpose of `wasmi` is to be used with [parity](https://github.com/paritytech/parity) (ethereum-like contracts in wasm) and with [Polkadot](https://github.com/paritytech/polkadot). However, `wasmi` is designed to be as flexible as possible and might be suited well for other purposes.

At the moment, the API is rather low-level (especially, in the part related to host functions). But some high-level API is on the roadmap.

# License

`wasmi` is primarily distributed under the terms of both the MIT
license and the Apache License (Version 2.0), at your choice.

See LICENSE-APACHE, and LICENSE-MIT for details.

# Build & Test

As `wasmi` contains a git submodule, you need to use `--recursive` for cloning or to checkout the submodule explicitly, otherwise the testing would fail.

```
git clone https://github.com/paritytech/wasmi.git --recursive
cd wasmi
cargo build
cargo test
```

# `no_std` support
This crate supports `no_std` environments.
Enable the `core` feature and disable default features:
```toml
[dependencies]
parity-wasm = {
	version = "0.31",
	default-features = false,
	features = "core"
}
```

The `core` feature requires the `core` and `alloc` libraries and a nightly compiler.
Also, code related to `std::error` is disabled.

Floating point operations in `no_std` use [`libm`](https://crates.io/crates/libm), which sometimes panics in debug mode (https://github.com/japaric/libm/issues/4).
So make sure to either use release builds or avoid WASM with floating point operations, for example by using [`deny_floating_point`](https://docs.rs/wasmi/0.4.0/wasmi/struct.Module.html#method.deny_floating_point).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `wasmi` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
