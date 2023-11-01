use leptos::server;
use leptos::ServerFnError;

use crate::widget;

#[server]
pub async fn load_jobs() -> Result<Vec<widget::Job>, ServerFnError> {
    let ids = osprei_storage::job_ids().await?;
    let mut jobs = Vec::new();
    for id in ids {
        let source = load_job_source(id).await?;
        let status = load_job_status(id).await?;
        let job = widget::Job {
            id,
            source,
            status,
        };
        jobs.push(job);
    }
    Ok(jobs)
}

#[server]
pub async fn load_executions() -> Result<Vec<widget::Execution>, ServerFnError> {
    let ids = load_execution_list().await?;
    let mut executions = Vec::new();
    for id in ids {
        let status = load_execution_status(id).await?;
        let duration = load_execution_duration(id).await?;
        let execution = widget::Execution {
            id,
            status,
            duration,
        };
        executions.push(execution);
    }
    Ok(executions)
}

#[server]
pub async fn load_job_list() -> Result<Vec<i64>, ServerFnError> {
    let jobs = osprei_storage::job_ids().await?;
    Ok(jobs)
}

#[server(AddJob)]
pub async fn add_job(source: String) -> Result<(), ServerFnError> {
    osprei_storage::job_create(source).await?;
    Ok(())
}

#[server(ExecuteJob)]
pub async fn execute_job(job_id: i64) -> Result<(), ServerFnError> {
    log::info!("Running job with id {}", job_id);
    let source = osprei_storage::job_source(job_id).await?;
    let execution_id = osprei_storage::execution_create(job_id).await?;
    tokio::spawn(async move {
        let stages = vec![];
        match osprei_execution::execute(source, stages).await {
            Ok(()) => {
                let _ = osprei_storage::execution_success(execution_id).await;
            }
            Err(_) => {
                let _ = osprei_storage::execution_failure(execution_id).await;
            }
        }
    });
    Ok(())
}

#[server]
pub async fn load_job_source(id: i64) -> Result<String, ServerFnError> {
    let source = osprei_storage::job_source(id).await?;
    Ok(source)
}

#[server]
pub async fn load_job_status(id: i64) -> Result<String, ServerFnError> {
    let status = osprei_storage::job_status(id).await?;
    let message = match status {
        None => "Not executed".to_string(),
        Some(osprei_storage::ExecutionStatus::Running) => "Running".to_string(),
        Some(osprei_storage::ExecutionStatus::Success) => "Success".to_string(),
        Some(osprei_storage::ExecutionStatus::Failure) => "Failure".to_string(),
        Some(osprei_storage::ExecutionStatus::Unknown) => "Unknown".to_string(),
    };
    Ok(message)
}

#[server]
pub async fn load_execution_list() -> Result<Vec<i64>, ServerFnError> {
    let executions = osprei_storage::execution_ids().await?;
    Ok(executions)
}

#[server]
pub async fn load_execution_status(id: i64) -> Result<String, ServerFnError> {
    let status = osprei_storage::execution_status(id).await?;
    let message = match status {
        osprei_storage::ExecutionStatus::Running => "Running".to_string(),
        osprei_storage::ExecutionStatus::Success => "Success".to_string(),
        osprei_storage::ExecutionStatus::Failure => "Failure".to_string(),
        osprei_storage::ExecutionStatus::Unknown => "Unknown".to_string(),
    };
    Ok(message)
}

#[server]
pub async fn load_execution_duration(id: i64) -> Result<Option<i64>, ServerFnError> {
    let duration = osprei_storage::execution_duration(id).await?;
    Ok(duration)
}


















