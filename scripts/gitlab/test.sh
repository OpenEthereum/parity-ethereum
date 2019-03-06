#!/bin/bash
echo "________Running test.sh________"
set -e # fail on any error
set -u # treat unset variables as error

FEATURES="json-tests,ci-skip-tests"
OPTIONS="--release"

echo "________Running Parity Full Test Suite________"
time cargo test $OPTIONS --features "$FEATURES" --locked --all --target $CARGO_TARGET
