#!/bin/bash

echo "starting local CI ..." &&
echo "    Formatting ..." &&
cargo +nightly fmt &&
echo "    Building ..." &&
cargo +stable build --workspace &&
echo "    Building no_std ..." &&
cargo +stable build --workspace --exclude wasmi_cli --exclude wasmi_wasi --no-default-features --target thumbv7em-none-eabi &&
echo "    Clippy ..." &&
cargo +stable clippy --workspace -- -D warnings &&
echo "    Docs ..." &&
cargo +stable doc --workspace --all-features --no-deps --document-private-items &&
echo "    Testing Spec ..." &&
cargo +stable test --release --quiet &&
echo "    Testing Package ..." &&
cargo +stable test --package wasmi --quiet &&
echo "-------------------------" &&
echo "CI passed"
