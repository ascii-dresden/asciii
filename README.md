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
* [ ] Displaying
* [ ] Settings
    * [ ] Merge HOME_DIR config with defaults
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
  ascii list                          # List current Projects
  ascii edit NAMES                    # Edit project
  ascii unarchive YEAR NAME           # reopen an archived project
  ascii archive NAME                  # Move project to archive
  ascii templates                     # List or add templates
  ascii csv                           # Equal to: ascii list --all --csv --sort=index --filter event/date:2015

  ascii calendar                      # Create a calendar file from all caterings named "invoicer.ics"
  ascii display NAMES                 # Shows information about a project in different ways
  ascii help [COMMAND]                # Describe available commands or one specific command
  ascii invoice NAMES                 # Create an invoice from project
  ascii offer NAMES                   # Create an offer from project
  ascii new NAME                      # Creating a new project
  ascii open NAMES                    # Open created documents
  ascii output                        # Equal to: ascii path --output
  ascii path                          # Return projects storage path
  ascii whoami                        # Invoke settings --show manager_name

  ascii add NAMES                     # Git Integration
  ascii pull                          # Git Integration
  ascii push                          # Git Integration
  ascii status                        # Git Integration
  ascii log                           # Git Integration
  ascii commit -m, --message=MESSAGE  # Git Integration

  ascii search QUERY                  # Search everything, only one query currently
  ascii settings                      # View settings

  ascii version                       # Display version

Options:
  -v, [--verbose], [--no-verbose]  # Change default in /home/hendrik/.ascii-invoicer.yml
```
