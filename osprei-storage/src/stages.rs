use crate::{db, Error};

pub const WORKSPACE_DIR: &str = "/workspace";
pub const CHECKOUT_DIR: &str = "/workspace/code";

pub struct Stage {
    pub id: i64,
    pub dependency: Option<i64>,
    pub definition: Definition,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Definition {
    pub name: String,
    pub command: Vec<String>,
    pub environment: Vec<EnvironmentVariable>,
    pub working_dir: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct EnvironmentVariable {
    pub name: String,
    pub value: String,
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
        let definition: Definition = serde_json::from_str(&definition)?;
        let stage = Stage {
            id,
            dependency,
            definition,
        };
        stages.push(stage);
    }
    Ok(stages)
}

pub async fn create(job_id: i64, dependency: i64, definition: Definition) -> Result<(), Error> {
    create_optional(job_id, Some(dependency), definition).await
}

pub(crate) async fn create_checkout(job_id: i64, source: String) -> Result<(), Error> {
    log::info!("Insert checkout for job ({job_id})");
    let command = checkout_command(source);
    let definition = Definition {
        name: "checkout".to_string(),
        command,
        environment: Vec::new(),
        working_dir: WORKSPACE_DIR.to_string(),
    };
    create_optional(job_id, None, definition).await
}

fn checkout_command(source: String) -> Vec<String> {
    vec![
        "git".to_string(),
        "clone".to_string(),
        source,
        CHECKOUT_DIR.to_string(),
    ]
}

async fn create_optional(
    job_id: i64,
    dependency: Option<i64>,
    definition: Definition,
) -> Result<(), Error> {
    let mut conn = db().await?;
    log::info!("Insert for job ({job_id})");
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
