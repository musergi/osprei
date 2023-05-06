use std::ops::Deref;

use gloo_net::http::Request;
use log::{debug, error, info};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub jobs: Vec<String>,
}

#[function_component]
pub fn JobList(props: &Props) -> Html {
    let cards: Html = props
        .jobs
        .iter()
        .map(|job_name| {
            html! {
                <JobCard job_name={job_name.clone()} />
            }
        })
        .collect();
    html! {
        <ul>
            {cards}
        </ul>
    }
}

#[derive(Properties, PartialEq, Debug)]
struct CardProps {
    job_name: String,
}

#[function_component]
fn JobCard(props: &CardProps) -> Html {
    debug!("Created component JobCard: {:?}", props);
    let CardProps { job_name } = props;
    let display_job_name: String = job_name
        .to_uppercase()
        .chars()
        .map(|c| match c {
            '_' => ' ',
            c => c,
        })
        .collect();

    let url = format!("http://localhost:10000/job/{}/executions", job_name);

    let last_execution_summary: UseStateHandle<Option<osprei::ExecutionSummary>> =
        use_state(|| None);
    let last_execution: UseStateHandle<Option<osprei::ExecutionDetails>> = use_state(|| None);

    {
        let last_execution_summary = last_execution_summary.clone();
        use_effect_with_deps(
            move |_| {
                info!("Reloading execution summary");
                let last_execution_summary = last_execution_summary.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let summaries: Vec<osprei::ExecutionSummary> = Request::get(&url)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    if let Some(execution) = summaries.into_iter().next() {
                        last_execution_summary.set(Some(execution));
                    };
                });
                || ()
            },
            (),
        );
    }

    {
        let last_execution_summary = last_execution_summary.clone();
        let last_execution = last_execution.clone();
        let deps = last_execution_summary.clone();

        use_effect_with_deps(
            move |_| {
                info!("Reloading execution");
                if let Some(execution_summary) = last_execution_summary.deref().clone() {
                    let url = format!("http://localhost:10000/execution/{}", execution_summary.id);
                    let last_execution = last_execution.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let execution: osprei::ExecutionDetails = Request::get(&url)
                            .send()
                            .await
                            .unwrap()
                            .json()
                            .await
                            .unwrap();
                        last_execution.set(Some(execution));
                    });
                };
                || ()
            },
            deps,
        );
    }

    let job_name = job_name.clone();
    let run_callback = Callback::from(move |_| {
        info!("Run pressed for {}", job_name);
        let url = format!("http://localhost:10000/job/{}/run", job_name);
        info!("Sending request to {}", url);
        wasm_bindgen_futures::spawn_local(async move {
            match Request::get(&url).send().await {
                Ok(response) => {
                    info!("Recieved response: {}", response.status());
                    match response.json::<i64>().await {
                        Ok(id) => info!("Successfully created execution with id {}", id),
                        Err(err) => {
                            error!("Deserialization error: {}", err);
                        }
                    }
                }
                Err(err) => {
                    error!("Request to {} failed: {}", url, err)
                }
            }
        });
    });

    let execution_time = last_execution_summary
        .deref()
        .clone()
        .map(|execution| execution.start_time)
        .unwrap_or_else(|| String::from("Not loaded"));
    let status = last_execution
        .deref()
        .clone()
        .map(|execution| match execution.status {
            Some(0) => String::from("Success"),
            Some(1) => String::from("Failure"),
            Some(2) => String::from("Canceled"),
            None => String::from("Executing"),
            _ => String::from("Unknown"),
        })
        .unwrap_or_else(|| String::from("Not loaded"));
    html! {
        <li class={"job-card"}>
            <h3>{ display_job_name }</h3>
            <p>{ "Last execution: " }<strong>{ execution_time }</strong></p>
            <p>{ "Status: " }<strong>{ status }</strong></p>
            <button onclick={run_callback}>{ "Run" }</button>
        </li>
    }
}
