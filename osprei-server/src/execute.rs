use chrono::DurationRound;
use log::{debug, error, info, warn};
use osprei::{EnvironmentDefinition, Job, Stage, StageExecutionSummary};
use std::collections::HashMap;

pub struct JobDescriptor {
    /// Clone directory for the source repo
    pub execution_dir: String,
    /// Git repository to clone
    pub source: String,
    /// Path within the repository of the job definition
    pub path: String,
}

impl JobDescriptor {
    pub async fn execute_job(
        self,
    ) -> Result<(osprei::ExecutionStatus, String, String), ExecutionError> {
        let JobDescriptor {
            execution_dir,
            source,
            path,
        } = self;
        if tokio::fs::remove_dir_all(&execution_dir).await.is_ok() {
            info!("Clean up directory: {}", execution_dir);
        }
        let StageResult {
            mut status,
            mut stdout,
            mut stderr,
        } = checkout_repo(&execution_dir, &source).await?;
        if status == osprei::ExecutionStatus::Success {
            info!("Code checkout complete for: {}", source);
            let definition_path = joined(&execution_dir, &path);
            let job_definition = read_job_definition(definition_path).await?;
            debug!("Read job definition: {:?}", job_definition);
            let stage_executor = StageExecutor::new(&execution_dir);
            for stage in job_definition.stages {
                debug!("Running stage: {:?}", stage);
                let result = stage_executor.execute(stage).await?;
                status = result.status;
                stdout += &result.stdout;
                stderr += &result.stderr;
            }
        }
        Ok((status, stdout, stderr))
    }
}

#[derive(Debug, Clone)]
struct StageResult {
    status: osprei::ExecutionStatus,
    stdout: String,
    stderr: String,
}

impl StageResult {
    fn convert_output(out: Vec<u8>) -> Result<String, ExecutionError> {
        String::from_utf8(out).map_err(|err| ExecutionError::StringConversionError(err))
    }
}

impl TryFrom<std::process::Output> for StageResult {
    type Error = ExecutionError;

    fn try_from(
        std::process::Output {
            status,
            stdout,
            stderr,
        }: std::process::Output,
    ) -> Result<Self, Self::Error> {
        let status = match status.code() {
            Some(0) => osprei::ExecutionStatus::Success,
            _ => osprei::ExecutionStatus::Failed,
        };
        Ok(StageResult {
            status,
            stdout: Self::convert_output(stdout)?,
            stderr: Self::convert_output(stderr)?,
        })
    }
}

struct StageExecutor<'a> {
    working_dir: &'a str,
}

