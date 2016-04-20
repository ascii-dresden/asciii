# asciii-rs

The advanced but simple commandline interface to invoice invocation.

Here I rewrite the original [ascii-invoicer](http://github.com/ascii-dresden/ascii-invoicer) in Rust. Why? Because!

## Status

This has evolved from a technical experiment into a full blown rewrite,
I'm currently working on restoring full functionality and release the first version with the number [3.0.0](https://github.com/hoodie/asciii-rs/milestones/3.0.0).

This should hopefully run smoothly on the same platforms that ascii2 currently runs on.


## Build

Just plain old `cargo build` will do.

### Features

Currently a few things are only build-features.
Use `cargo build --features lints` to build with [clippy](https://github.com/Manishearth/rust-clippy).
Use `cargo build --features nightly` to use nightly features:
  * currently only system allocators

Use `cargo build --features debug` to enable debug prints, this is sorta deprecated, it is only necessary to debug config, because is initialized by lazy_static before even the logger is set up.

### Release
To build a release ready version use `cargo build --release`.


## Logging

asciii uses rust [env_logger](http://doc.rust-lang.org/log/env_logger).
To enable logging you have to set `ASCIII_LOG=debug`.
Besides `debug`, you can also use `trace`, `warn` or `error`.
You can enable logging per-module logging like this: `ASCIII_LOG=storage=debug`.
Modules are all top-level files and folders in `src/`.

## Technical TODO

* [ ] test on windows and mac (https://github.com/japaric/rust-everywhere)
* [x] build on Raspberry Pi
* [ ] see if you can `#[inline]` to improve perfomance
* [ ] break up code into crates
  * [ ] config
  * [ ] yaml helpers macro-DSL
  * [ ] templating
  * [ ] utilities etc

### make use if these crates (optional)

* rayon / simple_parallel
* sparkline
* xdg / xdg-basedir
* env_logger
* filetime, for make like features
* itertools
* multimap
* cool faces
* open
* notify-rust
* colored

## Side Effects

While working on this I had the chance to also contribute to a number of crates that asciii depends on.
These include:

* [prettytables-rs](https://github.com/phsym/prettytable-rs/)
* [yaml-rust](https://github.com/chyh1990/yaml-rust)
* [currency](https://github.com/Tahler/rust-lang-currency)
* [open-rs](https://github.com/byron/open-rs)
