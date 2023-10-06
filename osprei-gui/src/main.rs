use leptos::*;

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

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum JobRowState {
    ToRun,
    Loaded(JobData),
}

impl From<Option<osprei::LastExecution>> for JobRowState {
    fn from(value: Option<osprei::LastExecution>) -> Self {
        JobRowState::from(JobData::from(value))
    }
}

impl From<JobData> for JobRowState {
    fn from(value: JobData) -> Self {
        JobRowState::Loaded(value)
    }
}

#[component]
fn JobRow(job: osprei::JobOverview) -> impl IntoView {
    let osprei::JobOverview {
        id,
        name,
        last_execution,
    } = job;
    let (job_row_state, set_job_row_state) = create_signal(JobRowState::from(last_execution));
    let job_data = create_resource(
        move || job_row_state.get(),
        move |state| async move {
            match state {
                JobRowState::ToRun => {
                    let execution_id = run_job(id).await;
                    let execution = load_execution(execution_id).await;
                    let job_data = JobData::from(execution);
                    set_job_row_state.set(JobRowState::from(job_data.clone()));
                    job_data
                }
                JobRowState::Loaded(loaded) => loaded,
            }
        },
    );

    let start_time = move || {
        job_data
            .map(|job| job.start_time.to_string())
            .unwrap_or("Loading".to_string())
    };

    let color = move || {
        if job_data
            .map(|job| job.status == ExecutionStatus::Success)
            .unwrap_or(false)
        {
            "green"
        } else {
            "false"
        }
    };

    let status = move || {
        job_data
            .map(|job| job.status.to_string())
            .unwrap_or("Loading".to_string())
    };

    let run = move |_| {
        set_job_row_state.set(JobRowState::ToRun);
    };

    view! {
        <tr>
            <td>{ name }</td>
            <td>{ start_time }</td>
            <td class={ color }>{ status }</td>
            <td class={ "run-button" } on:click=run>{ "Run" }</td>
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

async fn load_execution(id: i64) -> osprei::ExecutionDetails {
    let body = reqwest::get(format!("http://localhost:8081/execution/{}", id))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    serde_json::from_str(&body).unwrap()
}
