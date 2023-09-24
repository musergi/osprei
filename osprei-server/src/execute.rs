use chrono::DurationRound;
use log::{debug, error, info, warn};
use osprei::{EnvironmentDefinition, Job, Schedule, Stage, StageExecutionSummary};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

use crate::persistance;

#[derive(Debug, Clone)]
pub struct Report {
    pub status: osprei::ExecutionStatus,
    pub stdout: String,
    pub stderr: String,
}

impl From<std::process::Output> for Report {
    fn from(value: std::process::Output) -> Self {
        let std::process::Output {
            status,
            stdout,
            stderr,
        } = value;
        let status = match status.code() {
            Some(0) => osprei::ExecutionStatus::Success,
            _ => osprei::ExecutionStatus::Failed,
        };
        let stdout = String::from_utf8_lossy(&stdout).to_string();
        let stderr = String::from_utf8_lossy(&stderr).to_string();
        Self {
            status,
            stdout,
            stderr,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    tx: mpsc::Sender<Message>,
}

impl Client {
    pub async fn execute(&self, job_id: i64) -> Result<i64, String> {
        let (tx, rx) = oneshot::channel();
        let message = Message::Execute {
            job_id,
            response: tx,
        };
        self.tx.send(message).await.unwrap();
        rx.await
            .unwrap()
            .ok_or_else(|| "failed to assign execution id".to_string())
    }

    pub async fn create_schedule(&self, job_id: i64, hour: u8, minute: u8) -> Result<i64, String> {
        let (tx, rx) = oneshot::channel();
        let message = Message::Schedule {
            job_id,
            hour,
            minute,
            response: tx,
        };
        self.tx.send(message).await.unwrap();
        rx.await.unwrap().map_err(|err| err.to_string())
    }
}

#[derive(Debug)]
pub enum Message {
    Execute {
        job_id: i64,
        response: oneshot::Sender<Option<i64>>,
    },
    Schedule {
        job_id: i64,
        hour: u8,
        minute: u8,
        response: oneshot::Sender<Result<i64, persistance::StorageError>>,
    },
}

pub struct Server {
    channel: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,
    persistance: mpsc::Sender<persistance::Message>,
    execution_dir: String,
}

impl Server {
    pub async fn new(
        execution_dir: String,
        persistance: mpsc::Sender<persistance::Message>,
    ) -> (mpsc::Sender<Message>, Self) {
        let (tx, rx) = mpsc::channel(128);
        let engine = Self {
            channel: rx,
            sender: tx.clone(),
            persistance,
            execution_dir,
        };
        let schedules = engine.get_all_schedules().await;
        for Schedule {
            job_id,
            hour,
            minute,
            ..
        } in schedules
        {
            engine.spawn_schedule(job_id, hour, minute);
        }
        (tx, engine)
    }

    fn spawn_schedule(&self, job_id: i64, hour: u8, minute: u8) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            match create_interval(hour, minute) {
                Ok(mut interval) => {
                    debug!("Created loop to run {}", job_id);
                    loop {
                        interval.tick().await;
                        let (response, receiver) = oneshot::channel();
                        sender
                            .send(Message::Execute { job_id, response })
                            .await
                            .unwrap_or_default();
                        match receiver.await {
                            Ok(Some(execution_id)) => {
                                info!("Schedule created execution: {}", execution_id);
                            }
                            _ => error!("Failed to create execution"),
                        }
                    }
                }
                Err(err) => {
                    error!("Could not schedule: {}", err);
                }
            }
        });
    }

    pub async fn serve(mut self) {
        while let Some(message) = self.channel.recv().await {
            match message {
                Message::Execute {
                    job_id,
                    response: respone,
                } => {
                    let job = self.get_job(job_id).await;
                    match job {
                        Some(job) => {
                            let execution_id = self.get_execution_id(job_id).await;
                            respone.send(execution_id).unwrap_or_default();
                            match execution_id {
                                Some(execution_id) => {
                                    let osprei::JobPointer { source, path, .. } = job;
                                    let report = JobDescriptor {
                                        execution_dir: self.execution_dir.clone(),
                                        source,
                                        path,
                                    }
                                    .execute_job()
                                    .await;
                                    match self.store_report(execution_id, report).await {
                                        Err(err) => error!("error storing result {}", err),
                                        _ => (),
                                    }
                                }
                                None => error!("Failed to create execution id"),
                            };
                        }
                        None => respone.send(None).unwrap_or_default(),
                    }
                }
                Message::Schedule {
                    job_id,
                    hour,
                    minute,
                    response,
                } => {
                    let (tx, rx) = oneshot::channel();
                    let message = persistance::Message::CreateSchedule(
                        persistance::request::Schedule {
                            job_id,
                            hour,
                            minute,
                        },
                        tx,
                    );
                    self.persistance.send(message).await.unwrap();
                    let schedule_id = rx.await.unwrap();
                    response.send(schedule_id).unwrap();
                    self.spawn_schedule(job_id, hour, minute);
                }
            }
        }
    }

    async fn get_job(&self, job_id: i64) -> Option<osprei::JobPointer> {
        let (tx, rx) = oneshot::channel();
        let message = persistance::Message::FetchJob(job_id, tx);
        self.persistance.send(message).await.ok()?;
        rx.await.ok()?.ok()
    }

    async fn get_execution_id(&self, job_id: i64) -> Option<i64> {
        let (tx, rx) = oneshot::channel();
        let message = persistance::Message::CreateExecution(job_id, tx);
        self.persistance.send(message).await.ok()?;
        rx.await.ok()?.ok()
    }

    async fn store_report(
        &self,
        execution_id: i64,
        report: Report,
    ) -> Result<(), persistance::StorageError> {
        let (tx, rx) = oneshot::channel();
        let Report {
            status,
            stdout,
            stderr,
        } = report;
        let request = persistance::request::Execution {
            id: execution_id,
            status,
            stdout,
            stderr,
        };
        let message = persistance::Message::SetExecution(request, tx);
        self.persistance.send(message).await.unwrap();
        rx.await.unwrap()
    }

    async fn get_all_schedules(&self) -> Vec<Schedule> {
        let (tx, rx) = oneshot::channel();
        self.persistance
            .send(persistance::Message::ListSchedules(tx))
            .await
            .unwrap_or_default();
        rx.await
            .map(|response| response.unwrap_or_default())
            .unwrap_or_default()
    }
}

