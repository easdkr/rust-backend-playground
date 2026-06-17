use std::sync::Arc;

use application::user::{
    AuthError, AuthService, JwtConfig, LoginCmd, LogoutRequest, RefreshTokenRequest, RegisterCmd,
    TokenRepository, UserService,
};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use infrastructure::cache::token_repository::ValkeyTokenRepository;
use infrastructure::persistence::seaorm::user_repository::{SeaOrmUserRepository, UserRepository};
use serde::Serialize;
use tracing::info;

mod state;
mod token_adapter;

use state::AppState;
use token_adapter::ValkeyTokenRepositoryAdapter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    libs::telemetry::init_tracing("auth_server=debug,tower_http=debug,axum::rejection=trace");

    info!("Starting Auth Server...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:***@127.0.0.1:5433/playground".to_string());
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
        jwt_config: Arc::new(jwt_config.clone()),
    };

    // Paths are mounted without the `/auth` prefix because the k8s ingress
    // strips `/auth` before forwarding requests to this service.
    let app = Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Auth server listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn register(
    State(user_service): State<Arc<UserService>>,
    Json(cmd): Json<RegisterCmd>,
) -> Result<Json<application::user::UserDto>, ErrorResponse> {
    let user = user_service.register(cmd).await.map_err(error_response)?;
    Ok(Json(user))
}

async fn login(
    State(auth_service): State<Arc<AuthService>>,
    Json(cmd): Json<LoginCmd>,
) -> Result<Json<application::user::TokenResponse>, ErrorResponse> {
    let pair = auth_service.login(cmd).await.map_err(error_response)?;
    let expires_in = auth_service.jwt_config().access_expiry_secs;
    Ok(Json(pair.into_response(expires_in)))
}

async fn refresh(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<application::user::TokenResponse>, ErrorResponse> {
    let pair = auth_service
        .refresh(&req.refresh_token)
        .await
        .map_err(error_response)?;
    let expires_in = auth_service.jwt_config().access_expiry_secs;
    Ok(Json(pair.into_response(expires_in)))
}

async fn logout(
    State(auth_service): State<Arc<AuthService>>,
    headers: HeaderMap,
    Json(req): Json<LogoutRequest>,
) -> Result<StatusCode, ErrorResponse> {
    let access_token = extract_bearer_token(&headers).unwrap_or("");
    auth_service
        .logout(access_token, req.refresh_token.as_deref())
        .await
        .map_err(error_response)?;
    Ok(StatusCode::NO_CONTENT)
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
}

#[derive(Serialize)]
struct ErrorResponseBody {
    success: bool,
    error: String,
}

type ErrorResponse = (StatusCode, Json<ErrorResponseBody>);

fn error_response(err: AuthError) -> ErrorResponse {
    let status = match &err {
        AuthError::InvalidCredentials
        | AuthError::UserNotFound
        | AuthError::TokenExpired
        | AuthError::TokenInvalid(_)
        | AuthError::TokenBlacklisted => StatusCode::UNAUTHORIZED,
        AuthError::UserAlreadyExists => StatusCode::CONFLICT,
        AuthError::InsufficientPermissions => StatusCode::FORBIDDEN,
        AuthError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (
        status,
        Json(ErrorResponseBody {
            success: false,
            error: err.to_string(),
        }),
    )
}
