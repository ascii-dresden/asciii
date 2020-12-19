#!/usr/bin/env sh 
wasmtime run --enable-all --env PWD=/home/hendrik/code/ascii/asciii --env HOME=/home/hendrik --dir /home/hendrik --enable-threads asciii.wasm $*