impl<'a> StageExecutor<'a> {
    fn new(working_dir: &'a str) -> StageExecutor<'a> {
        StageExecutor { working_dir }
    }

    async fn execute(&'a self, stage: Stage) -> Result<StageResult, ExecutionError> {
        debug!("Running stage: {:?}", stage);
        let Stage {
            cmd,
            args,
            path,
            env,
        } = stage;
        let path = joined(self.working_dir, &path);
        let env: HashMap<_, _> = env
            .into_iter()
            .map(|EnvironmentDefinition { key, value }| (key, value))
            .collect();
        let output = tokio::process::Command::new(&cmd)
            .args(&args)
            .current_dir(&path)
            .envs(env)
            .output()
            .await
            .map_err(|err| {
                error!("Error occured when spawning suprocess, dumping details.");
                error!("Command: {}", cmd);
                error!("Arguments: {:?}", args);
                error!("Path: {}", path);
                ExecutionError::SubProccess(err)
            })?;
        StageResult::try_from(output)
    }
}

async fn checkout_repo(execution_dir: &str, source: &str) -> Result<StageResult, ExecutionError> {
    let output = tokio::process::Command::new("git")
        .arg("clone")
        .arg(source)
        .arg(execution_dir)
        .output()
        .await
        .map_err(ExecutionError::SubProccess)?;
    StageResult::try_from(output)
}

async fn read_job_definition(path: String) -> Result<Job, ExecutionError> {
    debug!("Reading job definition from {}", path);
    let job_definition = tokio::fs::read_to_string(&path)
        .await
        .map_err(|err| ExecutionError::MissingDefinition { path, err })?;
    Ok(serde_json::from_str(&job_definition)?)
}

fn joined(base: &str, suffix: &str) -> String {
    std::path::PathBuf::from(base)
        .join(suffix)
        .to_string_lossy()
        .to_string()
}

pub async fn write_result(
    execution_id: i64,
    stage_summaries: &[StageExecutionSummary],
    store: &dyn crate::persistance::Storage,
) {
    let any_failed = stage_summaries.iter().any(|output| output.status != 0);
    let status = match any_failed {
        false => osprei::ExecutionStatus::Success,
        true => osprei::ExecutionStatus::Failed,
    };
    let result = store.set_execution_status(execution_id, status).await;
    if let Err(err) = result {
        warn!(
            "Failed to store status for execution ({}): {}",
            execution_id, err
        );
    }
}

pub async fn write_error(execution_id: i64, store: &dyn crate::persistance::Storage) {
    let result = store
        .set_execution_status(execution_id, osprei::ExecutionStatus::InvalidConfig)
        .await;
    if let Err(err) = result {
        warn!(
            "Failed to store error for execution ({}): {}",
            execution_id, err
        );
    }
}

pub async fn schedule_all(
    persistance: crate::persistance::Persistances,
    path_builder: crate::PathBuilder,
) {
    match persistance.boxed().get_all_schedules().await {
        Ok(schedules) => {
            for schedule in schedules {
                let osprei::Schedule {
                    job_id,
                    hour,
                    minute,
                    ..
                } = schedule;
                match persistance.boxed().fetch_job(job_id).await {
                    Ok(job) => {
                        debug!("Scheduling {} for {}h{}", job.name, hour, minute);
                        schedule_job(job, hour, minute, path_builder.clone(), persistance.boxed())
                            .await;
                    }
                    Err(err) => {
                        error!("Failed to schedule {}: {}", job_id, err);
                    }
                }
            }
        }
        Err(err) => {
            error!("Failed to fetch schedules: {}", err);
            error!("No jobs will be scheduled")
        }
    }
}

pub async fn schedule_job(
    job: osprei::JobPointer,
    hour: u8,
    minute: u8,
    path_builder: crate::PathBuilder,
    store: Box<dyn crate::persistance::Storage>,
) {
    let osprei::JobPointer {
        id,
        name,
        source,
        path,
    } = job;

    tokio::spawn(async move {
        match create_interval(hour, minute) {
            Ok(mut interval) => {
                debug!("Created loop to run {}", name);
                loop {
                    interval.tick().await;
                    match store.create_execution(id).await {
                        Ok(execution_id) => {
                            let descriptor = JobDescriptor {
                                execution_dir: path_builder.workspace(&name),
                                source: source.clone(),
                                path: path.clone(),
                            };
                            match descriptor.execute_job().await {
                                Ok((status, stdout, stderr)) => {
                                    if let Err(err) = store
                                        .set_execution_result(execution_id, stdout, stderr, status)
                                        .await
                                    {
                                        error!("An error occured storing execution result: {}", err)
                                    }
                                }
                                Err(err) => {
                                    error!("An error occurred during job executions: {}", err)
                                }
                            }
                        }
                        Err(err) => error!("Error creating execution: {}", err),
                    }
                }
            }
            Err(err) => {
                error!("Could not schedule: {}", err);
            }
        }
    });
}

fn create_interval(hour: u8, minute: u8) -> Result<tokio::time::Interval, IntervalCreationError> {
    let now = chrono::Utc::now();
    let mut start = now
        .duration_trunc(chrono::Duration::days(1))?
        .checked_add_signed(chrono::Duration::minutes(hour as i64 * 60 + minute as i64))
        .ok_or(IntervalCreationError::TimestampArithmetic)?;
    if start < now {
        start = start
            .checked_add_signed(chrono::Duration::days(1))
            .ok_or(IntervalCreationError::TimestampArithmetic)?;
    }
    let offset = start.signed_duration_since(now);
    Ok(tokio::time::interval_at(
        tokio::time::Instant::now() + offset.to_std()?,
        std::time::Duration::from_secs(24 * 60 * 60),
    ))
}

#[derive(Debug)]
enum IntervalCreationError {
    Truncation(chrono::RoundingError),
    TimestampArithmetic,
    DateCast(chrono::OutOfRangeError),
}

impl std::fmt::Display for IntervalCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Truncation(err) => write!(f, "failed to perform daily truncation: {}", err),
            Self::TimestampArithmetic => write!(f, "failed to perform specified offset"),
            Self::DateCast(err) => write!(f, "failed to cast date: {}", err),
        }
    }
}

impl From<chrono::RoundingError> for IntervalCreationError {
    fn from(value: chrono::RoundingError) -> Self {
        IntervalCreationError::Truncation(value)
    }
}

impl From<chrono::OutOfRangeError> for IntervalCreationError {
    fn from(value: chrono::OutOfRangeError) -> Self {
        IntervalCreationError::DateCast(value)
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    SubProccess(std::io::Error),
    OutputWriteError(OutputWriteError),
    MissingDefinition { path: String, err: std::io::Error },
    DefinitionSyntaxError(serde_json::Error),
    StringConversionError(std::string::FromUtf8Error),
}

impl From<OutputWriteError> for ExecutionError {
    fn from(value: OutputWriteError) -> Self {
        Self::OutputWriteError(value)
    }
}

impl From<serde_json::Error> for ExecutionError {
    fn from(value: serde_json::Error) -> Self {
        Self::DefinitionSyntaxError(value)
    }
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExecutionError::SubProccess(err) => write!(f, "error starting process: {}", err),
            ExecutionError::OutputWriteError(err) => write!(f, "error writing output: {}", err),
            ExecutionError::MissingDefinition { path, err } => {
                write!(f, "error reading definition: {}: {}", err, path)
            }
            ExecutionError::DefinitionSyntaxError(err) => {
                write!(f, "definition syntax error: {}", err)
            }
            ExecutionError::StringConversionError(err) => {
                write!(f, "failed to convert output to string: {}", err)
            }
        }
    }
}

impl std::error::Error for ExecutionError {}

#[derive(Debug)]
pub struct OutputWriteError(String, tokio::io::Error);

impl std::fmt::Display for OutputWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "failed to write stage output: {}: {}", self.0, self.1)
    }
}

impl std::error::Error for OutputWriteError {}

#[cfg(test)]
mod test {
    use super::create_interval;

    #[tokio::test]
    async fn test_interval_generation() {
        create_interval(12, 0).unwrap();
    }
}
