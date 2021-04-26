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

pub(crate) struct ExperimentDatabase {
    pub(crate) pool: Pool,
}

impl ExperimentDatabase {
    pub fn from_db_config(db_config: DatabaseConfig) -> Self {
        let url = format!(
            "mysql://{}:{}@{}/{}_experiments",
            db_config.username, db_config.password, db_config.host, db_config.username
        );
        let pool = Pool::new(url);
        Self { pool }
    }
    pub async fn create_table(&self, table_name: &str) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;

        // Let's create a table for payments.
        conn.exec_drop(
            r"CREATE TABLE ? (
                    id int NOT NULL AUTO_INCREMENT, 
                    command VARCHAR(500) NOT NULL, 
                    status int NOT NULL, 
                    PRIMARY KEY (id))",
            (table_name,),
        )
        .await?;
        Ok(())
    }
}
