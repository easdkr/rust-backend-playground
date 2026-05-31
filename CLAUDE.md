# CLAUDE.md

Project instructions for Claude Code and other Anthropic tooling.

> Shared agent rules also live in [AGENTS.md](./AGENTS.md). Prefer that file as the single source of truth when both are present.

## Quick context

Layered Rust workspace: **Tokio · Axum 0.7 · SeaORM 1.1 · PostgreSQL 17 · Valkey 8**.

```
crates/
├── infrastructure/   # persistence, cache, db
├── application/      # services, DTOs, validation
├── api/              # HTTP handlers & routes
├── server/           # API binary (cargo run)
└── batch/            # periodic post-count worker
```

Work only under `crates/`. The root `src/` tree is legacy and must not be modified.

## Before making changes

1. Read the relevant module in each layer (e.g. `post` spans all crates).
2. Run `cargo check` after edits; use `cargo clippy` when touching non-trivial logic.
3. Ensure `docker compose up -d` if you need live DB/Valkey for manual testing.

## Layer responsibilities

| Layer | May import | Must not import |
|:---|:---|:---|
| `infrastructure` | sea-orm, redis, chrono | axum, application, api |
| `application` | infrastructure | axum, api |
| `api` | application, axum | server, batch |
| `server`, `batch` | application, infrastructure, api (server) | — |

## Patterns to follow

### Service (application)

```rust
pub struct PostService {
    repo: Arc<dyn PostRepository>,
}

impl PostService {
    pub async fn create(&self, cmd: CreatePostCmd) -> Result<PostDto, String> {
        validate_title(&cmd.title)?;
        let saved = self.repo.create(cmd.title, cmd.content).await?;
        Ok(PostDto::from(saved))
    }
}
```

### Handler (api)

```rust
pub async fn create_post(
    State(state): State<AppState>,
    Json(payload): Json<CreatePostCmd>,
) -> AppResult<Json<PostDto>> {
    let post = state.post_service.create(payload).await.map_err(AppError::BadRequest)?;
    Ok(Json(post))
}
```

### Wiring (server)

```rust
let post_repo: Arc<dyn PostRepository> = Arc::new(SeaOrmPostRepository::new(db_conn));
let post_service = Arc::new(PostService::new(post_repo));
let app_state = AppState { post_service };
let app = configure_routes(app_state);
```

## Common tasks

### Run locally

```bash
cp .env.example .env
docker compose up -d
cargo run                  # API on :3000
cargo run -p batch         # background worker
```

### Add workspace dependency

Add once in root `Cargo.toml` under `[workspace.dependencies]`, then reference in crate `Cargo.toml`:

```toml
some-crate.workspace = true
```

### Docker full stack

```bash
docker compose --profile app up -d --build
```

## Guardrails

- Never commit `.env` or secrets.
- Keep HTTP concerns out of `application` and `infrastructure`.
- Preserve `Result<_, String>` in services/repos unless migrating the whole stack to a typed error.
- Use `tracing` for logs; default filters are set in each binary's `main.rs`.
- Avoid drive-by refactors, extra abstractions, or README updates unless asked.

## Verification checklist

After substantive changes:

```bash
cargo fmt
cargo check --workspace
cargo clippy --workspace -- -D warnings   # when feasible
cargo test --workspace                    # if tests exist
```

Manual smoke test (API running):

```bash
curl -X POST http://127.0.0.1:3000/posts \
  -H "Content-Type: application/json" \
  -d '{"title":"test","content":"hello"}'
curl http://127.0.0.1:3000/posts
```
