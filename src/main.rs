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
