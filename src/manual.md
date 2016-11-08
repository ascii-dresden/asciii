The **official asciii Handbook**.

This user documentation has been ripped off straight from the [original
README](https://github.com/ascii-dresden/ascii-invoicer/), so please forgive if you find any
mistakes or unimplemented material,
please **[file an issue immediately](https://github.com/hoodie/asciii-rs/issues/new)**.

1. [Introduction](#introduction)
1. [Installation](#installation)
1. [Usage](#usage)
1. [File Format](#file-format)
1. [File Structure](#file-structure)

# ascii invoicer

## Introduction

The ascii-invoicer is a command-line tool that manages projects and stores them not in a database but in folders.
New projects can be created from templates and are stored in the working directory.
Projects can be archived, each year will have its own archive.
A project consists of a folder containing a yaml file describing it and a number of attached files,
such tex files.
Projects can contain products and personal.
You can create preliminary offers and invoices from your projects.

## Installation

### Archlinux User Repository

You find the AUR package [asciii-git](https://aur.archlinux.org/packages/asciii-git), or contact me personally about the inofficial repo with the binary packages.
Debian and Windows packages are on the way.

### Build Requirements

* rustc ≥ 1.11.0
* cargo
* optionally: for full feature completeness you also need cmake to build libgit2

Just run `cargo build --release` or `cargo install --path .`

### Requirements

* linux, mac osx, windows7+
* git for sync
* pdflatex/xelatex to produce documents
* an editor that can highlight yaml


## Usage

You should be able to learn everything there is to know about the command line interface by just typing in `asciii help`
Each of these sections starts with a list of commands.
Read the help to each command with `asciii help [COMMAND]` to find out about all parameters, especially *list* has quite a few of them.

### Get started with

```bash
asciii help [COMMAND]               # Describe available commands or one specific command
asciii list                         # List current Projects
asciii show NAMES                   # Shows information about a project in different ways
```

### Project Life-Cycle


```bash
asciii new NAME                     # Creating a new project
asciii edit NAMES                   # Edit project
asciii make NAME                    # Creates an Offer

asciii edit NAMES                   # Edit project
asciii make NAME                    # Creates an Invoice

asciii archive NAME                 # Move project to archive
asciii unarchive YEAR NAME          # reopen an archived project
asciii delete NAME                  # If you really have to
```

### GIT Features

```bash
asciii add NAMES
asciii commit
asciii pull / push
asciii cleanup
asciii status, log, diff, stash, pop
```

These commands behave similar to the original git commands.
The only difference is that you select projects just like you do with other ascii commands (see edit, display, offer, invoice).
Commit uses -m (like in git) but unlike git does not (yet) open an editor if you leave out the message.

#### CAREFUL:
These commands are meant as a convenience, they ARE NOT however a *complete* replacement for git!
You should always pull before you start working and push right after you are done in order to avoid merge conflicts.
If you do run into such problems go to storage directory `cd $(ascii path)` and resolve them using git.

Personal advice N°1: use `git pull --rebase`

Personal advice N°2: add this to your `.bash_aliases`:
`alias agit="git --git-dir=$(ascii path)/.git --work-tree=$(ascii path)"`

### More Details

The commands `asciii list` and `asciii display` (equals `ascii show`) allow to display all sorts of details from a project.
You can describe the exact field you wish to view via path like string.
To display the clients email for instance

```yaml
client:
  email: jon.doe@example.com
```

you pass in:

`
asciii show -d client/email` will display the clients email.
`asciii show -d invoice/date` will display the date of the invoice.

`asciii list --details` will add columns to the table.
For example try `asciii list --details client/email`.
As some fields are computed you have to use a different syntax to access them,
try for instance `asciii list -d ClientFullName`.
For a full list run `asciii list --computed`.


### Exporting
Currently `asciii` only supports csv export.
You can export the entire list of projects in a year with

```bash
asciii csv [year]     # Prints a CSV list of current year into CSV
asciii list --csv     # prints the same configuration (sorted, filtered) as `list` would.
```

You can pipe the csv into column (`asciii csv | column -ts\;`) to display the table in you terminal.

### Miscellaneous

```bash
asciii path      # Return projects storage path
asciii config -e # Edit configuration
asciii templates # List or add templates
asciii whoami    # Invoke settings --show user/name
asciii version   # Display version
```

## File Format

Every project consists of a project folder containig at least a `.yml` file.
[yaml](https://en.wikipedia.org/w/YAML)] is a structured file format, similar to json.
Infact: it is a superset of json.

### Document structure

A project file contains several sections, most of which you neither have to fill out manually nor right away be a valid project. The

#### Client

This describes the clients name and address. Please note that the field `client/address` mentions the clients name too.
The field `client/title` is used to determine the clients gender, so the first word must be one of the listed options.
`client/email` is not required though highly recommended.

```yaml
client:
  title: Mr # or: "Mrs", "Ms", "Herr", "Frau" - after which anything can follow
  first_name: John
  last_name:  Doe

  email:
  address: |
    John Doe
    Nöthnitzerstraße 46
    01187 Dresden
```

The event files can be filled

```yaml
event:
```


```yaml
offer:
```
```yaml
invoice:
```
```yaml
products:
```
```yaml
hours:
```

### Products

```yaml
  "Sekt  (0,75l)":
    amount: 4
    price: 6.0
    sold: 2
  "Belegte Brötchen":
    amount: 90
    price: 1.16
```

```yaml
cataloge:
  product: &kaffee       { name: Kaffee          , price: 2.5  , unit: 1l  }
products:
  *kaffee:
    amount: 60
```

## File Structure

Your config-file is located in ~/.asciii.yml but you can also access it using `asciii config --edit`.
The projects directory contains working, archive and templates. If you start with a blank slate you might want to put the templates folder into the storage folder (not well tested yet).

By default in your `path` folder you fill find:

```bash
caterings
├── archive
│   ├── 2013
│   │   ├── Foobar1
│   │   │   └── Foobar1.yml
│   │   └── Foobar2
│   │       ├── Foobar2.yml
│   │       └── R007 Foobar2 2013-02-11.tex
│   └── 2014
│       ├── canceled_foobar1
│       │   ├── A20141009-1 foobar.tex
│       │   └── foobar1.yml
│       ├── R029_foobar2
│       │   └── R029 foobar2 2014-09-10.tex
│       └── R036_foobar3
│           ├── foobar3.yml
│           └── R036 foobar3 2014-10-08.tex
├── templates
│   ├── default.yml.erb
│   └── document.tex.erb
└── working
    ├── Foobar1
    │   ├── A20141127-1 Foobar1.tex
    │   └── Foobar1.yml
    ├── Foobar2
    │   ├── A20141124-1 Foobar2.tex
    │   └── Foobar2.yml
    └── Foobar3
        ├── A20140325-1 Foobar3.tex
        ├── A20140327-1 Foobar3.tex
        ├── R008 Foobar3 2014-03-31.tex
        └── Foobar3.yml
```

## Aliases

// * `list`: `-l`, `l`, `ls`, `dir`
// * `display`: `-d`, `show`
// * `archive`: `close`
// * `invoice`: `-l`
// * `offer`: `-o`
// * `settings`: `config`
// * `log`: `history`

## Pro tips

1. Check out `repl asciii`!
You should copy [repl-file](src/repl/ascii) into ~/.repl/ascii and install rlwrap to take advantage of all the repl goodness such as autocompletion and history.

2. Check out `xclip`!
You can pipe the output of `ascii show` or `ascii show --csv` to xclip and paste to your email program or into a spreadsheet tool like libreoffice calc.


## Known Issues

Some strings may cause problems when rendering latex, e.g.
a client called `"ABC GmbH & Co. KG"`.
The `"&"` causes latex to fail, `\&"` bugs the yaml parser but `"\\&"` will do the trick.
asciii list -dCaterers -fCaterers:hendrik
