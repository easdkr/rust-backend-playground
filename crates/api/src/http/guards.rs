use application::user::{AccessTokenClaims, Permission, Role};

use crate::error::AppError;
use crate::http::extractors::AuthenticatedUser;

pub fn require_permission(
    user: AuthenticatedUser,
    permission: Permission,
) -> Result<AuthenticatedUser, AppError> {
    if user.claims.permissions.contains(&permission) {
        Ok(user)
    } else {
        Err(AppError::Forbidden(format!(
            "Missing permission: {permission:?}"
        )))
    }
}

pub fn require_role(user: AuthenticatedUser, role: Role) -> Result<AuthenticatedUser, AppError> {
    if user.claims.role == role {
        Ok(user)
    } else {
        Err(AppError::Forbidden(format!("{role:?} role required")))
    }
}

pub fn check_permission(
    claims: &AccessTokenClaims,
    permission: Permission,
) -> Result<(), AppError> {
    if claims.permissions.contains(&permission) {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!(
            "Missing permission: {permission:?}"
        )))
    }
}
