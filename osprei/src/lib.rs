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
    /// Environment variables for execution
    pub env: Vec<EnvironmentDefinition>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct EnvironmentDefinition {
    /// Name of the variable to set
    pub key: String,
    /// Value of the variable to set
    pub value: String,
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
    pub status: Option<ExecutionStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ExecutionStatus {
    Success,
    Failed,
    InvalidConfig,
}

impl From<i64> for ExecutionStatus {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::Success,
            1 => Self::Failed,
            2 => Self::InvalidConfig,
            _ => panic!("Corrupt database"),
        }
    }
}

impl From<ExecutionStatus> for i64 {
    fn from(value: ExecutionStatus) -> Self {
        match value {
            ExecutionStatus::Success => 0,
            ExecutionStatus::Failed => 1,
            ExecutionStatus::InvalidConfig => 2,
        }
    }
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LastExecutionResponse {
    /// If executed the last execution
    pub last: Option<LastExecution>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LastExecution {
    /// Unique identifier for the execution
    pub id: i64,
    /// Start time of the executio
    pub start_time: String,
    /// Status of the execution
    pub status: Option<ExecutionStatus>,
}
