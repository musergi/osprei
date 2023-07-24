use std::convert::Infallible;

use log::error;
use osprei::{JobCreationRequest, JobPointer};

use crate::{execute, persistance::Storage, PathBuilder};

pub async fn get_jobs(job_store: Box<dyn Storage>) -> Result<impl warp::Reply, Infallible> {
    let jobs = job_store.list_jobs().await.unwrap();
    Ok(warp::reply::json(&jobs))
}

pub async fn get_job(
    job_id: i64,
    job_store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let job_ptr = job_store.fetch_job(job_id).await.unwrap();
    Ok(warp::reply::json(&job_ptr))
}

pub async fn post_job(
    request: JobCreationRequest,
    job_store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let JobCreationRequest { name, source, path } = request;
    let job_id = job_store.store_job(name, source, path).await.unwrap();
    Ok(warp::reply::json(&job_id))
}

pub async fn get_job_run(
    job_id: i64,
    path_builder: PathBuilder,
    store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let JobPointer {
        name, source, path, ..
    } = store.fetch_job(job_id).await.unwrap();
    let execution_id = store.create_execution(job_id).await.unwrap();
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
    Ok(warp::reply::json(&execution_id))
}

pub async fn get_job_executions(
    job_id: i64,
    store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let executions = store.last_executions(job_id, 10).await.unwrap();
    Ok(warp::reply::json(&executions))
}

pub async fn get_execution(
    execution_id: i64,
    store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let execution = store.get_execution(execution_id).await.unwrap();
    Ok(warp::reply::json(&execution))
}

pub async fn get_job_schedule(
    job_id: i64,
    store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let (reply, status) = match store.get_schedules(job_id).await {
        Ok(schedules) => (warp::reply::json(&schedules), warp::http::StatusCode::OK),
        Err(_) => (
            warp::reply::json(&ApiError::new(String::from("Internal error"))),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    };
    Ok(warp::reply::with_status(reply, status))
}

pub async fn post_job_schedule(
    job_id: i64,
    request: osprei::ScheduleRequest,
    path_builder: PathBuilder,
    store: Box<dyn Storage>,
) -> Result<impl warp::Reply, Infallible> {
    let (reply, status) = match store.create_daily(job_id, request.clone()).await {
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
            (warp::reply::json(&id), warp::http::StatusCode::OK)
        }
        Err(err) => {
            let message = err.to_string();
            (
                warp::reply::json(&ApiError::new(message)),
                warp::http::StatusCode::NOT_FOUND,
            )
        }
    };
    Ok(warp::reply::with_status(reply, status))
}

#[derive(Debug, serde::Serialize)]
pub struct ApiError {
    message: String,
}

impl ApiError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
