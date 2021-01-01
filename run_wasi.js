#!/usr/bin/env node --experimental-wasi-unstable-preview1
'use strict';

const fs = require('fs');

const { WASI } = require('wasi');

const wasi = new WASI({
  args: process.argv.slice(1),
  env: process.env,
  preopens: {
    '/Users/hendrik/': '/Users/hendrik/'
  }
});
const importObject = { wasi_snapshot_preview1: wasi.wasiImport };

(async () => {
  const wasm = await WebAssembly.compile(fs.readFileSync('./asciii.wasm'));
  const instance = await WebAssembly.instantiate(wasm, importObject);

  wasi.start(instance);
})();
