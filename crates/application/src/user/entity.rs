use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Editor,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    PostCreate,
    PostUpdate,
    PostDelete,
    PostPublish,
    UserManage,
}

impl Role {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Role::Admin),
            "editor" => Some(Role::Editor),
            "user" => Some(Role::User),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Editor => "editor",
            Role::User => "user",
        }
    }

    pub fn permissions(&self) -> Vec<Permission> {
        match self {
            Role::Admin => vec![
                Permission::PostCreate,
                Permission::PostUpdate,
                Permission::PostDelete,
                Permission::PostPublish,
                Permission::UserManage,
            ],
            Role::Editor => vec![
                Permission::PostCreate,
                Permission::PostUpdate,
                Permission::PostDelete,
                Permission::PostPublish,
            ],
            Role::User => vec![],
        }
    }
}
