use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use infrastructure::cache::post_cache::PostCache;
use infrastructure::persistence::seaorm::post_repository::{PostListFilter, PostRepository};
use infrastructure::persistence::seaorm::post_revision_repository::PostRevisionRepository;
use serde::{Deserialize, Serialize};

use super::dto::{
    BulkPostAction, BulkPostResult, BulkPostResultItem, ListPostsQuery, PostDto, PostRevisionDto,
    SearchPostsQuery,
};
use super::entity::{Post, PostAuthor, PostWithAuthor};
use super::error::PostError;
use super::mapper::domain_from_record;
use super::slug;

const POST_KEY_PREFIX: &str = "playground:post:";
const POST_TTL_SECS: u64 = 300;
const LIST_KEY_PREFIX: &str = "playground:posts:list:";
const LIST_TTL_SECS: u64 = 60;

#[derive(Serialize, Deserialize)]
struct CachedPage {
    data: Vec<PostWithAuthor>,
    next_cursor: Option<i32>,
    has_more: bool,
}

pub struct PostService {
    repo: Arc<dyn PostRepository>,
    cache: Arc<dyn PostCache>,
    revision_repo: Option<Arc<dyn PostRevisionRepository>>,
}

impl PostService {
    pub fn new(
        repo: Arc<dyn PostRepository>,
        cache: Arc<dyn PostCache>,
        revision_repo: Option<Arc<dyn PostRevisionRepository>>,
    ) -> Self {
        Self {
            repo,
            cache,
            revision_repo,
        }
    }

    pub async fn create(
        &self,
        current_user_id: String,
        cmd: super::dto::CreatePostCmd,
    ) -> Result<PostWithAuthor, PostError> {
        let mut post = cmd.into_domain(current_user_id.clone());
        post.slug = Some(
            self.ensure_unique_slug(post.slug.as_deref(), post.title.as_str(), None)
                .await?,
        );
        let saved = self
            .repo
            .create(
                post.title,
                post.content,
                post.status.as_str().to_string(),
                current_user_id,
                post.slug.clone(),
                post.excerpt.clone(),
            )
            .await
            .map_err(PostError::repo)?;
        let domain = domain_from_record(saved)?;
        let with_author = PostWithAuthor {
            post: domain,
            author: None,
        };
        self.write_through(&with_author).await;
        self.invalidate_lists().await;
        Ok(with_author)
    }

    pub async fn list(&self) -> Result<Vec<Post>, PostError> {
        let records = self.repo.find_all().await.map_err(PostError::repo)?;
        records.into_iter().map(domain_from_record).collect()
    }

    pub async fn find_many(
        &self,
        query: ListPostsQuery,
    ) -> Result<crate::pagination::CursorPage<PostDto>, PostError> {
        query.validate().map_err(PostError::Domain)?;

        let hash = self.query_hash(&query);
        let key = format!("{LIST_KEY_PREFIX}{hash}");
        if let Ok(Some(raw)) = self.cache.get(&key).await
            && let Ok(page) = serde_json::from_str::<CachedPage>(&raw)
        {
            return Ok(self.to_cursor_page(page));
        }

        let filter = PostListFilter {
            cursor: query.cursor,
            limit: query.limit,
            status: query.status.map(|s| s.as_str().to_string()),
            author_id: query.author_id,
            q: query.q,
            sort: query.sort.as_str().to_string(),
            include_deleted: query.include_deleted.unwrap_or(false),
        };
        let result = self
            .repo
            .find_many_with_author(filter)
            .await
            .map_err(PostError::repo)?;

        let page = CachedPage {
            data: result
                .data
                .into_iter()
                .map(|row| self.row_to_with_author(row))
                .collect(),
            next_cursor: result.next_cursor,
            has_more: result.has_more,
        };
        if let Ok(json) = serde_json::to_string(&page) {
            let _ = self.cache.put(&key, &json, LIST_TTL_SECS).await;
        }

        Ok(self.to_cursor_page(page))
    }

    pub async fn search_fts(
        &self,
        query: SearchPostsQuery,
    ) -> Result<crate::pagination::CursorPage<PostDto>, PostError> {
        query.validate().map_err(PostError::Domain)?;

        let result = self
            .repo
            .search_fts(&query.q, query.cursor, query.limit)
            .await
            .map_err(PostError::repo)?;

        let page = CachedPage {
            data: result
                .data
                .into_iter()
                .map(|row| self.row_to_with_author(row))
                .collect(),
            next_cursor: result.next_cursor,
            has_more: result.has_more,
        };

        Ok(self.to_cursor_page(page))
    }