#[async_trait::async_trait]
trait JobSystemInteface {
    async fn cleanup(&self, checkout_path: &str);
    async fn checkout_code(
        &self,
        checkout_path: &str,
        source: &str,
        path: &str,
    ) -> Result<(Report, Job), Report>;
    async fn execute_stage(&self, checkout_path: &str, stage: Stage) -> Report;
}

struct LocalJobSystem;

#[async_trait::async_trait]
impl JobSystemInteface for LocalJobSystem {
    async fn cleanup(&self, checkout_path: &str) {
        if tokio::fs::remove_dir_all(checkout_path).await.is_ok() {
            info!("Clean up directory: {}", checkout_path);
        }
    }

    async fn checkout_code(
        &self,
        checkout_path: &str,
        source: &str,
        path: &str,
    ) -> Result<(Report, Job), Report> {
        let output = tokio::process::Command::new("git")
            .arg("clone")
            .arg(source)
            .arg(checkout_path)
            .output()
            .await
            .map_err(|err| Report {
                status: osprei::ExecutionStatus::InvalidConfig,
                stdout: "".to_string(),
                stderr: err.to_string(),
            })?;
        let mut report = Report::from(output);
        let definition_path = joined(checkout_path, path);
        match read_job_definition(definition_path).await {
            Ok(job) => Ok((report, job)),
            Err(err) => {
                report.status = osprei::ExecutionStatus::InvalidConfig;
                report.stderr += &err.to_string();
                report.stderr += "\n";
                Err(report)
            }
        }
    }

    async fn execute_stage(&self, checkout_path: &str, stage: Stage) -> Report {
        debug!("Running stage: {:?}", stage);
        let Stage {
            cmd,
            args,
            path,
            env,
        } = stage;
        let path = joined(checkout_path, &path);
        let env: HashMap<_, _> = env
            .into_iter()
            .map(|EnvironmentDefinition { key, value }| (key, value))
            .collect();
        match tokio::process::Command::new(&cmd)
            .args(&args)
            .current_dir(&path)
            .envs(env)
            .output()
            .await
        {
            Ok(output) => Report::from(output),
            Err(err) => {
                error!("Error occured when spawning suprocess, dumping details.");
                error!("Command: {}", cmd);
                error!("Arguments: {:?}", args);
                error!("Path: {}", path);
                Report {
                    status: osprei::ExecutionStatus::InvalidConfig,
                    stdout: "".to_string(),
                    stderr: err.to_string(),
                }
            }
        }
    }
}

