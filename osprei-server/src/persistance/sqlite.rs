use log::error;
use osprei::{ExecutionDetails, JobPointer};
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
        sqlx::query(
            "
                CREATE TABLE IF NOT EXISTS
                    jobs
                (
                    id INTEGER PRIMARY KEY,
                    name TEXT,
                    source TEXT,
                    path TEXT
                )
            ",
        )
        .execute(&mut conn)
        .await?;
        sqlx::query(
            "
                CREATE TABLE IF NOT EXISTS
                    executions
                (
                    id INTEGER PRIMARY KEY,
                    job_id INTEGER,
                    start_time TIMESTAMP,
                    status INTEGER,
                    stdout TEXT,
                    stderr TEXT,
                    FOREIGN KEY (job_id) REFERENCES jobs(id)
                )
            ",
        )
        .execute(&mut conn)
        .await?;
        sqlx::query(
            "
                CREATE TABLE IF NOT EXISTS
                    schedules
                (
                    id INTEGER PRIMARY KEY,
                    job_id INTEGER,
                    hour INTEGER,
                    minute INTEGER,
                    FOREIGN KEY (job_id) REFERENCES jobs(id)
                )
            ",
        )
        .execute(&mut conn)
        .await?;
        Ok(DatabasePersistance { pool })
    }
}

#[async_trait::async_trait]
impl Storage for DatabasePersistance {
    async fn list_jobs_new(&self) -> StoreResult<Vec<osprei::JobOverview>> {
        let mut conn = self.pool.acquire().await?;
        let jobs = sqlx::query(
            "
            SELECT 
                jid,
                name,
                eid,
                start_time,
                status
            FROM (
                SELECT
                    jobs.id AS jid,
                    name,
                    executions.id AS eid,
                    start_time,
                    status,
                    row_number() OVER ( partition BY jobs.id ORDER BY start_time ) AS score
                FROM (
                    jobs
                    LEFT JOIN executions
                )
            )
            WHERE  score = 1
            ",
        )
        .fetch_all(&mut conn)
        .await?
        .into_iter()
        .map(|row| {
            let id: i64 = row.get(0);
            let name: String = row.get(1);
            let execution_id: Option<i64> = row.get(2);
            let start_time: Option<String> = row.get(3);
            let status: Option<i64> = row.get(4);
            let status = status.map(&osprei::ExecutionStatus::from);
            let last_execution = match (execution_id, start_time) {
                (Some(id), Some(start_time)) => Some(osprei::LastExecution {
                    id,
                    start_time,
                    status,
                }),
                _ => None,
            };
            osprei::JobOverview {
                id,
                name,
                last_execution,
            }
        })
        .collect();
        Ok(jobs)
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

    async fn set_execution_result(
        &self,
        id: i64,
        stdout: String,
        stderr: String,
        execution_status: osprei::ExecutionStatus,
    ) -> StoreResult<()> {
        let value: i64 = execution_status.into();
        let mut conn = self.pool.acquire().await?;
        sqlx::query(
            "
            UPDATE
                executions
            SET
                status = $2,
                stdout = $3,
                stderr = $4
            WHERE id = $1;
            ",
        )
        .bind(id)
        .bind(value)
        .bind(stdout)
        .bind(stderr)
        .execute(&mut conn)
        .await?;
        Ok(())
    }

    async fn get_execution(&self, id: i64) -> StoreResult<osprei::ExecutionDetails> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query(
            "
            SELECT
                executions.id,
                jobs.name,
                start_time,
                status,
                stdout,
                stderr
            FROM
                executions
                JOIN jobs
                    ON jobs.id = executions.job_id
            WHERE
                executions.id = $1
            ",
        )
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
                status,
                stdout: row.get(4),
                stderr: row.get(5),
            }
        })
        .ok_or_else(|| StorageError::UserError(String::from("Invalid execution id")))
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
}

impl From<sqlx::Error> for StorageError {
    fn from(value: sqlx::Error) -> Self {
        Self::InternalError(value.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::DatabasePersistance;
    use crate::test_store;

    async fn create() -> DatabasePersistance {
        DatabasePersistance::new(":memory:").await.unwrap()
    }

    test_store!(create);
}
