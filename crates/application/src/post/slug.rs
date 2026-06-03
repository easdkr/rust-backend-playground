/// 제목/사용자 입력을 URL 친화적인 slug로 변환합니다.
/// - 영숫자(영문·한글·일본어 한자 등 `char::is_alphanumeric`)는 보존합니다.
/// - 공백/하이픈/언더스코어는 단일 하이픈으로 축약합니다.
/// - 양 끝 하이픈은 제거합니다.
pub fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_sep = false;
    for c in input.chars() {
        if c.is_alphanumeric() {
            for low in c.to_lowercase() {
                out.push(low);
            }
            prev_sep = false;
        } else if c.is_whitespace() || c == '-' || c == '_' {
            if !prev_sep && !out.is_empty() {
                out.push('-');
                prev_sep = true;
            }
        }
    }
    out.trim_matches('-').to_string()
}

/// base slug가 이미 사용 중일 때 `-2`, `-3`, ... 형태로 충돌을 회피합니다.
pub async fn ensure_unique<F, Fut>(base: &str, exists: F) -> String
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    if !exists(base.to_string()).await {
        return base.to_string();
    }
    let mut n: usize = 2;
    loop {
        let candidate = format!("{base}-{n}");
        if !exists(candidate.clone()).await {
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_ascii_title() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  Rust  Backend  "), "rust-backend");
        assert_eq!(slugify("foo_bar-baz"), "foo-bar-baz");
    }

    #[test]
    fn slugify_unicode() {
        assert_eq!(slugify("안녕하세요 World"), "안녕하세요-world");
    }

    #[test]
    fn slugify_collapses_separators() {
        assert_eq!(slugify("a   ---   b"), "a-b");
    }

    #[tokio::test]
    async fn ensure_unique_suffixes() {
        let taken = |s: String| async move { s == "rust" || s == "rust-2" || s == "rust-3" };
        assert_eq!(ensure_unique("rust", taken).await, "rust-4");
    }
}
