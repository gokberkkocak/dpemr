mod show;
mod table;

use anyhow::Result;
use mysql_async::Pool;
use serde::Deserialize;
use std::{path::Path, sync::Arc};

#[derive(Deserialize)]
pub(crate) struct DatabaseConfig {
    client: DatabaseClient,
}

#[derive(Deserialize)]
struct DatabaseClient {
    host: String,
    user: String,
    password: String,
}

impl DatabaseConfig {
    pub(crate) async fn from_config_file(file_name: &Path) -> Result<DatabaseConfig> {
        let file_contents = String::from_utf8(tokio::fs::read(file_name).await?)?;
        let dbc: DatabaseConfig = toml::from_str(&file_contents)?;
        Ok(dbc)
    }

    pub(crate) fn get_user(&self) -> &str {
        &self.client.user
    }

    pub(crate) fn get_password(&self) -> &str {
        &self.client.password
    }

    pub(crate) fn get_host(&self) -> &str {
        &self.client.host
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ExperimentDatabase {
    pub(crate) pool: Pool,
    table_name: Arc<String>,
}

impl ExperimentDatabase {
    pub fn from_db_config(db_config: DatabaseConfig, table_name: String) -> Self {
        let url = format!(
            "mysql://{}:{}@{}/{}_dpemr_experiments",
            db_config.get_user(),
            db_config.get_password(),
            db_config.get_host(),
            db_config.get_user()
        );
        let pool = Pool::new(url);
        Self {
            pool,
            table_name: Arc::new(table_name),
        }
    }
}

#[derive(Copy, Clone)]
pub enum ExperimentStatus {
    NotRunning = 0,
    Running = 1,
    SuccessFinished = 2,
    FailedFinished = 3,
    TimedOut = 4,
}

impl ExperimentStatus {
    fn new(status_code: usize) -> Self {
        match status_code {
            0 => ExperimentStatus::NotRunning,
            1 => ExperimentStatus::Running,
            2 => ExperimentStatus::SuccessFinished,
            3 => ExperimentStatus::FailedFinished,
            4 => ExperimentStatus::TimedOut,
            // SAFETY: If experiment status only constructed for status codes from db table, it's guaranteed to be bounded. 
            _ => unreachable!(),
        }
    }

    fn to_db_code(self) -> usize {
        self as usize
    }
}

impl std::fmt::Display for ExperimentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExperimentStatus::NotRunning => write!(f, "Available"),
            ExperimentStatus::Running => write!(f, "Running"),
            ExperimentStatus::SuccessFinished => write!(f, "Success"),
            ExperimentStatus::FailedFinished => write!(f, "Failure"),
            ExperimentStatus::TimedOut => write!(f, "Timeout"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: usize,
    pub command: Arc<String>,
}
