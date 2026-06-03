use std::sync::Arc;

use chrono::{DateTime, Utc};

use infrastructure::persistence::seaorm::tag::Tag;
use infrastructure::persistence::seaorm::tag_repository::TagRepository;

use super::dto::TagDto;
use super::super::post::slug;

pub struct TagService {
    repo: Arc<dyn TagRepository>,
}

impl TagService {
    pub fn new(repo: Arc<dyn TagRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(&self) -> Result<Vec<TagDto>, String> {
        let tags = self.repo.list().await?;
        Ok(tags.into_iter().map(into_dto).collect())
    }

    pub async fn create(&self, name: String, slug_in: Option<String>) -> Result<TagDto, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("tag name must not be empty".to_string());
        }
        let slug_value = match slug_in {
            Some(s) if !s.is_empty() => s,
            _ => slug::slugify(trimmed),
        };
        if slug_value.is_empty() {
            return Err("cannot derive slug from tag name".to_string());
        }
        let tag = self.repo.upsert(trimmed.to_string(), slug_value).await?;
        Ok(into_dto(tag))
    }

    pub async fn list_for_post(&self, post_id: i32) -> Result<Vec<TagDto>, String> {
        let tags = self.repo.list_for_post(post_id).await?;
        Ok(tags.into_iter().map(into_dto).collect())
    }

    pub async fn set_for_post(
        &self,
        post_id: i32,
        tag_ids: Vec<i32>,
    ) -> Result<Vec<TagDto>, String> {
        let tags = self.repo.set_for_post(post_id, tag_ids).await?;
        Ok(tags.into_iter().map(into_dto).collect())
    }
}

fn into_dto(t: Tag) -> TagDto {
    TagDto {
        id: t.id,
        name: t.name,
        slug: t.slug,
        created_at: format_dt(t.created_at),
    }
}

fn format_dt(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}
