use super::{ExperimentDatabase, ExperimentStatus};

use anyhow::Result;
use mysql_async::prelude::*;

impl ExperimentDatabase {
    pub async fn print_stats(&self) -> Result<()> {
        let mut conn = self.pool.get_conn().await?;
        let results: Vec<ExperimentStatus> = conn
            .query_map(format!("SELECT status from {}", self.table_name), |x| {
                ExperimentStatus::new(x)
            })
            .await?;
        let name_vec = vec!["Available", "Running", "Success", "Failed", "Timeout"];
        let mut result_vec = vec![0; 5];
        for i in results {
            // SAFETY: Database constraints ensures that status is always below 5.
            result_vec[i.to_db_code()] += 1;
        }
        for i in 0..name_vec.len() {
            println!("{}: {}", name_vec[i], result_vec[i])
        }
        Ok(())
    }

    pub async fn print_all_jobs(&self) -> Result<()> {
        let mut conn = self.pool.get_conn().await?;
        let results: Vec<(String, ExperimentStatus)> = conn
            .query_map(
                format!("SELECT command, status from {}", self.table_name),
                |(c, s)| (c, ExperimentStatus::new(s)),
            )
            .await?;
        if results.is_empty() {
            println!("Database is empty.")
        } else {
            println!("Command, Status");
            for (cmd, status) in results {
                println!("{}, {}", cmd, status);
            }
        }
        Ok(())
    }
}
