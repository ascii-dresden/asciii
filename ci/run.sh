#!/bin/sh

set -ex

main() {
  echo "toolchain versions\n------------------"

  rustc -vV
  cargo -vV

  cargo check --target $TARGET --features full_tool
  cargo build --target $TARGET --features full_tool --release
  cargo test --target $TARGET --features full_tool
}

if [ -z "$SKIP_TESTS" ]; then
  main
fi
