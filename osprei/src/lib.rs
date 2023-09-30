#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Job {
    /// Path to the repository
    pub source: String,
    /// Image to run the job in
    pub image: String,
    /// Command to run
    pub command: String,
    /// Arguments passed to the command
    pub arguments: Vec<String>,
    /// Working directory, relative to source root
    pub working_directory: String,
    /// Environment variables for execution
    pub environment: Vec<EnvironmentDefinition>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
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
    /// Definition of the job
    pub definition: Job,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct JobCreationRequest {
    /// A human readable identifier for the job
    pub name: String,
    /// Git repository for the job
    pub definition: Job,
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
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct LastExecution {
    /// Unique identifier for the execution
    pub id: i64,
    /// Start time of the executio
    pub start_time: String,
    /// Status of the execution
    pub status: Option<ExecutionStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct JobOverview {
    pub id: i64,
    pub name: String,
    pub last_execution: Option<LastExecution>,
}
