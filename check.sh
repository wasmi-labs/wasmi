#!/usr/bin/env bash

set -eux

cd $(dirname $0)

cargo check --tests

cd -
