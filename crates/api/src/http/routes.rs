use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use libs::http::HttpAspects;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use super::auth;
use super::handlers;
use super::middleware::auth::{jwt_auth_middleware, require_admin_middleware};
use crate::http::state::AppState;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rust Backend Playground API",
        description = "Tokio + Axum + SeaORM(PostgreSQL) + Valkey 기반 API",
        version = "0.1.0",
        contact(name = "API Support", email = "support@example.com")
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server")
    ),
    paths(
        handlers::list_posts,
        handlers::create_post,
        handlers::get_post,
        handlers::update_post,
        handlers::publish_post,
        handlers::delete_post,
    ),
    components(
        schemas(
            application::post::dto::PostDto,
            application::post::dto::CreatePostCmd,
            application::post::dto::UpdatePostCmd,
            application::post::dto::ListPostsQuery,
            application::pagination::CursorPage<application::post::dto::PostDto>,
        )
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

pub fn configure_routes(state: AppState) -> Router {
    let public = Router::new()
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh));

    let protected = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route(
            "/posts",
            get(handlers::list_posts).post(handlers::create_post),
        )
        .route(
            "/posts/:id",
            get(handlers::get_post)
                .put(handlers::update_post)
                .delete(handlers::delete_post),
        )
        .route("/posts/:id/publish", post(handlers::publish_post))
        .route_layer(from_fn_with_state(state.clone(), jwt_auth_middleware));

    let admin = Router::new()
        .route("/admin/users", get(auth::list_users))
        .route_layer(from_fn_with_state(state.clone(), require_admin_middleware))
        .route_layer(from_fn_with_state(state.clone(), jwt_auth_middleware));

    let scalar = Scalar::with_url("/scalar", ApiDoc::openapi());

    let router = Router::new()
        .route("/", get(welcome_handler))
        .merge(scalar)
        .merge(public)
        .merge(protected)
        .merge(admin)
        .with_state(state);

    HttpAspects::new().apply(router)
}

async fn welcome_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(
        r#"
        <!DOCTYPE html>
        <html lang="ko">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Rust Backend Playground</title>
            <style>
                body {
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    background: linear-gradient(135deg, #1e1e2f 0%, #11111c 100%);
                    color: #e2e8f0;
                    margin: 0;
                    padding: 0;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    min-height: 100vh;
                }
                .container {
                    background: rgba(255, 255, 255, 0.05);
                    backdrop-filter: blur(10px);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    border-radius: 16px;
                    padding: 40px;
                    max-width: 600px;
                    width: 90%;
                    box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.37);
                }
                h1 {
                    color: #ff8c00;
                    margin-top: 0;
                    display: flex;
                    align-items: center;
                    font-size: 2.5rem;
                }
                h1::after {
                    content: " 🦀";
                }
                p {
                    line-height: 1.6;
                    color: #cbd5e1;
                }
                .tag {
                    display: inline-block;
                    background: #a78bfa;
                    color: #1e1e2f;
                    padding: 2px 8px;
                    border-radius: 9999px;
                    font-size: 0.8rem;
                    font-weight: bold;
                    margin-bottom: 16px;
                }
                .endpoint-list {
                    background: rgba(0, 0, 0, 0.2);
                    padding: 20px;
                    border-radius: 8px;
                    margin-top: 24px;
                }
                .endpoint {
                    font-family: 'Courier New', Courier, monospace;
                    margin-bottom: 12px;
                    display: flex;
                    align-items: center;
                }
                .method {
                    display: inline-block;
                    padding: 4px 8px;
                    border-radius: 4px;
                    font-size: 0.8rem;
                    font-weight: bold;
                    margin-right: 12px;
                    width: 70px;
                    text-align: center;
                }
                .get { background-color: #2e7d32; color: #e8f5e9; }
                .post { background-color: #1565c0; color: #e3f2fd; }
                .put { background-color: #f57f17; color: #fffde7; }
                .delete { background-color: #c62828; color: #ffebee; }
                .path {
                    color: #a78bfa;
                }
                .desc {
                    margin-left: auto;
                    color: #94a3b8;
                    font-size: 0.9rem;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <span class="tag">Layered Workspace</span>
                <h1>Rust Backend Playground</h1>
                <p>Tokio + Axum + SeaORM(PostgreSQL) + Valkey 기반의 <strong>레이어드 workspace</strong> 아키텍처입니다.</p>
                <p>application → infrastructure / api → server 계층으로 crate가 분리되어 있습니다.</p>
                
                <div class="endpoint-list">
                    <div class="endpoint">
                        <span class="method post">POST</span>
                        <span class="path">/auth/login</span>
                        <span class="desc">로그인</span>
                    </div>
                    <div class="endpoint">
                        <span class="method get">GET</span>
                        <span class="path">/posts</span>
                        <span class="desc">전체 포스트 조회 (JWT)</span>
                    </div>
                    <div class="endpoint">
                        <span class="method post">POST</span>
                        <span class="path">/posts</span>
                        <span class="desc">새 포스트 생성 (Editor+)</span>
                    </div>
                    <div class="endpoint">
                        <span class="method get">GET</span>
                        <span class="path">/admin/users</span>
                        <span class="desc">사용자 목록 (Admin)</span>
                    </div>
                </div>
            </div>
        </body>
        </html>
        "#,
    )
}
