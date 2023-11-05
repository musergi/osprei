use crate::server::*;
use crate::widget::StageForm;
use crate::widget::Stages;
use leptos::*;
use leptos_router::*;

#[component]
pub fn job() -> impl IntoView {
    let params = use_params_map();
    let job_id = move || params.with(|p| p.get("id").cloned().unwrap_or_default());

    let (dependency, set_dependency) = create_signal(None::<i64>);

    let add_stage = create_server_action::<AddStage>();

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
                {move || {
                    stages
                        .get()
                        .map(|stages| {
                            stages
                                .map(|stages| {
                                    view! { <Stages stages set_as_parent=set_dependency/> }
                                })
                        })
                }}
                {move || view! { <StageForm dependency=dependency.get() action=add_stage/> }}
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
