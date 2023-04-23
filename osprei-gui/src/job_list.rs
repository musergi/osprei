use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub jobs: Vec<String>,
}

#[function_component]
pub fn JobList(props: &Props) -> Html {
    html! {
        <ul>
            {
                props
                    .jobs
                    .clone()
                    .into_iter()
                    .map(|job| html!{ <li>{ job }</li> })
                    .collect::<Html>()
            }
        </ul>
    }
}
