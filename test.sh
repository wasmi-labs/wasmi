#!/usr/bin/env bash

set -eux

cd $(dirname $0)

# Make sure that the testsuite submodule is checked out.
git submodule update --init wasmi/tests/spec/testsuite

time cargo test

cd -
