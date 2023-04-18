use yew::prelude::*;

#[function_component]
pub fn Missing() -> Html {
    html! {
        <div>
            <h3>{ "No backend found..." }</h3>
            <form>
                <label for="url">{{"Server url"}}</label>
                <input type="text" id="ul" name="url" />
                <input type="submit" value="Try"/>
            </form>
        </div>
    }
}
