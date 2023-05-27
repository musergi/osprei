use leptos::{ev::SubmitEvent, html::Input, Resource, *};
use log::info;

pub fn osprei_gui(cx: Scope) -> impl IntoView {
    let (is_adding, set_is_adding) = create_signal(cx, false);
    let jobs = create_resource(cx, || (), load_jobs);
    let (req, set_req) = create_signal(cx, None);
    let job_creator = create_resource(cx, req, move |req| create_job(req, jobs, set_req));

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
                let req = osprei::JobCreationRequest { name, source, path };
                set_req.set(Some(req));
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
            .map(
                |osprei::JobPointer { name, .. }: osprei::JobPointer| view! {cx, <li>{ name }</li>},
            )
            .collect_view(cx)
    };

    view! { cx,
        <ul>
            <li>"Job 1"</li>
            <li>"Job 2"</li>
            { jobs }
            { last }
        </ul>
    }
}

async fn load_jobs(_: ()) -> Vec<osprei::JobPointer> {
    let url = "http://localhost:10000/job";
    let job_ids: Vec<i64> = request(url).await.unwrap_or_default();
    let mut jobs: Vec<osprei::JobPointer> = Vec::new();
    for job_id in job_ids {
        let url = format!("http://localhost:10000/job/{}", job_id);
        let job = request(&url).await.unwrap();
        jobs.push(job);
    }
    for job in jobs.iter() {
        info!("Found job: {}", job.name);
    }
    info!("Loaded all jobs");
    jobs
}

async fn create_job(
    req: Option<osprei::JobCreationRequest>,
    jobs: Resource<(), Vec<osprei::JobPointer>>,
    set_req: WriteSignal<Option<osprei::JobCreationRequest>>,
) -> Result<(), String> {
    match req {
        Some(job_req) => {
            set_req.set(None);
            let url = "http://localhost:10000/job";
            let job_id: i64 = reqwasm::http::Request::post(url)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&job_req).unwrap())
                .send()
                .await
                .map_err(|err| format!("Request error: {}", err))?
                .json()
                .await
                .map_err(|err| format!("Deserialization error: {}", err))?;
            info!("Created job with id: {}", job_id);
            jobs.refetch();
            Ok(())
        }
        None => Ok(()),
    }
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
