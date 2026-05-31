# AGENTS.md

AI coding agents working in this repository should follow these instructions.

## Project overview

Rust backend playground: **Tokio + Axum + SeaORM (PostgreSQL 17) + Valkey 8**.

Cargo workspace with layered crates. Active code lives under `crates/` — ignore the legacy root `src/` directory.

| Crate | Role |
|:---|:---|
| `infrastructure` | DB connection, SeaORM entities/repos, Valkey cache |
| `application` | Use cases, DTOs, input validation, batch logic |
| `api` | HTTP handlers, routing, API error mapping |
| `server` | HTTP binary entrypoint (`default-members`) |
| `batch` | Background worker binary |

## Dependency rules

Respect crate boundaries. Do not introduce circular dependencies.

```
server / batch  →  api (server only)  →  application  →  infrastructure
```

- **`infrastructure`**: SeaORM, Redis/Valkey, `db::connect`. No Axum or HTTP code.
- **`application`**: Business logic and DTOs. Depends on `infrastructure` for repository traits and models.
- **`api`**: Thin HTTP layer. Handlers delegate to `PostService`; map errors via `AppError`.
- **`server` / `batch`**: Wire dependencies (`Arc<dyn PostRepository>`, services) and run binaries.

When adding a new domain feature, follow the existing `post` module layout in each crate.

## Commands

```bash
# Start DB + Valkey only (local Rust dev)
docker compose up -d

# Run API server (workspace root)
cargo run
cargo run --release

# Run batch worker
cargo run -p batch

# Full stack in containers
docker compose --profile app up -d --build

# Check / test / lint
cargo check
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

Copy `.env.example` to `.env` before local runs. Never commit `.env`.

## Environment variables

| Variable | Default (if unset) | Used by |
|:---|:---|:---|
| `DATABASE_URL` | `postgres://postgres:postgrespassword@127.0.0.1:5433/playground` | server, batch |
| `VALKEY_URL` | none | batch (optional cache sync) |
| `HOST` | `127.0.0.1` | server |
| `PORT` | `3000` | server |
| `RUST_LOG` | crate-specific filter in `main.rs` | server, batch |

## Coding conventions

### Rust style

- Edition **2024**, stable toolchain.
- Shared deps go in root `[workspace.dependencies]`; crate `Cargo.toml` uses `{ workspace = true }`.
- Prefer `tracing` over `println!` for runtime logs.
- Use `async_trait` for repository traits; implementations return `Result<_, String>` (existing pattern).
- Keep handlers thin: extract state → call service → map to JSON/`AppError`.

### Error handling

- **Application layer**: `Result<T, String>` with descriptive messages.
- **API layer**: Map to `AppError` (`NotFound`, `BadRequest`, `DatabaseError`) in `crates/api/src/error.rs`.
- Do not leak internal DB errors to clients; log with `tracing::error!`.

### Repository pattern

- Define trait + SeaORM impl in `crates/infrastructure/src/persistence/seaorm/`.
- Inject as `Arc<dyn PostRepository>` in `server` / `batch` entrypoints.
- Schema init runs in `infrastructure::db::connect` (auto-creates `posts` table).

### Adding a new resource (checklist)

1. SeaORM entity + repository trait/impl → `infrastructure`
2. DTOs + service (+ batch if needed) → `application`
3. Handlers + routes + `AppState` field → `api`
4. Wire in `server/src/main.rs` (and `batch` if applicable)

## Scope and change discipline

- Minimize diff scope; match existing naming and module structure.
- Do not refactor unrelated crates or move repository traits unless explicitly requested.
- Do not add tests unless they cover meaningful behavior or the user asks.
- Do not create commits, push, or edit `README.md` unless requested.

## API reference

| Method | Path | Body |
|:---|:---|:---|
| GET | `/` | — |
| GET | `/posts` | — |
| POST | `/posts` | `{ "title", "content" }` |
| GET | `/posts/:id` | — |
| PUT | `/posts/:id` | `{ "title"?, "content"? }` |
| DELETE | `/posts/:id` | — |
