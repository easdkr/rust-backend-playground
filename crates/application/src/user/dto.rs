use chrono::{DateTime, Utc};
use libs::env::env_u64;
use serde::{Deserialize, Serialize};

use super::entity::{Permission, Role};

#[derive(Debug, Deserialize)]
pub struct LoginCmd {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterCmd {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,
    pub role: Role,
    pub permissions: Vec<Permission>,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String,
    pub exp: usize,
    pub jti: String,
}

#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: DateTime<Utc>,
    pub refresh_token_expires_at: DateTime<Utc>,
    pub refresh_token_jti: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: Role,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProfileCmd {
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_expiry_secs: u64,
    pub refresh_expiry_secs: u64,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-key-min-32-chars-long!!".to_string()),
            access_expiry_secs: env_u64("JWT_ACCESS_EXPIRY", 900),
            refresh_expiry_secs: env_u64("JWT_REFRESH_EXPIRY", 604_800),
        }
    }
}

impl TokenPair {
    pub fn into_response(self, access_expiry_secs: u64) -> TokenResponse {
        TokenResponse {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            token_type: "Bearer",
            expires_in: access_expiry_secs,
        }
    }
}
