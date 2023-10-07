use crate::leptos_dom::logging::console_log;
use leptos::*;

const JOB_POLLING_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

fn main() {
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}

#[component]
fn App() -> impl IntoView {
    let jobs = create_resource(|| (), |_| async move { load_jobs().await });
    set_interval(
        move || {
            console_log("Refetching jobs");
            jobs.refetch();
        },
        JOB_POLLING_INTERVAL,
    );
    view! {
        {move || match jobs.get() {
            Some(jobs) => view! { <JobTable jobs/> }.into_view(),
            None => view! { <p>{ "Loading..." }</p> }.into_view()
        }}
    }
}

#[component]
fn JobTable(jobs: Vec<osprei::JobOverview>) -> impl IntoView {
    view! {
        <table class="JobTable">
            <tr>
                <th>{ "Name" }</th>
                <th>{ "Start Time" }</th>
                <th>{ "Status" }</th>
                <th>{ "Actions" }</th>
            </tr>
            {move || jobs.iter()
                .cloned()
                .map(|job| view! { <JobRow job/> })
                .collect_view()}
        </table>
    }
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct JobData {
    start_time: String,
    status: ExecutionStatus,
}

impl From<Option<osprei::LastExecution>> for JobData {
    fn from(value: Option<osprei::LastExecution>) -> Self {
        let status = value
            .as_ref()
            .map(|value| value.status)
            .clone()
            .map(|value| ExecutionStatus::from(value))
            .unwrap_or_default();
        let start_time = value.map(|value| value.start_time).unwrap_or_default();
        JobData { start_time, status }
    }
}

impl From<osprei::ExecutionDetails> for JobData {
    fn from(value: osprei::ExecutionDetails) -> Self {
        let osprei::ExecutionDetails {
            start_time, status, ..
        } = value;
        let status = ExecutionStatus::from(status);
        JobData { start_time, status }
    }
}

#[component]
fn JobRow(job: osprei::JobOverview) -> impl IntoView {
    let osprei::JobOverview {
        id,
        name,
        last_execution,
    } = job;
    let job_data = JobData::from(last_execution);

    let color = if job_data.status == ExecutionStatus::Success {
        "green"
    } else {
        "false"
    };

    let run = create_action(|id| {
        let id = *id;
        async move {
            run_job(id).await;
        }
    });

    view! {
        <tr>
            <td>{ name }</td>
            <td>{ job_data.start_time.clone() }</td>
            <td class={ color }>{ job_data.status.to_string() }</td>
            <td class={ "run-button" } on:click=move |e| {
                e.prevent_default();
                run.dispatch(id);
            }>
                { "Run" }
            </td>
        </tr>
    }
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum ExecutionStatus {
    Success,
    Failed,
    Error,
    Running,
    NotExecuted,
}

impl From<Option<osprei::ExecutionStatus>> for ExecutionStatus {
    fn from(value: Option<osprei::ExecutionStatus>) -> Self {
        value
            .map(|value| match value {
                osprei::ExecutionStatus::Success => ExecutionStatus::Success,
                osprei::ExecutionStatus::Failed => ExecutionStatus::Failed,
                osprei::ExecutionStatus::InvalidConfig => ExecutionStatus::Error,
            })
            .unwrap_or(ExecutionStatus::Running)
    }
}

impl Default for ExecutionStatus {
    fn default() -> Self {
        ExecutionStatus::NotExecuted
    }
}

impl ToString for ExecutionStatus {
    fn to_string(&self) -> String {
        match self {
            ExecutionStatus::Success => "Success".to_string(),
            ExecutionStatus::Failed => "Failed".to_string(),
            ExecutionStatus::Error => "Error".to_string(),
            ExecutionStatus::Running => "Running".to_string(),
            ExecutionStatus::NotExecuted => "Not executed".to_string(),
        }
    }
}

async fn load_jobs() -> Vec<osprei::JobOverview> {
    let body = reqwest::get("http://localhost:8081/job")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    serde_json::from_str(&body).unwrap()
}

async fn run_job(id: i64) -> i64 {
    let body = reqwest::get(format!("http://localhost:8081/job/{}/run", id))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    serde_json::from_str(&body).unwrap()
}
