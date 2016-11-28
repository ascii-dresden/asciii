# asciii

The **a**dvanced but **s**imple **c**ommandline **i**nterface for **i**nvoice **i**nvocation.

Here I rewrite the original [ascii-invoicer](http://github.com/ascii-dresden/ascii-invoicer) in Rust. Why? Because!

## Introduction

The ascii-invoicer is a command-line tool that manages projects and stores them not in a database but in a folder structure. New projects can be created from templates and are stored in a working directory. Projects can be archived, each year will have its own archive. A project consists of a folder containing a yaml file describing it and a number of attached files, such tex files. Projects can contain products and personal. You can create preliminary offers and invoices from your projects.


## Installation

### Archlinux
You can install the package `asciii-git` from the AUR.

### Using cargo
Just plain old `cargo install --git https://github.com/ascii-dresden/asciii` will do.

### Requirements

You need at least `rustc`, `cargo`, `cmake` and `git` to run this.


## Usage
After installation simply run `asciii` and it will present you with a list of possible subcommands. `asciii help list` will give you a comprehensive explanation of  what `asciii list` does.

You can also run `asciii doc` which will take you to the complete [online user and development documentation](http://ascii-dresden.github.io/asciii/doc).
Further information may be found in the [README of version 2.5](https://github.com/ascii-dresden/ascii-invoicer/blob/master/README.md)


### Logging

`asciii` uses Rusts [env_logger](http://doc.rust-lang.org/log/env_logger).
To enable logging you have to set `ASCIII_LOG=debug`.
Besides `debug`, you can also use `trace`, `warn` or `error`.
You can enable logging per-module logging like this: `ASCIII_LOG=storage=debug`.
Modules are all top-level files and folders in `src/`.
