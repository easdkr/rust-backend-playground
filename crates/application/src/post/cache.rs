use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::entity::PostWithAuthor;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CachedPostPage {
    pub data: Vec<PostWithAuthor>,
    pub next_cursor: Option<i32>,
    pub has_more: bool,
}

#[async_trait]
pub trait PostCache: Send + Sync {
    async fn get_post(&self, id: i32) -> Result<Option<PostWithAuthor>, String>;
    async fn put_post(&self, value: &PostWithAuthor) -> Result<(), String>;
    async fn invalidate_post(&self, id: i32) -> Result<(), String>;

    async fn get_list(&self, query_hash: &str) -> Result<Option<CachedPostPage>, String>;
    async fn put_list(&self, query_hash: &str, page: &CachedPostPage) -> Result<(), String>;
    async fn invalidate_all_lists(&self) -> Result<(), String>;
}

/// 테스트·캐시 미사용 환경에서 쓰는 no-op 구현.
pub struct NoopPostCache;

#[async_trait]
impl PostCache for NoopPostCache {
    async fn get_post(&self, _id: i32) -> Result<Option<PostWithAuthor>, String> {
        Ok(None)
    }
    async fn put_post(&self, _value: &PostWithAuthor) -> Result<(), String> {
        Ok(())
    }
    async fn invalidate_post(&self, _id: i32) -> Result<(), String> {
        Ok(())
    }
    async fn get_list(&self, _query_hash: &str) -> Result<Option<CachedPostPage>, String> {
        Ok(None)
    }
    async fn put_list(&self, _query_hash: &str, _page: &CachedPostPage) -> Result<(), String> {
        Ok(())
    }
    async fn invalidate_all_lists(&self) -> Result<(), String> {
        Ok(())
    }
}
