use std::{convert::Infallible, net::SocketAddr};

use clap::Parser;
use log::{error, info};
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
        job_path,
        data_path,
        address,
    } = Config::read(&args.config_path);
    let path_builder = osprei_server::PathBuilder::new(job_path, data_path);
    let persistance =
        osprei_server::database::DatabasePersistance::new(path_builder.database_path()).await;
    build_workspace(path_builder.workspace_dir()).await;
    let job_list = warp::path!("job")
        .and(with_string(path_builder.job_path().to_string()))
        .and_then(list_jobs);
    let job_get = warp::path!("job" / String)
        .and(with_string(path_builder.job_path().to_string()))
        .and_then(get_job);
    let job_run = warp::path!("job" / String / "run")
        .and(with_persistance(persistance.clone()))
        .and(with_string(path_builder.job_path().to_string()))
        .and(with_string(path_builder.workspace_dir().to_string()))
        .and_then(job_run);
    let execution_list = warp::path!("job" / String / "executions")
        .and(with_persistance(persistance.clone()))
        .and_then(job_executions);
    let execution_get = warp::path!("execution" / i64)
        .and(with_persistance(persistance.clone()))
        .and_then(execution_details);
    warp::serve(
        warp::any()
            .and(
                job_list
                    .or(job_get)
                    .or(job_run)
                    .or(execution_list)
                    .or(execution_get),
            )
            .with(warp::cors().allow_any_origin()),
    )
    .run(address.parse::<SocketAddr>().unwrap())
    .await;
}

async fn build_workspace(workspace_dir: &str) {
    tokio::fs::create_dir_all(workspace_dir).await.unwrap();
}

fn with_string(
    string: String,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || string.clone())
}

fn with_persistance(
    db: osprei_server::database::DatabasePersistance,
) -> impl Filter<
    Extract = (osprei_server::database::DatabasePersistance,),
    Error = std::convert::Infallible,
> + Clone {
    warp::any().map(move || db.clone())
}

async fn execution_details(
    execution_id: i64,
    persistance: impl osprei_server::database::Persistance,
) -> Result<impl warp::Reply, Infallible> {
    let execution = persistance.get_execution(execution_id).await;
    Ok(warp::reply::json(&execution))
}

async fn job_executions(
    job_name: String,
    persistance: impl osprei_server::database::Persistance,
) -> Result<impl warp::Reply, Infallible> {
    let executions = persistance.last_executions(job_name, 10).await;
    Ok(warp::reply::json(&executions))
}

async fn list_jobs(job_dir: String) -> Result<impl warp::Reply, Infallible> {
    let jobs: Vec<_> = jobs(job_dir)
        .await
        .into_iter()
        .map(|osprei::Job { name, .. }| name)
        .collect();
    Ok(warp::reply::json(&jobs))
}

async fn get_job(job_name: String, job_dir: String) -> Result<impl warp::Reply, Infallible> {
    let job = jobs(job_dir)
        .await
        .into_iter()
        .find(|job| job.name == job_name)
        .unwrap();
    Ok(warp::reply::json(&job))
}

async fn jobs(job_dir: String) -> Vec<osprei::Job> {
    let mut entries = tokio::fs::read_dir(job_dir).await.unwrap();
    let mut jobs: Vec<osprei::Job> = Vec::new();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        let path = entry.path().to_str().unwrap().to_string();
        let string = tokio::fs::read_to_string(path).await.unwrap();
        let job = serde_json::from_str(&string).unwrap();
        jobs.push(job);
    }
    jobs
}

async fn job_run(
    job_name: String,
    persistance: impl osprei_server::database::Persistance + Send + Sync + 'static,
    job_dir: String,
    data_dir: String,
) -> Result<impl warp::Reply, Infallible> {
    let job = jobs(job_dir)
        .await
        .into_iter()
        .find(|job| job.name == job_name)
        .unwrap();
    let index = persistance.create_execution(job_name.clone()).await;
    let mut job_runner = osprei::JobRunner::new(&job_name, &data_dir);
    {
        let execution_id = index.clone();
        tokio::spawn(async move {
            match job_runner.execute(job).await {
                Ok(res) => {
                    info!("Successfull execution of {}: {:?}", job_name, res);
                    let status = match res.iter().any(|status| status.status != 0) {
                        true => 1,
                        false => 0,
                    };
                    persistance.set_execution_status(execution_id, status).await;
                }
                Err(err) => {
                    error!("An error occured while executing: {}: {}", job_name, err);
                    persistance.set_execution_status(execution_id, 1).await;
                }
            }
        });
    }
    Ok(warp::reply::json(&index))
}

#[derive(Debug, Serialize)]
struct ExecutionList {
    executions: Vec<ExecutionSummary>,
}

#[derive(Debug, Serialize)]
struct ExecutionSummary {
    id: i64,
    timestamp: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    job_path: String,
    data_path: String,
    address: String,
}

impl Config {
    fn read(path: &str) -> Self {
        let file = std::fs::File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }
}
