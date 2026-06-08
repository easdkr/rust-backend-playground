/// JWT 토큰 검증 (공유 로직)
/// 실제로는 application::user::AuthService를 사용하거나,
/// libs에 공통 JWT 검증 모듈을 두는 것이 좋습니다.
pub async fn validate_token(token: &str) -> Result<String, String> {
    // TODO: JWT 시크릿으로 토큰 검증 구현
    // 지금은 단순히 토큰을 user_id로 간주 (개발용)
    if token.is_empty() || token == "null" {
        return Err("Empty token".to_string());
    }
    Ok(token.to_string())
}
