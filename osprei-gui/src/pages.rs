use crate::server::*;
use crate::widget::ExecutionTable;
use crate::widget::JobTable;
use crate::widget::Stages;
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

#[component]
pub fn job() -> impl IntoView {
    let params = use_params_map();
    let job_id = move || params.with(|p| p.get("id").cloned().unwrap_or_default());
    let source = create_resource(job_id, |id| async move {
        load_job_source(id.parse().unwrap()).await
    });
    let status = create_resource(job_id, |id| async move {
        load_job_status(id.parse().unwrap()).await
    });
    let stages = create_resource(job_id, |id| async move {
        load_stages(id.parse().unwrap()).await
    });
    view! {
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| view! { <ErrorTemplate errors/> }>
                <p>{move || source.get()}</p>
                <p>{move || status.get()}</p>
                {move || stages.get().map(|stages| stages.map(|stages| view! { <Stages stages/> }))}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn error_template(#[prop(optional)] errors: Option<RwSignal<Errors>>) -> impl IntoView {
    match errors {
        Some(errors) => {
            let errors = move || {
                errors
                    .get()
                    .into_iter()
                    .map(|error| view! { <li>{error.1.to_string()}</li> })
                    .collect_view()
            };
            view! { <ul>{errors}</ul> }.into_view()
        }
        None => view! { <p>"No errors"</p> }.into_view(),
    }
}
