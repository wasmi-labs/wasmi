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

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `wasmi` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
