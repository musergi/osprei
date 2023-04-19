use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
pub fn Missing() -> Html {
    let url = use_state(|| String::from("http://localhost:10000"));
    let url_input_ref = use_node_ref();

    let submit_callback = {
        let url = url.clone();
        let url_input_ref = url_input_ref.clone();
        move |e: SubmitEvent| {
            e.prevent_default();
            url.set(url_input_ref.cast::<HtmlInputElement>().unwrap().value())
        }
    };
    html! {
        <div>
            <h3>{ "No backend found..." }</h3>
            <p>{ "Attempted connection to: "}{ (*url).clone() }</p>
            <form onsubmit={submit_callback}>
                <label for="url">{{"Server url"}}</label>
                <input type="text" id="url" name="url" value={ (*url).clone() } ref={url_input_ref} />
                <input type="submit" value="Try"/>
            </form>
        </div>
    }
}
