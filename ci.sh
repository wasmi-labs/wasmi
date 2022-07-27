#!/bin/bash

echo "starting local CI ..." &&
echo "    Formatting ..." &&
cargo +nightly fmt &&
echo "    Building ..." &&
cargo build --workspace &&
echo "    Building no_std ..." &&
cargo build --workspace --exclude wasmi_cli --no-default-features --target thumbv7em-none-eabi &&
echo "    Clippy ..." &&
cargo clippy --workspace -- -D warnings &&
echo "    Docs ..." &&
cargo doc --workspace --all-features --no-deps --document-private-items &&
echo "    Testing Spec ..." &&
cargo test --release --quiet &&
echo "    Testing Package ..." &&
cargo test --package wasmi --quiet &&
echo "-------------------------" &&
echo "CI passed"
