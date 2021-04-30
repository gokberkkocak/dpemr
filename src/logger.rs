use crate::process::ProcessResult;

use std::sync::Arc;
use tokio::{fs::File, io::AsyncWriteExt, sync::mpsc::Receiver};


pub(crate) struct Logger;

impl Logger {
    pub(crate) async fn new(
        writer_rx: Receiver<ProcessResult>,
        log_folder: String,
    ) -> Result<(), anyhow::Error> {
        let log_folder = Arc::new(log_folder);
        tokio::fs::create_dir_all(log_folder.as_str()).await?;
        let _logger_task = tokio::spawn(Logger::worker_writer(
            writer_rx,
            // map.clone(),
            log_folder.clone(),
        ));
        Ok(())
    }

    async fn worker_writer(
        mut rx: Receiver<ProcessResult>,
        log_folder: Arc<String>,
    ) -> Result<(), anyhow::Error> {
        while let Some(p) = rx.recv().await {
            let mut stdout_file = File::create(format!("{}/{}.out", log_folder, p.id)).await?;
            stdout_file.write_all(p.stdout.as_bytes()).await?;
            let mut stderr_file = File::create(format!("{}/{}.err", log_folder, p.id)).await?;
            stderr_file.write_all(p.stderr.as_bytes()).await?;
        }
        Ok(())
    }
}
