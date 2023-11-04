use log::info;

use crate::{db, Error, ExecutionStatus};

pub async fn create(job_id: i64) -> Result<i64, Error> {
    let mut conn = db().await?;
    log::info!("Insert execution with job ({job_id})");
    let execution_id = sqlx::query!(
        "
            INSERT INTO executions
            (job, start_time)
            VALUES ($1, datetime('now'))
            ",
        job_id
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();
    info!("Inserted");
    Ok(execution_id)
}

pub async fn ids() -> Result<Vec<i64>, Error> {
    let mut conn = db().await?;
    log::info!("Get ids");
    struct Query {
        id: i64,
    }
    let ids = sqlx::query_as!(
        Query,
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

pub async fn status(id: i64) -> Result<ExecutionStatus, Error> {
    let mut conn = db().await?;
    log::info!("Get execution ({id}) status");
    struct Query {
        status: Option<i64>,
    }
    let status: ExecutionStatus = sqlx::query_as!(
        Query,
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

pub async fn success(id: i64) -> Result<(), Error> {
    set_status(id, 0).await
}

pub async fn failure(id: i64) -> Result<(), Error> {
    set_status(id, 1).await
}

async fn set_status(id: i64, status: i64) -> Result<(), Error> {
    let mut conn = db().await?;
    log::info!("Set execution ({id}) status ({status})");
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

pub async fn duration(id: i64) -> Result<Option<i64>, Error> {
    let mut conn = db().await?;
    log::info!("Get execution ({id}) duration");
    struct Query {
        duration: Option<i64>,
    }
    let duration = sqlx::query_as!(
        Query,
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
