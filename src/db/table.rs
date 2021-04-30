use super::{ExperimentDatabase, ExperimentStatus, Job};

use mysql_async::{prelude::*, Conn};
use rand::{seq::SliceRandom, thread_rng};
use std::{path::Path, sync::Arc};

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

    pub async fn lock_table(&self) -> Result<Conn, anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;
        conn.query_drop(format!("LOCK TABLE {} WRITE", self.table_name))
            .await?;
        Ok(conn)
    }

    pub async fn unlock_table(&self, mut conn: Conn) -> Result<(), anyhow::Error> {
        conn.query_drop("UNLOCK TABLES").await?;
        Ok(())
    }

    pub async fn change_status_given_ids_with_lock(
        &self,
        ids: Vec<usize>,
        new_status: ExperimentStatus,
        conn: &mut Conn,
    ) -> Result<(), anyhow::Error> {
        let params = ids.iter().map(|i| {
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

    pub async fn change_status_given_ids(
        &self,
        ids: Vec<usize>,
        new_status: ExperimentStatus,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;
        let params = ids.iter().map(|i| {
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

    pub async fn get_available_jobs_with_lock(
        &self,
        nb_jobs: usize,
        shuffle: bool,
        conn: &mut Conn,
    ) -> Result<Vec<Job>, anyhow::Error> {
        let cmd = if shuffle {
            format!(
                "SELECT id, command from {} WHERE status = :status ORDER BY RAND() LIMIT :limit ",
                self.table_name
            )
        } else {
            format!(
                "SELECT id, command from {} WHERE status = :status LIMIT :limit",
                self.table_name
            )
        };
        let jobs: Vec<Job> = conn
            .exec_map(
                cmd,
                params! {
                    "status" => ExperimentStatus::NotRunning.to_db_code(),
                    "limit" => nb_jobs
                },
                |(id, command)| Job {
                    id,
                    command: Arc::new(command),
                },
            )
            .await?;

        Ok(jobs)
    }

    pub async fn get_number_of_available_jobs(&self) -> Result<Option<usize>, anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;
        let job_count: Option<usize> = conn
            .exec_first(
                format!(
                    "SELECT COUNT(*) FROM {} WHERE status = :status",
                    self.table_name
                ),
                params! {
                    "status" => ExperimentStatus::NotRunning.to_db_code(),
                },
            )
            .await?;

        Ok(job_count)
    }
}
