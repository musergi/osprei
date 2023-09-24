use tokio::sync::mpsc;
use warp::Filter;
use warp::Reply;

use crate::execute;
use crate::persistance;
use crate::views;

pub fn routes(
    persistance: mpsc::Sender<persistance::Message>,
    execution: mpsc::Sender<execute::Message>,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let get_jobs = warp::path!("job")
        .and(warp::get())
        .and(with_sender(persistance.clone()))
        .and_then(views::get_jobs);
    let get_job = warp::path!("job" / i64)
        .and(warp::get())
        .and(with_sender(persistance.clone()))
        .and_then(views::get_job);
    let post_job = warp::path!("job")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_sender(persistance.clone()))
        .and_then(views::post_job);
    let get_job_run = warp::path!("job" / i64 / "run")
        .and(warp::get())
        .and(with_sender(execution.clone()))
        .and_then(views::get_job_run);
    let get_execution = warp::path!("execution" / i64)
        .and(warp::get())
        .and(with_sender(persistance.clone()))
        .and_then(views::get_execution);
    let post_job_schedule = warp::path!("job" / i64 / "schedule")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_sender(execution.clone()))
        .and_then(views::post_job_schedule);
    let get_stdout = warp::path!("execution" / i64 / "stdout")
        .and(warp::get())
        .and(with_sender(persistance.clone()))
        .and_then(views::get_stdout);
    let get_stderr = warp::path!("execution" / i64 / "stderr")
        .and(warp::get())
        .and(with_sender(persistance.clone()))
        .and_then(views::get_stderr);
    get_jobs
        .or(get_job)
        .or(post_job)
        .or(get_job_run)
        .or(get_execution)
        .or(post_job_schedule)
        .or(get_stdout)
        .or(get_stderr)
}

fn with_sender<T>(
    sender: mpsc::Sender<T>,
) -> impl Filter<Extract = (mpsc::Sender<T>,), Error = std::convert::Infallible> + Clone
where
    T: std::marker::Send,
{
    warp::any().map(move || sender.clone())
}
