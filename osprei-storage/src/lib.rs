use sqlx::Connection;

pub mod job;

pub mod execution;

pub mod stages;
pub use stages::Stage;

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
            Error::Sqlx(err) => write!(f, "sqlx: {}", err),
            Error::Serde(err) => write!(f, "serde: {}", err),
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
