use std::{convert::Infallible, io::Write, net::SocketAddr};

use clap::Parser;
use log::{error, info};
use serde::{Deserialize, Serialize};
use warp::{reject::Reject, Filter};

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
    let job_list = warp::path!("job")
        .and(with_job_dir(job_path.clone()))
        .and_then(list_jobs);
    let job_get = warp::path!("job" / String)
        .and(with_job_dir(job_path.clone()))
        .and_then(get_job);
    let job_run = warp::path!("job" / String / "run")
        .and(with_job_dir(job_path.clone()))
        .and(with_job_dir(data_path.clone()))
        .and_then(job_run);
    warp::serve(job_list.or(job_get).or(job_run))
        .run(address.parse::<SocketAddr>().unwrap())
        .await;
}

fn with_job_dir(
    job_dir: String,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || job_dir.clone())
}

async fn list_jobs(job_dir: String) -> Result<impl warp::Reply, Infallible> {
    let jobs: Vec<_> = jobs(job_dir)
        .await
        .into_iter()
        .map(|Job { name, .. }| name)
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

async fn jobs(job_dir: String) -> Vec<Job> {
    let mut entries = tokio::fs::read_dir(job_dir).await.unwrap();
    let mut jobs: Vec<Job> = Vec::new();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        let job = Job::read(entry.path().to_str().unwrap());
        jobs.push(job);
    }
    jobs
}

async fn job_run(
    job_name: String,
    job_dir: String,
    data_dir: String,
) -> Result<impl warp::Reply, Infallible> {
    let job = jobs(job_dir)
        .await
        .into_iter()
        .find(|job| job.name == job_name)
        .unwrap();
    job.arun(&data_dir).await;
    Ok(warp::reply::json(&String::from("ok")))
}

struct Model {
    job_dir: String,
    data_dir: String,
}

impl Model {
    fn new(job_dir: String, data_dir: String) -> Model {
        Model { job_dir, data_dir }
    }
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

#[derive(Deserialize, Serialize, Debug)]
struct Job {
    name: String,
    source: String,
    command: String,
    args: Vec<String>,
}

impl Job {
    fn read(path: &str) -> Self {
        let file = std::fs::File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }

    async fn arun(&self, base_path: &str) {
        let mut buf = std::path::PathBuf::from(base_path);
        buf.push(&self.name);
        let repo_dir = buf.as_path().to_str().unwrap();
        let output = tokio::process::Command::new("git")
            .arg("clone")
            .arg(&self.source)
            .arg(repo_dir)
            .output()
            .await
            .unwrap();
        info!(
            "{}: checkout exit status: {}",
            self.name,
            output.status.code().unwrap_or(128)
        );
        if output.status.success() {
            let mut cmd = tokio::process::Command::new(&self.command);
            for arg in self.args.iter() {
                cmd.arg(arg);
            }
            let command_output = cmd.current_dir(repo_dir).output().await.unwrap();
            info!(
                "{}: command exit status: {}",
                self.name,
                command_output.status.code().unwrap_or(128)
            );
        }
    }

    fn run(&self, base_path: &str) {
        let mut buf = std::path::PathBuf::from(base_path);
        buf.push(&self.name);
        let repo_dir = buf.as_path().to_str().unwrap();

        let result = match std::fs::metadata(repo_dir)
            .map(|metadata| metadata.is_dir())
            .unwrap_or(false)
        {
            true => match self.update(repo_dir) {
                Ok(clone_output) => {
                    let mut cmd = std::process::Command::new(&self.command);
                    for arg in self.args.iter() {
                        cmd.arg(arg);
                    }
                    let command_output = cmd.current_dir(repo_dir).output().unwrap();
                    ExecutionResult::Execution {
                        clone_output,
                        command_output,
                    }
                }
                Err(output) => ExecutionResult::SourceFailure { output },
            },
            false => match self.clone(repo_dir) {
                Ok(clone_output) => {
                    let mut cmd = std::process::Command::new(&self.command);
                    for arg in self.args.iter() {
                        cmd.arg(arg);
                    }
                    let command_output = cmd.current_dir(repo_dir).output().unwrap();
                    ExecutionResult::Execution {
                        clone_output,
                        command_output,
                    }
                }
                Err(output) => ExecutionResult::SourceFailure { output },
            },
        };

        let mut buf = std::path::PathBuf::from(base_path);
        buf.push("results");
        buf.push(&self.name);
        buf.push(format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(buf.as_path()).unwrap();
        buf.push("output.txt");
        self.write_result(result, buf.as_path().to_str().unwrap());
    }

    fn update(&self, repo_dir: &str) -> Result<std::process::Output, std::process::Output> {
        let output = std::process::Command::new("git")
            .arg("pull")
            .current_dir(repo_dir)
            .output()
            .unwrap();
        if output.status.success() {
            Ok(output)
        } else {
            Err(output)
        }
    }

    fn clone(&self, repo_dir: &str) -> Result<std::process::Output, std::process::Output> {
        let output = std::process::Command::new("git")
            .arg("clone")
            .arg(&self.source)
            .arg(repo_dir)
            .output()
            .unwrap();
        if output.status.success() {
            Ok(output)
        } else {
            Err(output)
        }
    }

    fn write_result(&self, result: ExecutionResult, path: &str) {
        let mut file = std::fs::File::create(path).unwrap();
        match result {
            ExecutionResult::SourceFailure { output } => {
                error!(
                    "Error occured when performing git checkout, more info: {}",
                    path
                );
                file.write_all("Source checkout failed\n".as_bytes())
                    .unwrap();
                file.write_all("git stdout:\n".as_bytes()).unwrap();
                file.write_all(&output.stdout).unwrap();
                file.write_all("git stderr:\n".as_bytes()).unwrap();
                file.write_all(&output.stderr).unwrap();
            }
            ExecutionResult::Execution {
                clone_output,
                command_output,
            } => {
                file.write_all("git stdout:\n".as_bytes()).unwrap();
                file.write_all(&clone_output.stdout).unwrap();
                file.write_all("git stderr:\n".as_bytes()).unwrap();
                file.write_all(&clone_output.stderr).unwrap();
                file.write_all("job stdout:\n".as_bytes()).unwrap();
                file.write_all(&command_output.stdout).unwrap();
                file.write_all("job stderr:\n".as_bytes()).unwrap();
                file.write_all(&command_output.stderr).unwrap();
                info!("Successfully executed job '{}' results: ", path);
            }
        }
    }
}

enum ExecutionResult {
    SourceFailure {
        output: std::process::Output,
    },
    Execution {
        clone_output: std::process::Output,
        command_output: std::process::Output,
    },
}
