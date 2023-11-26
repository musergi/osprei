use crate::{server::*, widget::*};
use leptos::*;
use leptos_router::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Job {
    pub id: i64,
    pub source: String,
    pub status: String,
}

type RunJobAction = Action<ExecuteJob, Result<(), ServerFnError>>;

#[component]
pub fn job_list(jobs: Vec<Job>, action: RunJobAction) -> impl IntoView {
    let jobs = jobs
        .into_iter()
        .map(|job| view! {<JobCard job action/>})
        .collect_view();
    view! {
        <div>{jobs}</div>
    }
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[component]
pub fn job_card(job: Job, action: RunJobAction) -> impl IntoView {
    let Job { id, source, status } = job;
    let title = source
        .split_once("://")
        .unwrap()
        .1
        .split("/")
        .last()
        .unwrap()
        .split_once(".")
        .unwrap()
        .0;
    let title = capitalize(title);
    let link = format!("/job/{id}");
    view! {
        <Card title>
            <p>{status}</p>
            <p>"1h ago"</p>
            <ActionButtons>
                <LinkButton button_type=ButtonType::Secondary link text="Details"/>
                <FormButton text="Run" action>
                    <input type="text" hidden=true name="job_id" value={id}/>
                </FormButton>
            </ActionButtons>
        </Card>
    }
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
