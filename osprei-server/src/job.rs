use log::{info, warn};

mod model {
    #[derive(Debug, serde::Deserialize)]
    pub struct Job {
        pub name: String,
        pub stages: Vec<Stage>,
    }

    #[derive(Debug, serde::Deserialize)]
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

pub struct JobRunner {
    source_dir: String,
    output_writer: OutputWriter,
}

impl JobRunner {
    pub async fn execute(
        &self,
        job: model::Job,
    ) -> Result<Vec<model::StageExecutionSummary>, ExecutionError> {
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

    async fn execute_stage(
        &self,
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
        &self,
        _cmd: String,
        _args: Vec<String>,
        _path: String,
    ) -> Result<model::StageExecutionSummary, ExecutionError> {
        todo!()
    }

    async fn execute_source(
        &self,
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
}

impl OutputWriter {
    async fn write(
        &self,
        output: &std::process::Output,
    ) -> Result<model::WrittenResult, OutputWriteError> {
        let stdout_path = self.write_stdout(&output.stdout).await?;
        let stderr_path = self.write_stderr(&output.stderr).await?;
        Ok(model::WrittenResult {
            stdout_path,
            stderr_path,
        })
    }

    async fn write_stdout(&self, stdout: impl AsRef<[u8]>) -> Result<String, OutputWriteError> {
        let mut buf = std::path::PathBuf::from(&self.result_dir);
        buf.push("stdout.txt");
        let path = buf.to_string_lossy().into_owned();
        tokio::fs::write(&path, stdout).await?;
        info!("Writen stdout to: {}", path);
        Ok(path)
    }

    async fn write_stderr(&self, stderr: impl AsRef<[u8]>) -> Result<String, OutputWriteError> {
        let mut buf = std::path::PathBuf::from(&self.result_dir);
        buf.push("stderr.txt");
        let path = buf.to_string_lossy().into_owned();
        tokio::fs::write(&path, stderr).await?;
        info!("Written stderr to: {}", path);
        Ok(path)
    }
}

#[derive(Debug)]
pub struct OutputWriteError(tokio::io::Error);

impl From<tokio::io::Error> for OutputWriteError {
    fn from(value: tokio::io::Error) -> Self {
        OutputWriteError(value)
    }
}

impl std::fmt::Display for OutputWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "failed to write stage output: {}", self.0)
    }
}

impl std::error::Error for OutputWriteError {}
