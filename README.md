# dpemr or dpr ![Tests](https://github.com/gokberkkocak/dpemr/actions/workflows/ci.yml/badge.svg)
## Distributed parallel for experiment management in Rust 

## Usage

```
dpr 0.1.0
Distributed parallel for experiment management in Rust

USAGE:
    dpr [FLAGS] [OPTIONS] --config <config> <SUBCOMMAND>

FLAGS:
    -d, --debug      Debug mode enables verbose printing
    -h, --help       Prints help information
    -s, --shuffle    Shuffle data when loading and/or running
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>            DB Configuration file
    -t, --table-name <table-name>    Table to use [default: experiments]

SUBCOMMANDS:
    edit    Edit the experiment table, insert new data and do maintenance
    help    Prints this message or the help of the given subcommand(s)
    run     Run experiments in parallel
    show    Print out stats or experiment details
```