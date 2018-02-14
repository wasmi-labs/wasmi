#!/usr/bin/env bash

set -eux

cd $(dirname $0)

time cargo test

cd -
