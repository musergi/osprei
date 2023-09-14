use warp::Filter;
use warp::Reply;

use crate::persistance;
use crate::views;
use crate::PathBuilder;

pub fn routes(
    path_builder: PathBuilder,
    persistance: persistance::Persistances,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let get_jobs = warp::path!("job")
        .and(warp::get())
        .and(persistance::with(persistance.clone()))
        .and_then(views::get_jobs);
    let get_job = warp::path!("job" / i64)
        .and(warp::get())
        .and(persistance::with(persistance.clone()))
        .and_then(views::get_job);
    let post_job = warp::path!("job")
        .and(warp::post())
        .and(warp::body::json())
        .and(persistance::with(persistance.clone()))
        .and_then(views::post_job);
    let get_job_run = warp::path!("job" / i64 / "run")
        .and(warp::get())
        .and(with_path_builder(path_builder.clone()))
        .and(persistance::with(persistance.clone()))
        .and_then(views::get_job_run);
    let get_execution = warp::path!("execution" / i64)
        .and(warp::get())
        .and(persistance::with(persistance.clone()))
        .and_then(views::get_execution);
    let post_job_schedule = warp::path!("job" / i64 / "schedule")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_path_builder(path_builder))
        .and(persistance::with(persistance.clone()))
        .and_then(views::post_job_schedule);
    let get_stdout = warp::path!("execution" / i64 / "stdout")
        .and(warp::get())
        .and(persistance::with(persistance.clone()))
        .and_then(views::get_stdout);
    let get_stderr = warp::path!("execution" / i64 / "stderr")
        .and(warp::get())
        .and(persistance::with(persistance))
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

fn with_path_builder(
    path_builder: PathBuilder,
) -> impl Filter<Extract = (PathBuilder,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || path_builder.clone())
}
