use redis::AsyncCommands;

use libs::error::IntoStringErr;
use libs::redis::connect_manager;

pub struct ValkeyTokenRepository {
    conn: redis::aio::ConnectionManager,
}

impl ValkeyTokenRepository {
    pub async fn connect(url: &str) -> Result<Self, String> {
        let conn = connect_manager(url).await?;
        Ok(Self { conn })
    }

    fn refresh_key(jti: &str) -> String {
        format!("refresh:{jti}")
    }

    fn blacklist_key(jti: &str) -> String {
        format!("blacklist:{jti}")
    }

    pub async fn store_refresh_token(
        &self,
        jti: &str,
        user_id: &str,
        expires_in_secs: u64,
    ) -> Result<(), String> {
        let mut conn = self.conn.clone();
        let key = Self::refresh_key(jti);
        conn.set_ex(key, user_id, expires_in_secs)
            .await
            .map_err_string()
    }

    pub async fn verify_refresh_token(&self, jti: &str, user_id: &str) -> Result<bool, String> {
        let mut conn = self.conn.clone();
        let key = Self::refresh_key(jti);
        let stored: Option<String> = conn.get(key).await.map_err_string()?;
        Ok(stored.map(|s| s == user_id).unwrap_or(false))
    }

    pub async fn revoke_refresh_token(&self, jti: &str) -> Result<(), String> {
        let mut conn = self.conn.clone();
        let key = Self::refresh_key(jti);
        conn.del(key).await.map_err_string()
    }

    pub async fn blacklist_access_token(
        &self,
        jti: &str,
        expires_in_secs: u64,
    ) -> Result<(), String> {
        let mut conn = self.conn.clone();
        let key = Self::blacklist_key(jti);
        conn.set_ex(key, "1", expires_in_secs)
            .await
            .map_err_string()
    }

    pub async fn is_blacklisted(&self, jti: &str) -> Result<bool, String> {
        let mut conn = self.conn.clone();
        let key = Self::blacklist_key(jti);
        conn.exists(key).await.map_err_string()
    }
}
