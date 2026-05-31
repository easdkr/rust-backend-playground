use std::sync::Arc;

use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::RngCore;

use crate::password;
use infrastructure::persistence::seaorm::user_repository::UserRepository;

use super::dto::{AccessTokenClaims, JwtConfig, LoginCmd, RefreshTokenClaims, TokenPair, UserDto};
use super::entity::{Permission, Role, User};
use super::error::AuthError;
use super::token_repository::TokenRepository;

fn user_from_record(
    record: infrastructure::persistence::seaorm::user::User,
) -> Result<User, AuthError> {
    let role = Role::parse(&record.role).ok_or_else(|| {
        AuthError::internal(format!("Invalid role stored for user {}", record.id))
    })?;

    Ok(User {
        id: record.id,
        username: record.username,
        email: record.email,
        password_hash: record.password_hash,
        role,
    })
}

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    token_repo: Arc<dyn TokenRepository>,
    jwt_config: JwtConfig,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        token_repo: Arc<dyn TokenRepository>,
        jwt_config: JwtConfig,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            jwt_config,
        }
    }

    pub fn jwt_config(&self) -> &JwtConfig {
        &self.jwt_config
    }

    pub async fn login(&self, cmd: LoginCmd) -> Result<TokenPair, AuthError> {
        let record = self
            .user_repo
            .find_by_username(&cmd.username)
            .await
            .map_err(AuthError::internal)?
            .ok_or(AuthError::InvalidCredentials)?;

        let user = user_from_record(record)?;

        verify_password(&cmd.password, &user.password_hash)?;

        self.issue_token_pair(&user).await
    }

    pub async fn logout(
        &self,
        access_token: &str,
        refresh_token: Option<&str>,
    ) -> Result<(), AuthError> {
        let claims = self.decode_access_token(access_token)?;

        if self
            .token_repo
            .is_blacklisted(&claims.jti)
            .await
            .map_err(AuthError::internal)?
        {
            return Err(AuthError::TokenBlacklisted);
        }

        let remaining_ttl = remaining_ttl_secs(claims.exp);

        if remaining_ttl > 0 {
            self.token_repo
                .blacklist_access_token(&claims.jti, remaining_ttl)
                .await
                .map_err(AuthError::internal)?;
        }

        if let Some(refresh) = refresh_token
            && let Ok(refresh_claims) = self.decode_refresh_token(refresh)
        {
            self.token_repo
                .revoke_refresh_token(&refresh_claims.jti)
                .await
                .map_err(AuthError::internal)?;
        }

        Ok(())
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<TokenPair, AuthError> {
        let claims = self.decode_refresh_token(refresh_token)?;

        let valid = self
            .token_repo
            .verify_refresh_token(&claims.jti, &claims.sub)
            .await
            .map_err(AuthError::internal)?;

        if !valid {
            return Err(AuthError::TokenInvalid(
                "Refresh token not found or revoked".to_string(),
            ));
        }

        let record = self
            .user_repo
            .find_by_id(&claims.sub)
            .await
            .map_err(AuthError::internal)?
            .ok_or(AuthError::UserNotFound)?;

        let user = user_from_record(record)?;

        self.token_repo
            .revoke_refresh_token(&claims.jti)
            .await
            .map_err(AuthError::internal)?;

        self.issue_token_pair(&user).await
    }

    pub async fn validate_access_token(&self, token: &str) -> Result<AccessTokenClaims, AuthError> {
        let claims = self.decode_access_token(token)?;

        if self
            .token_repo
            .is_blacklisted(&claims.jti)
            .await
            .map_err(AuthError::internal)?
        {
            return Err(AuthError::TokenBlacklisted);
        }

        Ok(claims)
    }

    pub fn has_permission(&self, claims: &AccessTokenClaims, permission: Permission) -> bool {
        claims.permissions.contains(&permission)
    }

    async fn issue_token_pair(&self, user: &User) -> Result<TokenPair, AuthError> {
        let now = Utc::now();
        let access_expires_at =
            now + chrono::Duration::seconds(self.jwt_config.access_expiry_secs as i64);
        let refresh_expires_at =
            now + chrono::Duration::seconds(self.jwt_config.refresh_expiry_secs as i64);

        let access_jti = generate_jti();
        let refresh_jti = generate_jti();

        let permissions = user.role.permissions();

        let access_claims = AccessTokenClaims {
            sub: user.id.clone(),
            role: user.role,
            permissions,
            exp: access_expires_at.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: access_jti,
        };

        let refresh_claims = RefreshTokenClaims {
            sub: user.id.clone(),
            exp: refresh_expires_at.timestamp() as usize,
            jti: refresh_jti.clone(),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_config.secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenInvalid(e.to_string()))?;

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_config.secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenInvalid(e.to_string()))?;

        self.token_repo
            .store_refresh_token(&refresh_jti, &user.id, self.jwt_config.refresh_expiry_secs)
            .await
            .map_err(AuthError::internal)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            access_token_expires_at: access_expires_at,
            refresh_token_expires_at: refresh_expires_at,
            refresh_token_jti: refresh_jti,
        })
    }

    fn decode_access_token(&self, token: &str) -> Result<AccessTokenClaims, AuthError> {
        let mut validation = Validation::default();
        validation.validate_exp = true;

        decode::<AccessTokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(map_jwt_error)
    }

    fn decode_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, AuthError> {
        let mut validation = Validation::default();
        validation.validate_exp = true;

        decode::<RefreshTokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(map_jwt_error)
    }
}

pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn list_users(&self) -> Result<Vec<super::dto::UserDto>, AuthError> {
        let users = self
            .user_repo
            .find_all()
            .await
            .map_err(AuthError::internal)?;

        Ok(users
            .into_iter()
            .filter_map(|record| user_from_record(record).ok())
            .map(|u| UserDto {
                id: u.id,
                username: u.username,
                email: u.email,
                role: u.role,
            })
            .collect())
    }
}

fn verify_password(password: &str, password_hash: &str) -> Result<(), AuthError> {
    match password::verify_password(password, password_hash) {
        Ok(true) => Ok(()),
        Ok(false) => Err(AuthError::InvalidCredentials),
        Err(e) => Err(AuthError::internal(e)),
    }
}

fn generate_jti() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn remaining_ttl_secs(exp: usize) -> u64 {
    let now = Utc::now().timestamp();
    let exp = exp as i64;
    (exp - now).max(0) as u64
}

fn map_jwt_error(err: jsonwebtoken::errors::Error) -> AuthError {
    use jsonwebtoken::errors::ErrorKind;

    match err.kind() {
        ErrorKind::ExpiredSignature => AuthError::TokenExpired,
        _ => AuthError::TokenInvalid(err.to_string()),
    }
}
