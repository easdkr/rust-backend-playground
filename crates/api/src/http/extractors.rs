use async_trait::async_trait;
use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;
use crate::http::validation::format_validation_errors;

/// JSON body extractor that maps parse failures to [`AppError::BadRequest`].
pub struct AppJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(AppJson(value)),
            Err(rejection) => Err(AppError::BadRequest(format!(
                "Invalid request body: {}",
                rejection.body_text()
            ))),
        }
    }
}

/// JSON body extractor that parses and runs [`Validate`] before the handler (ValidationPipe-style).
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                AppError::BadRequest(format!("Invalid request body: {}", rejection.body_text()))
            })?;

        value
            .validate()
            .map_err(|errors| AppError::BadRequest(format_validation_errors(&errors)))?;

        Ok(ValidatedJson(value))
    }
}
