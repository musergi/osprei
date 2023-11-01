use crate::server::*;
use leptos::*;
use leptos_router::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Job {
    pub id: i64,
    pub source: String,
    pub status: String,
}

#[component]
pub fn job_table(
    jobs: Vec<Job>,
    action: Action<ExecuteJob, Result<(), ServerFnError>>,
) -> impl IntoView {
    let rows = jobs
        .into_iter()
        .map(|job| view! { <Row job action/> })
        .collect_view();
    view! {
        <table class="job-table">
            <Header/>
            {rows}
        </table>
    }
}

#[component]
fn header() -> impl IntoView {
    view! {
        <tr>
            <th>"Id"</th>
            <th>"Source"</th>
            <th>"Status"</th>
            <th>"Action"</th>
        </tr>
    }
}

#[component]
fn row(job: Job, action: Action<ExecuteJob, Result<(), ServerFnError>>) -> impl IntoView {
    let Job { id, source, status } = job;
    view! {
        <td>{id}</td>
        <td>{source}</td>
        <td>{status}</td>
        <td>
            <RunButton id action/>
        </td>
    }
}

#[component]
fn run_button(id: i64, action: Action<ExecuteJob, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action>
            <input type="text" value=id hidden=true name="job_id"/>
            <input type="submit" value="Run"/>
        </ActionForm>
    }
}

#[component]
pub fn ExecutionTable(execution_ids: Vec<i64>) -> impl IntoView {
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
            {rows}
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
        <Suspense fallback=move || view! { <></> }>
            <tr>
                <td>{id}</td>
                <td>{move || status.get()}</td>
                <td>{duration_string}</td>
            </tr>
        </Suspense>
    }
}





