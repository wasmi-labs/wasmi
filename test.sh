#!/usr/bin/env bash

set -eux

EXTRA_ARGS=""

if [ -n "${TARGET-}" ]; then
    # Tests build in debug mode are prohibitively
    # slow when ran under emulation so that
    # e.g. Travis CI will hit timeouts.
    EXTRA_ARGS="--release --target=${TARGET}"
    export RUSTFLAGS="--cfg debug_assertions"
fi

cd $(dirname $0)

time cargo test --all ${EXTRA_ARGS}

cd -
