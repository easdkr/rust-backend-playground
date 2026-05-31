use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use application::user::TokenRepository;
use async_trait::async_trait;

#[derive(Default)]
struct TokenStore {
    refresh: HashMap<String, String>,
    blacklist: HashMap<String, ()>,
}

pub struct InMemoryTokenRepository {
    store: Mutex<TokenStore>,
}

impl InMemoryTokenRepository {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(TokenStore::default()),
        }
    }
}

#[async_trait]
impl TokenRepository for InMemoryTokenRepository {
    async fn store_refresh_token(
        &self,
        jti: &str,
        user_id: &str,
        _expires_in_secs: u64,
    ) -> Result<(), String> {
        self.store
            .lock()
            .map_err(|e| e.to_string())?
            .refresh
            .insert(jti.to_string(), user_id.to_string());
        Ok(())
    }

    async fn verify_refresh_token(&self, jti: &str, user_id: &str) -> Result<bool, String> {
        let store = self.store.lock().map_err(|e| e.to_string())?;
        Ok(store
            .refresh
            .get(jti)
            .map(|stored| stored == user_id)
            .unwrap_or(false))
    }

    async fn revoke_refresh_token(&self, jti: &str) -> Result<(), String> {
        self.store
            .lock()
            .map_err(|e| e.to_string())?
            .refresh
            .remove(jti);
        Ok(())
    }

    async fn blacklist_access_token(&self, jti: &str, _expires_in_secs: u64) -> Result<(), String> {
        self.store
            .lock()
            .map_err(|e| e.to_string())?
            .blacklist
            .insert(jti.to_string(), ());
        Ok(())
    }

    async fn is_blacklisted(&self, jti: &str) -> Result<bool, String> {
        let store = self.store.lock().map_err(|e| e.to_string())?;
        Ok(store.blacklist.contains_key(jti))
    }
}

pub fn test_token_repository() -> Arc<dyn TokenRepository> {
    Arc::new(InMemoryTokenRepository::new())
}
