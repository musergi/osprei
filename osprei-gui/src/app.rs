use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/osprei-gui.css"/>
        <Title text="Welcome to Leptos"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <header>
                <h1>"Osprei"</h1>
            </header>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let add_job = create_server_action::<AddJob>();
    let execute_job = create_server_action::<ExecuteJob>();
    let job_list = create_resource(
        move || add_job.version().get(),
        |_| async { load_job_list().await },
    );
    view! {
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            <div>
                <h2>"Jobs"</h2>
                {move || job_list
                    .get()
                    .map(|job_ids| job_ids
                        .map(|job_ids| view!{<JobTable job_ids execute_job/>})
                    )
                }
                <ActionForm action=add_job>
                    <label>
                        "Source"
                        <input type="text" name="source"/>
                    </label>
                    <input type="submit" value="Add"/>
                </ActionForm>
            </div>
            <div>
                <h2>"Executions"</h2>
            </div>
        </Suspense>

    }
}

#[component]
fn JobTable(
    job_ids: Vec<i64>,
    execute_job: Action<ExecuteJob, Result<(), ServerFnError>>,
) -> impl IntoView {
    let jobs = job_ids
        .into_iter()
        .map(|id| view! {<JobRow id execute_job/>})
        .collect_view();
    view! {
        <table class="job-table">
            <tr>
                <th>"Id"</th>
                <th>"Source"</th>
                <th>"Action"</th>
            </tr>
            { jobs }
        </table>
    }
}

#[component]
fn JobRow(id: i64, execute_job: Action<ExecuteJob, Result<(), ServerFnError>>) -> impl IntoView {
    let source = create_resource(|| (), move |_| async move { load_job(id).await });
    view! {
        <Suspense fallback=move || view! { <> }>
            <tr>
                <td>{ id }</td>
                <td>{ source.get() }</td>
                <td>
                    <ActionForm action=execute_job>
                        <input type="text" value={ id } hidden={ true } name="job_id"/>
                        <input type="submit" value="Run"/>
                    </ActionForm>
                </td>
            </tr>
        </Suspense>
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Connection, SqliteConnection};

        pub async fn db() -> Result<SqliteConnection, ServerFnError> {
            let url = std::env::var("DATABASE_URL").unwrap();
            Ok(SqliteConnection::connect(&url).await?)
        }
    }
}

struct JobId {
    id: i64,
}

#[server(LoadJobList, "/api")]
async fn load_job_list() -> Result<Vec<i64>, ServerFnError> {
    log::info!("Getting database");
    let mut conn = db().await?;
    log::info!("Loading jobs");
    let jobs = sqlx::query_as!(
        JobId,
        "
        SELECT id
        FROM jobs
        "
    )
    .fetch_all(&mut conn)
    .await?
    .into_iter()
    .map(|job| job.id)
    .collect();
    Ok(jobs)
}

#[server(AddJob, "/api")]
pub async fn add_job(source: String) -> Result<(), ServerFnError> {
    log::info!("Adding job with source {}", source);
    let mut conn = db().await?;
    sqlx::query!("INSERT INTO jobs (source) VALUES ($1)", source)
        .execute(&mut conn)
        .await?;
    log::info!("Added");
    Ok(())
}

#[server(ExecuteJob, "/api")]
pub async fn execute_job(job_id: i64) -> Result<(), ServerFnError> {
    log::info!("Running job with id {}", job_id);
    let source = load_job(job_id).await?;
    tokio::spawn(async move {
        log::info!("Connecting to docker server");
        let docker = docker_api::Docker::new("unix:///var/run/docker.sock")
            .map_err(|err| log::error!("Failed to connect to docker: {}", err))
            .unwrap();
        log::info!("Connected");
        let volume = docker
            .volumes()
            .create(&Default::default())
            .await
            .map_err(|err| log::error!("Failed to create docker volume: {}", err))
            .unwrap();
        let volume = docker_api::Volume::new(docker.clone(), volume.name);
        log::info!("Created volume: {}", volume.name());
        let opts = docker_api::opts::ContainerCreateOpts::builder()
            .image("rust:latest")
            .volumes(vec![format!("{}:/workspaces", volume.name())])
            .working_dir("/workspaces")
            .command(vec!["git", "clone", &source, "code"])
            .build();
        let container = docker
            .containers()
            .create(&opts)
            .await
            .map_err(|err| log::error!("Failed to create container: {}", err))
            .unwrap();
        log::info!("Created container: {}", container.id());
        container
            .start()
            .await
            .map_err(|err| log::error!("Failed to start container: {}", err))
            .unwrap();
        container
            .wait()
            .await
            .map_err(|err| log::error!("Failed to wait container: {}", err))
            .unwrap();
        container
            .delete()
            .await
            .map_err(|err| log::error!("Failed to delete container: {}", err))
            .unwrap();
        log::info!("Deleted container: {}", container.id());
        let opts = docker_api::opts::ContainerCreateOpts::builder()
            .image("rust:latest")
            .volumes(vec![format!("{}:/workspaces", volume.name())])
            .working_dir("/workspaces/code")
            .command(vec!["cargo", "test"])
            .build();
        let container = docker
            .containers()
            .create(&opts)
            .await
            .map_err(|err| log::error!("Failed to create container: {}", err))
            .unwrap();
        log::info!("Created container: {}", container.id());
        container
            .start()
            .await
            .map_err(|err| log::error!("Failed to start container: {}", err))
            .unwrap();
        container
            .wait()
            .await
            .map_err(|err| log::error!("Failed to wait container: {}", err))
            .unwrap();
        container
            .delete()
            .await
            .map_err(|err| log::error!("Failed to delete container: {}", err))
            .unwrap();
        log::info!("Deleted container: {}", container.id());
        volume
            .delete()
            .await
            .map_err(|err| log::error!("Failed to delete volume: {}", err))
            .unwrap();
        log::info!("Deleted volume: {}", volume.name());
    });
    Ok(())
}

struct JobSource {
    source: String,
}

#[server]
async fn load_job(id: i64) -> Result<String, ServerFnError> {
    log::info!("Getting database");
    let mut conn = db().await?;
    log::info!("Loading job {}", id);
    let job = sqlx::query_as!(
        JobSource,
        "
        SELECT source
        FROM jobs
        WHERE id = $1
        ",
        id
    )
    .fetch_one(&mut conn)
    .await?;
    Ok(job.source)
}
