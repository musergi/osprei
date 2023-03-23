use std::io::Write;

use clap::Parser;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

/// Osprei CI server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File path to the configuration
    #[arg(short, long)]
    config_path: String,
}

fn main() {
    let args = Args::parse();
    pretty_env_logger::init();
    info!("Reading configuration from {}", args.config_path);
    let config = Config::read(&args.config_path);
    debug!("Configuration: {:?}", config);
    match std::fs::read_dir(&config.job_path) {
        Ok(job_paths) => {
            for path in job_paths {
                let job = Job::read(path.unwrap().path().to_str().unwrap());
                debug!("Found job: {:?}", job);
                job.run(&config.data_path);
            }
        }
        Err(err) => error!("Job dir not found: {}", err),
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    job_path: String,
    data_path: String,
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
