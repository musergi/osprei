use crate::{db, Error};
use osprei_data::{EnvironmentVariable, StageDefinition};

pub const WORKSPACE_DIR: &str = "/workspace";
pub const CHECKOUT_DIR: &str = "/workspace/code";
pub const GIT_IMAGE: &str = "ghcr.io/musergi/osprei-git:latest";
const SOURCE_ENV_VAR_NAME: &str = "SOURCE";

pub struct Stage {
    pub id: i64,
    pub dependency: Option<i64>,
    pub definition: osprei_data::StageDefinition,
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
        let definition: StageDefinition = serde_json::from_str(&definition)?;
        let stage = Stage {
            id,
            dependency,
            definition,
        };
        stages.push(stage);
    }
    Ok(stages)
}

pub async fn create(
    job_id: i64,
    dependency: i64,
    definition: StageDefinition,
) -> Result<(), Error> {
    create_optional(job_id, Some(dependency), definition).await
}

pub(crate) async fn create_checkout(job_id: i64, source: String) -> Result<(), Error> {
    log::info!("Insert checkout for job ({job_id})");
    let definition = StageDefinition {
        name: "checkout".to_string(),
        image: GIT_IMAGE.to_string(),
        environment: vec![EnvironmentVariable {
            name: SOURCE_ENV_VAR_NAME.to_string(),
            value: source,
        }],
        working_dir: WORKSPACE_DIR.to_string(),
    };
    create_optional(job_id, None, definition).await
}

async fn create_optional(
    job_id: i64,
    dependency: Option<i64>,
    definition: StageDefinition,
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
