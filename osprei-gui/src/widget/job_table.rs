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
            <DetailsButton id/>
        </td>
    }
}

#[component]
fn run_button(id: i64, action: Action<ExecuteJob, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action>
            <input type="text" value=id hidden=true name="job_id"/>
            <input class="run-button" type="submit" value="Run"/>
        </ActionForm>
    }
}

#[component]
fn details_button(id: i64) -> impl IntoView {
    view! {
        <A class="details-button" href=format!("/job/{id}")>
            "Details"
        </A>
    }
}
