mod api;
mod config;
mod core;
mod db;
mod ssh;

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use rust_embed::Embed;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::AppConfig;

#[derive(Embed)]
#[folder = "web/dist"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub config: Arc<AppConfig>,
    pub log_broadcast: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "xforge=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load .env if present
    let _ = dotenvy::dotenv();

    let config = AppConfig::from_env();
    let config = Arc::new(config);

    // Initialize database
    let db = db::init_pool(&config.database_url).await?;

    // Ensure default admin user exists with proper bcrypt hash
    let admin_exists: Option<(String,)> =
        sqlx::query_as("SELECT id FROM users WHERE username = 'admin'")
            .fetch_optional(&db)
            .await?;

    if admin_exists.is_none() {
        let hash = bcrypt::hash("admin", bcrypt::DEFAULT_COST)?;
        sqlx::query("INSERT INTO users (id, username, password_hash, role) VALUES (?, ?, ?, ?)")
            .bind("00000000-0000-0000-0000-000000000001")
            .bind("admin")
            .bind(&hash)
            .bind("admin")
            .execute(&db)
            .await?;
        tracing::info!("Created default admin user (admin/admin)");
    }

    // Create broadcast channel for log streaming
    let (log_tx, _) = broadcast::channel::<String>(1000);

    let state = AppState {
        db,
        config: config.clone(),
        log_broadcast: log_tx,
    };

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/ws", get(api::ws::ws_handler));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/api/dashboard", get(api::jobs::dashboard))
        .route("/api/servers", get(api::servers::list_servers))
        .route("/api/servers", post(api::servers::create_server))
        .route("/api/servers/:id", get(api::servers::get_server))
        .route("/api/servers/:id", put(api::servers::update_server))
        .route("/api/servers/:id", delete(api::servers::delete_server))
        .route(
            "/api/servers/:id/health",
            post(api::servers::health_check),
        )
        .route("/api/recipes", get(api::recipes::list_recipes))
        .route("/api/recipes/:name", get(api::recipes::get_recipe))
        .route("/api/jobs", get(api::jobs::list_jobs))
        .route("/api/jobs", post(api::jobs::create_job))
        .route("/api/jobs/:id", get(api::jobs::get_job))
        .route("/api/jobs/:id/cancel", post(api::jobs::cancel_job))
        // Marketplace
        .route("/api/sources", get(api::sources::list_sources))
        .route("/api/sources", post(api::sources::add_source))
        .route("/api/sources/:id", delete(api::sources::delete_source))
        .route("/api/sources/:id/sync", post(api::sources::sync_source))
        .route(
            "/api/sources/:source_id/recipes/:slug/install",
            post(api::sources::install_recipe),
        )
        // Audit log
        .route("/api/audit", get(api::sources::list_audit_logs))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            api::auth::auth_middleware,
        ));

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback(static_handler)
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("xForge server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn static_handler(
    uri: axum::http::Uri,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    let path = uri.path().trim_start_matches('/');

    // Try to serve the exact file
    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            [(axum::http::header::CONTENT_TYPE, mime.as_ref().to_string())],
            content.data.into_owned(),
        )
            .into_response();
    }

    // Fallback to index.html for SPA routing
    if let Some(content) = Assets::get("index.html") {
        let mime = mime_guess::from_path("index.html").first_or_octet_stream();
        return (
            [(axum::http::header::CONTENT_TYPE, mime.as_ref().to_string())],
            content.data.into_owned(),
        )
            .into_response();
    }

    (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response()
}
