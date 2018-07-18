# Configuration


## the config command

`asciii config --help`

```
asciii-config 
Show and edit your config

USAGE:
    asciii config [FLAGS] [OPTIONS]

FLAGS:
    -d, --default     Show default config
    -e, --edit        Edit your config
    -h, --help        Prints help information
    -i, --init        Create config file.
    -l, --location    Show the location of the config file
    -V, --version     Prints version information

OPTIONS:
        --editor <editor>        Override the configured editor
        --set-root <set root>    set the root folder in the config
    -s, --show <show>            Show a specific config value
```

## the config file

`vim ~/.asciii.yml` || `asciii config --edit`

```yaml
---
user:
  name: Hendrik Sollich
  editor: #gvim -p

list:
  colors:    true
  verbose:   true
  sort:      index
  gitstatus: true
  extra_details: [OurBad]

path: ~/ascii/
output_path: ~/ascii/output

dirs:
  storage: caterings

template: default # default template

```
