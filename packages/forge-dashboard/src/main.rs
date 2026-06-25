use leptos::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <div class="forge-dashboard">
            <header><h1>"Forge Dashboard"</h1></header>
            <nav>
                <a href="/">"Overview"</a>
                <a href="/sessions">"Sessions"</a>
                <a href="/audit">"Audit"</a>
                <a href="/analytics">"Analytics"</a>
                <a href="/meta">"Meta"</a>
            </nav>
            <main><Outlet/></main>
        </div>
    }
}

fn main() { mount_to_body(App); }
