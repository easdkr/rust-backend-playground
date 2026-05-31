use axum::{
    extract::{Path, State},
    Json,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryOrder, Set, IntoActiveModel};
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::entities::{prelude::Post, post};
use crate::error::{AppResult, AppError};

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

impl From<post::Model> for PostResponse {
    fn from(model: post::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            content: model.content,
            created_at: model.created_at.to_rfc3339(),
        }
    }
}

// GET /posts
pub async fn list_posts(
    State(db): State<DatabaseConnection>,
) -> AppResult<Json<Vec<PostResponse>>> {
    let posts = Post::find()
        .order_by_desc(post::Column::CreatedAt)
        .all(&db)
        .await?;
        
    let response = posts.into_iter().map(PostResponse::from).collect();
    Ok(Json(response))
}

// POST /posts
pub async fn create_post(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreatePostRequest>,
) -> AppResult<Json<PostResponse>> {
    if payload.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title cannot be empty".to_string()));
    }

    let new_post = post::ActiveModel {
        title: Set(payload.title),
        content: Set(payload.content),
        created_at: Set(Utc::now()),
        ..Default::default()
    };

    let inserted = new_post.insert(&db).await?;
    Ok(Json(PostResponse::from(inserted)))
}

// GET /posts/:id
pub async fn get_post(
    State(db): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> AppResult<Json<PostResponse>> {
    let post = Post::find_by_id(id)
        .one(&db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Post with id {} not found", id)))?;
        
    Ok(Json(PostResponse::from(post)))
}

// PUT /posts/:id
pub async fn update_post(
    State(db): State<DatabaseConnection>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdatePostRequest>,
) -> AppResult<Json<PostResponse>> {
    let post = Post::find_by_id(id)
        .one(&db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Post with id {} not found", id)))?;

    let mut active_model = post.into_active_model();
    
    if let Some(title) = payload.title {
        if title.trim().is_empty() {
            return Err(AppError::BadRequest("Title cannot be empty".to_string()));
        }
        active_model.title = Set(title);
    }
    
    if let Some(content) = payload.content {
        active_model.content = Set(content);
    }

    let updated = active_model.update(&db).await?;
    Ok(Json(PostResponse::from(updated)))
}

// DELETE /posts/:id
pub async fn delete_post(
    State(db): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> AppResult<Json<serde_json::Value>> {
    let delete_result = Post::delete_by_id(id).exec(&db).await?;
    
    if delete_result.rows_affected == 0 {
        return Err(AppError::NotFound(format!("Post with id {} not found", id)));
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Post with id {} deleted successfully", id)
    })))
}
