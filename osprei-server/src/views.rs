use crate::execute;
use crate::persistance;
use crate::persistance::StorageError;
use osprei::JobCreationRequest;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub async fn get_jobs(
    persistance: mpsc::Sender<persistance::Message>,
) -> Result<impl warp::Reply, Infallible> {
    let (tx, rx) = oneshot::channel();
    persistance
        .send(persistance::Message::ListJobs(tx))
        .await
        .unwrap();
    let jobs = rx.await.unwrap();
    reply(jobs)
}

pub async fn get_job(
    job_id: i64,
    persistance: mpsc::Sender<persistance::Message>,
) -> Result<impl warp::Reply, Infallible> {
    let (tx, rx) = oneshot::channel();
    persistance
        .send(persistance::Message::FetchJob(job_id, tx))
        .await
        .unwrap();
    let job_ptr = rx.await.unwrap();
    reply(job_ptr)
}

pub async fn post_job(
    request: JobCreationRequest,
    persistance: mpsc::Sender<persistance::Message>,
) -> Result<impl warp::Reply, Infallible> {
    let JobCreationRequest { name, source, path } = request;
    let (tx, rx) = oneshot::channel();
    persistance
        .send(persistance::Message::StoreJob(
            persistance::request::Job { name, source, path },
            tx,
        ))
        .await
        .unwrap();
    let job_id = rx.await.unwrap();
    reply(job_id)
}

pub async fn get_job_run(
    job_id: i64,
    engine: mpsc::Sender<execute::Message>,
) -> Result<impl warp::Reply, Infallible> {
    let (tx, rx) = oneshot::channel();
    engine
        .send(execute::Message::Execute {
            job_id,
            response: tx,
        })
        .await
        .unwrap();
    let execution_id = rx.await.unwrap();
    reply(Ok(execution_id))
}

pub async fn get_execution(
    execution_id: i64,
    persistance: mpsc::Sender<persistance::Message>,
) -> Result<impl warp::Reply, Infallible> {
    let (tx, rx) = oneshot::channel();
    persistance
        .send(persistance::Message::GetExecution(execution_id, tx))
        .await
        .unwrap();
    let execution = rx.await.unwrap();
    reply(execution)
}

pub async fn post_job_schedule(
    job_id: i64,
    request: osprei::ScheduleRequest,
    engine: mpsc::Sender<execute::Message>,
) -> Result<impl warp::Reply, Infallible> {
    let (tx, rx) = oneshot::channel();
    let osprei::ScheduleRequest { hour, minute } = request;
    engine
        .send(execute::Message::Schedule {
            job_id,
            hour,
            minute,
            response: tx,
        })
        .await
        .unwrap();
    let id = rx.await.unwrap();
    reply(id)
}

pub async fn get_stdout(
    execution_id: i64,
    persistance: mpsc::Sender<persistance::Message>,
) -> Result<impl warp::Reply, Infallible> {
    let (tx, rx) = oneshot::channel();
    persistance
        .send(persistance::Message::GetStdout(execution_id, tx))
        .await
        .unwrap();
    let stdout = rx.await.unwrap();
    reply(stdout)
}

pub async fn get_stderr(
    execution_id: i64,
    persistance: mpsc::Sender<persistance::Message>,
) -> Result<impl warp::Reply, Infallible> {
    let (tx, rx) = oneshot::channel();
    persistance
        .send(persistance::Message::GetStdout(execution_id, tx))
        .await
        .unwrap();
    let stderr = rx.await.unwrap();
    reply(stderr)
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
