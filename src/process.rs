use crate::db::{ExperimentDatabase, ExperimentStatus, Job};

use std::{
    process::{ExitStatus, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
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
    pub task: JoinHandle<Result<()>>,
}

impl ExperimentProcess {
    pub(crate) async fn new(
        job: Job,
        experiment_db: ExperimentDatabase,
        writer_tx: Sender<ProcessResult>,
    ) -> Result<ExperimentProcess> {
        let task = task::spawn(ExperimentProcess::middle_layer(
            job.clone(),
            experiment_db,
            writer_tx,
        ));
        Ok(ExperimentProcess { job, task })
    }

    async fn middle_layer(
        job: Job,
        experiment_db: ExperimentDatabase,
        writer_tx: Sender<ProcessResult>,
    ) -> Result<()> {
        let worker_result = ExperimentProcess::worker(job.clone(), experiment_db).await;
        let end_result = match worker_result {
            std::result::Result::Ok(res) => res,
            std::result::Result::Err(e) => ProcessResult {
                job,
                code: -1,
                stdout: String::new(),
                stderr: e.to_string(),
            },
        };
        writer_tx.send(end_result).await?;
        Ok(())
    }

    async fn worker(job: Job, experiment_db: ExperimentDatabase) -> Result<ProcessResult> {
        // shlex might be necessary
        let mut s = job.command.split_ascii_whitespace();
        let cmd = s
            .next()
            .ok_or(0)
            .map_err(|_| anyhow::Error::new(ProcessError::StartProcess))?;
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
            job,
            code: return_code,
            stdout,
            stderr,
        })
    }

    async fn process_std(mut child: Child) -> Result<(String, String)> {
        let mut stdout = String::new();
        BufReader::new(
            child
                .stdout
                .take()
                .ok_or(0)
                .map_err(|_| anyhow::Error::new(ProcessError::FetchOutput))?,
        )
        .read_to_string(&mut stdout)
        .await?;
        let mut stderr = String::new();
        BufReader::new(
            child
                .stderr
                .take()
                .ok_or(0)
                .map_err(|_| anyhow::Error::new(ProcessError::FetchOutput))?,
        )
        .read_to_string(&mut stderr)
        .await?;
        Ok((stdout, stderr))
    }

    async fn process_res(
        job_id: usize,
        res: ExitStatus,
        experiment_db: ExperimentDatabase,
    ) -> Result<i32> {
        if res.success() {
            experiment_db
                .change_status_given_ids(vec![job_id], ExperimentStatus::SuccessFinished)
                .await?;
        } else if res.code().filter(|&c| c == TIMEOUT_RETURN_CODE).is_some() {
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
            _ => Err(anyhow::Error::new(ProcessError::GetReturnCode)),
        }
    }
}
#[derive(Error, Debug)]
enum ProcessError {
    #[error("Cannot get return code of subprocess")]
    GetReturnCode,
    #[error("Cannot start subprocess because of command is invalid")]
    StartProcess,
    #[error("Cannot get stdout/stderr of the process")]
    FetchOutput,
}
#[derive(Debug)]
pub struct ProcessResult {
    pub job: Job,
    code: i32,
    pub stdout: String,
    pub stderr: String,
}
