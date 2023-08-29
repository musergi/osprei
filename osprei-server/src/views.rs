use std::convert::Infallible;

use log::error;
use osprei::{JobCreationRequest, JobPointer};

use crate::{
    execute,
    persistance::{Storage, StorageError},
    PathBuilder,
};

pub async fn get_jobs(job_store: Box<dyn Storage>) -> Result<impl warp::Reply, Infallible> {
    let jobs = job_store.list_jobs().await;
    reply(jobs)
}

pub async fn get_job(
    job_id: i64,
    job_store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let job_ptr = job_store.fetch_job(job_id).await;
    reply(job_ptr)
}

pub async fn post_job(
    request: JobCreationRequest,
    job_store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let JobCreationRequest { name, source, path } = request;
    let job_id = job_store.store_job(name, source, path).await;
    reply(job_id)
}

pub async fn get_last_job_execution(
    job_id: i64,
    job_store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let execution = job_store.get_last_execution(job_id).await;
    reply(execution)
}

pub async fn get_job_run(
    job_id: i64,
    path_builder: PathBuilder,
    store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let execution_id = run_job(job_id, path_builder, store).await;
    reply(execution_id)
}

async fn run_job(
    job_id: i64,
    path_builder: PathBuilder,
    store: Box<dyn Storage>,
) -> Result<impl serde::Serialize, StorageError> {
    let JobPointer {
        name, source, path, ..
    } = store.fetch_job(job_id).await?;
    let execution_id = store.create_execution(job_id).await?;
    let execution_dir = path_builder.workspace(&name);
    let result_dir = path_builder.results(&name, execution_id);
    let descriptor = execute::JobDescriptor {
        execution_dir,
        result_dir,
        source,
        path,
    };
    {
        let execution_id = execution_id;
        tokio::spawn(async move {
            match descriptor.execute_job().await {
                Ok(outputs) => {
                    execute::write_result(execution_id, &outputs, store.as_ref()).await;
                }
                Err(err) => {
                    error!("An error occurred during job executions: {}", err);
                    execute::write_error(execution_id, store.as_ref()).await;
                }
            }
        });
    }
    Ok(execution_id)
}

pub async fn get_job_executions(
    job_id: i64,
    store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let executions = store.last_executions(job_id, 10).await;
    reply(executions)
}

pub async fn get_execution(
    execution_id: i64,
    store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let execution = store.get_execution(execution_id).await;
    reply(execution)
}

pub async fn get_job_schedule(
    job_id: i64,
    store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let schedules = store.get_schedules(job_id).await;
    reply(schedules)
}

pub async fn post_job_schedule(
    job_id: i64,
    request: osprei::ScheduleRequest,
    path_builder: PathBuilder,
    store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let id = match store.create_daily(job_id, request.clone()).await {
        Ok(id) => {
            match store.fetch_job(job_id).await {
                Ok(job) => {
                    let osprei::ScheduleRequest { hour, minute } = request;
                    execute::schedule_job(job, hour, minute, path_builder, store).await;
                }
                Err(err) => {
                    error!("failed to schedule: job not found: {}", err);
                }
            }
            Ok(id)
        }
        Err(err) => Err(err),
    };
    reply(id)
}

fn reply(
    reply: Result<impl serde::Serialize, StorageError>,
) -> Result<impl warp::Reply, Infallible> {
    let (reply, status) = match reply {
        Ok(reply) => (warp::reply::json(&reply), warp::http::StatusCode::OK),
        Err(err) => {
            let status = match err {
                StorageError::UserError(_) => warp::http::StatusCode::NOT_FOUND,
                StorageError::InternalError(_) => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            let message: ApiError = err.into();
            (warp::reply::json(&message), status)
        }
    };
    Ok(warp::reply::with_status(reply, status))
}

#[derive(Debug, serde::Serialize)]
pub struct ApiError {
    message: String,
}

impl From<StorageError> for ApiError {
    fn from(value: StorageError) -> Self {
        let message = match value {
            StorageError::UserError(err) => err,
            StorageError::InternalError(_) => String::from("Internal error"),
        };
        ApiError { message }
    }
}
