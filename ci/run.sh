#!/bin/sh

set -ex

main() {
  echo "toolchain versions\n------------------"

  rustc -vV
  cargo -vV

  cargo build --target $TARGET --features travis_compatible
  cargo test --target $TARGET --features travis_compatible
}

if [ -z "$SKIP_TESTS" ]; then
  main
fi
