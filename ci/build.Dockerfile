FROM scorpil/rust

# this image never gets pushed
# so run prepare in minimal steps
# allow to retain most of them
RUN apt-get update
RUN apt-get install -y cmake 
RUN apt-get install -y git
RUN apt-get install -y zlib1g-dev
RUN apt-get install -y binutils