use async_trait::async_trait;
use libs::error::IntoStringErr;
use redis::AsyncCommands;

/// 도메인 비의존적인 캐시 인터페이스. application/service 측에서 직렬화한 JSON을 다룹니다.
#[async_trait]
pub trait PostCache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>, String>;
    async fn put(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), String>;
    async fn invalidate(&self, key: &str) -> Result<(), String>;
    async fn invalidate_list(&self, list_prefix: &str) -> Result<(), String>;
}

/// 캐시 미사용 환경 (테스트 등) 에서 쓰는 no-op 구현.
pub struct NoopPostCache;

#[async_trait]
impl PostCache for NoopPostCache {
    async fn get(&self, _key: &str) -> Result<Option<String>, String> {
        Ok(None)
    }
    async fn put(&self, _key: &str, _value: &str, _ttl_secs: u64) -> Result<(), String> {
        Ok(())
    }
    async fn invalidate(&self, _key: &str) -> Result<(), String> {
        Ok(())
    }
    async fn invalidate_list(&self, _list_prefix: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Valkey/Redis 기반 캐시 구현.
/// - 단건 키: `{prefix}{id}` (서비스가 prefix 결정)
/// - 목록 키: `{prefix}list:{hash}` → `LIST_KEYS_SET` SET에 등록되어 일괄 무효화
pub struct ValkeyPostCache {
    conn: redis::aio::ConnectionManager,
    list_keys_set: String,
}

impl ValkeyPostCache {
    pub async fn connect(url: &str) -> Result<Self, String> {
        let conn = libs::redis::connect_manager(url).await?;
        Ok(Self {
            conn,
            list_keys_set: "playground:posts:list:keys".to_string(),
        })
    }
}

#[async_trait]
impl PostCache for ValkeyPostCache {
    async fn get(&self, key: &str) -> Result<Option<String>, String> {
        let mut conn = self.conn.clone();
        conn.get(key).await.map_err_string()
    }

    async fn put(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), String> {
        let mut conn = self.conn.clone();
        let _: () = conn.set_ex(key, value, ttl_secs).await.map_err_string()?;
        if key.starts_with("playground:posts:list:") {
            let _: () = conn.sadd(&self.list_keys_set, key).await.map_err_string()?;
            let _: () = conn
                .expire(&self.list_keys_set, (ttl_secs as i64) * 4)
                .await
                .map_err_string()?;
        }
        Ok(())
    }

    async fn invalidate(&self, key: &str) -> Result<(), String> {
        let mut conn = self.conn.clone();
        let _: () = conn.del(key).await.map_err_string()?;
        Ok(())
    }

    async fn invalidate_list(&self, list_prefix: &str) -> Result<(), String> {
        let mut conn = self.conn.clone();
        let keys: Vec<String> = conn.smembers(&self.list_keys_set).await.map_err_string()?;
        let matching: Vec<String> = keys
            .into_iter()
            .filter(|k| k.starts_with(list_prefix))
            .collect();
        if !matching.is_empty() {
            let _: () = conn.del(matching).await.map_err_string()?;
        }
        Ok(())
    }
}
