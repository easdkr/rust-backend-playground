use std::sync::Arc;

use api::http::AppState;
use api::http::routes::configure_routes;
use application::post::service::PostService;
use infrastructure::persistence::seaorm::post_repository::{PostRepository, SeaOrmPostRepository};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "rust_backend_playground=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .init();

    info!("Starting Rust Backend Playground...");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgrespassword@127.0.0.1:5433/playground".to_string()
    });
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr_str = format!("{}:{}", host, port);

    info!("Connecting to database...");
    let db_conn = infrastructure::db::connect(&database_url).await?;
    info!("Database connected and schema initialized successfully!");

    let post_repo: Arc<dyn PostRepository> = Arc::new(SeaOrmPostRepository::new(db_conn));
    let post_service = Arc::new(PostService::new(post_repo));
    let app_state = AppState { post_service };

    let app = configure_routes(app_state);

    let listener = tokio::net::TcpListener::bind(&addr_str).await?;
    info!("Listening on http://{}", addr_str);
    axum::serve(listener, app).await?;

    Ok(())
}
