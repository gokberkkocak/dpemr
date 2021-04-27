use once_cell::sync::OnceCell;
use structopt::StructOpt;
use thiserror::Error;
mod db;

use std::path::PathBuf;

pub static DEBUG_MODE: OnceCell<bool> = OnceCell::new();

/// Distributed parallel for experiment management in Rust
#[derive(StructOpt, Debug)]
#[structopt(name = "dpr")]
struct Opt {
    /// DB Configuration file.
    #[structopt(short, long)]
    config: PathBuf,
    /// Debug mode enables verbose printing
    #[structopt(short, long)]
    debug: bool,
    /// Table to use
    #[structopt(short, long, default_value = "experiments")]
    table_name: String,
    /// Shuffle data when loading and/or running
    #[structopt(short, long)]
    shuffle: bool,
    /// Subcommand to choose
    #[structopt(subcommand)]
    command: Command,
}
#[derive(StructOpt, Debug)]
enum Command {
    /// Edit the experiment table, insert new data and do maintenance
    Edit {
        /// Create or empty the table in DB
        #[structopt(short,long)]
        table: bool,
        /// Commands file to load
        #[structopt(short = "l", long = "load")]
        commands_file_to_load: Option<PathBuf>,
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
    },
    /// Run experiments in parallel
    Run {
        /// Time (in seconds) frequency to check db for new jobs
        #[structopt(short, long, default_value = "15")]
        freq: i32,
        /// Enforce timeout by timeout command in secs. Can be used in --load and --run
        #[structopt(short, long)]
        timeout: Option<i32>,
        /// When used with --timeout, it can requeue task which went timeout for t*2 seconds
        #[structopt(short = "q", long)]
        auto_requeue: bool,
        /// Number of parallel of jobs on run mode
        #[structopt(short = "j", long = "jobs", default_value = "1")]
        nb_jobs: i32,
        /// Keep it running even though the DB is empty and no tasks are running
        #[structopt(short, long)]
        keep_running: bool,
        /// Additional parallel arguments to pass. Use quotes
        #[structopt(long)]
        extra_args: Option<String>,
    },
    /// Print out stats or experiment details
    Show {
        /// Print Experiment statistics
        #[structopt(long)]
        stats: bool,
        /// Print all experiments in the DB
        #[structopt(long)]
        all: bool,
    },
}

#[derive(Error, Debug)]
enum OnceCellError {
    #[error("Invalid config file. Please check your config file.")]
    CouldNotSetDebugCell,
}

#[tokio::main]
pub async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::from_args();
    let db_config = db::DatabaseConfig::from_config_file(&opt.config).await?;
    let experiment_db = db::ExperimentDatabase::from_db_config(db_config, opt.table_name);

    if opt.debug {
        DEBUG_MODE
            .set(true)
            .map_err(|_| anyhow::Error::new(OnceCellError::CouldNotSetDebugCell))?;
    } else {
        DEBUG_MODE
            .set(false)
            .map_err(|_| anyhow::Error::new(OnceCellError::CouldNotSetDebugCell))?;
    }
    match opt.command {
        Command::Edit {
            table,
            commands_file_to_load,
            reset_running,
            reset_failed,
            reset_timeout,
            reset_all,
        } => {
            if table {
                experiment_db.create_table().await?;
            }
            if let Some(commands_file) = commands_file_to_load {
                experiment_db
                    .load_commands(&commands_file, opt.shuffle)
                    .await?;
            }
        }
        Command::Run {
            freq,
            timeout,
            auto_requeue,
            nb_jobs,
            keep_running,
            extra_args,
        } => {}
        Command::Show { stats, all } => {}
    }
    experiment_db.pool.disconnect().await?;
    Ok(())
}
