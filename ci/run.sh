#!/bin/sh

set -ex

echo "toolchain versions\n------------------"

rustc -vV
cargo -vV

cargo build --target $TARGET
cargo build --target $TARGET --release

if [ -z "$SKIP_TESTS" ]; then
  cargo test --target $TARGET
fi
