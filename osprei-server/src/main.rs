use log::error;

use clap::Parser;
use log::debug;
use osprei_server::config::Config;
use osprei_server::routes::routes;
use osprei_server::{execute, persistance};
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
    let config = Config::read(&args.config_path);
    match config {
        Ok(Config {
            address,
            data_path,
            persistance,
        }) => {
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
                execute::schedule_job(job, hour, minute, path_builder.clone(), persistance.boxed())
                    .await;
            }
            warp::serve(
                warp::any().and(routes(path_builder, persistance)).with(
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
            .run(address)
            .await;
        }
        Err(err) => error!("Config error: {}", err),
    };
}

async fn build_workspace(workspace_dir: &str) {
    tokio::fs::create_dir_all(workspace_dir).await.unwrap();
}
