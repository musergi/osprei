use yew::prelude::*;
use log::{error, info, debug};
use gloo_net::http::Request;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub jobs: Vec<String>,
}

#[function_component]
pub fn JobList(props: &Props) -> Html {
    let cards: Html = props
        .jobs
        .iter()
        .map(|job_name| html!{
            <JobCard job_name={job_name.clone()} job_status={"Not executed"}/>
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
    job_status: String,
}

#[function_component]
fn JobCard(props: &CardProps) -> Html {
    debug!("Created component JobCard: {:?}", props);
    let CardProps {job_name, job_status} = props;
    let display_job_name: String = job_name.to_uppercase().chars().map(|c| match c {
        '_' => ' ',
        c => c,
    }).collect();
    let job_name = job_name.clone();
    let run_callback = Callback::from(move |_| {
        info!("Run pressed for {}", job_name);
        let url = format!("http://localhost:10000/job/{}/run", job_name);
        info!("Sending request to {}", url);
        wasm_bindgen_futures::spawn_local(async move {
            match Request::get(&url)
                .send()
                .await {
                Ok(response) => {
                    info!("Recieved response: {}", response.status());
                    match response.json::<i64>().await {
                        Ok(id) => info!("Successfully created execution with id {}", id),
                        Err(err) => {
                            error!("Deserialization error: {}", err);
                        }
                    }
                },
                Err(err) => {
                    error!("Request to {} failed: {}", url, err)
                }
                
            }
        });
    });
    html! {
        <li class={"job-card"}>
            <h3>{ display_job_name }</h3>
            <p>{ job_status }</p>
            <button onclick={run_callback}>{ "Run" }</button>
        </li>
    }
}

