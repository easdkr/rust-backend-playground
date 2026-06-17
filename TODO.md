# 남은 작업 정리

## 현재 완료 상태
- ✅ feat-1: 게시글 좋아요 (Like)
- ✅ feat-2: 댓글 알림 (Comment → Notification)
- ✅ feat-3: 사용자 프로필 확장 (bio, avatar_url, last_login_at)
- ✅ feat-4: Rate Limiting (Redis sliding window)
- ✅ feat-5: 에디터 파일 업로드 (multipart 이미지 업로드)
- ✅ feat-6: PostgreSQL FTS 검색
- ✅ feat-7: 게시글 버전 관리 (Post Revision)

> **참고**: `crates/e2e/tests/comments_e2e.rs`의 댓글 API 테스트는 아직 댓글 엔드포인트가 구현되지 않아 6건이 실패합니다. 본 TODO 항목(feat-6/feat-7)과는 무관한 사전 존재 이슈입니다.

---

## feat-6: PostgreSQL FTS 검색 — 완료

### 수정 사항
- `crates/infrastructure/src/persistence/seaorm/post_repository.rs`
  - `search_fts`의 `row.try_get`를 `try_get_by` / `Option<T>` 기반으로 수정
  - 누락된 `published_at`, `search_vector` 필드 추가
  - 사용자/게시글 컬럼 매핑을 alias 기반으로 정리
  - `to_tsquery`/`to_tsvector` 설정을 `korean` → `simple`로 변경해 기본 PostgreSQL 이미지와 호환
- `crates/application/src/post/dto.rs`: `SearchPostsQuery` 추가
- `crates/application/src/post/service.rs`: `search_fts` 서비스 메서드 추가
- `crates/api/src/http/handlers.rs`: `search_posts` 핸들러 추가
- `crates/api/src/http/routes.rs`: `GET /posts/search` 라우트 및 OpenAPI 등록

## feat-7: 게시글 버전 관리 — 완료

### 수정 사항
- 마이그레이션
  - `migration/src/m20240101_000013_add_version_to_post_revisions.rs` 추가
  - `post_revisions` 테이블에 `version` 컬럼 추가
- 엔티티
  - `crates/infrastructure/src/persistence/seaorm/post_revision.rs` 추가
  - 실제 마이그레이션 스키마와 일치하도록 `editor_id`, `status`, `version` 포함
- 리포지토리
  - `crates/infrastructure/src/persistence/seaorm/post_revision_repository.rs` 추가
  - `save_revision`, `find_by_post_id`, `find_by_id`, `find_by_post_id_and_version` 구현
- 서비스
  - `crates/application/src/post/service.rs`에 `revision_repo` 주입
  - `update` 시 이전 내용을 `post_revisions`에 자동 저장
  - `list_revisions`, `get_revision`, `restore_revision` 메서드 추가
- API
  - `crates/api/src/http/handlers.rs`에 revision 핸들러 추가
  - `crates/api/src/http/routes.rs`에 다음 라우트 등록
    - `GET /posts/:id/revisions`
    - `GET /posts/:id/revisions/:version`
    - `POST /posts/:id/revisions/:version/restore`
- DTO
  - `crates/application/src/post/dto.rs`에 `PostRevisionDto` 추가

## 부수 수정 (빌드/테스트 정리)

- E2E 테스트컨테이너 PostgreSQL 이미지를 `11-alpine` → `15-alpine`로 업그레이드
  - `crates/e2e/src/lib.rs`
- `posts` 조회 시 `users` 테이블 중복 조인으로 인한 500 오류 수정
  - `crates/infrastructure/src/persistence/seaorm/post_repository.rs`
- 삭제된 게시글을 `GET /posts/:id`로 조회할 수 있던 문제 수정
- `PostDto`에 `comment_count`, `has_more_comments` 추가 (기존 E2E 테스트와 맞춤)
- Clippy 경고 및 `cargo fmt` 정리
  - `libs`, `migration`, `infrastructure`, `application`, `api`, `websocket-server`

## 빌드/검증 명령어

```bash
cd /Users/june/workspace/personal/rust-backend-playground
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 현재 테스트 상태

- `cargo test --workspace`: unit + `posts_e2e` + `welcome_e2e` 통과
- `comments_e2e`: 댓글 엔드포인트 미구현으로 6건 실패 (사전 존재 이슈)
