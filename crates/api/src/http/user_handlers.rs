use std::sync::Arc;

use application::user::dto::UpdateProfileCmd;
use application::user::service::UserService;
use axum::{Json, extract::State};

use crate::error::{AppError, AppResult};
use crate::http::extractors::AuthenticatedUser;

pub async fn get_profile(
    State(user_service): State<Arc<UserService>>,
    user: AuthenticatedUser,
) -> AppResult<Json<application::user::dto::UserDto>> {
    let users = user_service
        .list_users()
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let me = users
        .into_iter()
        .find(|u| u.id == user.claims.sub)
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(me))
}

pub async fn update_profile(
    State(user_service): State<Arc<UserService>>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateProfileCmd>,
) -> AppResult<Json<application::user::dto::UserDto>> {
    let updated = user_service
        .update_profile(&user.claims.sub, payload)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(updated))
}
