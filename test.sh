#!/usr/bin/env bash

set -eux

EXTRA_ARGS=""

if [ -n "${TARGET-}" ]; then
    EXTRA_ARGS="--target=${TARGET} -- --test-threads=1"
fi

cd $(dirname $0)

time cargo test --all ${EXTRA_ARGS}

cd -
