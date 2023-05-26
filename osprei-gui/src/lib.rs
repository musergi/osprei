use leptos::{ev::SubmitEvent, html::Input, *};
use log::info;

pub fn osprei_gui(cx: Scope) -> impl IntoView {
    let (is_adding, set_is_adding) = create_signal(cx, false);
    let jobs = create_resource(cx, || (), load_jobs);

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
                info!("Submitted {}, {}, {}", name, source, path);
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

    view! { cx,
        <ul>
            <li>"Job 1"</li>
            <li>"Job 2"</li>
            { last }
        </ul>
    }
}

async fn load_jobs(_: ()) -> Vec<osprei::JobPointer> {
    let url = "http://localhost:10000/job";
    let job_ids: Vec<i64> = reqwasm::http::Request::get(url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let mut jobs = Vec::new();
    for job_id in job_ids {
        let url = format!("http://localhost:10000/job/{}", job_id);
        let job = reqwasm::http::Request::get(&url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        jobs.push(job);
    }
    jobs
}
