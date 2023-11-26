use leptos::*;
use leptos_router::*;
use serde::{de::DeserializeOwned, Serialize};

pub enum ButtonType {
    Primary,
    Secondary,
}

impl Default for ButtonType {
    fn default() -> Self {
        Self::Primary
    }
}

#[component]
pub fn button(
    #[prop(optional)] button_type: Option<ButtonType>,
    children: Children,
) -> impl IntoView {
    match button_type.unwrap_or_default() {
        ButtonType::Primary => view! {
            <button class="button primary">{children()}</button>
        },
        ButtonType::Secondary => view! {
            <button class="button secondary">{children()}</button>
        },
    }
}

#[component]
pub fn link_button(
    #[prop(optional)] button_type: Option<ButtonType>,
    link: String,
    #[prop(into)] text: String,
) -> impl IntoView {
    match button_type.unwrap_or_default() {
        ButtonType::Primary => view! {
            <A class="button primary" href={link}>{text}</A>
        },
        ButtonType::Secondary => view! {
            <A class="button secondary" href={link}>{text}</A>
        },
    }
}

#[component]
pub fn form_button<I, O>(
    #[prop(optional)] button_type: Option<ButtonType>,
    action: Action<I, Result<O, ServerFnError>>,
    #[prop(into)] text: String,
    children: Children,
) -> impl IntoView
where
    I: Clone + ServerFn + 'static,
    O: Clone + Serialize + DeserializeOwned + 'static,
{
    let input = match button_type.unwrap_or_default() {
        ButtonType::Primary => view! {
            <input class="button primary" type="submit" value={text}/>
        },
        ButtonType::Secondary => view! {
            <input class="button secondary" type="submit" value={text}/>
        },
    };
    view! {
        <ActionForm action>
            {children()}
            {input}
        </ActionForm>
    }
}
