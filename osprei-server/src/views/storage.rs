use log::error;
use sqlx::Row;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

impl Error {
    pub fn to_reply(self) -> impl warp::Reply {
        match self {
            Error::DatabaseError(err) => {
                error!("{}", err);
                let status = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
                let message = osprei::Error {
                    status: status.as_u16() as i64,
                    message: "Internal server error".to_string(),
                };
                let reply = warp::reply::json(&message);
                warp::reply::with_status(reply, status)
            }
            Error::NotFound => {
                let status = warp::http::StatusCode::NOT_FOUND;
                let message = osprei::Error {
                    status: status.as_u16() as i64,
                    message: "Not found".to_string(),
                };
                let reply = warp::reply::json(&message);
                warp::reply::with_status(reply, status)
            }
        }
    }
}
pub struct Storage {
    pool: sqlx::SqlitePool,
}

const GET_JOBS_QUERY: &str = "
        SELECT 
            jid,
            name,
            eid,
            start_time,
            status
        FROM (
            SELECT
                jobs.id AS jid,
                name,
                executions.id AS eid,
                start_time,
                status,
                row_number() OVER ( partition BY jobs.id ORDER BY start_time DESC, executions.id DESC) AS score
            FROM (
                jobs
                LEFT JOIN executions
                     ON (jobs.id = job_id)
            )
        )
        WHERE  score = 1
    ";

const GET_JOB_QUERY: &str = "
        SELECT 
            id,
            name,
            definition
        FROM jobs
        WHERE id = $1
    ";

mod adapter;

impl Storage {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_jobs(&self) -> Result<impl serde::Serialize, Error> {
        let mut conn = self.pool.acquire().await?;
        let jobs: Vec<_> = sqlx::query(GET_JOBS_QUERY)
            .fetch_all(&mut conn)
            .await?
            .into_iter()
            .map(adapter::JobOverview::from)
            .map(osprei::JobOverview::from)
            .collect();
        Ok(jobs)
    }

    pub async fn get_job(&self, job_id: i64) -> Result<impl serde::Serialize, Error> {
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(GET_JOB_QUERY)
            .bind(job_id)
            .fetch_optional(&mut conn)
            .await?
            .ok_or(Error::NotFound)?;
        let definition: String = row.get(2);
        let definition = serde_json::from_str(&definition).unwrap();
        let job = osprei::JobPointer {
            id: row.get(0),
            name: row.get(1),
            definition,
        };
        Ok(job)
    }
}
