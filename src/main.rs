use db::ExperimentStatus;
use once_cell::sync::OnceCell;
use process::ExperimentProcess;
use structopt::StructOpt;
use thiserror::Error;
use tokio::task;
mod db;
mod process;

use std::{path::PathBuf, sync::atomic::Ordering, time::Duration};

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
        #[structopt(short, long, group = "reset")]
        table: bool,
        /// Commands file to load
        #[structopt(short = "l", long = "load")]
        commands_file_to_load: Option<PathBuf>,
        /// Reset running jobs to available in DB
        #[structopt(long, group = "reset")]
        reset_running: bool,
        /// Reset failed jobs to available in DB
        #[structopt(long, group = "reset")]
        reset_failed: bool,
        /// Reset timed out jobs to available in DB
        #[structopt(long, group = "reset")]
        reset_timeout: bool,
        /// Reset all jobs to available in DB
        #[structopt(long, group = "reset")]
        reset_all: bool,
    },
    /// Run experiments in parallel
    Run {
        /// Time (in seconds) frequency to check db for new jobs
        #[structopt(short, long, default_value = "15")]
        freq: usize,
        /// Number of parallel of jobs on run mode
        #[structopt(short = "j", long = "jobs", default_value = "1")]
        nb_jobs: usize,
        /// Keep it running even though the DB is empty and no tasks are running
        #[structopt(short, long)]
        keep_running: bool,
        /// Dump command line outputs of tasks.
        #[structopt(short, long)]
        log_folder: Option<String>,
    },
    /// Print out stats or experiment details
    Show {
        /// Print Experiment statistics
        #[structopt(long, group = "print")]
        stats: bool,
        /// Print all experiments in the DB
        #[structopt(long, group = "print")]
        all: bool,
    },
}

#[derive(Error, Debug)]
enum OnceCellError {
    #[error("Could not setup inner flag with OnceCell")]
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
            } else if reset_running {
                experiment_db
                    .reset_jobs_with_status(ExperimentStatus::Running)
                    .await?;
            } else if reset_failed {
                experiment_db
                    .reset_jobs_with_status(ExperimentStatus::FailedFinished)
                    .await?;
            } else if reset_timeout {
                experiment_db
                    .reset_jobs_with_status(ExperimentStatus::TimedOut)
                    .await?;
            } else if reset_all {
                experiment_db.reset_all_jobs().await?;
            }
            if let Some(commands_file) = commands_file_to_load {
                experiment_db
                    .load_commands(&commands_file, opt.shuffle)
                    .await?;
            }
        }
        Command::Run {
            freq,
            nb_jobs,
            keep_running,
            log_folder,
        } => {
            let mut processes = vec![];
            while let Some(_) = experiment_db
                .get_number_of_available_jobs()
                .await?
                .and_then(|nb| {
                    if nb > 0 || keep_running {
                        Some(nb)
                    } else {
                        None
                    }
                })
            {
                let nb_available = nb_jobs - process::GLOBAL_JOB_COUNT.load(Ordering::SeqCst);
                let jobs = experiment_db
                    .get_available_jobs(nb_available, opt.shuffle)
                    .await?;
                experiment_db
                    .change_status_given_ids(
                        jobs.iter().map(|j| j.id).collect(),
                        ExperimentStatus::Running,
                    )
                    .await?;
                for j in jobs {
                    let e = ExperimentProcess::new(j, experiment_db.clone(), log_folder.clone())
                        .await?;
                    processes.push(e);
                }
                tokio::time::sleep(Duration::from_secs(freq as u64)).await;
            }
            for p in processes {
                let _ = task::spawn(async {
                    p.task
                        .await
                        .expect("Wrapping up last processes failed in a child process");
                })
                .await;
            }
        }
        Command::Show { stats, all } => {
            if stats {
                experiment_db.print_stats().await?;
            } else if all {
                experiment_db.print_all_jobs().await?;
            }
        }
    }
    experiment_db.pool.disconnect().await?;
    Ok(())
}
