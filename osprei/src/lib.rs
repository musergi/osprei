use log::{info, warn};

pub use model::Job;
pub use model::Stage;

mod model {
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
}

struct ExecutionPaths {
    source_dir: String,
    result_dir: String,
}

impl ExecutionPaths {
    fn new(job_name: &str, data_path: &str) -> Self {
        let source_dir = ExecutionPaths::build_source_dir(data_path, job_name).unwrap();
        let result_dir = ExecutionPaths::build_result_dir(data_path, job_name).unwrap();
        Self {
            source_dir,
            result_dir,
        }
    }

    fn build_source_dir(data_path: &str, job_name: &str) -> Option<String> {
        let mut buf = std::path::PathBuf::from(data_path);
        buf.push(job_name);
        buf.push("workspace");
        buf.push(job_name);
        let path = buf.to_str()?.to_string();
        Some(path)
    }

    fn build_result_dir(data_path: &str, job_name: &str) -> Option<String> {
        let current_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Clock before epoch")
            .as_nanos();
        let mut buf = std::path::PathBuf::from(data_path);
        buf.push(format!("{}-{}", job_name, current_epoch));
        let path = buf.to_str()?.to_string();
        Some(path)
    }
}

pub struct JobRunner {
    source_dir: String,
    output_writer: OutputWriter,
}

impl JobRunner {
    pub fn new(job_name: &str, data_path: &str) -> Self {
        let ExecutionPaths {
            source_dir,
            result_dir,
        } = ExecutionPaths::new(job_name, data_path);
        let output_writer = OutputWriter::new(result_dir);
        Self {
            source_dir,
            output_writer,
        }
    }

    pub async fn execute(
        &mut self,
        job: model::Job,
    ) -> Result<Vec<model::StageExecutionSummary>, ExecutionError> {
        self.prepare_environment().await;
        let mut summaries = Vec::new();
        for stage in job.stages {
            let summary = self.execute_stage(stage).await?;
            let is_ok = summary.status == 0;
            summaries.push(summary);
            if !is_ok {
                return Ok(summaries);
            }
        }
        Ok(summaries)
    }

    async fn prepare_environment(&self) {
        tokio::fs::remove_dir_all(self.source_dir.clone())
            .await
            .expect("Delete previous environment");
    }

    async fn execute_stage(
        &mut self,
        stage: model::Stage,
    ) -> Result<model::StageExecutionSummary, ExecutionError> {
        let summary = match stage {
            model::Stage::Command { cmd, args, path } => {
                self.execute_cmd_stage(cmd, args, path).await?
            }
            model::Stage::Source { repository_url } => self.execute_source(repository_url).await?,
        };
        Ok(summary)
    }

    async fn execute_cmd_stage(
        &mut self,
        cmd: String,
        args: Vec<String>,
        path: String,
    ) -> Result<model::StageExecutionSummary, ExecutionError> {
        let mut buf = std::path::PathBuf::from(&self.source_dir);
        buf.push(path);
        let output = tokio::process::Command::new(cmd)
            .args(args)
            .current_dir(buf.as_path())
            .output()
            .await?;
        let logs = self.output_writer.write(&output).await?;
        if !output.status.success() {
            warn!("An error occured while running command");
            warn!("stdout: {}", logs.stdout_path);
            warn!("stderr: {}", logs.stderr_path);
        }
        let status = output.status.code().unwrap();
        Ok(model::StageExecutionSummary { status, logs })
    }

    async fn execute_source(
        &mut self,
        repo_url: String,
    ) -> Result<model::StageExecutionSummary, ExecutionError> {
        let output = tokio::process::Command::new("git")
            .args(vec!["clone", &repo_url, &self.source_dir])
            .output()
            .await?;
        let logs = self.output_writer.write(&output).await?;
        if !output.status.success() {
            warn!("An error occured while checking out code");
            warn!("stdout: {}", logs.stdout_path);
            warn!("stderr: {}", logs.stderr_path);
        }
        let status = output.status.code().unwrap();
        Ok(model::StageExecutionSummary { status, logs })
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    CommandSpawnError(std::io::Error),
    OutputDumpError(OutputWriteError),
}

impl From<OutputWriteError> for ExecutionError {
    fn from(value: OutputWriteError) -> Self {
        Self::OutputDumpError(value)
    }
}

impl From<std::io::Error> for ExecutionError {
    fn from(value: std::io::Error) -> Self {
        Self::CommandSpawnError(value)
    }
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CommandSpawnError(err) => write!(f, "failed to spawn process: {}", err),
            Self::OutputDumpError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for ExecutionError {}

struct OutputWriter {
    result_dir: String,
    counter: u32,
}

impl OutputWriter {
    fn new(result_dir: String) -> Self {
        Self {
            result_dir,
            counter: 0,
        }
    }

    async fn write(
        &mut self,
        output: &std::process::Output,
    ) -> Result<model::WrittenResult, OutputWriteError> {
        self.ensure_result_dir().await?;
        let stdout_path = self.write_stdout(&output.stdout).await?;
        let stderr_path = self.write_stderr(&output.stderr).await?;
        self.counter += 1;
        Ok(model::WrittenResult {
            stdout_path,
            stderr_path,
        })
    }

    async fn ensure_result_dir(&self) -> Result<(), OutputWriteError> {
        tokio::fs::create_dir_all(self.result_dir.clone())
            .await
            .map_err(|err| OutputWriteError(self.result_dir.clone(), err))
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
