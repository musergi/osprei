use clap::Parser;
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
    println!("Reading configuration from {}", args.config_path);
    let config = Config::read(&args.config_path);
    println!("Configuration: {:?}", config);
    match std::fs::read_dir(&config.job_path) {
        Ok(job_paths) => {
            for path in job_paths {
                let job = Job::read(path.unwrap().path().to_str().unwrap());
                println!("Found job: {:?}", job);
                job.run();
            }
        }
        Err(err) => println!("Job dir not found: {}", err)
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
    command: String,
    args: Vec<String>,
}

impl Job {
    fn read(path: &str) -> Self {
        let file = std::fs::File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }

    fn run(&self) {
        let mut cmd = std::process::Command::new(&self.command);
        for arg in self.args.iter() {
            cmd.arg(arg);
        }
        let output = cmd.output().unwrap();
        println!("{:?}", output);
    }
}
