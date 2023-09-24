use warp::Filter;
use warp::Reply;

use crate::views;

pub fn routes(
    pool: sqlx::SqlitePool,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let get_jobs = warp::path!("job")
        .and(warp::get())
        .and(with_pool(pool.clone()))
        .and_then(views::get_jobs);
    let get_job = warp::path!("job" / i64)
        .and(warp::get())
        .and(with_pool(pool.clone()))
        .and_then(views::get_job);
    let post_job = warp::path!("job")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_pool(pool.clone()))
        .and_then(views::post_job);
    let get_job_run = warp::path!("job" / i64 / "run")
        .and(warp::get())
        .and(with_pool(pool.clone()))
        .and_then(views::get_job_run);
    let get_execution = warp::path!("execution" / i64)
        .and(warp::get())
        .and(with_pool(pool.clone()))
        .and_then(views::get_execution);
    let post_job_schedule = warp::path!("job" / i64 / "schedule")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_pool(pool.clone()))
        .and_then(views::post_job_schedule);
    let get_stdout = warp::path!("execution" / i64 / "stdout")
        .and(warp::get())
        .and(with_pool(pool.clone()))
        .and_then(views::get_stdout);
    let get_stderr = warp::path!("execution" / i64 / "stderr")
        .and(warp::get())
        .and(with_pool(pool.clone()))
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

fn with_pool(
    pool: sqlx::SqlitePool,
) -> impl Filter<Extract = (sqlx::SqlitePool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}
