#!/usr/bin/env bash

set -ex

cd $(dirname $0)

time cargo test $CARGOFLAGS

cd -
