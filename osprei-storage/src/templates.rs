use crate::{db, Error};

pub async fn names() -> Result<Vec<String>, Error> {
    let mut conn = db().await?;
    log::info!("Get names");
    struct Query {
        name: String,
    }
    let names = sqlx::query_as!(
        Query,
        "
            SELECT name
            FROM templates
        "
    )
    .fetch_all(&mut conn)
    .await?
    .into_iter()
    .map(|query| query.name)
    .collect();
    Ok(names)
}
