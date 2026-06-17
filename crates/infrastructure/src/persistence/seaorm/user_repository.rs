use async_trait::async_trait;
use chrono::Utc;
use libs::error::IntoStringErr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use super::user::{Entity, User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, String>;
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, String>;
    async fn find_all(&self) -> Result<Vec<User>, String>;
    async fn create_user(
        &self,
        id: &str,
        username: &str,
        email: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<User, String>;
    async fn update_profile(
        &self,
        id: &str,
        bio: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<User, String>;
    async fn update_last_login(&self, id: &str) -> Result<(), String>;
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

    async fn create_user(
        &self,
        id: &str,
        username: &str,
        email: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<User, String> {
        let now = Utc::now();
        let am = super::user::ActiveModel {
            id: Set(id.to_string()),
            username: Set(username.to_string()),
            email: Set(email.to_string()),
            password_hash: Set(password_hash.to_string()),
            role: Set(role.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            bio: Set(None),
            avatar_url: Set(None),
            last_login_at: Set(None),
        };
        am.insert(&self.db).await.map_err_string()
    }

    async fn update_profile(
        &self,
        id: &str,
        bio: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<User, String> {
        let user = self.find_by_id(id).await?.ok_or("User not found")?;
        let mut am: super::user::ActiveModel = user.into();
        am.bio = Set(bio);
        am.avatar_url = Set(avatar_url);
        am.updated_at = Set(Utc::now());
        am.update(&self.db).await.map_err_string()
    }

    async fn update_last_login(&self, id: &str) -> Result<(), String> {
        let user = self.find_by_id(id).await?.ok_or("User not found")?;
        let mut am: super::user::ActiveModel = user.into();
        am.last_login_at = Set(Some(Utc::now()));
        am.update(&self.db).await.map_err_string()?;
        Ok(())
    }
}
