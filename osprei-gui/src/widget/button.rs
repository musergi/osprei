use leptos::*;

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
