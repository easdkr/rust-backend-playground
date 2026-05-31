use std::sync::Arc;

use api::http::AppState;
use api::http::routes::configure_routes;
use application::post::service::PostService;
use application::user::{AuthService, JwtConfig, TokenRepository, UserService};
use infrastructure::cache::token_repository::ValkeyTokenRepository;
use infrastructure::persistence::seaorm::post_repository::{PostRepository, SeaOrmPostRepository};
use infrastructure::persistence::seaorm::user_repository::{SeaOrmUserRepository, UserRepository};
use tracing::info;

mod token_adapter;

use token_adapter::ValkeyTokenRepositoryAdapter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    libs::telemetry::init_tracing(
        "rust_backend_playground=debug,tower_http=debug,axum::rejection=trace",
    );

    info!("Starting Rust Backend Playground...");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgrespassword@127.0.0.1:5433/playground".to_string()
    });
    let valkey_url =
        std::env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr_str = format!("{}:{}", host, port);

    info!("Connecting to database...");
    let db_conn = infrastructure::db::connect(&database_url).await?;
    info!("Database connected and schema initialized successfully!");

    info!("Connecting to Valkey...");
    let valkey_repo = ValkeyTokenRepository::connect(&valkey_url).await?;
    info!("Valkey connected successfully!");

    let jwt_config = JwtConfig::from_env();

    let post_repo: Arc<dyn PostRepository> = Arc::new(SeaOrmPostRepository::new(db_conn.clone()));
    let user_repo: Arc<dyn UserRepository> = Arc::new(SeaOrmUserRepository::new(db_conn));
    let token_repo: Arc<dyn TokenRepository> =
        Arc::new(ValkeyTokenRepositoryAdapter::new(valkey_repo));

    let post_service = Arc::new(PostService::new(post_repo));
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        token_repo,
        jwt_config.clone(),
    ));
    let user_service = Arc::new(UserService::new(user_repo));

    let app_state = AppState {
        post_service,
        auth_service,
        user_service,
        jwt_config: Arc::new(jwt_config),
    };

    let app = configure_routes(app_state);

    let listener = tokio::net::TcpListener::bind(&addr_str).await?;
    info!("Listening on http://{}", addr_str);
    axum::serve(listener, app).await?;

    Ok(())
}
