#!/bin/bash

export HFUZZ_RUN_ARGS="--max_file_size 2048"

die() { echo "$*"; exit 1; }

command -v wasm || die "spec interpreter 'wasm' is not on PATH";

rustup run nightly cargo hfuzz run hfuzz
