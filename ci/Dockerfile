FROM frolvlad/alpine-glibc
MAINTAINER \
[Mathias Kaufmann <steigr@ascii.coffee>]

RUN  apk add --no-cache --virtual .runtime-dep \
       git

ADD  target/release/asciii /bin/asciii

ENTRYPOINT ["asciii"]