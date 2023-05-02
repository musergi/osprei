#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Job {
    pub name: String,
    pub stages: Vec<Stage>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum Stage {
    Command {
        cmd: String,
        args: Vec<String>,
        path: String,
    },
    Source {
        repository_url: String,
    },
}

#[derive(Debug)]
pub struct StageExecutionSummary {
    pub status: i32,
    pub logs: WrittenResult,
}

#[derive(Debug)]
pub struct WrittenResult {
    pub stdout_path: String,
    pub stderr_path: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ExecutionSummary {
    pub id: i64,
    pub start_time: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ExecutionDetails {
    pub execution_id: i64,
    pub job_name: String,
    pub start_time: String,
    pub status: Option<i64>,
}
