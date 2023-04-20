use std::ops::Deref;

use gloo_net::http::Request;
use yew::prelude::*;

mod job_list;
mod missing;

#[function_component]
pub fn App() -> Html {
    let jobs = use_state(|| vec![]);
    {
        let jobs = jobs.clone();
        use_effect_with_deps(
            move |_| {
                let jobs = jobs.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_jobs: Vec<String> = Request::get("http://localhost:10000/job")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    jobs.set(fetched_jobs);
                });
                || ()
            },
            (),
        );
    }
    html! {
        <job_list::JobList jobs={ jobs.deref().clone() } />
    }
}
