use mysql_async::prelude::*;

use super::ExperimentStatus;

use super::ExperimentDatabase;

impl ExperimentDatabase {
    pub async fn print_stats(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;
        let results: Vec<ExperimentStatus> = conn
            .query_map(format!("SELECT status from {}", self.table_name), |x| {
                ExperimentStatus::new(x)
            })
            .await?;
        let name_vec = vec!["Available", "Running", "Success", "Failed", "Timeout"];
        let mut result_vec = vec![0; 5];
        for i in results {
            result_vec[i.to_db_code()] += 1;
        }
        for i in 0..5 {
            println!("{}: {}", name_vec[i], result_vec[i])
        }
        Ok(())
    }

    pub async fn print_all_jobs(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get_conn().await?;
        let results: Vec<(String, ExperimentStatus)> = conn
            .query_map(
                format!("SELECT (command, status) from {}", self.table_name),
                |(c, s)| (c, ExperimentStatus::new(s)),
            )
            .await?;
        println!("Command, Status");
        for (c, e) in results {
            println!("{},{}", c, e);
        }
        Ok(())
    }
}