    pub async fn get(&self, id: i32) -> Result<PostWithAuthor, PostError> {
        self.get_with_author(id).await
    }

    pub async fn get_with_author(&self, id: i32) -> Result<PostWithAuthor, PostError> {
        let key = format!("{POST_KEY_PREFIX}{id}");
        if let Ok(Some(raw)) = self.cache.get(&key).await
            && let Ok(wa) = serde_json::from_str::<PostWithAuthor>(&raw)
        {
            return Ok(wa);
        }
        let row = self
            .repo
            .find_by_id_with_author(id)
            .await
            .map_err(PostError::repo)?
            .ok_or(PostError::NotFound(id))?;
        let wa = self.row_to_with_author(row);
        self.write_through(&wa).await;
        Ok(wa)
    }

    pub async fn get_by_slug(&self, slug_value: &str) -> Result<PostWithAuthor, PostError> {
        let row = self
            .repo
            .find_by_slug_with_author(slug_value)
            .await
            .map_err(PostError::repo)?
            .ok_or_else(|| PostError::NotFoundBySlug(slug_value.to_string()))?;
        Ok(self.row_to_with_author(row))
    }

    pub async fn update(
        &self,
        id: i32,
        current_user_id: &str,
        cmd: super::dto::UpdatePostCmd,
    ) -> Result<PostWithAuthor, PostError> {
        let record = self
            .repo
            .find_by_id(id)
            .await
            .map_err(PostError::repo)?
            .ok_or(PostError::NotFound(id))?;
        let mut post = domain_from_record(record.clone())?;
        Self::verify_ownership(&post, current_user_id)?;

        if let Some(ref revision_repo) = self.revision_repo {
            revision_repo
                .save_revision(
                    id,
                    current_user_id.to_string(),
                    record.title,
                    record.content,
                    record.excerpt,
                    record.status,
                )
                .await
                .map_err(PostError::repo)?;
        }

        if let Some(ref requested) = cmd.slug {
            post.slug = Some(
                self.ensure_unique_slug(Some(requested.as_str()), post.title.as_str(), Some(id))
                    .await?,
            );
        }

        post.apply_update(cmd.title, cmd.content, cmd.excerpt, None);

        let saved = self
            .repo
            .update(
                id,
                post.title.clone(),
                post.content.clone(),
                post.excerpt.clone(),
                post.slug.clone(),
                post.status.as_str().to_string(),
            )
            .await
            .map_err(PostError::repo)?;
        let updated = domain_from_record(saved)?;
        let with_author = PostWithAuthor {
            post: updated,
            author: None,
        };
        self.write_through(&with_author).await;
        self.invalidate_lists().await;
        Ok(with_author)
    }

    pub async fn list_revisions(&self, post_id: i32) -> Result<Vec<PostRevisionDto>, PostError> {
        self.repo
            .find_by_id(post_id)
            .await
            .map_err(PostError::repo)?
            .ok_or(PostError::NotFound(post_id))?;

        let Some(ref revision_repo) = self.revision_repo else {
            return Ok(Vec::new());
        };

        let revisions = revision_repo
            .find_by_post_id(post_id)
            .await
            .map_err(PostError::repo)?;
        Ok(revisions.into_iter().map(PostRevisionDto::from).collect())
    }

    pub async fn get_revision(
        &self,
        post_id: i32,
        version: i32,
    ) -> Result<PostRevisionDto, PostError> {
        self.repo
            .find_by_id(post_id)
            .await
            .map_err(PostError::repo)?
            .ok_or(PostError::NotFound(post_id))?;

        let Some(ref revision_repo) = self.revision_repo else {
            return Err(PostError::Domain(
                "Revision history is disabled".to_string(),
            ));
        };

        let revision = revision_repo
            .find_by_post_id_and_version(post_id, version)
            .await
            .map_err(PostError::repo)?
            .ok_or_else(|| {
                PostError::Domain(format!("Revision {version} not found for post {post_id}"))
            })?;
        Ok(PostRevisionDto::from(revision))
    }

