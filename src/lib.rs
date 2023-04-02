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

mod database {
    #[async_trait::async_trait]
    pub trait Persistance {
        async fn create_execution(&self, job_name: String) -> i64;
        async fn set_execution_status(&self, execution_id: i64, execution_status: i64);
    }

    #[derive(Debug, Clone)]
    pub struct DatabasePersistance {
        tx: tokio::sync::mpsc::Sender<DatabaseMessage>,
    }

    #[async_trait::async_trait]
    impl Persistance for DatabasePersistance {
        async fn create_execution(&self, job_name: String) -> i64 {
            todo!()
        }

        async fn set_execution_status(&self, execution_id: i64, execution_status: i64) {
            todo!()
        }
    }

    struct DatabaseMessage;
}
