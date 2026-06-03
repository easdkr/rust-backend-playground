use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    Json,
    extract::{FromRef, FromRequest, FromRequestParts, Path, Request},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use application::post::error::PostError;
use application::post::service::PostService;
use application::user::AccessTokenClaims;

use crate::error::AppError;
use crate::http::validation::format_validation_errors;

/// Authenticated user claims populated by JWT middleware.
pub struct AuthenticatedUser {
    pub claims: AccessTokenClaims,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AccessTokenClaims>()
            .cloned()
            .map(|claims| AuthenticatedUser { claims })
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()))
    }
}

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

fn map_post_error(err: PostError) -> AppError {
    match err {
        PostError::NotFound(id) => AppError::NotFound(format!("Post with id {id} not found")),
        PostError::NotFoundBySlug(slug) => {
            AppError::NotFound(format!("Post with slug '{slug}' not found"))
        }
        PostError::OwnershipError => AppError::Forbidden("Not the owner of this post".to_string()),
        PostError::SlugConflict(slug) => {
            AppError::BadRequest(format!("Slug '{slug}' is already taken"))
        }
        PostError::Domain(msg) => AppError::BadRequest(msg),
        PostError::Internal(msg) => {
            tracing::error!("Post operation failed: {msg}");
            AppError::DatabaseError(msg)
        }
    }
}

/// Marker type for post ownership verification via [`OwnershipGuard`].
pub struct PostResource;

#[async_trait]
pub trait ResourceOwnership: Send + Sync + 'static {
    async fn verify(
        parts: &mut Parts,
        user_id: &str,
        post_service: &PostService,
    ) -> Result<(), AppError>;
}

#[async_trait]
impl ResourceOwnership for PostResource {
    async fn verify(
        parts: &mut Parts,
        user_id: &str,
        post_service: &PostService,
    ) -> Result<(), AppError> {
        let Path(id) = Path::<i32>::from_request_parts(parts, &())
            .await
            .map_err(|_| AppError::BadRequest("Invalid post id".to_string()))?;

        let post = post_service.get(id).await.map_err(map_post_error)?;

        match &post.post.user_id {
            Some(owner_id) if owner_id == user_id => Ok(()),
            _ => Err(AppError::Forbidden(
                "Not the owner of this post".to_string(),
            )),
        }
    }
}

/// AOP-style extractor that verifies resource ownership before the handler runs.
pub struct OwnershipGuard<P>(PhantomData<P>);

#[async_trait]
impl<P, S> FromRequestParts<S> for OwnershipGuard<P>
where
    P: ResourceOwnership,
    S: Send + Sync,
    Arc<PostService>: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        let post_service = Arc::<PostService>::from_ref(state);
        P::verify(parts, &user.claims.sub, &post_service).await?;
        Ok(OwnershipGuard(PhantomData))
    }
}
