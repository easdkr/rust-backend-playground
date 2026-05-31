mod context;
mod jobs;
mod scheduler;

use std::sync::Arc;

use application::post::batch::PostBatchService;
use context::BatchContext;
use infrastructure::persistence::seaorm::post_repository::{PostRepository, SeaOrmPostRepository};
use scheduler::Scheduler;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "rust_backend_playground_batch=debug,infrastructure=info".into()
            }),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgrespassword@127.0.0.1:5433/playground".to_string()
    });
    let valkey_url = std::env::var("VALKEY_URL").ok();

    info!("Starting Rust Backend Playground batch worker...");

    info!("Connecting to database...");
    let db_conn = infrastructure::db::connect(&database_url).await?;
    info!("Database connected.");

    let post_repo: Arc<dyn PostRepository> = Arc::new(SeaOrmPostRepository::new(db_conn));
    let post_batch_service = Arc::new(PostBatchService::new(post_repo));
    let ctx = Arc::new(BatchContext::new(post_batch_service, valkey_url.as_deref()).await);

    let scheduler = Scheduler::new().await?;
    jobs::post_count::register(&scheduler, Arc::clone(&ctx)).await?;

    scheduler.run_until_shutdown().await?;

    Ok(())
}
