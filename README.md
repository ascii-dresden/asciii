# asciii

[![build](https://img.shields.io/github/actions/workflow/status/ascii-dresden/asciii/ci.yml?branch=main)](https://github.com/ascii-dresden/asciii/actions?query=workflow%3A"Continuous+Integration")
[![license](https://img.shields.io/crates/l/asciii.svg)](https://crates.io/crates/asciii/)
[![crates.io](https://img.shields.io/crates/d/asciii.svg)](https://crates.io/crates/asciii)
[![version](https://img.shields.io/crates/v/asciii.svg)](https://crates.io/crates/asciii/)
[![documentation](https://docs.rs/asciii/badge.svg)](https://docs.rs/asciii/)

The **a**dvanced but **s**imple **c**ommandline **i**nterface for **i**nvoice **i**nvocation.

NoSql, blockchain based, serverless, cross-platform project management tool.

Here I rewrite the original [ascii-invoicer](http://github.com/ascii-dresden/ascii-invoicer) in Rust. Why? Because!

## Introduction

The ascii-invoicer is a command-line tool that manages projects and stores them not in a database but in a folder structure. New projects can be created from templates and are stored in a working directory. Projects can be archived, each year will have its own archive. A project consists of a folder containing a yaml file describing it and a number of attached files, such tex files. Projects can contain products and personal. You can create preliminary offers and invoices from your projects.


## Installation

To use \(experimental\) features use the `--features` flag.
Please check the `Cargo.toml` for an up-to-date list of features.
To get a full cli tool build with the feature `full_tool`.

Minimum required version of rustc: `1.32`

### Archlinux

You can install the package `asciii-git` from the AUR.

### macOS

You can install asciii via homebrew.

```sh
$ brew tap ascii-dresden/formulae
$ brew install asciii
```

### Using cargo

Just plain old `cargo install --git https://github.com/ascii-dresden/asciii` or `cargo install asciii` will do.

### Requirements

You need at least `rustc`, `cargo`, `cmake`, `git` and `zlib1g-dev` to run this. If you want to use `webapp` feature make sure `yarn` is installed.

## Development

**Hint!** After the first build you can removed the content of `build.rs`s `fn main()` during dev for significantly improved compile times :D

### Ubuntu 16.04

We recommend installing [rustup](https://github.com/rust-lang-nursery/rustup.rs) with the following command:

```sh
$ curl https://sh.rustup.rs -sSf | sh
```

This installs `rustc`, `cargo`, `rustup` and other standard tools.

#### Ubuntu Requirements

You need at least `cmake` and `zlib1g-dev` to run this.

## Usage
After installation simply run `asciii` and it will present you with a list of possible subcommands. `asciii help list` will give you a comprehensive explanation of  what `asciii list` does.

You can also run `asciii doc` which will take you to the complete [online user and development documentation](http://ascii-dresden.github.io/asciii/).
Further information may be found in the [README of version 2.5](https://github.com/ascii-dresden/ascii-invoicer/blob/master/README.md)

### web server

`ASCIII_LOG=debug cargo +nightly run --no-default-features --features server --bin asciii-server --release`
`ASCIII_LOG=debug cargo +nightly run --no-default-features --features webapp --bin asciii-server --release`

### Logging

`asciii` uses Rusts [env_logger](http://doc.rust-lang.org/log/env_logger).
To enable logging you have to set `ASCIII_LOG=debug`.
Besides `debug`, you can also use `trace`, `warn` or `error`.
You can enable logging per-module logging like this: `ASCIII_LOG=storage=debug`.
Modules are all top-level files and folders in `src/`.

### Localization
When you build with the `"localize"` feature then the `lang/default.pot` should be updated automatically during the build. If you have `gettext` installed you can run 

```
msgmerge -U lang/de.po lang/default.pot
```

to update the german local file `lang/de.po`. Now you only need to update any empty field and check the file in as well.

## Features
asciii comes with different sets of feature configurations, most of which are there to speed up development time. By default most useful features are turned on, such as `webapp`, `shell` and `localization`. You can build asciii with a reduced features set by passing these arguments to `cargo build`:

### `--no-default-features --features full_tool` 
* everything except the `webserver`

### `--no-default-features --features mini_tool` 
* no `shell`, `git_statuses`, `localization`, `meta` or `serde`, just `cli` and `document_export`

Please check `Cargo.toml` for all features.
