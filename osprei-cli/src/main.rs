use clap::{Parser, Subcommand};
use reqwest::Client;

/// CLI to manage your osprei server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// Server to which to connect to send the requests
    #[arg(short, long)]
    server: String,

    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    JobsList,
    JobsAdd(JobAddArgs),
    JobsRun { job_id: i64 },
    JobsDescribe { job_id: i64 },
    JobsSchedule(JobScheduleArgs),
}

#[derive(Parser, Debug)]
struct JobAddArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    source: String,
    #[arg(long)]
    path: String,
}

#[derive(Parser, Debug)]
struct JobScheduleArgs {
    #[arg(long)]
    id: i64,
    #[arg(long)]
    hour: u8,
    #[arg(long)]
    minute: u8,
}

#[tokio::main]
async fn main() {
    let Args { server, commands } = Args::parse();
    match commands {
        Commands::JobsList => {
            let url = format!("{}/job", server);
            let body = reqwest::get(&url).await.unwrap().text().await.unwrap();
            let job_ids: Vec<i64> = serde_json::from_str(&body).unwrap();
            if job_ids.is_empty() {
                println!("No jobs yet, try adding a job.");
            } else {
                for job_id in job_ids {
                    let url = format!("{}/job/{}", server, job_id);
                    let body = reqwest::get(&url).await.unwrap().text().await.unwrap();
                    let osprei::JobPointer { id, name, .. } = serde_json::from_str(&body).unwrap();
                    println!("({}) {}", id, name);
                }
            }
        }
        Commands::JobsAdd(JobAddArgs { name, source, path }) => {
            let url = format!("{}/job", server);
            let req = osprei::JobCreationRequest { name, source, path };
            let response = reqwest::Client::new()
                .post(&url)
                .body(serde_json::to_string(&req).unwrap())
                .send()
                .await
                .unwrap();
            if response.status().is_success() {
                println!("Created job {}.", req.name);
            } else {
                println!("Failed to create job.");
            }
        }
        Commands::JobsRun { job_id } => {
            let url = format!("{}/job/{}/run", server, job_id);
            let response = reqwest::get(url).await.unwrap().text().await.unwrap();
            let execution_id: i64 = serde_json::from_str(&response).unwrap();
            println!("Created execution with id {}.", execution_id);
        }
        Commands::JobsDescribe { job_id } => {
            let url = format!("{}/job/{}", server, job_id);
            let body = reqwest::get(&url).await.unwrap().text().await.unwrap();
            let osprei::JobPointer {
                id,
                name,
                source,
                path,
            } = serde_json::from_str(&body).unwrap();
            println!("{}", name);
            println!("Id: {}", id);
            println!("Source: {}", source);
            println!("Path: {}", path);
            let url = format!("{}/job/{}/executions", server, job_id);
            let body = reqwest::get(&url).await.unwrap().text().await.unwrap();
            let executions: Vec<osprei::ExecutionSummary> = serde_json::from_str(&body).unwrap();
            if let Some(execution) = executions.first() {
                let url = format!("{}/execution/{}", server, execution.id);
                let body = reqwest::get(&url).await.unwrap().text().await.unwrap();
                let execution: osprei::ExecutionDetails = serde_json::from_str(&body).unwrap();
                let status = match execution.status {
                    None => "Running",
                    Some(0) => "Success",
                    _ => "Failure",
                };
                println!("Last execution: {} UTC ({})", execution.start_time, status);
            } else {
                println!("Not executed yet.");
            }
        }
        Commands::JobsSchedule(JobScheduleArgs { id, hour, minute }) => {
            let url = format!("{}/job/{}/schedule", server, id);
            let req = osprei::ScheduleRequest { hour, minute };
            let response = Client::new()
                .post(&url)
                .body(serde_json::to_string(&req).unwrap())
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            let schedule_id: i64 = serde_json::from_str(&response).unwrap();
            println!("Created schedule with id {}.", schedule_id);
        }
    }
}
