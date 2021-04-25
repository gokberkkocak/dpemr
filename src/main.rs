use std::path::PathBuf;

use structopt::StructOpt;

/// Distributed parallel in Rust for experiment management.
#[derive(StructOpt, Debug)]
#[structopt(name = "dpr")]
struct Opt {
    /// DB Configuration file. 
    #[structopt(short, long)]
    config: PathBuf,
    /// Create or empty the table in DB
    #[structopt(short = "b", long = "table")]
    table: bool,
    /// Commands file to load
    #[structopt(short = "l", long = "load")]
    commands_file_to_load: PathBuf,
    /// Time (in seconds) frequency to check db for new jobs (default: 15)
    #[structopt(short, long, default_value = "15")]
    freq: i32,
    /// Enforce timeout by timeout command in secs. Can be used in --load and --run
    #[structopt(short, long)]
    timeout: i32,
    /// When used with --timeout, it can requeue task which went timeout for t*2 seconds
    #[structopt(short = "q", long)]
    auto_requeue: bool,
    /// Debug mode enables verbose printing
    #[structopt(short, long)]
    debug: bool,
    /// Shuffle data when loading and/or running
    #[structopt(short, long)]
    shuffle: bool,
    /// Run experiments from DB
    #[structopt(short, long)]
    run: bool,
    /// Number of parallel of jobs to run (user) (default: 1)
    #[structopt(short = "j", long = "jobs", default_value = "1")]
    nb_jobs: i32,
    /// Table to use for experiment (default: experiments)
    #[structopt(short = "u", long = "use-table", default_value = "experiments")]
    table_name: String,
    /// Keep it running even though the DB is empty and no tasks are running
    #[structopt(short, long)]
    keep_running: bool,
    /// Additional parallel arguments to pass. Use quotes
    #[structopt(long)]
    extra_args: String,
    /// Print Experiment statistics
    #[structopt(long)]
    stats: bool,
    /// Reset running jobs to available in DB
    #[structopt(long)]
    reset_running: bool,
    /// Reset failed jobs to available in DB
    #[structopt(long)]
    reset_failed: bool,
    /// Reset timed out jobs to available in DB
    #[structopt(long)]
    reset_timeout: bool,
    /// Reset all jobs to available in DB
    #[structopt(long)]
    reset_all: bool,

}

#[tokio::main]
pub async fn main() {
    let opt = Opt::from_args();
}
