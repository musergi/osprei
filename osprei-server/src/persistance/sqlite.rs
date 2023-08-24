use log::error;
use osprei::{ExecutionDetails, ExecutionSummary, JobPointer};
use sqlx::Row;

use super::{Storage, StorageError, StoreResult};

#[derive(Debug, Clone)]
pub struct DatabasePersistance {
    pool: sqlx::SqlitePool,
}

impl DatabasePersistance {
    pub async fn new(database_path: &str) -> StoreResult<DatabasePersistance> {
        let url = database_path.to_string() + "?mode=rwc";
        let pool = match sqlx::sqlite::SqlitePool::connect(&url).await {
            Ok(v) => v,
            Err(err) => {
                error!("Error while connecting to database: {:?}", err);
                error!("Database url: {}", url);
                panic!("DB connection error");
            }
        };
        let mut conn = pool.acquire().await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS jobs (id INTEGER PRIMARY KEY, name TEXT, source TEXT, path TEXT)")
            .execute(&mut conn)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS executions (id INTEGER PRIMARY KEY, job_id INTEGER, start_time TIMESTAMP, status INTEGER, FOREIGN KEY (job_id) REFERENCES jobs(id))")
            .execute(&mut conn)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS schedules (id INTEGER PRIMARY KEY, job_id INTEGER, hour INTEGER, minute INTEGER, FOREIGN KEY (job_id) REFERENCES jobs(id))")
            .execute(&mut conn)
            .await?;
        Ok(DatabasePersistance { pool })
    }
}

#[async_trait::async_trait]
impl Storage for DatabasePersistance {
    async fn list_jobs(&self) -> StoreResult<Vec<i64>> {
        let mut conn = self.pool.acquire().await?;
        let ids = sqlx::query("SELECT id FROM jobs")
            .fetch_all(&mut conn)
            .await?
            .into_iter()
            .map(|row| row.get(0))
            .collect();
        return Ok(ids);
    }

    async fn store_job(&self, name: String, source: String, path: String) -> StoreResult<i64> {
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query("INSERT INTO jobs (name, source, path) VALUES ($1, $2, $3)")
            .bind(name)
            .bind(source)
            .bind(path)
            .execute(&mut conn)
            .await?
            .last_insert_rowid();
        Ok(id)
    }

    async fn fetch_job(&self, id: i64) -> StoreResult<JobPointer> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query("SELECT id, name, source, path FROM jobs WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut conn)
            .await?
            .map(|row| JobPointer {
                id: row.get(0),
                name: row.get(1),
                source: row.get(2),
                path: row.get(3),
            })
            .ok_or_else(|| StorageError::UserError(String::from("Invalid job id")))
    }

    async fn fetch_job_description(&self, _id: i64) -> StoreResult<osprei::Job> {
        todo!()
    }

    async fn create_execution(&self, job_id: i64) -> StoreResult<i64> {
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query(
            "INSERT INTO executions (job_id, start_time) VALUES ($1, CURRENT_TIMESTAMP)",
        )
        .bind(job_id)
        .execute(&mut conn)
        .await?
        .last_insert_rowid();
        Ok(id)
    }

    async fn set_execution_status(
        &self,
        id: i64,
        execution_status: osprei::ExecutionStatus,
    ) -> StoreResult<()> {
        let value: i64 = execution_status.into();
        let mut conn = self.pool.acquire().await?;
        sqlx::query("UPDATE executions SET status = $2 WHERE id = $1;")
            .bind(id)
            .bind(value)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    async fn get_execution(&self, id: i64) -> StoreResult<osprei::ExecutionDetails> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query("SELECT executions.id, jobs.name, start_time, status FROM executions JOIN jobs ON jobs.id = executions.job_id WHERE executions.id = $1")
            .bind(id)
            .fetch_optional(&mut conn)
            .await?
            .map(|row| {
                let status_encoded: Option<i64> = row.get(3);
                let status = status_encoded.map(osprei::ExecutionStatus::from);
                ExecutionDetails {
                    execution_id: row.get(0),
                    job_name: row.get(1),
                    start_time: row.get(2),
                    status
                }
            }).ok_or_else(|| StorageError::UserError(String::from("Invalid execution id")))
    }

    async fn last_executions(
        &self,
        job_id: i64,
        limit: usize,
    ) -> StoreResult<Vec<osprei::ExecutionSummary>> {
        let mut conn = self.pool.acquire().await?;
        let executions = sqlx::query("SELECT id, start_time FROM executions WHERE job_id = $1 ORDER BY start_time DESC LIMIT $2")
            .bind(job_id)
            .bind(limit as i64)
            .fetch_all(&mut conn)
            .await?
            .into_iter()
            .map(|row| ExecutionSummary {
                id: row.get(0),
                start_time: row.get(1)
            })
            .collect();
        Ok(executions)
    }

    async fn create_daily(
        &self,
        job_id: i64,
        request: osprei::ScheduleRequest,
    ) -> StoreResult<i64> {
        let osprei::ScheduleRequest { hour, minute } = request;
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query("INSERT INTO schedules (job_id, hour, minute) VALUES ($1, $2, $3)")
            .bind(job_id)
            .bind(hour)
            .bind(minute)
            .execute(&mut conn)
            .await?
            .last_insert_rowid();
        Ok(id)
    }

    async fn get_schedules(&self, job_id: i64) -> StoreResult<Vec<osprei::Schedule>> {
        let mut conn = self.pool.acquire().await?;
        let schedules =
            sqlx::query("SELECT id, job_id, hour, minute FROM schedules WHERE job_id = $1")
                .bind(job_id)
                .fetch_all(&mut conn)
                .await?
                .into_iter()
                .map(|row| osprei::Schedule {
                    schedule_id: row.get(0),
                    job_id: row.get(1),
                    hour: row.get(2),
                    minute: row.get(3),
                })
                .collect();
        Ok(schedules)
    }

    async fn get_all_schedules(&self) -> StoreResult<Vec<osprei::Schedule>> {
        let mut conn = self.pool.acquire().await?;
        let schedules = sqlx::query("SELECT id, job_id, hour, minute FROM schedules")
            .fetch_all(&mut conn)
            .await?
            .into_iter()
            .map(|row| osprei::Schedule {
                schedule_id: row.get(0),
                job_id: row.get(1),
                hour: row.get(2),
                minute: row.get(3),
            })
            .collect();
        Ok(schedules)
    }

    async fn get_last_execution(&self, _job_id: i64) -> StoreResult<Option<osprei::LastExecution>> {
        todo!()
    }
}

impl From<sqlx::Error> for StorageError {
    fn from(value: sqlx::Error) -> Self {
        Self::InternalError(value.to_string())
    }
}

#[cfg(test)]
mod test {
    use crate::test_store;
    use super::DatabasePersistance;

    async fn create() -> DatabasePersistance {
        DatabasePersistance::new(":memory:").await.unwrap()
    }

    test_store!(create);
}
