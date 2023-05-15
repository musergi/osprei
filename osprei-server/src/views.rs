use std::convert::Infallible;

use log::error;
use osprei::{JobCreationRequest, JobPointer};

use crate::{
    execute,
    persistance::{ExecutionStore, JobStore},
    PathBuilder,
};

pub async fn get_jobs(job_store: impl JobStore) -> Result<impl warp::Reply, Infallible> {
    let jobs = job_store.list_jobs().await;
    Ok(warp::reply::json(&jobs))
}

pub async fn get_job(
    job_id: i64,
    job_store: impl JobStore,
) -> Result<impl warp::Reply, Infallible> {
    let job_ptr = job_store.fetch_job(job_id).await;
    Ok(warp::reply::json(&job_ptr))
}

pub async fn post_job(
    request: JobCreationRequest,
    job_store: impl JobStore,
) -> Result<impl warp::Reply, Infallible> {
    let JobCreationRequest { name, source, path } = request;
    let job_id = job_store.store_job(name, source, path).await;
    Ok(warp::reply::json(&job_id))
}

pub async fn get_job_run(
    job_id: i64,
    path_builder: PathBuilder,
    store: impl JobStore + ExecutionStore + std::marker::Send + std::marker::Sync + 'static,
) -> Result<impl warp::Reply, Infallible> {
    let JobPointer {
        name, source, path, ..
    } = store.fetch_job(job_id).await;
    let execution_id = store.create_execution(job_id).await;
    let execution_dir = path_builder.workspace(&name);
    let result_dir = path_builder.results(&name, execution_id);
    let options = execute::JobExecutionOptions {
        execution_dir,
        result_dir,
        source,
        path,
    };
    {
        let execution_id = execution_id.clone();
        tokio::spawn(async move {
            match execute::execute_job(options).await {
                Ok(outputs) => {
                    let any_failed = outputs.iter().any(|output| output.status != 0);
                    let status = match any_failed {
                        false => 0,
                        true => 1,
                    };
                    store.set_execution_status(execution_id, status).await;
                }
                Err(err) => error!("An error occurred during job executions: {}", err),
            }
        });
    }
    Ok(warp::reply::json(&execution_id))
}

pub async fn get_job_executions(
    job_id: i64,
    store: impl JobStore + ExecutionStore,
) -> Result<impl warp::Reply, Infallible> {
    let executions = store.last_executions(job_id, 10).await;
    Ok(warp::reply::json(&executions))
}

pub async fn get_execution(
    execution_id: i64,
    store: impl JobStore + ExecutionStore,
) -> Result<impl warp::Reply, Infallible> {
    let execution = store.get_execution(execution_id).await;
    Ok(warp::reply::json(&execution))
}
