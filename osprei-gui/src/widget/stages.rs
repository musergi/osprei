use leptos::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Stage {
    pub id: i64,
    pub dependency: Option<i64>,
    pub description: String,
}

#[component]
pub fn stages(stages: Vec<Stage>, set_as_parent: WriteSignal<Option<i64>>) -> impl IntoView {
    let root = StageWithChildren::new(&stages);
    view! {
        <div style="text-align: left">
            <ul>
                <Node node=root set_as_parent/>
            </ul>
        </div>
    }
}

#[component]
fn node(node: StageWithChildren, set_as_parent: WriteSignal<Option<i64>>) -> impl IntoView {
    let StageWithChildren { id, name, children } = node;
    let children = children
        .into_iter()
        .map(|node| {
            view! { <Node node set_as_parent/> }
        })
        .collect_view();
    let set = move |_| {
        set_as_parent.set(Some(id));
    };
    view! {
        <li class="stage">
            <strong>"(" {id} ")"</strong>
            <span>{name}</span>
            <button class="add-stage-button" on:click=set>
                "Add"
            </button>
            <ul>{children}</ul>
        </li>
    }
}

struct StageWithChildren {
    id: i64,
    name: String,
    children: Vec<StageWithChildren>,
}

impl StageWithChildren {
    fn new(stages: &[Stage]) -> StageWithChildren {
        StageWithChildren::new_recursive(stages, 0)
    }

    fn new_recursive(stages: &[Stage], idx: usize) -> StageWithChildren {
        let Stage {
            id, description, ..
        } = stages.get(idx).expect("invalid reference");
        let children = stages
            .iter()
            .enumerate()
            .filter(|(_, stage)| stage.dependency.map(|dep| dep == *id).unwrap_or(false))
            .map(|(idx, _)| StageWithChildren::new_recursive(stages, idx))
            .collect();
        StageWithChildren {
            id: *id,
            name: description.to_string(),
            children,
        }
    }
}
