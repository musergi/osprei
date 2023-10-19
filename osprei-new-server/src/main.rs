use axum::{
    routing::{get, post},
    Router,
};
use handler::get_jobs;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use osprei_app::*;

mod handler;

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let gui = Router::new()
        .route("/gui/*fn_name", post(leptos_axum::handle_server_fns))
        .leptos_routes(&leptos_options, routes, App)
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    let url = std::env::var("DATABASE_URL").unwrap();
    let pool = sqlx::SqlitePool::connect(&url).await.unwrap();
    let api = Router::new()
        .route(
            "/job",
            get({
                let pool = pool.clone();
                move || get_jobs(pool)
            })
            .post({
                let pool = pool.clone();
                move |job| post_job(job, pool)
            }),
        )
        .route(
            "/job/:id",
            get({
                let pool = pool.clone();
                move |job_id| get_job(job_id, pool)
            }),
        )
        .route(
            "/job/:id/run",
            post({
                let pool = pool.clone();
                move |job_id| post_job_run(job_id, pool)
            }),
        );

    log::info!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
        .serve(gui.nest("/api", api).into_make_service())
        .await
        .unwrap();
}

use axum::response::Response as AxumResponse;
use axum::{
    body::{boxed, Body, BoxBody},
    extract::State,
    http::{Request, Response, StatusCode, Uri},
    response::IntoResponse,
};
use osprei_app::error_template::AppError;
use osprei_app::error_template::ErrorTemplate;
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::handler::{get_job, post_job, post_job_run};

pub async fn file_and_error_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> AxumResponse {
    let root = options.site_root.clone();
    let res = get_static_file(uri.clone(), &root).await.unwrap();

    if res.status() == StatusCode::OK {
        res.into_response()
    } else {
        let mut errors = Errors::default();
        errors.insert_with_default_key(AppError::NotFound);
        let handler = leptos_axum::render_app_to_stream(
            options.to_owned(),
            move || view! { <ErrorTemplate outside_errors=errors.clone()/> },
        );
        handler(req).await.into_response()
    }
}

async fn get_static_file(uri: Uri, root: &str) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // This path is relative to the cargo root
    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {err}"),
        )),
    }
}
