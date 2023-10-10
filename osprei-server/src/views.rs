use docker_api::Docker;
use osprei::JobCreationRequest;
use sqlx::Row;
use std::convert::Infallible;
use warp::reply::Reply;

mod storage;
use storage::Storage;

mod execution;
use execution::dispatch_execution;

fn reply<S: serde::Serialize>(result: Result<S, storage::Error>) -> impl warp::Reply {
    match result {
        Ok(serializable) => warp::reply::json(&serializable).into_response(),
        Err(err) => err.to_reply().into_response(),
    }
}

pub async fn get_jobs(pool: sqlx::SqlitePool) -> Result<impl warp::Reply, Infallible> {
    let jobs = Storage::new(pool).get_jobs().await;
    let reply = reply(jobs);
    Ok(reply)
}

pub async fn get_job(job_id: i64, pool: sqlx::SqlitePool) -> Result<impl warp::Reply, Infallible> {
    let jobs = Storage::new(pool).get_job(job_id).await;
    let reply = reply(jobs);
    Ok(reply)
}

pub async fn post_job(
    request: JobCreationRequest,
    pool: sqlx::SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    let id = Storage::new(pool).post_job(request).await;
    let reply = reply(id);
    Ok(reply)
}

pub async fn get_job_run(
    job_id: i64,
    pool: sqlx::SqlitePool,
    docker: Docker,
) -> Result<impl warp::Reply, Infallible> {
    let execution_id = job_run(job_id, pool, docker).await;
    let reply = reply(execution_id);
    Ok(reply)
}

async fn job_run(
    job_id: i64,
    pool: sqlx::SqlitePool,
    docker: Docker,
) -> Result<impl serde::Serialize, storage::Error> {
    let storage = Storage::new(pool);
    let (id, pointer) = storage.create_execution(job_id).await.unwrap();
    dispatch_execution(id, pointer, storage, docker);
    Ok(id)
}

pub async fn get_execution(
    execution_id: i64,
    pool: sqlx::SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    let execution = Storage::new(pool).get_execution(execution_id).await;
    let reply = reply(execution);
    Ok(reply)
}

pub async fn post_job_schedule(
    job_id: i64,
    request: osprei::ScheduleRequest,
    pool: sqlx::SqlitePool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let osprei::ScheduleRequest { hour, minute } = request;
    let mut conn = pool.acquire().await.unwrap();
    let id = sqlx::query("INSERT INTO schedules (job_id, hour, minute) VALUES ($1, $2, $3)")
        .bind(job_id)
        .bind(hour)
        .bind(minute)
        .execute(&mut conn)
        .await
        .unwrap()
        .last_insert_rowid();
    Ok(warp::reply::json(&id))
}

pub async fn get_stdout(
    execution_id: i64,
    pool: sqlx::SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    let mut conn = pool.acquire().await.unwrap();
    let opt_stdout: Option<String> = sqlx::query("SELECT stdout FROM executions WHERE id = $1")
        .bind(execution_id)
        .fetch_one(&mut conn)
        .await
        .unwrap()
        .get(0);
    Ok(warp::reply::json(&opt_stdout.unwrap_or_default()))
}

pub async fn get_stderr(
    execution_id: i64,
    pool: sqlx::SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    let mut conn = pool.acquire().await.unwrap();
    let opt_stderr: Option<String> = sqlx::query("SELECT stderr FROM executions WHERE id = $1")
        .bind(execution_id)
        .fetch_one(&mut conn)
        .await
        .unwrap()
        .get(0);
    Ok(warp::reply::json(&opt_stderr.unwrap_or_default()))
}
