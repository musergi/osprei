use log::{error, warn};

use clap::Parser;
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
            let (persistance_channel, persistance) =
                persistance::Persistance::new(persistance).await.unwrap();
            tokio::spawn(async move {
                persistance.serve().await;
            });
            build_workspace(path_builder.workspaces()).await;
            let (engine_channel, engine) = execute::Server::new(
                path_builder.workspaces().to_string(),
                persistance_channel.clone(),
            )
            .await;
            tokio::spawn(async move {
                engine.serve().await;
            });
            warp::serve(
                warp::any()
                    .and(routes(persistance_channel, engine_channel))
                    .with(cors())
                    .with(warp::log("api")),
            )
            .run(address)
            .await;
        }
        Err(err) => error!("Config error: {}", err),
    };
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
