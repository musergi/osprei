use crate::server::AddStage;
use leptos::*;
use leptos_router::*;

#[component]
pub fn stage_form(
    dependency: Option<i64>,
    action: Action<AddStage, Result<(), ServerFnError>>,
) -> impl IntoView {
    match dependency {
        None => view! { <p>"Press a stage to add a new one depending on it"</p> }.into_view(),
        Some(dependency) => view! { <DependencyStageForm dependency action/> }.into_view(),
    }
}

#[component]
fn dependency_stage_form(
    dependency: i64,
    action: Action<AddStage, Result<(), ServerFnError>>,
) -> impl IntoView {
    view! {
        <ActionForm class="add-stage-form" action>
            <input name="job_id" type="number" value=1 hidden=true/>
            <label>"Depends on" <input type="number" name="dependency" value={dependency} readonly/></label>
            <label>"Name" <input type="text" name="name"/></label>
            <label>"Template" <input type="text" name="template"/></label>
            <input type="submit" value="Add"/>
        </ActionForm>
    }
}
