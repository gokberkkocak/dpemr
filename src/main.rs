mod db;
mod logger;
mod process;

use db::ExperimentStatus;
use logger::Logger;
use process::ExperimentProcess;

use std::{path::PathBuf, sync::atomic::Ordering, time::Duration};
use structopt::StructOpt;
use tokio::sync::mpsc;

/// Distributed parallel for experiment management in Rust
#[derive(StructOpt, Debug)]
#[structopt(name = "dpr")]
struct Opt {
    /// DB Configuration file.
    #[structopt(short, long)]
    config: PathBuf,
    /// Table to use
    #[structopt(short = "n", long, default_value = "experiments")]
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
        #[structopt(short = "t", long, group = "reset")]
        create_table: bool,
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

#[tokio::main]
pub async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::from_args();
    let db_config = db::DatabaseConfig::from_config_file(&opt.config).await?;
    let experiment_db = db::ExperimentDatabase::from_db_config(db_config, opt.table_name);

    match opt.command {
        Command::Edit {
            create_table,
            commands_file_to_load,
            reset_running,
            reset_failed,
            reset_timeout,
            reset_all,
        } => {
            if create_table {
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
            let writer_tx = if let Some(log_folder) = log_folder {
                // let (watcher_tx, watcher_rx) = channel(10);
                let (writer_tx, writer_rx) = mpsc::channel(10);
                let _ = Logger::new(writer_rx, log_folder).await?;
                Some(writer_tx)
            } else {
                None
            };
            while let Some(_) = experiment_db
                .get_number_of_available_jobs()
                .await?
                .filter(|&nb| nb > 0 || keep_running)
            {
                let nb_available = nb_jobs - process::GLOBAL_JOB_COUNT.load(Ordering::SeqCst);
                if nb_available > 0 {
                    let mut conn = experiment_db.lock_table().await?;
                    let jobs = experiment_db
                        .get_available_jobs_with_lock(nb_available, opt.shuffle, &mut conn)
                        .await?;
                    experiment_db
                        .change_status_given_ids_with_lock(
                            jobs.iter().map(|j| j.id).collect(),
                            ExperimentStatus::Running,
                            &mut conn,
                        )
                        .await?;
                    experiment_db.unlock_table(conn).await?;
                    for j in jobs {
                        ExperimentProcess::new(j, experiment_db.clone(), writer_tx.clone()).await?;
                    }
                }
                tokio::time::sleep(Duration::from_secs(freq as u64)).await;
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
