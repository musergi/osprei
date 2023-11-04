use crate::{db, Error};

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

pub async fn for_job(job_id: i64) -> Result<Vec<Stage>, Error> {
    let mut conn = db().await?;
    log::info!("Get stages for job ({job_id})");
    struct Query {
        id: i64,
        dependency: Option<i64>,
        definition: String,
    }
    let query = sqlx::query_as!(
        Query,
        "
            SELECT id, dependency, definition
            FROM stages
            WHERE job = $1 ORDER BY id
            ",
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

pub async fn create(job_id: i64, dependency: i64, definition: Defintion) -> Result<(), Error> {
    let mut conn = db().await?;
    log::info!("Insert stage for job ({job_id})");
    let definition = serde_json::to_string(&definition)?;
    sqlx::query!(
        "
            INSERT INTO stages
            (job, dependency, definition)
            VALUES ($1, $2, $3)
            ",
        job_id,
        dependency,
        definition
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}
