#!/bin/bash

export HFUZZ_RUN_ARGS="--max_file_size 2048"

rustup run nightly cargo hfuzz run hfuzz
