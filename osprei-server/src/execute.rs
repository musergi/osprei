use chrono::DurationRound;
use log::{debug, error, info};
use osprei::{Job, Stage, StageExecutionSummary};

pub struct JobExecutionOptions {
    /// Clone directory for the source repo
    pub execution_dir: String,
    /// Dump directory for this execution output
    pub result_dir: String,
    /// Git repository to clone
    pub source: String,
    /// Path within the repository of the job definition
    pub path: String,
}

pub async fn execute_job(
    JobExecutionOptions {
        execution_dir,
        result_dir,
        source,
        path,
    }: JobExecutionOptions,
) -> Result<Vec<StageExecutionSummary>, ExecutionError> {
    if tokio::fs::remove_dir_all(&execution_dir).await.is_ok() {
        info!("Clean up directory: {}", execution_dir);
    }
    let output_writer = OutputWriter::new(result_dir).await?;
    let mut output_builder = OutputBuilder::new(output_writer);
    checkout_repo(&mut output_builder, &execution_dir, &source).await?;
    if output_builder.last_stage_successful() {
        info!("Code checkout complete for: {}", source);
        let definition_path = joined(&execution_dir, &path);
        debug!("Reading job definition from {}", definition_path);
        let job_definition = tokio::fs::read_to_string(&path).await.map_err(|err| {
            ExecutionError::MissingDefinition {
                path: definition_path,
                err,
            }
        })?;
        let job_definition: Job = serde_json::from_str(&job_definition)?;
        debug!("Read job definition: {:?}", job_definition);
        for stage in job_definition.stages {
            debug!("Running stage: {:?}", stage);
            let Stage { cmd, args, path } = stage;
            let path = joined(&execution_dir, &path);
            let output = tokio::process::Command::new(&cmd)
                .args(&args)
                .current_dir(&path)
                .output()
                .await
                .map_err(|err| {
                    error!("Error occured when spawning suprocess, dumping details.");
                    error!("Command: {}", cmd);
                    error!("Arguments: {:?}", args);
                    error!("Path: {}", path);
                    ExecutionError::SubProccess(err)
                })?;
            output_builder.add(&output).await?;
            if !output_builder.last_stage_successful() {
                break;
            }
        }
    }
    Ok(output_builder.build())
}

async fn checkout_repo(output_builder: &mut OutputBuilder, execution_dir: &str, source: &str) -> Result<(), ExecutionError> {
    let output = tokio::process::Command::new("git")
        .arg("clone")
        .arg(source)
        .arg(execution_dir)
        .output()
        .await
        .map_err(ExecutionError::SubProccess)?;
    output_builder.add(&output).await?;
    Ok(())
}

fn joined(base: &str, suffix: &str) -> String {
    std::path::PathBuf::from(base)
        .join(suffix)
        .to_string_lossy()
        .to_string()
}

struct OutputBuilder {
    outputs: Vec<StageExecutionSummary>,
    output_writer: OutputWriter,
}

impl OutputBuilder {
    fn new(output_writer: OutputWriter) -> Self {
        OutputBuilder {
            outputs: Vec::new(),
            output_writer,
        }
    }

    fn last_stage_successful(&self) -> bool {
        self.outputs
            .last()
            .map(|summary| summary.status == 0)
            .unwrap_or(true)
    }

    async fn add(&mut self, output: &std::process::Output) -> Result<(), OutputWriteError> {
        let logs = self.output_writer.write(output).await?;
        let summary = StageExecutionSummary {
            status: output.status.code().unwrap_or(-1),
            logs,
        };
        self.outputs.push(summary);
        Ok(())
    }

    fn build(self) -> Vec<StageExecutionSummary> {
        self.outputs
    }
}

pub async fn write_result(
    execution_id: i64,
    stage_summaries: &[StageExecutionSummary],
    store: &dyn crate::persistance::Store,
) {
    let any_failed = stage_summaries.iter().any(|output| output.status != 0);
    let status = match any_failed {
        false => 0,
        true => 1,
    };
    store.set_execution_status(execution_id, status).await;
}

