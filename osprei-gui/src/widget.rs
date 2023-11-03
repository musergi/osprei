mod job_table;
pub use job_table::Job;
pub use job_table::JobTable;

mod execution_table;
pub use execution_table::Execution;
pub use execution_table::ExecutionTable;

use leptos::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Stage {
    pub id: i64,
    pub dependency: Option<i64>,
    pub description: String,
}

#[component]
pub fn stages(stages: Vec<Stage>) -> impl IntoView {
    let arena = Arena::new(stages);
    view! {
        <div style="text-align: left">
            <ul>
            <Node arena idx=0/>
            </ul>
        </div>
    }
}

#[component]
fn node(arena: Arena, idx: usize) -> impl IntoView {
    let Node { children, name, .. } = arena.nodes.get(idx).unwrap();
    let children = children
        .into_iter()
        .map(|&child| {
            let arena = arena.clone();
            view! {<Node arena idx=child/>}
        })
        .collect_view();
    view! { <li>{name} <ul>{children}</ul></li>}
}

#[derive(Clone)]
struct Arena {
    nodes: Vec<Node>,
}

impl Arena {
    fn new(stages: Vec<Stage>) -> Arena {
        let mut nodes: Vec<_> = stages
            .iter()
            .map(|stage| Node {
                children: Vec::new(),
                stage: stage.id,
                name: stage.description.clone(),
            })
            .collect();
        for (idx, stage) in stages.iter().enumerate() {
            if let Some(dependency) = stage.dependency {
                let src = nodes.iter().position(|n| n.stage == dependency).unwrap();
                nodes.get_mut(src).unwrap().children.push(idx);
            }
        }
        Arena { nodes }
    }
}

#[derive(Clone)]
struct Node {
    children: Vec<usize>,
    stage: i64,
    name: String,
}
