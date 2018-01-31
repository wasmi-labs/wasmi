#!/usr/bin/env bash

set -eux

cd $(dirname $0)

time cargo test
time cargo test --manifest-path=spec/Cargo.toml

cd -
