#!/bin/sh

set -ex

export PATH=/travis-rust/bin:$PATH
export LD_LIBRARY_PATH=/travis-rust/lib:$LD_LIBRARY_PATH
export CARGO_TARGET_DIR=`pwd`/target
export CARGO_HOME=`pwd`/target/cargo-home

exec sh ci/run.sh