pub struct JobDescriptor {
    /// Clone directory for the source repo
    pub execution_dir: String,
    /// Git repository to clone
    pub source: String,
    /// Path within the repository of the job definition
    pub path: String,
}

impl JobDescriptor {
    pub async fn execute_job(self) -> Report {
        self.execute_job_inner(LocalJobSystem).await
    }

    async fn execute_job_inner(self, system: impl JobSystemInteface) -> Report {
        let JobDescriptor {
            execution_dir,
            source,
            path,
        } = self;
        system.cleanup(&execution_dir).await;
        let report = match system.checkout_code(&execution_dir, &source, &path).await {
            Ok((mut report, job)) => {
                for stage in job.stages {
                    debug!("Running stage: {:?}", stage);
                    let stage_report = system.execute_stage(&execution_dir, stage).await;
                    report.status = stage_report.status;
                    report.stdout += &stage_report.stdout;
                    report.stderr += &stage_report.stderr;
                    if report.status != osprei::ExecutionStatus::Success {
                        break;
                    }
                }
                report
            }
            Err(report) => report,
        };
        system.cleanup(&execution_dir).await;
        report
    }
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
                            let Report {
                                status,
                                stdout,
                                stderr,
                            } = descriptor.execute_job().await;
                            if let Err(err) = store
                                .set_execution_result(execution_id, stdout, stderr, status)
                                .await
                            {
                                error!("An error occured storing execution result: {}", err)
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
    use tokio::sync::mpsc;

    use super::*;

    #[tokio::test]
    async fn test_interval_generation() {
        create_interval(12, 0).unwrap();
    }

    struct MessageSenderJobSystem {
        checkout: Result<(Report, Job), Report>,
        stage: Report,
        tx: tokio::sync::mpsc::Sender<JobSystemMessage>,
    }

    #[derive(Debug, PartialEq)]
    enum JobSystemMessage {
        Checkout,
        Cleanup,
        Stage,
    }

    #[async_trait::async_trait]
    impl JobSystemInteface for MessageSenderJobSystem {
        async fn cleanup(&self, _checkout_path: &str) {
            self.tx
                .send(JobSystemMessage::Cleanup)
                .await
                .expect("coult not send cleanup message");
        }

        async fn checkout_code(
            &self,
            _checkout_path: &str,
            _source: &str,
            _path: &str,
        ) -> Result<(Report, Job), Report> {
            self.tx
                .send(JobSystemMessage::Checkout)
                .await
                .expect("could not send checkout message");
            self.checkout.clone()
        }

        async fn execute_stage(&self, _checkout_path: &str, _stage: Stage) -> Report {
            self.tx
                .send(JobSystemMessage::Stage)
                .await
                .expect("could not send stage");
            self.stage.clone()
        }
    }

    fn setup() -> (
        JobDescriptor,
        MessageSenderJobSystem,
        mpsc::Receiver<JobSystemMessage>,
    ) {
        let job_ptr = JobDescriptor {
            execution_dir: "/var/osprei/test".to_string(),
            source: "https://github.com/user/repo.git".to_string(),
            path: "ci/test.json".to_string(),
        };
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let report = Report {
            status: osprei::ExecutionStatus::Success,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        let job = osprei::Job {
            stages: vec![osprei::Stage {
                cmd: String::default(),
                args: Vec::new(),
                env: Vec::new(),
                path: String::default(),
            }],
        };
        let checkout = Ok((report, job));
        let stage = Report {
            status: osprei::ExecutionStatus::Success,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        let system = MessageSenderJobSystem {
            checkout,
            stage,
            tx,
        };
        (job_ptr, system, rx)
    }

    async fn consume(mut rx: mpsc::Receiver<JobSystemMessage>) -> Vec<JobSystemMessage> {
        let mut messages = Vec::new();
        while let Some(message) = rx.recv().await {
            messages.push(message);
        }
        messages
    }

    #[tokio::test]
    async fn when_checkout_fails_then_failure_is_returned() {
        let (job_ptr, mut system, _rx) = setup();
        system.checkout = Err(Report {
            status: osprei::ExecutionStatus::Failed,
            stdout: "".to_string(),
            stderr: "".to_string(),
        });
        let report = job_ptr.execute_job_inner(system).await;
        assert_eq!(report.status, osprei::ExecutionStatus::Failed);
    }

    #[tokio::test]
    async fn when_checkout_fails_then_cleanup_is_called() {
        let (job_ptr, mut system, rx) = setup();
        system.checkout = Err(Report {
            status: osprei::ExecutionStatus::Failed,
            stdout: "".to_string(),
            stderr: "".to_string(),
        });
        let _report = job_ptr.execute_job_inner(system).await;
        let seq = consume(rx).await;
        assert_eq!(
            seq,
            vec![
                JobSystemMessage::Cleanup,
                JobSystemMessage::Checkout,
                JobSystemMessage::Cleanup
            ]
        );
    }

    #[tokio::test]
    async fn when_checkout_succeeds_then_stage_is_executed() {
        let (job_ptr, system, rx) = setup();
        let _report = job_ptr.execute_job_inner(system).await;
        let seq = consume(rx).await;
        assert_eq!(
            seq,
            vec![
                JobSystemMessage::Cleanup,
                JobSystemMessage::Checkout,
                JobSystemMessage::Stage,
                JobSystemMessage::Cleanup
            ]
        );
    }

    #[tokio::test]
    async fn when_checkout_and_stage_succeed_then_success_is_returned() {
        let (job_ptr, system, _rx) = setup();
        let report = job_ptr.execute_job_inner(system).await;
        assert_eq!(report.status, osprei::ExecutionStatus::Success);
    }

    #[tokio::test]
    async fn when_multiple_stage_then_all_are_executed() {
        let (job_ptr, mut system, rx) = setup();
        let checkout = system.checkout.as_mut().unwrap();
        checkout.1.stages.push(Stage {
            cmd: "".to_string(),
            args: Vec::new(),
            env: Vec::new(),
            path: "".to_string(),
        });
        let _report = job_ptr.execute_job_inner(system).await;
        let seq = consume(rx).await;
        assert_eq!(
            seq,
            vec![
                JobSystemMessage::Cleanup,
                JobSystemMessage::Checkout,
                JobSystemMessage::Stage,
                JobSystemMessage::Stage,
                JobSystemMessage::Cleanup
            ]
        );
    }

    #[tokio::test]
    async fn when_multiple_stages_and_first_fails_short_circuit() {
        let (job_ptr, mut system, rx) = setup();
        let checkout = system.checkout.as_mut().unwrap();
        checkout.1.stages.push(Stage {
            cmd: "".to_string(),
            args: Vec::new(),
            env: Vec::new(),
            path: "".to_string(),
        });
        system.stage.status = osprei::ExecutionStatus::Failed;
        let _report = job_ptr.execute_job_inner(system).await;
        let seq = consume(rx).await;
        assert_eq!(
            seq,
            vec![
                JobSystemMessage::Cleanup,
                JobSystemMessage::Checkout,
                JobSystemMessage::Stage,
                JobSystemMessage::Cleanup
            ]
        );
    }

    #[tokio::test]
    async fn when_multiple_stages_and_first_fails_then_failure_ir_returned() {
        let (job_ptr, mut system, _rx) = setup();
        let checkout = system.checkout.as_mut().unwrap();
        checkout.1.stages.push(Stage {
            cmd: "".to_string(),
            args: Vec::new(),
            env: Vec::new(),
            path: "".to_string(),
        });
        system.stage.status = osprei::ExecutionStatus::Failed;
        let report = job_ptr.execute_job_inner(system).await;
        assert_eq!(report.status, osprei::ExecutionStatus::Failed);
    }

    #[tokio::test]
    async fn when_multiplt_stages_output_is_concatenation_of_outputs() {
        let (job_ptr, mut system, _rx) = setup();
        let checkout = system.checkout.as_mut().unwrap();
        checkout.0.stdout = "a".to_string();
        checkout.0.stderr = "A".to_string();
        system.stage.stdout = "b".to_string();
        system.stage.stderr = "B".to_string();
        let report = job_ptr.execute_job_inner(system).await;
        assert_eq!(report.stdout, "ab");
        assert_eq!(report.stderr, "AB");
    }
}
