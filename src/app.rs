use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::hooks::use_navigate;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use log::error;
use reqwest;
use serde::{Deserialize, Serialize};
use web_sys;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos-full-stack.css" />

        // sets the document title
        <Title text="Leptos Full-Stack" />

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                    <Route path=StaticSegment("users") view=UsersPage />
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let navigate = use_navigate();

    let on_click = move |_| navigate("/users", Default::default());

    view! {
        <h1>"Leptos Full-Stack"</h1>
        <button on:click=on_click>"Start"</button>
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: i64,
    name: String,
    email: String,
}

#[component]
pub fn UsersPage() -> impl IntoView {
    let (users, set_users) = signal(Vec::<User>::new());
    let (name, set_name) = signal(String::new());
    let (email, set_email) = signal(String::new());

    // Fetch users from backend when component mounts
    Effect::new(move |_| {
        spawn_local(async move {
            let base_url = web_sys::window()
                .and_then(|win| win.location().origin().ok())
                .unwrap_or_else(|| "http://localhost:3000".to_string());

            let api_url = format!("{}/api/users", base_url);

            match reqwest::get(&api_url).await {
                Ok(response) => match response.json::<Vec<User>>().await {
                    Ok(data) => set_users.set(data),
                    Err(err) => error!("Failed to parse JSON: {:?}", err),
                },
                Err(err) => error!("Request failed: {:?}", err),
            }
        });
    });

    // Function to handle form submission
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default(); // Prevent page reload

        let name = name.get();
        let email = email.get();

        if name.is_empty() || email.is_empty() {
            error!("Name and email cannot be empty");
            return;
        }

        let new_user = User {
            id: 0, // Backend will assign ID
            name: name.clone(),
            email: email.clone(),
        };

        spawn_local(async move {
            let base_url = web_sys::window()
                .and_then(|win| win.location().origin().ok())
                .unwrap_or_else(|| "http://localhost:3000".to_string());

            let api_url = format!("{}/api/users", base_url);

            match reqwest::Client::new()
                .post(&api_url)
                .json(&new_user)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    if let Ok(created_user) = response.json::<User>().await {
                        set_users.update(|users| users.push(created_user));
                        set_name.set(String::new());
                        set_email.set(String::new());
                    }
                }
                Ok(response) => error!("Failed to create user: {:?}", response.status()),
                Err(err) => error!("Request failed: {:?}", err),
            }
        });
    };

    // Function to delete a user
    let delete_user = move |user_id: i64| {
        spawn_local(async move {
            let base_url = web_sys::window()
                .and_then(|win| win.location().origin().ok())
                .unwrap_or_else(|| "http://localhost:3000".to_string());

            let api_url = format!("{}/api/users/{}", base_url, user_id);

            match reqwest::Client::new().delete(&api_url).send().await {
                Ok(response) if response.status().is_success() => {
                    set_users.update(|users| users.retain(|user| user.id != user_id));
                }
                Ok(response) => error!("Failed to delete user: {:?}", response.status()),
                Err(err) => error!("Request failed: {:?}", err),
            }
        });
    };

    view! {
        <h1>"User Management"</h1>
        <form on:submit=on_submit>
            <label>"Name: "</label>
            <input
                type="text"
                on:input=move |e| set_name.set(event_target_value(&e))
                value=name.get()
            />
            " "
            <label>"Email: "</label>
            <input
                type="email"
                on:input=move |e| set_email.set(event_target_value(&e))
                value=email.get()
            />
            " "
            <button type="submit">"Add User"</button>
        </form>

        <ul>
            {move || {
                users
                    .get()
                    .iter()
                    .map(|user| {
                        let user_id = user.id;
                        view! {
                            <li>
                                <strong>"ID: "</strong>
                                {user.id}
                                " - "
                                <strong>"Name: "</strong>
                                {user.name.clone()}
                                " - "
                                <strong>"Email: "</strong>
                                {user.email.clone()}
                                " "
                                <button on:click=move |_| delete_user(user_id)>"Delete"</button>
                            </li>
                        }
                    })
                    .collect_view()
            }}
        </ul>
    }
}
