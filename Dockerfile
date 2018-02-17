FROM ubuntu:16.04 AS builder

# can be overridden for special toolchain\
# yields completly new image when changed
ARG TOOLCHAIN=nightly-2018-02-15
ARG TARGET=x86_64-unknown-linux-musl

# install build deps
RUN export DEBIAN_FRONTEND=noninteractive \
 && apt-get update \
 && apt-get install -y \
        build-essential \
        cmake \
        curl \
        file \
        git \
        musl-dev \
        musl-tools \
        libsqlite-dev \
        libssl-dev \
        pkgconf \
        sudo \
        xutils-dev \
        patchelf \
        \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/* \
 && useradd rust --user-group --create-home --shell /bin/bash --groups sudo \
 # allow sudo without a password.
 && echo '%sudo   ALL=(ALL:ALL) NOPASSWD:ALL' | install -D -m 0600 /dev/stdin /etc/sudoers.d/nopasswd

USER rust
RUN  mkdir -p /home/rust/libs /home/rust/src \
 # set the default target
 &&  printf '[build]\ntarget = "%s"' "$TARGET" \
     | install -D -o rust /dev/stdin /home/rust/.cargo/config

ENV PATH=/home/rust/.cargo/bin:/usr/local/musl/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- -y --default-toolchain $TOOLCHAIN \
 && rustup target add $TARGET

WORKDIR /home/rust/libs

RUN echo "Building OpenSSL" \
 && VERS=1.0.2l \
 && curl -O https://www.openssl.org/source/openssl-$VERS.tar.gz \
 && tar xvzf openssl-$VERS.tar.gz && cd openssl-$VERS \
 && env CC=musl-gcc ./Configure no-shared no-zlib -fPIC --prefix=/usr/local/musl linux-x86_64 \
 && env C_INCLUDE_PATH=/usr/local/musl/include/ make depend \
 && make && sudo make install \
 && cd .. && rm -rf openssl-$VERS.tar.gz openssl-$VERS \
 && echo "Building zlib" \
 && VERS=1.2.11 \
 && cd /home/rust/libs \
 && curl -LO http://zlib.net/zlib-$VERS.tar.gz \
 && tar xzf zlib-$VERS.tar.gz && cd zlib-$VERS \
 && CC=musl-gcc ./configure --static --prefix=/usr/local/musl \
 && make && sudo make install \
 && cd .. && rm -rf zlib-$VERS.tar.gz zlib-$VERS

ENV OPENSSL_DIR=/usr/local/musl/ \
    OPENSSL_INCLUDE_DIR=/usr/local/musl/include/ \
    DEP_OPENSSL_INCLUDE=/usr/local/musl/include/ \
    OPENSSL_LIB_DIR=/usr/local/musl/lib/ \
    OPENSSL_STATIC=1 \
    PKG_CONFIG_ALLOW_CROSS=true \
    PKG_CONFIG_ALL_STATIC=true

WORKDIR /home/rust/src

ADD Cargo.toml Cargo.toml
ADD Cargo.lock Cargo.lock

ARG BUILDMODE=dev
RUN printf 'fn main() {}' | tee build.rs | install -D /dev/stdin src/main.rs \
 && trap 'rm src/main.rs build.rs' EXIT \
 && BUILDMODE=$(bash -c 'echo ${BUILDMODE/dev/}') \
 && cargo build ${BUILDMODE:+--$BUILDMODE} --all-features

ADD build.rs build.rs 
ADD lang lang
ADD src src
ADD tests tests

ENV BINARY=server

RUN BUILDMODE=$(bash -c 'echo ${BUILDMODE/dev/}') \
 && cargo build ${BUILDMODE:+--$BUILDMODE} --features $BINARY

ADD examples examples

RUN BUILDMODE=$(bash -c 'echo ${BUILDMODE/dev/}') \
 && cargo build ${BUILDMODE:+--$BUILDMODE} --example $BINARY --features $BINARY

# alter interpreter
RUN set -x; BUILDMODE=$(bash -c 'echo ${BUILDMODE/dev/debug}') \
 && ARTIFACT="$(find target/$TARGET/$BUILDMODE/ -name $BINARY -type f | head -1)" \
 && patchelf --set-interpreter /lib/ld-musl-x86_64.so.1 "$ARTIFACT"

# strip binary
RUN set -x; BUILDMODE=$(bash -c 'echo ${BUILDMODE/dev/debug}') \
 && ARTIFACT="$(find target/$TARGET/$BUILDMODE/ -name $BINARY -type f | head -1)" \
 && strip -s "$ARTIFACT"

# provide artifacts at same place
RUN set -x; BUILDMODE=$(bash -c 'echo ${BUILDMODE/dev/debug}') \
 && ARTIFACT="$(find target/$TARGET/$BUILDMODE/ -name $BINARY -type f | head -1)" \
 && sudo install -D -m 0755 -o root -g root "$ARTIFACT" /dist/bin/$BINARY

FROM alpine:3.7 AS compressor
RUN  apk add --no-cache upx
COPY --from=builder /dist /dist
# for some reason upxd binary segfaults, so leave it for now
# ARG UPX_ARGS=-6
# RUN find /dist -type f | xargs -n1 -t upx ${UPX_ARGS}

FROM alpine:3.7 AS composer
RUN  apk add --no-cache tini
RUN  adduser -h / -S -D asciii
# files for tini
RUN  install -D -m 0755 -o 0 -g 0 /sbin/tini                                                /stage1/bin/tini
RUN  install -D -m 0755 -o 0 -g 0 /lib/ld-musl-x86_64.so.1                                  /stage1/lib/ld-musl-x86_64.so.1
RUN  grep -e root -e asciii -e nobody /etc/passwd | install -D -o 0 -g 0 -m 0644 /dev/stdin /stage1/etc/passwd
# files for $BINARY
RUN  install -D -m 0755 -o 0 -g 0 /lib/libz.so.1     /stage2/lib/libz.so.1
COPY --from=compressor /dist/                        /stage2/

FROM scratch
USER asciii
VOLUME /.asciii_projects
ENV  ROCKET_ENV=production \
     ROCKET_PORT=8080 \
     ROCKET_ADDRESS=0.0.0.0 \
     ROCKET_SECRET_KEY=GVH5hYGVw1xTQyUHtx8MrDkuUmYSBJFbMsGwdSREQwk= \
     CORS_ALLOWED_ORIGINS="http://localhost:8080"

COPY --from=composer /stage1 /
ENTRYPOINT ["tini","--"]

COPY --from=composer /stage2 /
CMD ["server"]
