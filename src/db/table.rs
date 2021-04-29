use std::path::Path;

use rand::{seq::SliceRandom, thread_rng};

use mysql_async::prelude::*;

use super::{ExperimentDatabase, ExperimentStatus};

struct TableEntry<'a> {
    command: &'a str,
    status: ExperimentStatus,
}

impl ExperimentDatabase {
    pub async fn create_table(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;
        conn.query_drop(format!(
            r"CREATE OR REPLACE TABLE {} (
                    id int NOT NULL AUTO_INCREMENT, 
                    command VARCHAR(500) NOT NULL, 
                    status int NOT NULL, 
                    CHECK(status<5),
                    PRIMARY KEY (id))",
            self.table_name
        ))
        .await?;
        Ok(())
    }
    pub async fn load_commands(
        &self,
        commands_file: &Path,
        shuffle: bool,
    ) -> Result<(), anyhow::Error> {
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
        if shuffle {
            let mut rng = thread_rng();
            table_entries.shuffle(&mut rng);
        }
        let params = table_entries.into_iter().map(|t| {
            params! {
                "command" => t.command,
                "status" => t.status.to_db_code(),
            }
        });
        conn.exec_batch(
            format!(
                r"INSERT INTO {}
                            (command,status) values (:command, :status)",
                self.table_name
            ),
            params,
        )
        .await?;
        Ok(())
    }

    pub async fn reset_all_jobs(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;
        let ids: Vec<usize> = conn
            .query(format!("SELECT id from {}", self.table_name))
            .await?;
        self.reset_given_ids(ids).await?;
        Ok(())
    }

    pub(crate) async fn reset_jobs_with_status(
        &self,
        status: ExperimentStatus,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;
        let ids: Vec<usize> = conn
            .exec(
                format!("SELECT id from {} WHERE status = :status", self.table_name),
                params! {
                    "status" => status.to_db_code(),
                },
            )
            .await?;
        self.reset_given_ids(ids).await?;
        Ok(())
    }

    async fn reset_given_ids(&self, ids: Vec<usize>) -> Result<(), anyhow::Error> {
        Ok(self
            .change_status_given_ids(ids, ExperimentStatus::NotRunning)
            .await?)
    }

    async fn change_status_given_ids(
        &self,
        ids: Vec<usize>,
        new_status: ExperimentStatus,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;
        let params = ids.into_iter().map(|i| {
            params! {
                "new_status" => new_status.to_db_code(),
                "id" => i,
            }
        });
        conn.exec_batch(
            format!(
                r"UPDATE {} SET status = :new_status WHERE id = :id",
                self.table_name
            ),
            params,
        )
        .await?;
        Ok(())
    }
}
