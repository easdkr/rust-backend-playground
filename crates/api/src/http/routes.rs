use axum::{Router, routing::get};
use libs::http::HttpAspects;

use super::handlers;
use crate::http::state::AppState;

pub fn configure_routes(state: AppState) -> Router {
    let router = Router::new()
        .route("/", get(welcome_handler))
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
        .route(
            "/posts/:id/publish",
            axum::routing::post(handlers::publish_post),
        )
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
                    <div class="endpoint">
                        <span class="method post">POST</span>
                        <span class="path">/posts/:id/publish</span>
                        <span class="desc">초안 포스트 발행</span>
                    </div>
                </div>
            </div>
        </body>
        </html>
        "#,
    )
}
