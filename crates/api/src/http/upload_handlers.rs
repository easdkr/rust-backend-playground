use std::path::PathBuf;

use axum::{Json, extract::Multipart};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::http::extractors::AuthenticatedUser;

#[derive(Debug, Clone, serde::Serialize)]
pub struct UploadResponse {
    pub url: String,
    pub filename: String,
    pub size: usize,
}

pub async fn upload_image(
    _user: AuthenticatedUser,
    mut multipart: Multipart,
) -> AppResult<Json<UploadResponse>> {
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
    fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create upload directory: {e}")))?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {e}")))?
    {
        let name = field.name().unwrap_or("unknown").to_string();
        if name != "image" {
            continue;
        }

        let filename = field.file_name().unwrap_or("unnamed").to_string();
        let ext = PathBuf::from(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin")
            .to_lowercase();

        // Only allow image extensions
        let allowed = ["jpg", "jpeg", "png", "gif", "webp", "svg"];
        if !allowed.contains(&ext.as_str()) {
            return Err(AppError::BadRequest(format!(
                "Invalid file extension: {ext}. Allowed: {allowed:?}"
            )));
        }

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read file bytes: {e}")))?;

        let size = data.len();
        if size > 10 * 1024 * 1024 {
            return Err(AppError::BadRequest(
                "File too large. Max 10MB allowed.".to_string(),
            ));
        }

        let stored_name = format!("{}.{}", Uuid::new_v4(), ext);
        let path = PathBuf::from(&upload_dir).join(&stored_name);

        let mut file = fs::File::create(&path)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create file: {e}")))?;
        file.write_all(&data)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to write file: {e}")))?;

        let base_url = std::env::var("UPLOAD_BASE_URL").unwrap_or_else(|_| "/uploads".to_string());
        let url = format!("{}/{}", base_url, stored_name);

        return Ok(Json(UploadResponse {
            url,
            filename: stored_name,
            size,
        }));
    }

    Err(AppError::BadRequest(
        "No image field found in multipart".to_string(),
    ))
}
