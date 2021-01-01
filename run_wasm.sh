#!/usr/bin/env sh 
wasmtime run --enable-all --env PWD=$PWD --env HOME=$HOME --dir $HOME --enable-threads asciii.wasm $*