pub async fn schedule_job(
    job: osprei::JobPointer,
    hour: u8,
    minute: u8,
    path_builder: crate::PathBuilder,
    store: Box<dyn crate::persistance::Store>,
) {
    let osprei::JobPointer {
        id,
        name,
        source,
        path,
    } = job;

    tokio::spawn(async move {
        debug!("Created loop to run {}", name);
        let mut interval = create_intervel(hour, minute);
        loop {
            interval.tick().await;
            let execution_id = store.create_execution(id).await;
            let options = JobExecutionOptions {
                execution_dir: path_builder.workspace(&name),
                result_dir: path_builder.results(&name, execution_id),
                source: source.clone(),
                path: path.clone(),
            };
            match execute_job(options).await {
                Ok(outputs) => {
                    write_result(execution_id, &outputs, store.as_ref()).await;
                }
                Err(err) => error!("An error occurred during job executions: {}", err),
            }
        }
    });
}

fn create_intervel(hour: u8, minute: u8) -> tokio::time::Interval {
    let now = chrono::Utc::now();
    let start = now
        .duration_trunc(chrono::Duration::days(1))
        .unwrap()
        .checked_add_signed(chrono::Duration::minutes(hour as i64 * 60 + minute as i64))
        .unwrap()
        .signed_duration_since(now);
    tokio::time::interval_at(
        tokio::time::Instant::now() + start.to_std().unwrap(),
        std::time::Duration::from_secs(24 * 60 * 60),
    )
}

#[derive(Debug)]
pub enum ExecutionError {
    SubProccess(std::io::Error),
    OutputWriteError(OutputWriteError),
    MissingDefinition { path: String, err: std::io::Error },
    DefinitionSyntaxError(serde_json::Error),
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
        }
    }
}

impl std::error::Error for ExecutionError {}

struct OutputWriter {
    result_dir: String,
    counter: u32,
}

impl OutputWriter {
    async fn new(result_dir: String) -> Result<Self, OutputWriteError> {
        tokio::fs::create_dir_all(&result_dir)
            .await
            .map_err(|err| OutputWriteError(result_dir.clone(), err))?;
        Ok(Self {
            result_dir,
            counter: 0,
        })
    }

    async fn write(
        &mut self,
        output: &std::process::Output,
    ) -> Result<osprei::WrittenResult, OutputWriteError> {
        let stdout_path = self.write_stdout(&output.stdout).await?;
        let stderr_path = self.write_stderr(&output.stderr).await?;
        self.counter += 1;
        Ok(osprei::WrittenResult {
            stdout_path,
            stderr_path,
        })
    }

    async fn write_stdout(&self, stdout: impl AsRef<[u8]>) -> Result<String, OutputWriteError> {
        let mut buf = std::path::PathBuf::from(&self.result_dir);
        buf.push(format!("stdout{}.txt", self.counter));
        let path = buf.to_string_lossy().into_owned();
        tokio::fs::write(&path, stdout)
            .await
            .map_err(|err| OutputWriteError(path.clone(), err))?;
        info!("Writen stdout to: {}", path);
        Ok(path)
    }

    async fn write_stderr(&self, stderr: impl AsRef<[u8]>) -> Result<String, OutputWriteError> {
        let mut buf = std::path::PathBuf::from(&self.result_dir);
        buf.push(format!("stderr{}.txt", self.counter));
        let path = buf.to_string_lossy().into_owned();
        tokio::fs::write(&path, stderr)
            .await
            .map_err(|err| OutputWriteError(path.clone(), err))?;
        info!("Written stderr to: {}", path);
        Ok(path)
    }
}

#[derive(Debug)]
pub struct OutputWriteError(String, tokio::io::Error);

impl std::fmt::Display for OutputWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "failed to write stage output: {}: {}", self.0, self.1)
    }
}

impl std::error::Error for OutputWriteError {}
