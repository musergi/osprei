use crate::{db, Error, ExecutionStatus};

pub async fn ids() -> Result<Vec<i64>, Error> {
    let mut conn = db().await?;
    log::info!("Get ids");
    struct Query {
        id: i64,
    }
    let jobs = sqlx::query_as!(
        Query,
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

pub async fn source(id: i64) -> Result<String, Error> {
    let mut conn = db().await?;
    log::info!("Get ({id}) source");
    struct Query {
        source: String,
    }
    let source = sqlx::query_as!(
        Query,
        "
        SELECT source
        FROM jobs
        WHERE id = $1
        ",
        id
    )
    .fetch_one(&mut conn)
    .await?
    .source;
    Ok(source)
}

pub async fn status(id: i64) -> Result<Option<ExecutionStatus>, Error> {
    let mut conn = db().await?;
    log::info!("Get ({id}) status");
    struct Query {
        status: Option<i64>,
    }
    let status: Option<ExecutionStatus> = sqlx::query_as!(
        Query,
        "
        SELECT status
        FROM (
            jobs
            INNER JOIN
                executions
                    ON executions.job = jobs.id
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

pub async fn create(source: String) -> Result<(), Error> {
    let mut conn = db().await?;
    log::info!("Insert ({source})");
    sqlx::query!(
        "
        INSERT INTO jobs
        (source)
        VALUES ($1)",
        source
    )
    .execute(&mut conn)
    .await?;
    log::info!("Inserted");
    Ok(())
}
