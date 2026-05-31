use crate::domain::post::entity::Post as DomainPost;
use crate::domain::post::repository::PostRepository;
use super::model::{Entity as PostEntity, Model as DbPost, ActiveModel as DbActivePost};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryOrder, Set};

pub struct SeaOrmPostRepository {
    db: DatabaseConnection,
}

impl SeaOrmPostRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

// Convert DB Model to Domain Entity
fn to_domain(model: DbPost) -> DomainPost {
    DomainPost {
        id: Some(model.id),
        title: model.title,
        content: model.content,
        created_at: model.created_at,
    }
}

#[axum::async_trait]
impl PostRepository for SeaOrmPostRepository {
    async fn find_all(&self) -> Result<Vec<DomainPost>, String> {
        let posts = PostEntity::find()
            .order_by_desc(super::model::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| e.to_string())?;
            
        Ok(posts.into_iter().map(to_domain).collect())
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<DomainPost>, String> {
        let post = PostEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?;
            
        Ok(post.map(to_domain))
    }

    async fn save(&self, post: DomainPost) -> Result<DomainPost, String> {
        if let Some(id) = post.id {
            // Update
            let db_model = DbActivePost {
                id: Set(id),
                title: Set(post.title),
                content: Set(post.content),
                ..Default::default()
            };
            let updated = db_model.update(&self.db).await.map_err(|e| e.to_string())?;
            Ok(to_domain(updated))
        } else {
            // Create
            let db_model = DbActivePost {
                title: Set(post.title),
                content: Set(post.content),
                created_at: Set(post.created_at),
                ..Default::default()
            };
            let inserted = db_model.insert(&self.db).await.map_err(|e| e.to_string())?;
            Ok(to_domain(inserted))
        }
    }

    async fn delete(&self, id: i32) -> Result<bool, String> {
        let res = PostEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?;
            
        Ok(res.rows_affected > 0)
    }
}
