pub mod execute;
pub mod persistance;
pub mod views;

#[derive(Debug, Clone)]
pub struct PathBuilder {
    workspace_dir: String,
    result_dir: String,
    _database_path: String,
}

impl PathBuilder {
    pub fn new(data_path: String) -> PathBuilder {
        let workspace_dir = build_workspace_path(&data_path);
        let _database_path = build_database_path(&data_path);
        let result_dir = build_result_path(&data_path);
        PathBuilder {
            workspace_dir,
            result_dir,
            _database_path,
        }
    }

    pub fn workspaces(&self) -> &str {
        &self.workspace_dir
    }

    pub fn workspace(&self, job_name: &str) -> String {
        let mut buf = std::path::PathBuf::from(&self.workspace_dir);
        buf.push(job_name);
        buf.to_string_lossy().to_string()
    }

    pub fn results(&self, job_name: &str, execution_id: i64) -> String {
        let mut buf = std::path::PathBuf::from(&self.result_dir);
        buf.push(job_name);
        buf.push(execution_id.to_string());
        buf.to_string_lossy().to_string()
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

fn build_result_path(data_path: &str) -> String {
    let mut buf = std::path::PathBuf::from(data_path);
    buf.push("results");
    buf.to_str().unwrap().to_string()
}