    pub async fn restore_revision(
        &self,
        post_id: i32,
        version: i32,
        current_user_id: &str,
    ) -> Result<PostWithAuthor, PostError> {
        let record = self
            .repo
            .find_by_id(post_id)
            .await
            .map_err(PostError::repo)?
            .ok_or(PostError::NotFound(post_id))?;
        let post = domain_from_record(record.clone())?;
        Self::verify_ownership(&post, current_user_id)?;

        let Some(ref revision_repo) = self.revision_repo else {
            return Err(PostError::Domain(
                "Revision history is disabled".to_string(),
            ));
        };

        let revision = revision_repo
            .find_by_post_id_and_version(post_id, version)
            .await
            .map_err(PostError::repo)?
            .ok_or_else(|| {
                PostError::Domain(format!("Revision {version} not found for post {post_id}"))
            })?;

        // 현재 상태를 먼저 버전으로 보존한 뒨 선택한 리비전으로 되돌립니다.
        revision_repo
            .save_revision(
                post_id,
                current_user_id.to_string(),
                record.title,
                record.content,
                record.excerpt,
                record.status,
            )
            .await
            .map_err(PostError::repo)?;

        let saved = self
            .repo
            .update(
                post_id,
                revision.title,
                revision.content,
                revision.excerpt,
                post.slug.clone(),
                revision.status,
            )
            .await
            .map_err(PostError::repo)?;
        let updated = domain_from_record(saved)?;
        let with_author = PostWithAuthor {
            post: updated,
            author: None,
        };
        self.write_through(&with_author).await;
        self.invalidate_lists().await;
        Ok(with_author)
    }

    pub async fn publish(&self, id: i32) -> Result<PostWithAuthor, PostError> {
        self.mutate_status(id, |p| p.publish()).await
    }

    pub async fn unpublish(&self, id: i32) -> Result<PostWithAuthor, PostError> {
        self.mutate_status(id, |p| p.unpublish()).await
    }

    pub async fn archive(&self, id: i32) -> Result<PostWithAuthor, PostError> {
        self.mutate_status(id, |p| p.archive()).await
    }

    pub async fn restore(&self, id: i32) -> Result<PostWithAuthor, PostError> {
        self.mutate_status(id, |p| p.restore()).await
    }

    async fn mutate_status<F>(&self, id: i32, f: F) -> Result<PostWithAuthor, PostError>
    where
        F: FnOnce(&mut Post) -> Result<(), PostError>,
    {
        let record = self
            .repo
            .find_by_id(id)
            .await
            .map_err(PostError::repo)?
            .ok_or(PostError::NotFound(id))?;
        let mut post = domain_from_record(record)?;
        f(&mut post)?;
        let saved = self
            .repo
            .update(
                id,
                post.title.clone(),
                post.content.clone(),
                post.excerpt.clone(),
                post.slug.clone(),
                post.status.as_str().to_string(),
            )
            .await
            .map_err(PostError::repo)?;
        let updated = domain_from_record(saved)?;
        let with_author = PostWithAuthor {
            post: updated,
            author: None,
        };
        self.write_through(&with_author).await;
        self.invalidate_lists().await;
        Ok(with_author)
    }

    pub async fn delete(&self, id: i32, current_user_id: &str) -> Result<(), PostError> {
        let record = self
            .repo
            .find_by_id(id)
            .await
            .map_err(PostError::repo)?
            .ok_or(PostError::NotFound(id))?;
        let post = domain_from_record(record)?;
        Self::verify_ownership(&post, current_user_id)?;

        let deleted = self.repo.soft_delete(id).await.map_err(PostError::repo)?;
        if !deleted {
            return Err(PostError::NotFound(id));
        }
        self.cache
            .invalidate(&format!("{POST_KEY_PREFIX}{id}"))
            .await
            .ok();
        self.invalidate_lists().await;
        Ok(())
    }

    /// hard delete — 관리자/배치에서 사용. 호출자는 권한을 검증해야 합니다.
    pub async fn hard_delete(&self, id: i32) -> Result<(), PostError> {
        let deleted = self.repo.delete(id).await.map_err(PostError::repo)?;
        if !deleted {
            return Err(PostError::NotFound(id));
        }
        self.cache
            .invalidate(&format!("{POST_KEY_PREFIX}{id}"))
            .await
            .ok();
        self.invalidate_lists().await;
        Ok(())
    }

    pub async fn increment_view(&self, id: i32) -> Result<(), PostError> {
        self.repo
            .increment_view_count(id)
            .await
            .map_err(PostError::repo)?;
        self.cache
            .invalidate(&format!("{POST_KEY_PREFIX}{id}"))
            .await
            .ok();
        Ok(())
    }

