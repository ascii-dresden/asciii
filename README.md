# asciii-rs

The advanced but simple commandline interface to invoice invocation.

Here I rewrite the original [ascii-invoicer](http://github.com/ascii-dresden/ascii-invoicer) in Rust. Why? Because!

Next steps: finish [3.0.0](https://github.com/hoodie/asciii-rs/milestones/3.0.0) and release a beta for testing.

```
  DOCUMENTS:
 sc ascii display NAMES                 # Shows information about a project in different ways
  ascii help [COMMAND]                # Describe available commands or one specific command
  ascii invoice NAMES                 # Create an invoice from project
  ascii offer NAMES                   # Create an offer from project
  ascii open NAMES                    # Open created documents

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
