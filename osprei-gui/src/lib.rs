use leptos::{ev::SubmitEvent, html::Input, *};
use log::info;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
enum Action {
    Run(i64),
    Add(osprei::JobCreationRequest),
}

#[derive(Debug, Default, Clone, PartialEq)]
struct ActionQueue {
    actions: VecDeque<(i64, Action)>,
}

impl ActionQueue {
    fn first(&self) -> Option<i64> {
        self.actions.front().map(|v| v.0)
    }

    fn fetch(self, id: i64) -> Option<Action> {
        self.actions.into_iter().find_map(
            |(action_id, action)| {
                if action_id == id {
                    Some(action)
                } else {
                    None
                }
            },
        )
    }

    fn remove(&mut self, id: i64) {
        let position = self
            .actions
            .iter()
            .position(|(action_id, _)| *action_id == id)
            .unwrap();
        self.actions.remove(position);
    }

    fn add(&mut self, action: Action) {
        let id = self
            .actions
            .iter()
            .map(|(idx, _)| idx)
            .max()
            .cloned()
            .unwrap_or_default()
            + 1;
        self.actions.push_back((id, action));
    }
}

pub fn osprei_gui(cx: Scope) -> impl IntoView {
    let (is_adding, set_is_adding) = create_signal(cx, false);
    let jobs = create_resource(cx, || (), load_jobs);
    let (action_queue, set_action_queue) = create_signal(cx, ActionQueue::default());
    let notifications = create_resource(
        cx,
        move || action_queue.get().first(),
        move |id| {
            fetch_notifications(id, action_queue, set_action_queue, move || {
                jobs.refetch();
            })
        },
    );

    let last = move || match is_adding() {
        true => {
            let name_ref: NodeRef<Input> = create_node_ref(cx);
            let source_ref: NodeRef<Input> = create_node_ref(cx);
            let path_ref: NodeRef<Input> = create_node_ref(cx);

            let on_submit = move |ev: SubmitEvent| {
                ev.prevent_default();

                let name = name_ref().unwrap().value();
                let source = source_ref().unwrap().value();
                let path = path_ref().unwrap().value();

                set_is_adding.set(false);
                info!("Submitting {}, {}, {}", name, source, path);
                let req = Action::Add(osprei::JobCreationRequest { name, source, path });
                set_action_queue.update(move |queue| {
                    queue.add(req);
                });
            };

            view! { cx,
                <li>
                    <form on:submit=on_submit>
                        <input type="text" placeholder="name" ref={ name_ref } />
                        <input type="text" placeholder="source" ref={ source_ref } />
                        <input type="text" placeholder="path" ref={ path_ref } />
                        <button>"Add"</button>
                    </form>
                </li>
            }
        }
        false => view! { cx,
            <li>
                <button on:click=move |_| { set_is_adding.set(true) }>"New"</button>
            </li>
        },
    };

    let jobs = move || {
        jobs.read(cx)
            .unwrap_or_default()
            .into_iter()
            .map(|job_id| view! { cx, <Job id={job_id} /> })
            .collect_view(cx)
    };

    view! { cx,
        <>
            <ul>
                { jobs }
                { last }
            </ul>
            <p>
                { move || notifications.read(cx).and_then(|c| c) }
            </p>
        </>
    }
}

async fn load_jobs(_: ()) -> Vec<i64> {
    let url = "http://localhost:10000/job";
    request(url).await.unwrap_or_default()
}

async fn request<T: serde::de::DeserializeOwned>(url: &str) -> Option<T> {
    match reqwasm::http::Request::get(&url).send().await {
        Ok(data) => match data.json().await {
            Ok(deserialized) => Some(deserialized),
            Err(err) => {
                error!("Failed to deserialize response: {}", err);
                None
            }
        },
        Err(err) => {
            error!("Request to server failed: {}: {}", url, err);
            None
        }
    }
}

#[component]
fn job(cx: Scope, id: i64) -> impl IntoView {
    let job = create_resource(cx, move || id, job_fetcher);

    move || match job.read(cx) {
        Some(Ok(osprei::JobPointer {
            name, source, path, ..
        })) => view! { cx,
            <div>
                <h3>{ name }</h3>
                <p>{ format!("{} with {}", source, path) }</p>
                <button>"Run"</button>
            </div>
        },
        _ => view! { cx,
            <div>
                <p>"Loading..."</p>
            </div>
        },
    }
}

async fn job_fetcher(job_id: i64) -> Result<osprei::JobPointer, String> {
    let url = format!("http://localhost:10000/job/{}", job_id);
    let job_pointer = reqwasm::http::Request::get(&url)
        .send()
        .await
        .map_err(|err| format!("Could not load job({}): {}", job_id, err))?
        .json()
        .await
        .map_err(|err| format!("Could not deserialize job({}): {}", job_id, err))?;
    Ok(job_pointer)
}

async fn fetch_notifications(
    id: Option<i64>,
    read: ReadSignal<ActionQueue>,
    write: WriteSignal<ActionQueue>,
    refresh: impl Fn(),
) -> Option<String> {
    match id {
        Some(id) => {
            info!("Running action with id: {}", id);
            match read.get().fetch(id) {
                Some(action) => {
                    info!("Found action: {:?}", action);
                    write.update(|w| {
                        w.remove(id);
                    });
                    refresh();
                    None
                }
                None => Some(format!("Action not found: {}", id)),
            }
        }
        None => {
            info!("Action queue empty, nothing to do...");
            None
        }
    }
}
