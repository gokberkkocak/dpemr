use crate::db::{ExperimentDatabase, ExperimentStatus, Job};

use std::{
    process::{ExitStatus, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

use thiserror::Error;

use tokio::{
    io::AsyncReadExt,
    io::BufReader,
    process::Child,
    sync::mpsc::Sender,
    task::{self, JoinHandle},
};

pub(crate) static GLOBAL_JOB_COUNT: AtomicUsize = AtomicUsize::new(0);

const TIMEOUT_RETURN_CODE: i32 = 124;

use tokio::process::Command;

#[derive(Debug)]
pub struct ExperimentProcess {
    pub job: Job,
    pub task: JoinHandle<Result<(), anyhow::Error>>,
}

impl ExperimentProcess {
    pub(crate) async fn new(
        job: Job,
        experiment_db: ExperimentDatabase,
        writer_tx: Option<Sender<ProcessResult>>,
    ) -> Result<ExperimentProcess, anyhow::Error> {
        let task = task::spawn(ExperimentProcess::middle_layer(
            job.clone(),
            experiment_db.clone(),
            writer_tx,
        ));
        Ok(ExperimentProcess { job, task })
    }

    async fn middle_layer(
        job: Job,
        experiment_db: ExperimentDatabase,
        writer_tx: Option<Sender<ProcessResult>>,
    ) -> Result<(), anyhow::Error> {
        let worker_result = ExperimentProcess::worker(job.clone(), experiment_db).await;
        let end_result = match worker_result {
            std::result::Result::Ok(res) => res,
            std::result::Result::Err(e) => ProcessResult {
                id: job.id,
                code: -1,
                stdout: String::new(),
                stderr: e.to_string(),
            },
        };
        if let Some(tx) = writer_tx {
            tx.send(end_result).await?;
        }
        Ok(())
    }

    async fn worker(
        job: Job,
        experiment_db: ExperimentDatabase,
    ) -> Result<ProcessResult, anyhow::Error> {
        // shlex might be necessary
        let mut s = job.command.split_ascii_whitespace();
        let cmd = s
            .next()
            .ok_or(0)
            .map_err(|_| anyhow::Error::new(ProcessError::CannotStartProcess))?;
        let mut child = Command::new(cmd)
            .args(s)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        GLOBAL_JOB_COUNT.fetch_add(1, Ordering::SeqCst);
        let res = child.wait().await?;
        GLOBAL_JOB_COUNT.fetch_sub(1, Ordering::SeqCst);
        let return_code = ExperimentProcess::process_res(job.id, res, experiment_db).await?;
        let (stdout, stderr) = ExperimentProcess::process_std(child).await?;
        print!("{}", stdout);
        print!("{}", stderr);
        Ok(ProcessResult {
            id: job.id,
            code: return_code,
            stdout,
            stderr,
        })
    }

    async fn process_std(mut child: Child) -> Result<(String, String), anyhow::Error> {
        let mut stdout = String::new();
        BufReader::new(
            child
                .stdout
                .take()
                .ok_or(0)
                .map_err(|_| anyhow::Error::new(ProcessError::CannotFetchOutput))?,
        )
        .read_to_string(&mut stdout)
        .await?;
        let mut stderr = String::new();
        BufReader::new(
            child
                .stderr
                .take()
                .ok_or(0)
                .map_err(|_| anyhow::Error::new(ProcessError::CannotFetchOutput))?,
        )
        .read_to_string(&mut stderr)
        .await?;
        Ok((stdout, stderr))
    }

    async fn process_res(
        job_id: usize,
        res: ExitStatus,
        experiment_db: ExperimentDatabase,
    ) -> Result<i32, anyhow::Error> {
        if res.success() {
            experiment_db
                .change_status_given_ids(vec![job_id], ExperimentStatus::SuccessFinished)
                .await?;
        } else if let Some(_) = res.code().filter(|&c| c == TIMEOUT_RETURN_CODE) {
            experiment_db
                .change_status_given_ids(vec![job_id], ExperimentStatus::TimedOut)
                .await?;
        } else {
            experiment_db
                .change_status_given_ids(vec![job_id], ExperimentStatus::FailedFinished)
                .await?;
        }
        match res.code() {
            Some(ret) => Ok(ret),
            _ => Err(anyhow::Error::new(ProcessError::CannotGetReturnCode)),
        }
    }
}
#[derive(Error, Debug)]
enum ProcessError {
    #[error("Cannot get return code of subprocess")]
    CannotGetReturnCode,
    #[error("Cannot start subprocess because of command is invalid")]
    CannotStartProcess,
    #[error("Cannot get stdout/stderr of the process")]
    CannotFetchOutput,
}
#[derive(Debug)]
pub struct ProcessResult {
    pub id: usize,
    code: i32,
    pub stdout: String,
    pub stderr: String,
}