    pub async fn bulk(
        &self,
        current_user_id: &str,
        ids: Vec<i32>,
        action: BulkPostAction,
    ) -> Result<BulkPostResult, PostError> {
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();
        for id in ids {
            match self.bulk_one(current_user_id, id, action).await {
                Ok(()) => succeeded.push(id),
                Err(e) => failed.push(BulkPostResultItem {
                    id,
                    success: false,
                    error: Some(e.to_string()),
                }),
            }
        }
        Ok(BulkPostResult { succeeded, failed })
    }

    async fn bulk_one(
        &self,
        current_user_id: &str,
        id: i32,
        action: BulkPostAction,
    ) -> Result<(), PostError> {
        match action {
            BulkPostAction::Publish => {
                self.publish(id).await?;
            }
            BulkPostAction::Unpublish => {
                self.verify_ownership_for(id, current_user_id).await?;
                self.unpublish(id).await?;
            }
            BulkPostAction::Archive => {
                self.verify_ownership_for(id, current_user_id).await?;
                self.archive(id).await?;
            }
            BulkPostAction::Restore => {
                self.verify_ownership_for(id, current_user_id).await?;
                self.restore(id).await?;
            }
            BulkPostAction::Delete => {
                self.delete(id, current_user_id).await?;
            }
        }
        Ok(())
    }

    pub async fn verify_ownership_for(
        &self,
        id: i32,
        current_user_id: &str,
    ) -> Result<(), PostError> {
        let record = self
            .repo
            .find_by_id(id)
            .await
            .map_err(PostError::repo)?
            .ok_or(PostError::NotFound(id))?;
        let post = domain_from_record(record)?;
        Self::verify_ownership(&post, current_user_id)
    }

    async fn ensure_unique_slug(
        &self,
        requested: Option<&str>,
        title: &str,
        exclude_id: Option<i32>,
    ) -> Result<String, PostError> {
        let base = match requested {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => slug::slugify(title),
        };
        if base.is_empty() {
            return Err(PostError::Domain(
                "Cannot derive a slug from the given title".to_string(),
            ));
        }
        let exists = |candidate: String| {
            let repo = Arc::clone(&self.repo);
            let exclude = exclude_id;
            async move { repo.slug_exists(&candidate, exclude).await.unwrap_or(false) }
        };
        let unique = slug::ensure_unique(&base, exists).await;
        if self
            .repo
            .slug_exists(&unique, exclude_id)
            .await
            .unwrap_or(false)
        {
            return Err(PostError::SlugConflict(unique));
        }
        Ok(unique)
    }

    fn row_to_with_author(
        &self,
        row: infrastructure::persistence::seaorm::post_repository::PostWithAuthorRow,
    ) -> PostWithAuthor {
        let post = match domain_from_record(row.post) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to map post record: {e}");
                Post::new(String::new(), String::new(), String::new())
            }
        };
        let author = row.author.map(|u| PostAuthor {
            id: u.id,
            username: u.username,
        });
        PostWithAuthor { post, author }
    }

    fn verify_ownership(post: &Post, current_user_id: &str) -> Result<(), PostError> {
        match &post.user_id {
            Some(owner_id) if owner_id == current_user_id => Ok(()),
            _ => Err(PostError::OwnershipError),
        }
    }

    async fn write_through(&self, wa: &PostWithAuthor) {
        let Some(id) = wa.post.id else { return };
        let key = format!("{POST_KEY_PREFIX}{id}");
        if let Ok(json) = serde_json::to_string(wa) {
            let _ = self.cache.put(&key, &json, POST_TTL_SECS).await;
        }
    }

    async fn invalidate_lists(&self) {
        let _ = self.cache.invalidate_list(LIST_KEY_PREFIX).await;
    }

    fn query_hash(&self, query: &ListPostsQuery) -> String {
        let mut hasher = DefaultHasher::new();
        query.cursor.hash(&mut hasher);
        query.limit.hash(&mut hasher);
        query.status.hash(&mut hasher);
        query.author_id.hash(&mut hasher);
        query.q.hash(&mut hasher);
        query.sort.hash(&mut hasher);
        query.include_deleted.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn to_cursor_page(&self, page: CachedPage) -> crate::pagination::CursorPage<PostDto> {
        let data = page.data.into_iter().map(PostDto::from).collect();
        crate::pagination::CursorPage::new(data, page.next_cursor, page.has_more)
    }
}
