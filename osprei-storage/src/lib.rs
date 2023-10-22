use sqlx::Connection;

pub async fn job_ids() -> Result<Vec<i64>, Error> {
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

pub async fn execution_create(job_id: i64) -> Result<i64, Error> {
    let mut conn = db().await?;
    log::info!("Creating execution for job {}", job_id);
    let execution_id = sqlx::query!("INSERT INTO executions (job) VALUES ($1)", job_id)
        .execute(&mut conn)
        .await?
        .last_insert_rowid();
    Ok(execution_id)
}

pub async fn execution_status(id: i64) -> Result<ExecutionStatus, Error> {
    let mut conn = db().await?;
    log::info!("Loading execution status {}", id);
    let status: ExecutionStatus = sqlx::query_as!(
        StatusQuery,
        "
        SELECT status
        FROM executions
        WHERE id = $1
        ",
        id
    )
    .fetch_one(&mut conn)
    .await?
    .status
    .into();
    Ok(status)
}

pub async fn execution_success(id: i64) -> Result<(), Error> {
    let mut conn = db().await?;
    log::info!("Setting execution {} to success", id);
    sqlx::query!(
        "
        UPDATE executions
        SET status = 0
        WHERE id = $1
        ",
        id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn execution_failure(id: i64) -> Result<(), Error> {
    let mut conn = db().await?;
    log::info!("Setting execution {} to failure", id);
    sqlx::query!(
        "
        UPDATE executions
        SET status = 1
        WHERE id = $1
        ",
        id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

struct StatusQuery {
    status: Option<i64>,
}

pub enum ExecutionStatus {
    Running,
    Success,
    Failure,
    Unknown,
}

impl From<Option<i64>> for ExecutionStatus {
    fn from(value: Option<i64>) -> ExecutionStatus {
        match value {
            None => Self::Running,
            Some(0) => Self::Success,
            Some(1) => Self::Failure,
            _ => Self::Unknown,
        }
    }
}

struct JobSource {
    source: String,
}

struct JobId {
    id: i64,
}

async fn db() -> Result<sqlx::SqliteConnection, Error> {
    let url = std::env::var("DATABASE_URL").unwrap();
    log::info!("Connecting to database: {}", url);
    let connection = sqlx::SqliteConnection::connect(&url).await?;
    log::info!("Connected to database");
    Ok(connection)
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
