use crate::server::*;
use leptos::*;
use leptos_router::*;
use crate::widget::JobTable;
use crate::widget::ExecutionTable;

#[component]
pub fn HomePage() -> impl IntoView {
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
pub fn job() -> impl IntoView {
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
