# dpemr or dpr ![Tests](https://github.com/gokberkkocak/dpemr/actions/workflows/ci.yml/badge.svg)
## Distributed parallel for experiment management in Rust 

## Dependencies
    gnu-parallel


## Usage

```
dpr 0.1.0
Distributed parallel in Rust for experiment management

USAGE:
    dpr [FLAGS] [OPTIONS] --load <commands-file-to-load> --config <config> --extra-args <extra-args> --timeout <timeout>

FLAGS:
    -q, --auto-requeue     When used with --timeout, it can requeue task which went timeout for t*2 seconds
    -d, --debug            Debug mode enables verbose printing
    -h, --help             Prints help information
    -k, --keep-running     Keep it running even though the DB is empty and no tasks are running
        --reset-all        Reset all jobs to available in DB
        --reset-failed     Reset failed jobs to available in DB
        --reset-running    Reset running jobs to available in DB
        --reset-timeout    Reset timed out jobs to available in DB
    -r, --run              Run experiments from DB
    -s, --shuffle          Shuffle data when loading and/or running
        --stats            Print Experiment statistics
    -b, --table            Create or empty the table in DB
    -V, --version          Prints version information

OPTIONS:
    -l, --load <commands-file-to-load>    Commands file to load
    -c, --config <config>                 DB Configuration file
        --extra-args <extra-args>         Additional parallel arguments to pass. Use quotes
    -f, --freq <freq>                     Time (in seconds) frequency to check db for new jobs (default: 15) [default:
                                          15]
    -j, --jobs <nb-jobs>                  Number of parallel of jobs to run (user) (default: 1) [default: 1]
    -u, --use-table <table-name>          Table to use for experiment (default: experiments) [default: experiments]
    -t, --timeout <timeout>               Enforce timeout by timeout command in secs. Can be used in --load and --run
```