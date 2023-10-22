use sqlx::Connection;

pub async fn job_ids() -> Result<Vec<i64>, Error> {
    log::info!("Getting database");
    let mut conn = db().await?;
    log::info!("Loading jobs");
    let jobs = sqlx::query_as!(
        JobId,
        "
        SELECT id
        FROM jobs
        "
    )
    .fetch_all(&mut conn)
    .await?
    .into_iter()
    .map(|job| job.id)
    .collect();
    Ok(jobs)
}

pub async fn job_source(id: i64) -> Result<String, Error> {
    log::info!("Getting database");
    let mut conn = db().await?;
    log::info!("Loading job {}", id);
    let job = sqlx::query_as!(
        JobSource,
        "
        SELECT source
        FROM jobs
        WHERE id = $1
        ",
        id
    )
    .fetch_one(&mut conn)
    .await?;
    Ok(job.source)
}

pub async fn job_create(source: String) -> Result<(), Error> {
    log::info!("Adding job with source {}", source);
    let mut conn = db().await?;
    sqlx::query!("INSERT INTO jobs (source) VALUES ($1)", source)
        .execute(&mut conn)
        .await?;
    log::info!("Added");
    Ok(())
} 

struct JobSource {
    source: String,
}

struct JobId {
    id: i64,
}

async fn db() -> Result<sqlx::SqliteConnection, Error> {
    let url = std::env::var("DATABASE_URL").unwrap();
    Ok(sqlx::SqliteConnection::connect(&url).await?)
}

#[derive(Debug)]
pub enum Error {
    Sqlx(sqlx::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Sqlx(err) => write!(f, "sqlx error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Error {
        Error::Sqlx(value)
    }
}
