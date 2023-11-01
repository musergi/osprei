use crate::error_template::{AppError, ErrorTemplate};
use crate::server::*;
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
                    <Route path="" view=HomePage />
                    <Route path="/job/:id" view=Job />
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
    let execution_list = create_resource(
        move || execute_job.version().get(),
        |_| async { load_execution_list().await },
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
                <ActionForm class="add-job-form" action=add_job>
                    <label>
                        "Source"
                        <input type="text" name="source"/>
                    </label>
                    <input type="submit" value="Add"/>
                </ActionForm>
            </div>
            <div>
                <h2>"Executions"</h2>
                {move || execution_list
                    .get()
                    .map(|execution_ids| execution_ids
                        .map(|execution_ids| view!(<ExecutionTable execution_ids/>))
                    )
                }
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
                <th>"Status"</th>
                <th>"Action"</th>
            </tr>
            { jobs }
        </table>
    }
}

#[component]
fn JobRow(id: i64, execute_job: Action<ExecuteJob, Result<(), ServerFnError>>) -> impl IntoView {
    let source = create_resource(|| (), move |_| async move { load_job_source(id).await });
    let status = create_resource(|| (), move |_| async move { load_job_status(id).await });
    view! {
        <Suspense fallback=move || view! { <> }>
            <tr>
                <td>{ id }</td>
                <td>{ move || source.get() }</td>
                <td>{ move || status.get() }</td>
                <td>
                    <ActionForm action=execute_job>
                        <input type="text" value={ id } hidden={ true } name="job_id"/>
                        <input type="submit" value="Run"/>
                    </ActionForm>
                    <A href=format!("/job/{id}")>"Details"</A>
                </td>
            </tr>
        </Suspense>
    }
}

#[component]
fn ExecutionTable(execution_ids: Vec<i64>) -> impl IntoView {
    let rows = execution_ids
        .into_iter()
        .map(|id| view! { <ExecutionRow id/> })
        .collect_view();
    view! {
        <table class="job-table">
            <tr>
                <th>"Id"</th>
                <th>"Status"</th>
                <th>"Duration"</th>
            </tr>
            { rows }
        </table>
    }
}

#[component]
fn ExecutionRow(id: i64) -> impl IntoView {
    let status = create_resource(
        || (),
        move |_| async move { load_execution_status(id).await },
    );
    let duration = create_resource(
        || (),
        move |_| async move { load_execution_duration(id).await },
    );
    let duration_string = move || {
        duration.get().map(|duration| {
            duration.map(|duration| {
                duration
                    .map(|v| format!("{v} secs"))
                    .unwrap_or("-".to_string())
            })
        })
    };
    view! {
        <Suspense fallback=move || view! { <> }>
            <tr>
                <td>{ id }</td>
                <td>{ move || status.get() }</td>
                <td>{ duration_string }</td>
            </tr>
        </Suspense>
    }
}

#[component]
fn job() -> impl IntoView {
    let params = use_params_map();
    let source = create_resource(
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        |id| async move { load_job_source(id.parse().unwrap()).await },
    );
    view! {
        <p>
            {move || source.get()}
        </p>
    }
}
