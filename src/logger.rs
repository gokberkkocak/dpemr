use crate::process::{ExperimentProcess, ProcessResult};

use std::{collections::HashMap, sync::Arc};
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    sync::{mpsc::Receiver, Mutex},
    task::JoinHandle,
};

pub(crate) struct TrackerLogger {
    pub(crate) track_task: JoinHandle<Result<(), anyhow::Error>>,
    pub(crate) active_jobs: Arc<Mutex<HashMap<usize, ExperimentProcess>>>,
}

impl TrackerLogger {
    pub(crate) async fn new(
        writer_rx: Receiver<ProcessResult>,
        log_folder: Option<String>,
    ) -> Result<TrackerLogger, anyhow::Error> {
        let active_jobs = Arc::new(Mutex::new(HashMap::new()));
        let log_folder = Arc::new(log_folder);
        if let Some(logs) = log_folder.as_ref() {
            tokio::fs::create_dir_all(logs).await?;
        }
        let track_task = tokio::spawn(TrackerLogger::worker_writer(
            writer_rx,
            active_jobs.clone(),
            log_folder.clone(),
        ));
        Ok(TrackerLogger {
            track_task,
            active_jobs,
        })
    }

    async fn worker_writer(
        mut rx: Receiver<ProcessResult>,
        active_jobs: Arc<Mutex<HashMap<usize, ExperimentProcess>>>,
        log_folder: Arc<Option<String>>,
    ) -> Result<(), anyhow::Error> {
        while let Some(p) = rx.recv().await {
            if let Some(logs) = log_folder.as_ref() {
                let mut stdout_file = File::create(format!("{}/{}.out", logs, p.job.id)).await?;
                stdout_file.write_all(p.stdout.as_bytes()).await?;
                let mut stderr_file = File::create(format!("{}/{}.err", logs, p.job.id)).await?;
                stderr_file.write_all(p.stderr.as_bytes()).await?;
            }
            let mut lock = active_jobs.lock().await;
            let p = (*lock).remove(&p.job.id);
            drop(lock);
            if let Some(p) = p {
                p.task.await??;
            }
        }
        Ok(())
    }

    pub(crate) async fn add_to_active_jobs(&self, p: ExperimentProcess) {
        let mut lock = self.active_jobs.lock().await;
        (*lock).insert(p.job.id, p);
    }

    pub(crate) async fn wait_all_to_finish(&self) -> Result<(), anyhow::Error> {
        let mut lock = self.active_jobs.lock().await;
        let ids = lock.keys().cloned().collect::<Vec<_>>();
        for id in ids {
            let p = (*lock).remove(&id);
            if let Some(p) = p {
                p.task.await??;
            }
        }
        Ok(())
    }
}
