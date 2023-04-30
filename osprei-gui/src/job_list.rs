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

#[derive(Properties, PartialEq)]
struct CardProps {
    job_name: String,
    job_status: String,
}

#[function_component]
fn JobCard(props: &CardProps) -> Html {
    let CardProps {job_name, job_status} = props;
    let job_name: String = job_name.to_uppercase().chars().map(|c| match c {
        '_' => ' ',
        c => c,
    }).collect();
    html! {
        <li class={"job-card"}>
            <h3>{ job_name }</h3>
            <p>{ job_status }</p>
            <button>{ "Run" }</button>
        </li>
    }
}

