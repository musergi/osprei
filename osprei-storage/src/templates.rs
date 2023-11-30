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

pub async fn for_name(name: String) -> Result<osprei_data::Template, Error> {
    let mut conn = db().await?;
    log::info!("Get for {name}");
    struct Query {
        definition: String,
    }
    let definition = sqlx::query_as!(
        Query,
        "
            SELECT definition
            FROM templates
            WHERE name = $1
        ",
        name
    )
    .fetch_one(&mut conn)
    .await?
    .definition;
    let definition = serde_json::from_str(&definition)?;
    Ok(definition)
}
