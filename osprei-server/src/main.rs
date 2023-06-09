use std::net::SocketAddr;

use clap::Parser;
use log::{debug, info};
use osprei_server::persistance::PersistanceConfig;
use osprei_server::{execute, persistance};
use osprei_server::{views, PathBuilder};
use serde::{Deserialize, Serialize};
use warp::Filter;

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
    info!("Reading configuration from {}", args.config_path);
    let Config {
        data_path,
        address,
        persistance,
    } = Config::read(&args.config_path);
    let path_builder = osprei_server::PathBuilder::new(data_path);
    let persistance = persistance::build(persistance).await;
    build_workspace(path_builder.workspaces()).await;
    for schedule in persistance.boxed().get_all_schedules().await {
        let osprei::Schedule {
            job_id,
            hour,
            minute,
            ..
        } = schedule;
        let job = persistance.boxed().fetch_job(job_id).await;
        debug!("Scheduling {} for {}h{}", job.name, hour, minute);
        execute::schedule_job(job, hour, minute, path_builder.clone(), persistance.boxed()).await;
    }
    let get_jobs = warp::path!("job")
        .and(warp::get())
        .and(persistance::with(persistance.clone()))
        .and_then(views::get_jobs);
    let get_job = warp::path!("job" / i64)
        .and(warp::get())
        .and(persistance::with(persistance.clone()))
        .and_then(views::get_job);
    let post_job = warp::path!("job")
        .and(warp::post())
        .and(warp::body::json())
        .and(persistance::with(persistance.clone()))
        .and_then(views::post_job);
    let get_job_run = warp::path!("job" / i64 / "run")
        .and(warp::get())
        .and(with_path_builder(path_builder.clone()))
        .and(persistance::with(persistance.clone()))
        .and_then(views::get_job_run);
    let get_job_executions = warp::path!("job" / i64 / "executions")
        .and(warp::get())
        .and(persistance::with(persistance.clone()))
        .and_then(views::get_job_executions);
    let get_execution = warp::path!("execution" / i64)
        .and(warp::get())
        .and(persistance::with(persistance.clone()))
        .and_then(views::get_execution);
    let get_job_schedule = warp::path!("job" / i64 / "schedule")
        .and(warp::get())
        .and(persistance::with(persistance.clone()))
        .and_then(views::get_job_schedule);
    let post_job_schedule = warp::path!("job" / i64 / "schedule")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_path_builder(path_builder))
        .and(persistance::with(persistance.clone()))
        .and_then(views::post_job_schedule);
    warp::serve(
        warp::any()
            .and(
                get_jobs
                    .or(get_job)
                    .or(post_job)
                    .or(get_job_run)
                    .or(get_job_executions)
                    .or(get_execution)
                    .or(get_job_schedule)
                    .or(post_job_schedule),
            )
            .with(
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
                    .allow_methods(vec!["GET", "POST"]),
            ),
    )
    .run(address.parse::<SocketAddr>().unwrap())
    .await;
}

async fn build_workspace(workspace_dir: &str) {
    tokio::fs::create_dir_all(workspace_dir).await.unwrap();
}

fn with_path_builder(
    path_builder: PathBuilder,
) -> impl Filter<Extract = (PathBuilder,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || path_builder.clone())
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    data_path: String,
    address: String,
    persistance: PersistanceConfig,
}

impl Config {
    fn read(path: &str) -> Self {
        let file = std::fs::File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }
}
