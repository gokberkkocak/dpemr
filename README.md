# dpemr or dpr ![Tests](https://github.com/gokberkkocak/dpemr/actions/workflows/ci.yml/badge.svg)
## Distributed parallel for experiment management in Rust 

Re-write of the project [dpem](https://github.com/gokberkkocak/dpem) in Rust with async tokio runtime. It doesn't require gnu-parallel as a dependency anymore. As a reason, it lacks some expressibility features from the original project due to removal of extra parallel args and some other features as well since handling timeouts or dynamic timeouts inside the experiment seems easier to manage. 

Since it's not depending gnu-parallel, it's technically possible to use this even on native Windows but I'm not sure that would be useful for anyone. 

## Usage

```
dpr 0.1.0
Distributed parallel for experiment management in Rust

USAGE:
    dpr [FLAGS] [OPTIONS] --config <config> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -s, --shuffle    Shuffle data when loading and/or running
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>            DB Configuration file
    -n, --table-name <table-name>    Table to use [default: experiments]

SUBCOMMANDS:
    edit    Edit the experiment table, insert new data and do maintenance
    help    Prints this message or the help of the given subcommand(s)
    run     Run experiments in parallel
    show    Print out stats or experiment details
```

### Edit Mode Usage

```
dpr-edit 0.1.0
Edit the experiment table, insert new data and do maintenance

USAGE:
    dpr --config <config> edit [FLAGS] [OPTIONS]

FLAGS:
    -t, --create-table     Create or empty the table in DB
    -h, --help             Prints help information
        --reset-all        Reset all jobs to available in DB
        --reset-failed     Reset failed jobs to available in DB
        --reset-running    Reset running jobs to available in DB
        --reset-timeout    Reset timed out jobs to available in DB
    -V, --version          Prints version information

OPTIONS:
    -l, --load <commands-file-to-load>    Commands file to load
```

### Run Mode Usage

```
dpr-run 0.1.0
Run experiments in parallel

USAGE:
    dpr --config <config> run [FLAGS] [OPTIONS]

FLAGS:
    -h, --help            Prints help information
    -k, --keep-running    Keep it running even though the DB is empty and no tasks are running
    -V, --version         Prints version information

OPTIONS:
    -f, --freq <freq>                Time (in seconds) frequency to check db for new jobs [default: 15]
    -l, --log-folder <log-folder>    Dump command line outputs of tasks
    -j, --jobs <nb-jobs>             Number of parallel of jobs on run mode [default: 1]
```

### Show Mode Usage

```
dpr-show 0.1.0
Print out stats or experiment details

USAGE:
    dpr --config <config> show [FLAGS]

FLAGS:
        --all        Print all experiments in the DB
    -h, --help       Prints help information
        --stats      Print Experiment statistics
    -V, --version    Prints version information
```