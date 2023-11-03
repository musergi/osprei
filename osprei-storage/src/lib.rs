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
    log::info!("Loading job source {}", id);
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

pub async fn job_status(id: i64) -> Result<Option<ExecutionStatus>, Error> {
    let mut conn = db().await?;
    log::info!("Loading job status {}", id);
    let status: Option<ExecutionStatus> = sqlx::query_as!(
        StatusQuery,
        "
        SELECT status
        FROM (
            jobs
            INNER JOIN
                executions ON executions.job = jobs.id
        )
        WHERE jobs.id = $1
        ORDER BY executions.id DESC
        LIMIT 1
        ",
        id
    )
    .fetch_optional(&mut conn)
    .await?
    .map(|query| query.status.into());
    Ok(status)
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
    let execution_id = sqlx::query!(
        "
        INSERT INTO executions (job, start_time)
        VALUES ($1, datetime('now'))
        ",
        job_id
    )
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
    execution_set_status(id, 0).await
}

pub async fn execution_failure(id: i64) -> Result<(), Error> {
    execution_set_status(id, 1).await
}

async fn execution_set_status(id: i64, status: i64) -> Result<(), Error> {
    let mut conn = db().await?;
    log::info!("Setting execution {} to {}", id, status);
    sqlx::query!(
        "
        UPDATE executions
        SET
            status = $2,
            end_time = datetime('now')
        WHERE id = $1
        ",
        id,
        status
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn execution_ids() -> Result<Vec<i64>, Error> {
    let mut conn = db().await?;
    log::info!("Getting executions");
    let ids = sqlx::query_as!(
        ExecutionQuery,
        "
        SELECT id
        FROM executions
        ORDER BY id DESC
        "
    )
    .fetch_all(&mut conn)
    .await?
    .into_iter()
    .map(|query| query.id)
    .collect();
    Ok(ids)
}

pub struct Stage {
    pub id: i64,
    pub dependency: Option<i64>,
    pub definition: Defintion,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Defintion {
    name: String,
    command: Vec<String>,
    environment: Vec<EnvironmentVariable>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct EnvironmentVariable {
    name: String,
    value: String,
}

pub async fn stages(job_id: i64) -> Result<Vec<Stage>, Error> {
    let mut conn = db().await?;
    log::info!("Fetch stages for job {job_id}");
    struct Query {
        id: i64,
        dependency: Option<i64>,
        definition: String,
    }
    let query = sqlx::query_as!(
        Query,
        "SELECT id, dependency, definition FROM stages WHERE job = $1",
        job_id
    )
    .fetch_all(&mut conn)
    .await?;
    let mut stages = Vec::with_capacity(query.len());
    for Query {
        id,
        dependency,
        definition,
    } in query
    {
        let definition: Defintion = serde_json::from_str(&definition)?;
        let stage = Stage {
            id,
            dependency,
            definition,
        };
        stages.push(stage);
    }
    Ok(stages)
}

pub async fn execution_duration(id: i64) -> Result<Option<i64>, Error> {
    let mut conn = db().await?;
    log::info!("Fetch execution duration for {}", id);
    let duration = sqlx::query_as!(
        DurationQuery,
        "
        SELECT unixepoch(end_time) - unixepoch(start_time) AS duration
        FROM executions
        WHERE id = $1
        ",
        id
    )
    .fetch_one(&mut conn)
    .await?
    .duration;
    Ok(duration)
}

pub async fn stage_create(
    job_id: i64,
    dependency: i64,
    definition: Defintion,
) -> Result<(), Error> {
    let mut conn = db().await?;
    log::info!("Creating stage for {job_id}");
    let definition = serde_json::to_string(&definition)?;
    sqlx::query!(
        "INSERT INTO stages (job, dependency, definition) VALUES ($1, $2, $3)",
        job_id,
        dependency,
        definition
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

struct DurationQuery {
    duration: Option<i64>,
}

struct ExecutionQuery {
    id: i64,
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
    Serde(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Sqlx(err) => write!(f, "sqlx error: {}", err),
            Error::Serde(err) => write!(f, "serde error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Error {
        Error::Sqlx(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Serde(value)
    }
}
