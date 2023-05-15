use std::net::SocketAddr;

use clap::Parser;
use log::info;
use osprei_server::persistance;
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
    let Config { data_path, address } = Config::read(&args.config_path);
    let path_builder = osprei_server::PathBuilder::new(data_path);
    let store = persistance::memory::MemoryStore::default();
    build_workspace(path_builder.workspaces()).await;
    let get_jobs = warp::path!("job")
        .and(warp::get())
        .and(persistance::memory::with(store.clone()))
        .and_then(views::get_jobs);
    let get_job = warp::path!("job" / i64)
        .and(warp::get())
        .and(persistance::memory::with(store.clone()))
        .and_then(views::get_job);
    let post_job = warp::path!("job")
        .and(warp::post())
        .and(warp::body::json())
        .and(persistance::memory::with(store.clone()))
        .and_then(views::post_job);
    let get_job_run = warp::path!("job" / i64 / "run")
        .and(warp::get())
        .and(with_path_builder(path_builder))
        .and(persistance::memory::with(store.clone()))
        .and_then(views::get_job_run);
    let get_job_executions = warp::path!("job" / i64 / "executions")
        .and(warp::get())
        .and(persistance::memory::with(store.clone()))
        .and_then(views::get_job_executions);
    let get_execution = warp::path!("execution" / i64)
        .and(warp::get())
        .and(persistance::memory::with(store))
        .and_then(views::get_execution);
    warp::serve(
        warp::any()
            .and(
                get_jobs
                    .or(get_job)
                    .or(post_job)
                    .or(get_job_run)
                    .or(get_job_executions)
                    .or(get_execution),
            )
            .with(warp::cors().allow_any_origin()),
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
}

impl Config {
    fn read(path: &str) -> Self {
        let file = std::fs::File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }
}
