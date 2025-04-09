use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::Json;
use axum::Router;
use dotenvy::dotenv;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_full_stack::app::*;
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};

mod models;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).expect("Failed to initialize logger");

    log::info!("Server starting...");

    dotenv().ok();

    let db = SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!().run(&db).await.expect("Migrations failed");

    let state = AppState { db };

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let cors = CorsLayer::new()
        .allow_origin(Any) // In Production: DO NOT Use Any!
        .allow_methods(Any) // In Production: Restrict Methods!
        .allow_headers(Any); // In Production: Restrict Headers!

    let app = Router::new()
        // API routes
        .route("/api/users", get(get_users).post(create_user))
        .route("/api/users/:id", delete(delete_user))
        .layer(cors)
        .with_state(state)
        // Leptos routes
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}

async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<models::User>>, (StatusCode, String)> {
    let users = sqlx::query_as::<_, models::User>("SELECT * FROM users")
        .fetch_all(&state.db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", err),
            )
        })?;

    Ok(Json(users))
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<models::UserRaw>,
) -> Result<Json<models::User>, (StatusCode, String)> {
    let result = sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind(&payload.name)
        .bind(&payload.email)
        .execute(&state.db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Insert error: {}", err),
            )
        })?;

    Ok(Json(models::User {
        id: result.last_insert_rowid(),
        name: payload.name,
        email: payload.email,
    }))
}

async fn delete_user(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Delete error: {}", err),
            )
        })?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "User not found".to_string()))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
