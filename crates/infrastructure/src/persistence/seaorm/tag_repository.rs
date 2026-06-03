use async_trait::async_trait;
use libs::error::IntoStringErr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};

use super::post_tag::{
    ActiveModel as PostTagActive, Column as PostTagCol, Entity as PostTagEntity,
};
use super::tag::{ActiveModel, Column, Entity, Tag};

#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tag>, String>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Tag>, String>;
    async fn list(&self) -> Result<Vec<Tag>, String>;
    /// 주어진 name/slug 후보에 대해 (없으면 만들고, 있으면 가져오기) 반환합니다.
    async fn upsert(&self, name: String, slug: String) -> Result<Tag, String>;
    async fn attach_to_post(&self, post_id: i32, tag_id: i32) -> Result<(), String>;
    async fn detach_from_post(&self, post_id: i32, tag_id: i32) -> Result<(), String>;
    async fn list_for_post(&self, post_id: i32) -> Result<Vec<Tag>, String>;
    async fn set_for_post(&self, post_id: i32, tag_ids: Vec<i32>) -> Result<Vec<Tag>, String>;
}

pub struct SeaOrmTagRepository {
    db: DatabaseConnection,
}

impl SeaOrmTagRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TagRepository for SeaOrmTagRepository {
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tag>, String> {
        Entity::find()
            .filter(Column::Slug.eq(slug))
            .one(&self.db)
            .await
            .map_err_string()
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Tag>, String> {
        Entity::find_by_id(id).one(&self.db).await.map_err_string()
    }

    async fn list(&self) -> Result<Vec<Tag>, String> {
        Entity::find().all(&self.db).await.map_err_string()
    }

    async fn upsert(&self, name: String, slug: String) -> Result<Tag, String> {
        if let Some(existing) = Entity::find()
            .filter(Column::Slug.eq(&slug))
            .one(&self.db)
            .await
            .map_err_string()?
        {
            return Ok(existing);
        }
        let active = ActiveModel {
            name: Set(name),
            slug: Set(slug),
            ..Default::default()
        };
        active.insert(&self.db).await.map_err_string()
    }

    async fn attach_to_post(&self, post_id: i32, tag_id: i32) -> Result<(), String> {
        let active = PostTagActive {
            post_id: Set(post_id),
            tag_id: Set(tag_id),
        };
        // PK 충돌 시 무시: ON CONFLICT DO NOTHING
        let stmt = sea_orm::sea_query::OnConflict::columns([PostTagCol::PostId, PostTagCol::TagId])
            .do_nothing()
            .to_owned();
        let _ = PostTagEntity::insert(active)
            .on_conflict(stmt)
            .exec(&self.db)
            .await
            .map_err_string()?;
        Ok(())
    }

    async fn detach_from_post(&self, post_id: i32, tag_id: i32) -> Result<(), String> {
        PostTagEntity::delete_many()
            .filter(PostTagCol::PostId.eq(post_id))
            .filter(PostTagCol::TagId.eq(tag_id))
            .exec(&self.db)
            .await
            .map_err_string()?;
        Ok(())
    }

    async fn list_for_post(&self, post_id: i32) -> Result<Vec<Tag>, String> {
        // post_tags 에서 tag_id 목록을 가져온 뒤 각 태그를 조회
        let tag_ids: Vec<i32> = PostTagEntity::find()
            .select_only()
            .column(PostTagCol::TagId)
            .filter(PostTagCol::PostId.eq(post_id))
            .into_tuple::<i32>()
            .all(&self.db)
            .await
            .map_err_string()
            .unwrap_or_default();
        if tag_ids.is_empty() {
            return Ok(Vec::new());
        }
        let tags: Vec<Tag> = Entity::find()
            .filter(Column::Id.is_in(tag_ids))
            .all(&self.db)
            .await
            .map_err_string()
            .unwrap_or_default();
        Ok(tags)
    }

    async fn set_for_post(&self, post_id: i32, tag_ids: Vec<i32>) -> Result<Vec<Tag>, String> {
        // 기존 연결 모두 제거 후 재삽입
        PostTagEntity::delete_many()
            .filter(PostTagCol::PostId.eq(post_id))
            .exec(&self.db)
            .await
            .map_err_string()?;
        for tag_id in &tag_ids {
            self.attach_to_post(post_id, *tag_id).await?;
        }
        let mut tags = Vec::new();
        for tag_id in &tag_ids {
            if let Some(t) = self.find_by_id(*tag_id).await? {
                tags.push(t);
            }
        }
        Ok(tags)
    }
}
