use docker_api::Docker;
use log::{error, warn};

use clap::Parser;
use config::Config;
use routes::routes;
use warp::Filter;

mod config;
mod routes;
mod views;

/// Osprei CI server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File path to the configuration
    #[arg(short, long)]
    config_path: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    pretty_env_logger::init();
    let config = Config::read(&args.config_path);
    match config {
        Ok(Config {
            address,
            data_path,
            persistance,
        }) => {
            let pool = create_database(&persistance).await.unwrap();
            build_workspace(&data_path).await;
            let docker = Docker::new("unix:///var/run/docker.sock").unwrap();
            warp::serve(
                warp::any()
                    .and(routes(pool, docker))
                    .with(cors())
                    .with(warp::log("api")),
            )
            .run(address)
            .await;
        }
        Err(err) => error!("Config error: {}", err),
    };
}

async fn create_database(database_path: &str) -> Result<sqlx::sqlite::SqlitePool, sqlx::Error> {
    let url = format!("{}?mode=rwc", database_path);
    let pool = sqlx::sqlite::SqlitePool::connect(&url).await?;
    let mut conn = pool.acquire().await?;
    sqlx::query(
        "
            CREATE TABLE IF NOT EXISTS
                jobs
            (
                id INTEGER PRIMARY KEY,
                name TEXT,
                definition TEXT
            )
        ",
    )
    .execute(&mut conn)
    .await?;
    sqlx::query(
        "
            CREATE TABLE IF NOT EXISTS
                executions
            (
                id INTEGER PRIMARY KEY,
                job_id INTEGER,
                start_time TIMESTAMP,
                status INTEGER,
                stdout TEXT,
                stderr TEXT,
                FOREIGN KEY (job_id) REFERENCES jobs(id)
            )
        ",
    )
    .execute(&mut conn)
    .await?;
    sqlx::query(
        "
            CREATE TABLE IF NOT EXISTS
                schedules
            (
                id INTEGER PRIMARY KEY,
                job_id INTEGER,
                hour INTEGER,
                minute INTEGER,
                FOREIGN KEY (job_id) REFERENCES jobs(id)
            )
        ",
    )
    .execute(&mut conn)
    .await?;
    Ok(pool)
}

fn cors() -> warp::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "User-Agent",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Content-Type",
        ])
        .allow_methods(vec!["GET", "POST"])
}

async fn build_workspace(workspace_dir: &str) {
    if let Err(err) = tokio::fs::create_dir_all(workspace_dir).await {
        warn!("could not build workspace: {}", err);
    }
}
