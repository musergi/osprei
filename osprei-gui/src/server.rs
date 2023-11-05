use leptos::server;
use leptos::ServerFnError;

use crate::widget;

const SQLX_IMAGE: &str = "ghcr.io/musergi/sqlx:latest";

#[server]
pub async fn load_jobs() -> Result<Vec<widget::Job>, ServerFnError> {
    let ids = osprei_storage::job::ids().await?;
    let mut jobs = Vec::new();
    for id in ids {
        let source = load_job_source(id).await?;
        let status = load_job_status(id).await?;
        let job = widget::Job { id, source, status };
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
pub async fn load_stages(job_id: i64) -> Result<Vec<widget::Stage>, ServerFnError> {
    let stages = osprei_storage::stages::for_job(job_id)
        .await?
        .into_iter()
        .map(
            |osprei_storage::Stage {
                 id,
                 dependency,
                 definition,
             }| widget::Stage {
                id,
                dependency,
                description: definition.name,
            },
        )
        .collect();
    Ok(stages)
}

#[server]
pub async fn load_job_list() -> Result<Vec<i64>, ServerFnError> {
    let jobs = osprei_storage::job::ids().await?;
    Ok(jobs)
}

#[server(AddStage)]
pub async fn add_stage(
    job_id: i64,
    name: String,
    dependency: i64,
    template: String,
) -> Result<(), ServerFnError> {
    log::info!("AddStage {job_id} {name} {dependency} {template}");
    let definition = match template.as_str() {
        "sqlx" => Some(osprei_data::StageDefinition {
            name,
            image: SQLX_IMAGE.to_string(),
            command: vec![
                "sqlx".to_string(),
                "database".to_string(),
                "setup".to_string(),
            ],
            environment: vec![osprei_data::EnvironmentVariable {
                name: "DATABASE_URL".to_string(),
                value: "sqlite:testing.db".to_string(),
            }],
            working_dir: osprei_storage::stages::CHECKOUT_DIR.to_string(),
        }),
        _ => None,
    };
    match definition {
        Some(definition) => osprei_storage::stages::create(job_id, dependency, definition).await?,
        None => log::warn!("Unknown template {template}"),
    }
    Ok(())
}

#[server(AddJob)]
pub async fn add_job(source: String) -> Result<(), ServerFnError> {
    osprei_storage::job::create(source).await?;
    Ok(())
}

#[server(ExecuteJob)]
pub async fn execute_job(job_id: i64) -> Result<(), ServerFnError> {
    log::info!("Running job with id {}", job_id);
    let stages: Vec<_> = osprei_storage::stages::for_job(job_id)
        .await?
        .into_iter()
        .map(|stage| stage.definition)
        .collect();
    let execution_id = osprei_storage::execution::create(job_id).await?;
    tokio::spawn(async move {
        match osprei_execution::execute(stages).await {
            Ok(()) => {
                let _ = osprei_storage::execution::success(execution_id).await;
            }
            Err(err) => {
                log::error!("Execution error: {err}");
                let _ = osprei_storage::execution::failure(execution_id).await;
            }
        }
    });
    Ok(())
}

#[server]
pub async fn load_job_source(id: i64) -> Result<String, ServerFnError> {
    let source = osprei_storage::job::source(id).await?;
    Ok(source)
}

#[server]
pub async fn load_job_status(id: i64) -> Result<String, ServerFnError> {
    let status = osprei_storage::job::status(id).await?;
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
    let executions = osprei_storage::execution::ids().await?;
    Ok(executions)
}

#[server]
pub async fn load_execution_status(id: i64) -> Result<String, ServerFnError> {
    let status = osprei_storage::execution::status(id).await?;
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
    let duration = osprei_storage::execution::duration(id).await?;
    Ok(duration)
}
