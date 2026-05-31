use redis::AsyncCommands;

use libs::error::IntoStringErr;
use libs::redis::connect_manager;

const POST_COUNT_KEY: &str = "playground:posts:count";

pub struct ValkeyCache {
    conn: redis::aio::ConnectionManager,
}

impl ValkeyCache {
    pub async fn connect(url: &str) -> Result<Self, String> {
        let conn = connect_manager(url).await?;
        Ok(Self { conn })
    }

    pub async fn set_post_count(&mut self, count: u64) -> Result<(), String> {
        self.conn.set(POST_COUNT_KEY, count).await.map_err_string()
    }
}
