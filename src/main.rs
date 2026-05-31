use axum::{
    routing::get,
    Router,
};
use tracing::info;

mod db;
mod entities;
mod error;
mod handlers;

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

    info!("Starting Rust Backend Playground...");

    // Get database URL and port from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://playground.db?mode=rwc".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr_str = format!("{}:{}", host, port);

    // Connect to database and setup schema
    info!("Connecting to database at {}...", database_url);
    let db = db::connect(&database_url).await?;
    info!("Database connected and schema initialized successfully!");

    // Build routes
    let app = Router::new()
        .route("/", get(welcome_handler))
        .route("/posts", get(handlers::list_posts).post(handlers::create_post))
        .route(
            "/posts/:id",
            get(handlers::get_post)
                .put(handlers::update_post)
                .delete(handlers::delete_post),
        )
        .with_state(db);

    // Run the axum server
    let listener = tokio::net::TcpListener::bind(&addr_str).await?;
    info!("Listening on http://{}", addr_str);
    axum::serve(listener, app).await?;

    Ok(())
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
                <h1>Rust Backend Playground</h1>
                <p>Tokio + Axum + SeaORM(SQLite) 기반의 고성능 비동기 Rust 백엔드 플레이그라운드 셋업이 완료되었습니다!</p>
                <p>아래 API 엔드포인트를 사용하여 블로그 포스트 CRUD를 바로 테스트해보실 수 있습니다:</p>
                
                <div class="endpoint-list">
                    <div class="endpoint">
                        <span class="method get">GET</span>
                        <span class="path">/posts</span>
                        <span class="desc">전체 포스트 조회</span>
                    </div>
                    <div class="endpoint">
                        <span class="method post">POST</span>
                        <span class="path">/posts</span>
                        <span class="desc">새 포스트 생성</span>
                    </div>
                    <div class="endpoint">
                        <span class="method get">GET</span>
                        <span class="path">/posts/:id</span>
                        <span class="desc">특정 포스트 조회</span>
                    </div>
                    <div class="endpoint">
                        <span class="method put">PUT</span>
                        <span class="path">/posts/:id</span>
                        <span class="desc">포스트 수정</span>
                    </div>
                    <div class="endpoint">
                        <span class="method delete">DELETE</span>
                        <span class="path">/posts/:id</span>
                        <span class="desc">포스트 삭제</span>
                    </div>
                </div>
            </div>
        </body>
        </html>
        "#,
    )
}
