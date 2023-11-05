use crate::server::*;
use crate::widget::ExecutionTable;
use crate::widget::JobTable;
use leptos::*;
use leptos_router::*;

#[component]
pub fn home() -> impl IntoView {
    let add_job = create_server_action::<AddJob>();
    let execute_job = create_server_action::<ExecuteJob>();
    let jobs = create_resource(
        move || (add_job.version().get(), execute_job.version().get()),
        |_| async { load_jobs().await },
    );
    let executions = create_resource(
        move || execute_job.version().get(),
        |_| async { load_executions().await },
    );

    view! {
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            <div>
                <h2>"Jobs"</h2>
                {move || {
                    jobs.get()
                        .map(|jobs| {
                            jobs.map(|jobs| {
                                view! { <JobTable jobs action=execute_job/> }
                            })
                        })
                }}
                <ActionForm class="add-job-form" action=add_job>
                    <label>"Source" <input type="text" name="source"/></label>
                    <input type="submit" value="Add"/>
                </ActionForm>
            </div>
            <div>
                <h2>"Executions"</h2>
                {move || {
                    executions
                        .get()
                        .map(|executions| {
                            executions.map(|executions| view! { <ExecutionTable executions/> })
                        })
                }}

            </div>
        </Suspense>
    }
}
