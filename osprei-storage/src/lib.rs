use sqlx::Connection;

pub mod job;

pub mod execution;
pub struct Stage {
    pub id: i64,
    pub dependency: Option<i64>,
    pub definition: Defintion,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Defintion {
    pub name: String,
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
        "SELECT id, dependency, definition FROM stages WHERE job = $1 ORDER BY id",
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
