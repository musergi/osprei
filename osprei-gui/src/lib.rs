use yew::prelude::*;

mod missing;

#[function_component]
pub fn App() -> Html {
    html! {
        <missing::Missing/>
    }
}
