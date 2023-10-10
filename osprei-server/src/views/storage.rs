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

const INSERT_JOB_QUERY: &str = "
    INSERT INTO jobs (
        name,
        definition
    ) VALUES (
        $1,
        $2
    )
";

const INSERT_EXECUTION_QUERY: &str = "
    INSERT INTO executions (
        job_id,
        start_time
    ) VALUES (
        $1,
        CURRENT_TIMESTAMP
    )
";

const UPDATE_EXECUTION_QUERY: &str = "
    UPDATE executions
    SET
        status = $2,
        stdout = $3,
        stderr = $4
    WHERE id = $1
";

const GET_EXECUTION_QUERY: &str = "
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

    pub async fn get_job(&self, job_id: i64) -> Result<osprei::JobPointer, Error> {
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

    pub async fn post_job(
        &self,
        request: osprei::JobCreationRequest,
    ) -> Result<impl serde::Serialize, Error> {
        let osprei::JobCreationRequest { name, definition } = request;
        let definition = serde_json::to_string(&definition).unwrap();
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query(INSERT_JOB_QUERY)
            .bind(name)
            .bind(definition)
            .execute(&mut conn)
            .await?
            .last_insert_rowid();
        Ok(id)
    }

    pub async fn create_execution(&self, job_id: i64) -> Result<(i64, osprei::JobPointer), Error> {
        let pointer = self.get_job(job_id).await?;
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query(INSERT_EXECUTION_QUERY)
            .bind(job_id)
            .execute(&mut conn)
            .await?
            .last_insert_rowid();
        Ok((id, pointer))
    }

    pub async fn update_execution(
        &self,
        execution_id: i64,
        status: i64,
        stdout: String,
        stderr: String,
    ) -> Result<(), Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query(UPDATE_EXECUTION_QUERY)
            .bind(execution_id)
            .bind(status)
            .bind(stdout)
            .bind(stderr)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn get_execution(&self, execution_id: i64) -> Result<impl serde::Serialize, Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query(GET_EXECUTION_QUERY)
            .bind(execution_id)
            .fetch_optional(&mut conn)
            .await?
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
            .ok_or(Error::NotFound)
    }
}
