use async_trait::async_trait;
use libs::error::IntoStringErr;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use super::user::{Entity, User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, String>;
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, String>;
    async fn find_all(&self) -> Result<Vec<User>, String>;
}

pub struct SeaOrmUserRepository {
    db: DatabaseConnection,
}

impl SeaOrmUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for SeaOrmUserRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, String> {
        Entity::find()
            .filter(super::user::Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err_string()
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>, String> {
        Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err_string()
    }

    async fn find_all(&self) -> Result<Vec<User>, String> {
        Entity::find()
            .order_by_asc(super::user::Column::Username)
            .all(&self.db)
            .await
            .map_err_string()
    }
}
