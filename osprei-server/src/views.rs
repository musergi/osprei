use docker_api::conn::TtyChunk;
use docker_api::opts::ContainerCreateOpts;
use docker_api::opts::ContainerRemoveOpts;
use docker_api::opts::LogsOpts;
use docker_api::Containers;
use docker_api::Docker;
use osprei::Job;
use osprei::JobCreationRequest;
use osprei::JobPointer;
use sqlx::Row;
use std::convert::Infallible;
use tokio_stream::StreamExt;
use warp::reply::Reply;

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

mod storage;
use storage::Storage;

pub async fn post_job(
    request: JobCreationRequest,
    pool: sqlx::SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    let JobCreationRequest { name, definition } = request;
    let definition = serde_json::to_string(&definition).unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let id = sqlx::query("INSERT INTO jobs (name, definition) VALUES ($1, $2)")
        .bind(name)
        .bind(definition)
        .execute(&mut conn)
        .await
        .unwrap()
        .last_insert_rowid();
    Ok(warp::reply::json(&id))
}

pub async fn get_job_run(
    job_id: i64,
    pool: sqlx::SqlitePool,
    docker: Docker,
) -> Result<impl warp::Reply, Infallible> {
    let mut conn = pool.acquire().await.unwrap();
    let pointer = sqlx::query("SELECT id, name, definition FROM jobs WHERE id = $1")
        .bind(job_id)
        .fetch_optional(&mut conn)
        .await
        .unwrap()
        .map(|row| {
            let definition: String = row.get(2);
            let definition = serde_json::from_str(&definition).unwrap();
            osprei::JobPointer {
                id: row.get(0),
                name: row.get(1),
                definition,
            }
        })
        .unwrap();
    let id =
        sqlx::query("INSERT INTO executions (job_id, start_time) VALUES ($1, CURRENT_TIMESTAMP)")
            .bind(job_id)
            .execute(&mut conn)
            .await
            .unwrap()
            .last_insert_rowid();
    {
        let execution_id = id.clone();
        tokio::spawn(async move {
            let JobPointer {
                name, definition, ..
            } = pointer;
            let Job {
                source,
                image,
                command,
                arguments,
                ..
            } = definition;
            let checkout_path = format!("/opt/osprei/var/workspace/{}", name);
            tokio::process::Command::new("git")
                .arg("clone")
                .arg(source)
                .arg(&checkout_path)
                .output()
                .await
                .unwrap();
            let volume = format!("{v}:{v}", v = checkout_path);
            let mut command = vec![command];
            command.extend(arguments);
            let opts = ContainerCreateOpts::builder()
                .name(&name)
                .image(image)
                .working_dir(&checkout_path)
                .command(command)
                .volumes(vec![volume])
                .build();
            let container = Containers::new(docker).create(&opts).await.unwrap();
            container.start().await.unwrap();
            let status = container.wait().await.unwrap().status_code;
            let status = if status == 0 { 0 } else { 1 };
            let opts = LogsOpts::builder().stdout(true).stderr(true).build();
            let mut stream = container.logs(&opts);
            let mut stdout: Vec<u8> = Vec::new();
            let mut stderr: Vec<u8> = Vec::new();
            while let Some(text) = stream.next().await {
                match text.unwrap() {
                    TtyChunk::StdOut(out) => stdout.extend(out),
                    TtyChunk::StdErr(out) => stderr.extend(out),
                    _ => (),
                }
            }
            let stdout = String::from_utf8_lossy(&stdout).to_string();
            let stderr = String::from_utf8_lossy(&stderr).to_string();
            sqlx::query(
                "UPDATE executions SET status = $2, stdout = $3, stderr = $4 WHERE id = $1",
            )
            .bind(execution_id)
            .bind(status)
            .bind(stdout)
            .bind(stderr)
            .execute(&mut conn)
            .await
            .unwrap();
            let opts = ContainerRemoveOpts::builder()
                .force(true)
                .volumes(true)
                .build();
            container.remove(&opts).await.unwrap();
        });
    }
    Ok(warp::reply::json(&id))
}

pub async fn get_execution(
    execution_id: i64,
    pool: sqlx::SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    let mut conn = pool.acquire().await.unwrap();
    let execution = sqlx::query(
        "
            SELECT
                executions.id,
                jobs.name,
                start_time,
                status,
                stdout,
                stderr
            FROM
                executions
                JOIN jobs
                    ON jobs.id = executions.job_id
            WHERE
                executions.id = $1
            ",
    )
    .bind(execution_id)
    .fetch_optional(&mut conn)
    .await
    .unwrap()
    .map(|row| {
        let status_encoded: Option<i64> = row.get(3);
        let status = status_encoded.map(osprei::ExecutionStatus::from);
        osprei::ExecutionDetails {
            execution_id: row.get(0),
            job_name: row.get(1),
            start_time: row.get(2),
            status,
            stdout: row.get(4),
            stderr: row.get(5),
        }
    })
    .unwrap();
    Ok(warp::reply::json(&execution))
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
