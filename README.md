# asciii-rs

The advanced but simple commandline interface to invoice invocation.

Here I rewrite the original [ascii-invoicer](http://github.com/ascii-dresden/ascii-invoicer) in Rust. Why? Because!

Next steps: finish [3.0.0](https://github.com/hoodie/asciii-rs/milestones/3.0.0) and release a beta for testing.

```
  DOCUMENTS:
  asciii display NAMES                 # Shows information about a project in different ways
  asciii help [COMMAND]                # Describe available commands or one specific command
  asciii invoice NAMES                 # Create an invoice from project
  asciii offer NAMES                   # Create an offer from project
  asciii open NAMES                    # Open created documents

  SUGAR:
  asciii csv                           # Equal to: ascii list --all --csv --sort=index --filter event/date:2015
  asciii calendar                      # Create a calendar file from all caterings named "invoicer.ics"

  GIT INTEGRATION:
  asciii add NAMES                     # Git Integration
  asciii pull                          # Git Integration
  asciii push                          # Git Integration
  asciii status                        # Git Integration
  asciii log                           # Git Integration
  asciii commit -m, --message=MESSAGE  # Git Integration

  asciii search QUERY                  # Search everything, only one query currently

  asciii version                       # Display version

Options:
  -v, [--verbose], [--no-verbose]  # Change default in $HOME/.asciii.yml
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
