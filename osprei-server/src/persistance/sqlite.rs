use log::{error, info};
use sqlx::Row;

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
        DatabasePersistance { pool }
    }
}

#[async_trait::async_trait]
impl super::Persistance for DatabasePersistance {
    async fn init(&self) {
        info!("Initializing database");
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS job ( id INTEGER PRIMARY KEY, job_name TEXT, source TEXT, path TEXT )").execute(&mut conn).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS execution ( id INTEGER PRIMARY KEY, start_time TIMESTAMP, end_time TIMESTAMP, status INTEGER, job_id INTEGER, FOREIGN KEY ( job_id ) REFERENCES job( id ) )").execute(&mut conn).await.unwrap();
        let canceled_executions =
            sqlx::query("UPDATE execution SET status = 2 WHERE status IS NULL")
                .execute(&mut conn)
                .await
                .unwrap()
                .rows_affected();
        if canceled_executions > 0 {
            info!(
                "Found unfinished jobs: marking jobs as cancelled: {}",
                canceled_executions
            );
        }
    }

    async fn create_execution(&self, job_name: String) -> i64 {
        info!("Creating database execution entry for: {}", job_name);
        let mut conn = self.pool.acquire().await.unwrap();
        let execution_id = sqlx::query(
            "INSERT INTO execution ( job_name, start_time ) VALUES ( ?1, datetime( 'now' ) )",
        )
        .bind(job_name)
        .execute(&mut conn)
        .await
        .unwrap()
        .last_insert_rowid();
        info!("Create execution with id: {}", execution_id);
        execution_id
    }

    async fn set_execution_status(&self, execution_id: i64, execution_status: i64) {
        info!(
            "Updating executions {} status to {}",
            execution_id, execution_status
        );
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("UPDATE execution SET status = ?2 WHERE id = ?1")
            .bind(execution_id)
            .bind(execution_status)
            .execute(&mut conn)
            .await
            .unwrap();
        info!("Updated executions {}", execution_id);
    }

    async fn get_execution(&self, execution_id: i64) -> osprei::ExecutionDetails {
        info!("Fetching execution with id {}", execution_id);
        let mut conn = self.pool.acquire().await.unwrap();
        let row =
            sqlx::query("SELECT id, job_name, start_time, status FROM execution WHERE id = ?1")
                .bind(execution_id)
                .fetch_one(&mut conn)
                .await
                .unwrap();
        let execution_id = row.try_get(0).unwrap();
        let job_name = row.try_get(1).unwrap();
        let start_time = row.try_get(2).unwrap();
        let status = row.try_get(3).unwrap();
        osprei::ExecutionDetails {
            execution_id,
            job_name,
            start_time,
            status,
        }
    }

    async fn last_executions(&self, job_name: String, limit: i64) -> Vec<osprei::ExecutionSummary> {
        info!(
            "Fetching last ({}) executions for job_name: {}",
            limit, job_name
        );
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query("SELECT id, start_time FROM execution WHERE job_name = ?1 ORDER BY start_time DESC LIMIT ?2").bind(job_name).bind(limit).fetch_all(&mut conn).await.unwrap().into_iter().map(|row| {
        let id = row.try_get(0).unwrap();
        let start_time = row.try_get(1).unwrap();
        osprei::ExecutionSummary { id, start_time}
    }).collect()
    }
}
