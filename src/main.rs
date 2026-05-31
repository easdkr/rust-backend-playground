use tracing::info;
use std::sync::Arc;
use crate::presentation::http::handlers::AppState;
use crate::application::post::service::PostService;
use crate::infrastructure::persistence::seaorm::post_repository::SeaOrmPostRepository;

mod db;
mod domain;
mod application;
mod infrastructure;
mod presentation;
mod error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_backend_playground=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .init();

    info!("Starting Rust Backend Playground (DDD)...");

    // Get database URL and port from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://playground.db?mode=rwc".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr_str = format!("{}:{}", host, port);

    // Connect to database and setup schema
    info!("Connecting to database...");
    let db_conn = db::connect(&database_url).await?;
    info!("Database connected and schema initialized successfully!");

    // Wire up dependencies (Dependency Injection)
    let post_repo = Arc::new(SeaOrmPostRepository::new(db_conn));
    let post_service = Arc::new(PostService::new(post_repo));
    let app_state = AppState { post_service };

    // Build presentation (routes)
    let app = presentation::http::routes::configure_routes(app_state);

    // Run the axum server
    let listener = tokio::net::TcpListener::bind(&addr_str).await?;
    info!("Listening on http://{}", addr_str);
    axum::serve(listener, app).await?;

    Ok(())
}
