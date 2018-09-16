# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # TODO Update this to build the artifacts that matter to you
    cross rustc --bin asciii --target $TARGET --features full_tool --release -- -C lto

	ls target/$TARGET/release/

    # TODO Update this to package the right artifacts
    if test "$TARGET" = "x86_64-pc-windows-gnu"
    then
        cp target/$TARGET/release/asciii.exe $stage/
    else
        cp target/$TARGET/release/asciii $stage/
    fi

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
