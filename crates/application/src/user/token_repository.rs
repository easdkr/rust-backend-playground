use async_trait::async_trait;

#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn store_refresh_token(
        &self,
        jti: &str,
        user_id: &str,
        expires_in_secs: u64,
    ) -> Result<(), String>;

    async fn verify_refresh_token(&self, jti: &str, user_id: &str) -> Result<bool, String>;

    async fn revoke_refresh_token(&self, jti: &str) -> Result<(), String>;

    async fn blacklist_access_token(&self, jti: &str, expires_in_secs: u64) -> Result<(), String>;

    async fn is_blacklisted(&self, jti: &str) -> Result<bool, String>;
}
