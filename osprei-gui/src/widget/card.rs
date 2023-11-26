use leptos::*;

#[component]
pub fn card(title: String, children: Children) -> impl IntoView {
    view! {
        <CardContainer>
            <CardContent>
                <CardTitle title/>
                {children()}
            </CardContent>
        </CardContainer>
    }
}

#[component]
fn card_container(children: Children) -> impl IntoView {
    let style = "
        width: 300px;
        border: 1px solid #ddd;
        border-radius: 8px;
        overflow: hidden;
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
        margin: 16px;
    ";

    view! {
        <div style={style}>
            {children()}
        </div>
    }
}

#[component]
fn card_content(children: Children) -> impl IntoView {
    let style = "
        padding: 16px;
        text-align: left;
    ";

    view! {
        <div style={style}>
            {children()}
        </div>
    }
}

#[component]
fn card_title(title: String) -> impl IntoView {
    let style = "
        font-size: 20px;
        text-align: left;
        margin-bottom: 8px;
    ";

    view! {
        <h2 style={style}>{title}</h2>
    }
}

#[component]
pub fn action_buttons(children: Children) -> impl IntoView {
    let style = "
        display: flex;
        justify-content: space-between;
        margin-top: 16px;
    ";

    view! {
        <div style={style}>
            {children()}
        </div>
    }
}
