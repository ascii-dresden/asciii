#!/bin/bash

FEATURES=(
    shell
    localization
    git_statuses
    meta_store
    cli
    serialization
    deserialization
    version_string
    serde_base
)

# for FEATURE in ${FEATURES[@]}
# do
#     echo cargo check --lib  --no-default-features --features $FEATURE
#     cargo  check --lib  --no-default-features --features $FEATURE
# done

for FEATURE in ${FEATURES[@]}
do
    echo cargo check --bin asciii --no-default-features --features $FEATURE
    cargo  check --bin asciii --no-default-features --features cli\ $FEATURE
done

