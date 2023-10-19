use leptos::*;
use leptos_meta::*;
use leptos_router::*;

pub mod error_template;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {


        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/start-axum-workspace.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let jobs = create_resource(|| (), |_| async { fetch_jobs().await });

    view! {
        {move || match jobs.get() {
            None => view! {<p>{ "Loading..." }</p>}.into_view(),
            Some(jobs) => view! {
                <ul>
                    {jobs.into_iter()
                        .map(|j| view!{<li>{ j }</li>})
                        .collect_view()}
                </ul>
            }.into_view()
        }}

    }
}

#[server(FetchJobs)]
async fn fetch_jobs() -> Result<Vec<i64>, ServerFnError> {
    Ok(vec![0, 1, 2, 3])
}
