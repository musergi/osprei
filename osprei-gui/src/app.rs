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
                <h1>"Jobs"</h1>
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
    let job_list = create_resource(
        move || add_job.version().get(),
        |_| async { load_job_list().await },
    );
    view! {
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            {move || job_list
                .get()
                .map(|job_ids| job_ids
                    .map(|job_ids| view!{<JobTable job_ids/>})
                )
            }
        </Suspense>
        <ActionForm action=add_job>
            <label>
                "Source"
                // `title` matches the `title` argument to `add_todo`
                <input type="text" name="source"/>
            </label>
            <input type="submit" value="Add"/>
        </ActionForm>
    }
}

#[component]
fn JobTable(job_ids: Vec<i64>) -> impl IntoView {
    let jobs = job_ids
        .into_iter()
        .map(|id| view! {<JobRow id/>})
        .collect_view();
    view! {
        <table class="job-table">
            <tr>
                <th>"Id"</th>
                <th>"Source"</th>
            </tr>
            { jobs }
        </table>
    }
}

#[component]
fn JobRow(id: i64) -> impl IntoView {
    let source = create_resource(|| (), move |_| async move { load_job(id).await });
    view! {
        <Suspense fallback=move || view! { <> }>
            <tr>
                <td>{ id }</td>
                <td>{ source.get() }</td>
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
    let mut conn = db().await?;
    sqlx::query!("INSERT INTO jobs (source) VALUES ($1)", source)
        .execute(&mut conn)
        .await?;
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
