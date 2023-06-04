use log::error;
use osprei::{ExecutionDetails, ExecutionSummary, Job, JobPointer};
use sqlx::Row;

use super::{ExecutionStore, JobStore, ScheduleStore, Store};

#[derive(Debug, Clone)]
pub struct DatabasePersistance {
    pool: sqlx::SqlitePool,
}

impl DatabasePersistance {
    pub async fn new(database_path: &str) -> DatabasePersistance {
        let url = database_path.to_string() + "?mode=rwc";
        let pool = match sqlx::sqlite::SqlitePool::connect(&url).await {
            Ok(v) => v,
            Err(err) => {
                error!("Error while connecting to database: {:?}", err);
                error!("Database url: {}", url);
                panic!("DB connection error");
            }
        };
        let mut conn = pool.acquire().await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS jobs (id INTEGER PRIMARY KEY, name TEXT, source TEXT, path TEXT)")
            .execute(&mut conn)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS executions (id INTEGER PRIMARY KEY, job_id INTEGER, start_time TIMESTAMP, status INTEGER, FOREIGN KEY (job_id) REFERENCES jobs(id))")
            .execute(&mut conn)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS schedules (id INTEGER PRIMARY KEY, job_id INTEGER, hour INTEGER, minute INTEGER, FOREIGN KEY (job_id) REFERENCES jobs(id))")
            .execute(&mut conn)
            .await
            .unwrap();
        DatabasePersistance { pool }
    }
}

#[async_trait::async_trait]
impl JobStore for DatabasePersistance {
    async fn list_jobs(&self) -> Vec<i64> {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("SELECT id FROM jobs")
            .fetch_all(&mut conn)
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get(0))
            .collect()
    }

    async fn store_job(&self, name: String, source: String, path: String) -> i64 {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("INSERT INTO jobs (name, source, path) VALUES ($1, $2, $3)")
            .bind(name)
            .bind(source)
            .bind(path)
            .execute(&mut conn)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    async fn fetch_job(&self, id: i64) -> JobPointer {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("SELECT id, name, source, path FROM jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&mut conn)
            .await
            .map(|row| JobPointer {
                id: row.get(0),
                name: row.get(1),
                source: row.get(2),
                path: row.get(3),
            })
            .unwrap()
    }

    async fn fetch_job_description(&self, _id: i64) -> Job {
        todo!()
    }
}

#[async_trait::async_trait]
impl ExecutionStore for DatabasePersistance {
    async fn create_execution(&self, job_id: i64) -> i64 {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("INSERT INTO executions (job_id, start_time) VALUES ($1, CURRENT_TIMESTAMP)")
            .bind(job_id)
            .execute(&mut conn)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    async fn set_execution_status(&self, id: i64, execution_status: i64) {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("UPDATE executions SET status = $2 WHERE id = $1;")
            .bind(id)
            .bind(execution_status)
            .execute(&mut conn)
            .await
            .unwrap();
    }

    async fn get_execution(&self, id: i64) -> ExecutionDetails {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("SELECT executions.id, jobs.name, start_time, status FROM executions JOIN jobs ON jobs.id = executions.job_id WHERE executions.id = $1")
            .bind(id)
            .fetch_one(&mut conn)
            .await
            .map(|row| ExecutionDetails {
                execution_id: row.get(0),
                job_name: row.get(1),
                start_time: row.get(2),
                status: row.get(3)
            })
            .unwrap()
    }

    async fn last_executions(&self, job_id: i64, limit: usize) -> Vec<ExecutionSummary> {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("SELECT id, start_time FROM executions WHERE job_id = $1 ORDER BY start_time DESC LIMIT $2")
            .bind(job_id)
            .bind(limit as i64)
            .fetch_all(&mut conn)
            .await
            .unwrap()
            .into_iter()
            .map(|row| ExecutionSummary {
                id: row.get(0),
                start_time: row.get(1)
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl ScheduleStore for DatabasePersistance {
    async fn create_daily(&self, job_id: i64, request: osprei::ScheduleRequest) -> i64 {
        let osprei::ScheduleRequest { hour, minute } = request;
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("INSERT INTO schedules (job_id, hour, minute) VALUES ($1, $2, $3)")
            .bind(job_id)
            .bind(hour)
            .bind(minute)
            .execute(&mut conn)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    async fn get_schedules(&self, job_id: i64) -> Vec<osprei::Schedule> {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("SELECT id, job_id, hour, minute FROM schedules WHERE job_id = $1")
            .bind(job_id)
            .fetch_all(&mut conn)
            .await
            .unwrap()
            .into_iter()
            .map(|row| osprei::Schedule {
                schedule_id: row.get(0),
                job_id: row.get(1),
                hour: row.get(2),
                minute: row.get(3),
            })
            .collect()
    }
    async fn get_all_schedules(&self) -> Vec<osprei::Schedule> {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("SELECT id, job_id, hour, minute FROM schedules")
            .fetch_all(&mut conn)
            .await
            .unwrap()
            .into_iter()
            .map(|row| osprei::Schedule {
                schedule_id: row.get(0),
                job_id: row.get(1),
                hour: row.get(2),
                minute: row.get(3),
            })
            .collect()
    }
}

impl Store for DatabasePersistance {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::persistance::tests::{test_execution_store, test_job_store, test_schedule_store};

    #[tokio::test]
    async fn test_sqlite_job_store() {
        let store = DatabasePersistance::new(":memory:").await;
        test_job_store(store).await;
    }

    #[tokio::test]
    async fn test_sqlite_execution_store() {
        let store = DatabasePersistance::new(":memory:").await;
        test_execution_store(store).await;
    }

    #[tokio::test]
    async fn test_sqlite_schedule_store() {
        let store = DatabasePersistance::new(":memory:").await;
        test_schedule_store(store).await;
    }
}
