use crate::server::*;
use leptos_router::*;
use leptos::*;

#[component]
pub fn JobTable(
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
