use application::user::TokenRepository;
use async_trait::async_trait;
use infrastructure::cache::token_repository::ValkeyTokenRepository;

pub struct ValkeyTokenRepositoryAdapter(ValkeyTokenRepository);

impl ValkeyTokenRepositoryAdapter {
    pub fn new(inner: ValkeyTokenRepository) -> Self {
        Self(inner)
    }
}

#[async_trait]
impl TokenRepository for ValkeyTokenRepositoryAdapter {
    async fn store_refresh_token(
        &self,
        jti: &str,
        user_id: &str,
        expires_in_secs: u64,
    ) -> Result<(), String> {
        self.0
            .store_refresh_token(jti, user_id, expires_in_secs)
            .await
    }

    async fn verify_refresh_token(&self, jti: &str, user_id: &str) -> Result<bool, String> {
        self.0.verify_refresh_token(jti, user_id).await
    }

    async fn revoke_refresh_token(&self, jti: &str) -> Result<(), String> {
        self.0.revoke_refresh_token(jti).await
    }

    async fn blacklist_access_token(&self, jti: &str, expires_in_secs: u64) -> Result<(), String> {
        self.0.blacklist_access_token(jti, expires_in_secs).await
    }

    async fn is_blacklisted(&self, jti: &str) -> Result<bool, String> {
        self.0.is_blacklisted(jti).await
    }
}
