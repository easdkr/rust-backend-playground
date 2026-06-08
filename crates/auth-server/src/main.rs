use std::sync::Arc;

use application::user::{AuthService, JwtConfig, TokenRepository, UserService};
use axum::{
    Router,
    routing::{get, post},
};
use infrastructure::cache::token_repository::ValkeyTokenRepository;
use infrastructure::persistence::seaorm::user_repository::{SeaOrmUserRepository, UserRepository};
use tracing::info;

mod state;
mod token_adapter;

use state::AppState;
use token_adapter::ValkeyTokenRepositoryAdapter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    libs::telemetry::init_tracing(
        "auth_server=debug,tower_http=debug,axum::rejection=trace",
    );

    info!("Starting Auth Server...");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:***@127.0.0.1:5433/playground".to_string()
    });
    let valkey_url =
        std::env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let host = std::env::var("AUTH_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("AUTH_PORT").unwrap_or_else(|_| "3002".to_string());
    let addr = format!("{}:{}", host, port);

    info!("Connecting to database...");
    let db_conn = infrastructure::db::connect(&database_url).await?;
    info!("Database connected!");

    info!("Connecting to Valkey...");
    let valkey_repo = ValkeyTokenRepository::connect(&valkey_url).await?;
    info!("Valkey connected!");

    let jwt_config = JwtConfig::from_env();

    let user_repo: Arc<dyn UserRepository> = Arc::new(SeaOrmUserRepository::new(db_conn));
    let token_repo: Arc<dyn TokenRepository> =
        Arc::new(ValkeyTokenRepositoryAdapter::new(valkey_repo));

    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        token_repo,
        jwt_config.clone(),
    ));
    let user_service = Arc::new(UserService::new(user_repo));

    let app_state = AppState {
        auth_service,
        user_service,
        jwt_config: Arc::new(jwt_config),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Auth server listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn login() -> &'static str {
    "login"
}

async fn refresh() -> &'static str {
    "refresh"
}

async fn logout() -> &'static str {
    "logout"
}
