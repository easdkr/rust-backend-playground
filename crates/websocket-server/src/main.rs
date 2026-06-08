use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Response,
    routing::get,
};
use libs::redis::connect_manager;
use redis::AsyncCommands;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tracing::{error, info, warn};

mod auth;

use auth::validate_token;

/// 연결된 클라이언트 소켓 관리
type UserSockets = Arc<RwLock<HashMap<String, Vec<tokio::sync::mpsc::UnboundedSender<Message>>>>>;

#[derive(Clone)]
struct AppState {
    user_sockets: UserSockets,
    redis_pub: redis::aio::ConnectionManager,
    redis_sub: redis::aio::ConnectionManager,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    libs::telemetry::init_tracing("websocket_server=debug,tower_http=debug");

    let valkey_url =
        std::env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let host = std::env::var("WS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("WS_PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("{}:{}", host, port);

    info!("Connecting to Valkey for Pub/Sub...");
    let redis_pub = connect_manager(&valkey_url).await?;
    let redis_sub = connect_manager(&valkey_url).await?;
    info!("Valkey connected!");

    let state = AppState {
        user_sockets: Arc::new(RwLock::new(HashMap::new())),
        redis_pub,
        redis_sub,
    };

    // Redis Pub/Sub 구독 태스크 시작
    let sub_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = redis_subscriber(sub_state).await {
            error!("Redis subscriber error: {e}");
        }
    });

    let app = Router::new()
        .route("/ws/notifications", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("WebSocket server listening on ws://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // 첫 메시지로 토큰 수신
    let token = match socket.recv().await {
        Some(Ok(Message::Text(text))) => text,
        _ => {
            warn!("WebSocket connection closed before authentication");
            let _ = socket.close().await;
            return;
        }
    };

    // 토큰 검증
    let user_id = match validate_token(&token).await {
        Ok(uid) => uid,
        Err(e) => {
            warn!("WebSocket auth failed: {e}");
            let _ = socket
                .send(Message::Text(
                    serde_json::json!({"error": "Unauthorized", "message": e }).to_string(),
                ))
                .await;
            let _ = socket.close().await;
            return;
        }
    };

    info!("WebSocket authenticated for user: {user_id}");

    // 채널 생성하여 소켓과 브로드캐스터 연결
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    {
        let mut sockets = state.user_sockets.write().await;
        sockets.entry(user_id.clone()).or_default().push(tx);
    }

    // Redis에 사용자 접속 알림 (선택적)
    let _ = state
        .redis_pub
        .clone()
        .publish::<_, _, ()>(
            format!("ws:user:connected:{}", user_id),
            "",
        )
        .await;

    // 수신 루프: rx에서 메시지를 받아 WebSocket으로 전송
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if socket.send(msg).await.is_err() {
                break;
            }
        }
    });

    // 클라이언트로부터 ping/기타 메시지 수신 (heartbeat 용도)
    let _ = send_task.await;

    // 연결 종료 시 정리
    {
        let mut sockets = state.user_sockets.write().await;
        if let Some(list) = sockets.get_mut(&user_id) {
            list.retain(|sender| !sender.is_closed());
            if list.is_empty() {
                sockets.remove(&user_id);
            }
        }
    }

    info!("WebSocket disconnected for user: {user_id}");
}

/// Redis Pub/Sub으로 다른 서버 인스턴스에서 온 알림 수신
async fn redis_subscriber(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open(std::env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()))?;
    let mut conn = client.get_async_connection().await?;
    let mut pubsub = conn.into_pubsub();
    pubsub.subscribe("notifications:broadcast").await?;

    let mut stream = pubsub.on_message();

    while let Some(msg) = stream.next().await {
        let payload: String = msg.get_payload()?;
        let notification: serde_json::Value = serde_json::from_str(&payload).unwrap_or_default();

        if let Some(user_id) = notification.get("user_id").and_then(|v| v.as_str()) {
            let sockets = state.user_sockets.read().await;
            if let Some(list) = sockets.get(user_id) {
                let msg = Message::Text(payload.clone());
                for sender in list {
                    let _ = sender.send(msg.clone());
                }
            }
        }
    }

    Ok(())
}
