# ascii-invoicer rust rewrite


Here I try to rewrite the original [ascii-invoicer](http://github.com/ascii-dresden/ascii-invoicer) in Rust. Why? Because?

1. performence (rust instead of ruby)
2. deployment (one bin instead of n gems)
3. windows support (I hope)
4. fun

## TODO Rewrite

* [ ] Milestone v3.0.0 *complete rewrite*
  * [x] basic project file management
      * [x] creating
      * [ ] git integration
        * [ ] stage1: pull, push, add, commit
        * [ ] stage2: remind of stale projects and sync backlog
      * [x] template management
      * [ ] command line control
  * [x] Validate
      * [x] Offer
      * [x] Invoice
      * [x] Archive, Payed, Calendar etc
      * [ ] Absofuckinglutely correct price/wages calculation
  * [ ] Output
      * [ ] Displaying
      * [ ] CSV and YAML output
        * [ ] full list equivalence
        * [ ] each project as csv (offer andor invoice)
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
  * [ ] pretty cli output
      * [ ] all sorts of list, csv, etc
  * [ ] complete this TODO
  * [ ] logging
  * [ ] Documentation!!!


## Vision v3.x.x

Features possible through rewrite:

* [ ] xdg basedir (less setup)
* [ ] much faster
  * [ ] caching
  * [ ] multi threading
* [ ] REST API with [rustless](http://rustless.org/) or so `asciii serve`
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
