# asciii-rs

The advanced but simple commandline interface to invoice invocation.

Here I try to rewrite the original [ascii-invoicer](http://github.com/ascii-dresden/ascii-invoicer) in Rust. Why? Because!

1. performence (rust instead of ruby)
2. deployment (one bin instead of n gems)
3. windows support (I hope)
4. fun and learning more rust

## TODO Rewrite

* [ ] Milestone v3.0.0 *complete rewrite*
  * [x] basic project file management
      * [x] creating
      * [ ] git integration
        * [x] stage1: pull, push, add, commit
        * [ ] stage2: remind of stale projects and sync backlog
      * [x] template management
      * [ ] command line control
        * [ ] fill things like payed_date
  * [ ] Listing
      * [ ] pretty cli output
          * [ ] all sorts of list, csv, etc
  * [x] Validate
      * [x] Offer
      * [x] Invoice
      * [x] Archive, Payed, Calendar etc
      * [ ] Access dynamic values as if they were static
        * things like invoice number
      * [ ] Absofuckinglutely correct price/wages calculation
  * [ ] Output
      * [ ] Displaying
      * [ ] CSV and YAML output
        * [ ] full `list` equivalence
        * [ ] each project as csv (offer and/or invoice)
      * [ ] Pdf export (Invoice, Offer, Checklist)
      * [ ] make like behavior:
        * [ ] `open` triggers `invoice` or `offer`
        * [ ] tell if project file is newer then pdf
  * [ ] Search!!
      * [ ] keyword search
      * [ ] filtering and stats
  * [x] Settings
      * [x] Merge HOME_DIR config with defaults
  * [x] project type (init/open/parse yml/index/etc)
      * [x] template filling
  * [ ] complete this TODO
  * [ ] logging
  * [ ] Documentation!!!
  * [ ] Be rustacious!
  * [ ] Releasing
    * [ ] Travis
    * [ ] Doc = dev + usage
    * [ ] auto build Debian and arch packages


## Vision v3.x.x

Features possible through rewrite:

* [ ] xdg basedir (less setup)
* [ ] much faster
  * [ ] caching
  * [ ] multi threading
* [ ] REST API with [rustless](http://rustless.org/) or something alike `asciii serve`
* [ ] Statistics (plot over time, etc)
* [ ] platform support
  * [ ] debian/archlinux packages
  * [ ] windows build
* [ ] pure library build
  * [ ] GUI
* [ ] third format ( machine readable/writable for GUI)
* [ ] markdown export (for git[hub|lab])


The following is a printout of `ascii help` from the [original ascii-invoicer](http://github.com/ascii-dresden/ascii-invoicer).

```
original ascii invoicer commands:
  MINIMUM
  ascii new NAME                      # Creating a new project
  ascii edit NAMES                    # Edit project
  ascii settings                      # View settings
  ascii list                          # List current Projects
  ascii archive NAME                  # Move project to archive
  ascii unarchive YEAR NAME           # reopen an archived project
  ascii templates                     # List or add templates

  DOCUMENTS:
  ascii display NAMES                 # Shows information about a project in different ways
  ascii help [COMMAND]                # Describe available commands or one specific command
  ascii invoice NAMES                 # Create an invoice from project
  ascii offer NAMES                   # Create an offer from project
  ascii open NAMES                    # Open created documents
  ascii output                        # Equal to: ascii path --output
  ascii path                          # Return projects storage path
  ascii whoami                        # Invoke settings --show manager_name

  SUGAR:
  ascii csv                           # Equal to: ascii list --all --csv --sort=index --filter event/date:2015
  ascii calendar                      # Create a calendar file from all caterings named "invoicer.ics"

  GIT INTEGRATION:
  ascii add NAMES                     # Git Integration
  ascii pull                          # Git Integration
  ascii push                          # Git Integration
  ascii status                        # Git Integration
  ascii log                           # Git Integration
  ascii commit -m, --message=MESSAGE  # Git Integration

  ascii search QUERY                  # Search everything, only one query currently

  ascii version                       # Display version

Options:
  -v, [--verbose], [--no-verbose]  # Change default in /home/hendrik/.ascii-invoicer.yml
```

## Technical TODO

* [ ] test on windows and mac
* [ ] build on raspberry py
* [ ] see if you can `#[inline]` to improve perfomance
* [ ] break up code into crates
  * [ ] config
  * [ ] yaml helpers
  * [ ] templating
  * [ ] utilities etc

### make use if these crates (optional)

* rayon / simple_parallel
* sparkline
* xdg / xdg-basedir
* env_logger
* filetime
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
