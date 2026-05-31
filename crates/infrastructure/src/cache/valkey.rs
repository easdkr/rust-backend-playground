use redis::AsyncCommands;
use redis::aio::ConnectionManager;

const POST_COUNT_KEY: &str = "playground:posts:count";

pub struct ValkeyCache {
    conn: ConnectionManager,
}

impl ValkeyCache {
    pub async fn connect(url: &str) -> Result<Self, String> {
        let client = redis::Client::open(url).map_err(|e| e.to_string())?;
        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Self { conn })
    }

    pub async fn set_post_count(&mut self, count: u64) -> Result<(), String> {
        self.conn
            .set(POST_COUNT_KEY, count)
            .await
            .map_err(|e| e.to_string())
    }
}
