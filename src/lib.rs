#[derive(Debug)]
pub struct PathBuilder {
    job_path: String,
    workspace_dir: String,
    database_path: String,
}

impl PathBuilder {
    pub fn new(job_path: String, data_path: String) -> PathBuilder {
        let workspace_dir = build_workspace_path(&data_path);
        let database_path = build_database_path(&data_path);
        PathBuilder {
            job_path,
            workspace_dir,
            database_path,
        }
    }

    pub fn job_path(&self) -> &str {
        &self.job_path
    }

    pub fn database_path(&self) -> &str {
        &self.database_path
    }

    pub fn workspace_dir(&self) -> &str {
        &self.workspace_dir
    }
}

fn build_workspace_path(data_path: &str) -> String {
    let mut buf = std::path::PathBuf::from(data_path);
    buf.push("workspaces");
    buf.to_str().unwrap().to_string()
}

fn build_database_path(data_path: &str) -> String {
    let mut buf = std::path::PathBuf::from(data_path);
    buf.push("data.sqlite");
    buf.to_str().unwrap().to_string()
}

pub mod database {
    use log::info;

    #[async_trait::async_trait]
    pub trait Persistance {
        async fn create_execution(&self, job_name: String) -> i64;
        async fn set_execution_status(&self, execution_id: i64, execution_status: i64);
    }

    #[derive(Debug, Clone)]
    pub struct DatabasePersistance {
        pool: sqlx::SqlitePool,
    }

    impl DatabasePersistance {
        pub async fn new(database_path: &str) -> DatabasePersistance {
            let pool = sqlx::sqlite::SqlitePool::connect(database_path)
                .await
                .unwrap();
            DatabasePersistance { pool }
        }
    }

    #[async_trait::async_trait]
    impl Persistance for DatabasePersistance {
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
    }
}
