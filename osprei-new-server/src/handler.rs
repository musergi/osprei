use axum::extract::Path;
use axum::Json;
use docker_api::opts::{ContainerCreateOpts, ContainerRemoveOpts, VolumeCreateOpts};
use docker_api::Docker;
use sqlx::SqlitePool;

pub async fn get_jobs(pool: SqlitePool) -> Json<Vec<i64>> {
    let mut conn = pool.acquire().await.unwrap();
    Json(
        sqlx::query!("SELECT id FROM jobs")
            .fetch_all(&mut *conn)
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.id)
            .collect(),
    )
}

pub async fn post_job(Json(job): Json<String>, pool: SqlitePool) -> Json<i64> {
    let mut conn = pool.acquire().await.unwrap();
    Json(
        sqlx::query!("INSERT INTO jobs (source) VALUES ($1)", job)
            .execute(&mut *conn)
            .await
            .unwrap()
            .last_insert_rowid(),
    )
}

pub async fn get_job(Path(job_id): Path<i64>, pool: SqlitePool) -> Json<Option<String>> {
    let mut conn = pool.acquire().await.unwrap();
    Json(
        sqlx::query!("SELECT source FROM jobs WHERE id = $1", job_id)
            .fetch_optional(&mut *conn)
            .await
            .unwrap()
            .map(|row| row.source),
    )
}

pub async fn post_job_run(Path(job_id): Path<i64>, pool: SqlitePool) -> Json<i64> {
    let mut conn = pool.acquire().await.unwrap();
    let source = sqlx::query!("SELECT source FROM jobs WHERE id = $1", job_id)
        .fetch_one(&mut *conn)
        .await
        .unwrap()
        .source;
    spawn_job(source, pool);
    Json(0)
}

fn spawn_job(source: String, _pool: SqlitePool) {
    tokio::spawn(async move {
        let docker = Docker::new("unix:///var/run/docker.sock").unwrap();
        let opts = VolumeCreateOpts::builder().build();
        let volume = docker.volumes().create(&opts).await.unwrap();
        log::info!("created volume: {}", volume.name);
        let opts = ContainerCreateOpts::builder()
            .image("rust:latest")
            .command(vec!["git", "clone", &source, "code"])
            .working_dir("/workspace")
            .volumes(vec![format!("{}:/workspace", volume.name)])
            .build();
        let container = docker.containers().create(&opts).await.unwrap();
        log::info!("created container: {}", container.id());
        container.start().await.unwrap();
        container.wait().await.unwrap();
        let opts = ContainerRemoveOpts::builder().build();
        container.remove(&opts).await.unwrap();
        let opts = ContainerCreateOpts::builder()
            .image("rust:latest")
            .command(vec!["cargo", "test"])
            .working_dir("/workspace/code")
            .volumes(vec![format!("{}:/workspace", volume.name)])
            .build();
        let container = docker.containers().create(&opts).await.unwrap();
        log::info!("created container: {}", container.id());
        container.start().await.unwrap();
        container.wait().await.unwrap();
        let opts = ContainerRemoveOpts::builder().build();
        container.remove(&opts).await.unwrap();
    });
}
