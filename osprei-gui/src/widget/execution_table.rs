use leptos::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Execution {
    pub id: i64,
    pub status: String,
    pub duration: Option<i64>,
}

#[component]
pub fn execution_table(executions: Vec<Execution>) -> impl IntoView {
    let rows = executions
        .into_iter()
        .map(|execution| view! { <Row execution/> })
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
            <th>"Status"</th>
            <th>"Duration"</th>
        </tr>
    }
}

#[component]
fn row(execution: Execution) -> impl IntoView {
    let Execution {
        id,
        status,
        duration,
    } = execution;
    let duration_string = duration
        .map(|duration| format!("{duration} secs"))
        .unwrap_or_default();
    view! {
        <tr>
            <td>{id}</td>
            <td>{status}</td>
            <td>{duration_string}</td>
        </tr>
    }
}

