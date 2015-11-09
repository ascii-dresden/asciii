# ascii-invoicer rust rewrite


Here I try to rewrite the original [ascii-invoicer](http://github.com/ascii-dresden/ascii-invoicer) in Rust. Why? Because?

1. performence (rust instead of ruby)
2. deployment (one bin instead of n gems)
3. windows support (I hope)
4. fun

## TODO

* [x] basic project file management
    * [x] creating
    * [ ] git integration
    * [ ] template management
* [ ] project type (init/open/parse yml/index/etc)
    * [ ] template filling
* [ ] invoiceing
* [ ] pretty cli output
    * [ ] all sorts of list, csv, etc
* [ ] pdf export
* [ ] complete this TODO


The following is a printout of `ascii help` from the [original ascii-invoicer](http://github.com/ascii-dresden/ascii-invoicer).

```
ascii invoicer commands:
  ascii add NAMES                     # Git Integration
  ascii archive NAME                  # Move project to archive
  ascii calendar                      # Create a calendar file from all caterings named "invoicer.ics"
  ascii commit -m, --message=MESSAGE  # Git Integration
  ascii csv                           # Equal to: ascii list --all --csv --sort=index --filter event/date:2015
  ascii display NAMES                 # Shows information about a project in different ways
  ascii edit NAMES                    # Edit project
  ascii help [COMMAND]                # Describe available commands or one specific command
  ascii invoice NAMES                 # Create an invoice from project
  ascii list                          # List current Projects
  ascii log                           # Git Integration
  ascii new NAME                      # Creating a new project
  ascii offer NAMES                   # Create an offer from project
  ascii open NAMES                    # Open created documents
  ascii output                        # Equal to: ascii path --output
  ascii path                          # Return projects storage path
  ascii pull                          # Git Integration
  ascii push                          # Git Integration
  ascii search QUERY                  # Search everything, only one query currently
  ascii settings                      # View settings
  ascii status                        # Git Integration
  ascii templates                     # List or add templates
  ascii unarchive YEAR NAME           # reopen an archived project
  ascii version                       # Display version
  ascii whoami                        # Invoke settings --show manager_name

Options:
  -v, [--verbose], [--no-verbose]  # Change default in /home/hendrik/.ascii-invoicer.yml
```
