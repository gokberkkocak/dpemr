use std::path::Path;

use mysql_async::{prelude::*, Pool};

use thiserror::Error;

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
                assert_eq!(split.len(), 2);
                if split[0] == "host" {
                    host = Some(split[1].trim_end().to_string());
                } else if split[0] == "user" {
                    username = Some(split[1].trim_end().to_string());
                } else if split[0] == "password" {
                    password = Some(split[1].trim_end().to_string());
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

struct TableEntry<'a> {
    command: &'a str,
    status: ExperimentStatus,
}

pub(crate) struct ExperimentDatabase {
    pub(crate) pool: Pool,
    table_name: String,
}

impl ExperimentDatabase {
    pub fn from_db_config(db_config: DatabaseConfig, table_name: String) -> Self {
        let url = format!(
            "mysql://{}:{}@{}/{}_experiments",
            db_config.username, db_config.password, db_config.host, db_config.username
        );
        let pool = Pool::new(url);
        Self { pool, table_name }
    }
    pub async fn create_table(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;
        // Let's create a table for payments.
        conn.exec_drop(
            r"CREATE TABLE ? (
                    id int NOT NULL AUTO_INCREMENT, 
                    command VARCHAR(500) NOT NULL, 
                    status int NOT NULL, 
                    PRIMARY KEY (id))",
            (&self.table_name,),
        )
        .await?;
        Ok(())
    }

    pub async fn load_commands(&self, commands_file: &Path) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;
        let file_contents = String::from_utf8(tokio::fs::read(commands_file).await?)?;
        let mut table_entries = vec![];
        for line in file_contents.lines() {
            let t = TableEntry {
                command: line.trim_end(),
                status: ExperimentStatus::NotRunning,
            };
            table_entries.push(t);
        }
        let params = table_entries.into_iter().map(|t| {
            params! {
                "command" => t.command,
                "status" => t.status.to_db_code(),
            }
        });

        conn.exec_batch(
            r"INSERT INTO {} 
                            (command,status) values (:command, :status)",
            params,
        )
        .await?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
enum ExperimentStatus {
    NotRunning,
    Running,
    SuccessFinished,
    FailedFinished,
    TimedOut,
}

impl ExperimentStatus {
    fn to_db_code(self) -> i32 {
        match self {
            ExperimentStatus::NotRunning => 0,
            ExperimentStatus::Running => 1,
            ExperimentStatus::SuccessFinished => 2,
            ExperimentStatus::FailedFinished => 3,
            ExperimentStatus::TimedOut => 4,
        }
    }
}
