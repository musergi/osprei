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

#[server]
async fn load_job_list() -> Result<Vec<i64>, ServerFnError> {
    let jobs = osprei_storage::job_ids().await?;
    Ok(jobs)
}

#[server(AddJob)]
pub async fn add_job(source: String) -> Result<(), ServerFnError> {
    osprei_storage::job_create(source).await?;
    Ok(())
}

#[server(ExecuteJob)]
pub async fn execute_job(job_id: i64) -> Result<(), ServerFnError> {
    log::info!("Running job with id {}", job_id);
    let source = osprei_storage::job_source(job_id).await?;
    let execution_id = osprei_storage::execution_create(job_id).await?;
    tokio::spawn(async move {
        let success = osprei_execution::execute(source).await;
        match success {
            true => {
                let _ = osprei_storage::execution_success(execution_id).await;
            }
            _ => {
                let _ = osprei_storage::execution_failure(execution_id).await;
            }
        }
    });
    Ok(())
}

#[server]
async fn load_job(id: i64) -> Result<String, ServerFnError> {
    let source = osprei_storage::job_source(id).await?;
    Ok(source)
}
