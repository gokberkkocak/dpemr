use std::path::Path;

use mysql_async::Pool;

use thiserror::Error;

mod show;
mod table;

pub(crate) struct DatabaseConfig {
    host: String,
    username: String,
    password: String,
}

impl<'a> DatabaseConfig {
    fn new(host: String, username: String, password: String) -> Self {
        Self {
            host,
            username,
            password,
        }
    }

    pub(crate) async fn from_config_file(file_name: &Path) -> Result<Self, anyhow::Error> {
        let file_contents = String::from_utf8(tokio::fs::read(file_name).await?)?;
        let mut host = None;
        let mut username = None;
        let mut password = None;
        for line in file_contents.lines() {
            if line.starts_with('#') {
                continue;
            } else if line.contains('=') {
                let split: Vec<&str> = line.split('=').collect();
                if split.len() >= 2 {
                    if split[0] == "host" {
                        host = Some(split[1].trim_end().to_string());
                    } else if split[0] == "user" {
                        username = Some(split[1].trim_end().to_string());
                    } else if split[0] == "password" {
                        password = Some(split[1].trim_end().to_string());
                    }
                }
            }
        }
        match (host, username, password) {
            (Some(h), Some(u), Some(p)) => Ok(Self::new(h, u, p)),
            _ => Err(anyhow::Error::new(ConfigFileError::InvalidConfigFile)),
        }
    }
}

#[derive(Error, Debug)]
enum ConfigFileError {
    #[error("Invalid config file. Please check your config file.")]
    InvalidConfigFile,
}

pub(crate) struct ExperimentDatabase {
    pub(crate) pool: Pool,
    table_name: String,
}

impl ExperimentDatabase {
    pub fn from_db_config(db_config: DatabaseConfig, table_name: String) -> Self {
        let url = format!(
            "mysql://{}:{}@{}/{}_dpem_experiments",
            db_config.username, db_config.password, db_config.host, db_config.username
        );
        let pool = Pool::new(url);
        Self { pool, table_name }
    }
}

#[derive(Copy, Clone)]
pub enum ExperimentStatus {
    NotRunning,
    Running,
    SuccessFinished,
    FailedFinished,
    TimedOut,
}

impl ExperimentStatus {
    fn new(status_code: usize) -> Self {
        match status_code {
            0 => ExperimentStatus::NotRunning,
            1 => ExperimentStatus::Running,
            2 => ExperimentStatus::SuccessFinished,
            3 => ExperimentStatus::FailedFinished,
            4 => ExperimentStatus::TimedOut,
            _ => unreachable!(),
        }
    }

    fn to_db_code(self) -> usize {
        match self {
            ExperimentStatus::NotRunning => 0,
            ExperimentStatus::Running => 1,
            ExperimentStatus::SuccessFinished => 2,
            ExperimentStatus::FailedFinished => 3,
            ExperimentStatus::TimedOut => 4,
        }
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
