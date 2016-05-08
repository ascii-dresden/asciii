//! The ascii invoicer handbook
//!
//! This user documentation has been ripped off straight from the [original
//! README](https://github.com/ascii-dresden/ascii-invoicer/), so please forgive if you find any
//! mistakes or unimplemented material,
//! please **[file an issue immediately](https://github.com/hoodie/asciii-rs/issues/new)**.
//!
//! # ascii invoicer
//!
//! ## Introduction
//!
//! The ascii-invoicer is a command-line tool that manages projects and stores them not in a database but in folders.
//! New projects can be created from templates and are stored in the working directory.
//! Projects can be archived, each year will have its own archive.
//! A project consists of a folder containing a yaml file describing it and a number of attached files,
//! such tex files.
//! Projects can contain products and personal.
//! You can create preliminary offers and invoices from your projects.
//!
//! ## Installation
//!
//! Get a current binary from github (TODO) or build it for your platform.
//!
//! ### DIY Building
//!
//! * rustc ≥ 1.5.0
//! * cargo
//! * optionally: for full feature completeness you also need everything to build libgit2 and libssl2
//!
//! ### Requirements
//!
//! * linux, mac osx, windows7+
//! * git for sync
//! * pdflatex/xelatex to produce documents
//! * an editor that can highlight yaml
//!
//! Just run `cargo build --release`, and copy the resulting `target/release/asciii` binary into your $PATH somewhere .
//!
//!
//! ## Usage
//!
//! Each of these sections starts with a list of commands.
//! Read the help to each command with `ascii help [COMMAND]` to find out about all parameters, especially *list* has quite a few of them.
//!
//! ### Get started with
//!
//! ```bash
//! asciii help [COMMAND]                # Describe available commands or one specific command
//! asciii list                          # List current Projects
//! asciii show NAMES                    # Shows information about a project in different ways
//! ```
//!
//! ### Project Life-Cycle
//!
//! ```bash
//! asciii new NAME                      # Creating a new project
//! asciii edit NAMES                    # Edit project
// asciii offer NAMES                   # Create an offer from project
// asciii invoice NAMES                 # Create an invoice from project
//! asciii archive NAME                  # Move project to archive
//! asciii unarchive YEAR NAME           # reopen an archived project
//! ```
//!
//! ### GIT Features
//!
//! ```bash
//! asciii add NAMES
//! asciii commit
//! asciii pull
//! asciii push
//! asciii status, log, diff
//! ```
//!
//! These commands behave similar to the original git commands.
//! The only difference is that you select projects just like you do with other ascii commands (see edit, display, offer, invoice).
//! Commit uses -m (like in git) but unlike git does not (yet) open an editor if you leave out the message.
//!
//! #### CAREFUL:
//! These commands are meant as a convenience, they ARE NOT however a *complete* replacement for git!
//! You should always pull before you start working and push right after you are done in order to avoid merge conflicts.
//! If you do run into such problems go to storage directory `cd $(ascii path)` and resolve them using git.
//!
//! Personal advice N°1: use `git pull --rebase`
//!
//! Personal advice N°2: add this to your `.bash_aliases`:
//! `alias agit="git --git-dir=$(ascii path)/.git --work-tree=$(ascii path)"`
//!
//! ### More Details
//!
//! The commands `ascii list` and `ascii display` (equals `ascii show`) allow to display all sorts of details from a project.
//! You can define sort of path through the document structure to the key you want to be displayed.
//! `ascii show -d client/email` will display the clients email.
//! `ascii show -d invoice/date` will display the date of the invoice.
//!
//! `ascii list --details` will add columns to the table.
//! For example try `ascii list --details client/fullname client/email`
//!
//!
//! ### Exporting
//!
//! ```bash
//! asciii calendar # Create a calendar file from all caterings named "invoicer.ics"
//! asciii csv      # Prints a CSV list of current year into CSV
//! ```
//! You can pipe the csv into column (`ascii csv | column -ts\;`) to display the table in you terminal.
//!
//! ### Miscellaneous
//!
//! ```bash
//! asciii path      # Return projects storage path
//! asciii settings  # View settings
//! asciii templates # List or add templates
//! asciii whoami    # Invoke settings --show manager_name
//! asciii version   # Display version
//! ```
//!
//! ## Filesstructure
//!
//! Your config-file is located in ~/.ascii-invoicer.yml but you can also access it using `ascii settings --edit` or even `ascii edit --settings`.
//! The projects directory contains working, archive and templates. If you start with a blank slate you might want to put the templates folder into the storage folder (not well tested yet).
//!
//! By default in your `path` folder you fill find:
//!
//! ```bash
//! caterings
//! ├── archive
//! │   ├── 2013
//! │   │   ├── Foobar1
//! │   │   │   └── Foobar1.yml
//! │   │   └── Foobar2
//! │   │       ├── Foobar2.yml
//! │   │       └── R007 Foobar2 2013-02-11.tex
//! │   └── 2014
//! │       ├── canceled_foobar1
//! │       │   ├── A20141009-1 foobar.tex
//! │       │   └── foobar1.yml
//! │       ├── R029_foobar2
//! │       │   └── R029 foobar2 2014-09-10.tex
//! │       └── R036_foobar3
//! │           ├── foobar3.yml
//! │           └── R036 foobar3 2014-10-08.tex
//! ├── templates
//! │   ├── default.yml.erb
//! │   └── document.tex.erb
//! └── working
//!     ├── Foobar1
//!     │   ├── A20141127-1 Foobar1.tex
//!     │   └── Foobar1.yml
//!     ├── Foobar2
//!     │   ├── A20141124-1 Foobar2.tex
//!     │   └── Foobar2.yml
//!     └── Foobar3
//!         ├── A20140325-1 Foobar3.tex
//!         ├── A20140327-1 Foobar3.tex
//!         ├── R008 Foobar3 2014-03-31.tex
//!         └── Foobar3.yml
//! ```
//!
//! ## Aliases
//!
// * `list`: `-l`, `l`, `ls`, `dir`
// * `display`: `-d`, `show`
// * `archive`: `close`
// * `invoice`: `-l`
// * `offer`: `-o`
// * `settings`: `config`
// * `log`: `history`
//!
//! ## Pro tips
//!
//! 1. Check out `repl asciii`!
//! You should copy [repl-file](src/repl/ascii) into ~/.repl/ascii and install rlwrap to take advantage of all the repl goodness such as autocompletion and history.
//!
//! 2. Check out `xclip`!
//! You can pipe the output of `ascii show` or `ascii show --csv` to xclip and paste to your email program or into a spreadsheet tool like libreoffice calc.
//!
//!
//! ## Known Issues
//!
//! Some strings may cause problems when rendering latex, e.g.
//! a client called `"ABC GmbH & Co. KG"`.
//! The `"&"` causes latex to fail, `\&"` bugs the yaml parser but `"\\&"` will do the trick.
//!
//!
//!
//!
//!
