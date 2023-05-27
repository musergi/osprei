#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Job {
    pub stages: Vec<Stage>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Stage {
    /// Command to run
    pub cmd: String,
    /// Arguments passed to the command
    pub args: Vec<String>,
    /// Working directory, relative to source root
    pub path: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct JobPointer {
    /// Unique identifier for the job
    pub id: i64,
    /// A human readable identifier for the job
    pub name: String,
    /// Git repository for the job
    pub source: String,
    /// Path, relative to the repository root, of the job definition
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct JobCreationRequest {
    /// A human readable identifier for the job
    pub name: String,
    /// Git repository for the job
    pub source: String,
    /// Path, relative to the repository root, of the job definition
    pub path: String,
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Schedule {
    /// Unique identifier for the schedule
    pub schedule_id: i64,
    /// Job the schedule runs
    pub job_id: i64,
    /// Hour of day the job runs
    pub hour: u8,
    /// Minute of day the job runs
    pub minute: u8,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ScheduleRequest {
    /// Hour of day the job runs
    pub hour: u8,
    /// Minute of day the job runs
    pub minute: u8,
}
