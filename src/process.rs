use std::{
    process::Stdio,
    sync::atomic::{AtomicUsize, Ordering},
};

use tokio::{
    fs::File,
    io::AsyncReadExt,
    io::AsyncWriteExt,
    io::BufReader,
    task::{self, JoinHandle},
};

pub(crate) static GLOBAL_JOB_COUNT: AtomicUsize = AtomicUsize::new(0);

const TIMEOUT_RETURN_CODE: i32 = 124;

use tokio::process::Command;

use crate::db::{ExperimentDatabase, ExperimentStatus, Job};

pub struct ExperimentProcess {
    pub job: Job,
    pub task: JoinHandle<()>,
}

impl ExperimentProcess {
    pub(crate) async fn new(
        job: Job,
        experiment_db: ExperimentDatabase,
        log_folder: Option<String>,
    ) -> Result<ExperimentProcess, anyhow::Error> {
        let job_clone = job.clone();
        let task = task::spawn(async move {
            let mut s = job_clone.command.split_ascii_whitespace();
            let mut child = Command::new(s.next().unwrap())
                .args(s)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Spawning process failed on child");
            GLOBAL_JOB_COUNT.fetch_add(1, Ordering::SeqCst);
            let res = child.wait().await.unwrap();
            GLOBAL_JOB_COUNT.fetch_sub(1, Ordering::SeqCst);
            let mut stdout = String::new();
            BufReader::new(child.stdout.take().expect("a"))
                .read_to_string(&mut stdout)
                .await
                .expect("Reading job output failed on child");
            let stderr = String::new();
            BufReader::new(child.stderr.take().expect("a"))
                .read_to_string(&mut stdout)
                .await
                .expect("Reading job output failed on child");
            if res.success() {
                experiment_db
                    .change_status_given_ids(vec![job_clone.id], ExperimentStatus::SuccessFinished)
                    .await
                    .expect("Job update failed on child process");
            } else if res.code().unwrap() == TIMEOUT_RETURN_CODE {
                experiment_db
                    .change_status_given_ids(vec![job_clone.id], ExperimentStatus::TimedOut)
                    .await
                    .expect("Job update failed on child process");
            } else {
                experiment_db
                    .change_status_given_ids(vec![job_clone.id], ExperimentStatus::FailedFinished)
                    .await
                    .expect("Job update failed on child process");
            }
            if let Some(log_folder) = log_folder {
                tokio::fs::create_dir_all(&log_folder)
                    .await
                    .expect("Cannot create dirs");
                let mut stdout_file = File::create(format!("{}/{}.out", log_folder, job_clone.id))
                    .await
                    .expect("File creation failed on child");
                stdout_file
                    .write_all(stdout.as_bytes())
                    .await
                    .expect("Writing to file failed on child");
                let mut stderr_file = File::create(format!("{}/{}.err", log_folder, job_clone.id))
                    .await
                    .expect("File creation failed on child");
                stderr_file
                    .write_all(stderr.as_bytes())
                    .await
                    .expect("Writing to file failed on child");
            }
        });
        Ok(ExperimentProcess { job, task })
    }
}
