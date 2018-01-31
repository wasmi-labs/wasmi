#!/usr/bin/env bash

set -eux

cd $(dirname $0)

cargo check --tests
cargo check \
	--manifest-path=spec/Cargo.toml \
	--tests

cd -
