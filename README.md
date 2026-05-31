# 🦀 Rust Backend Playground (PostgreSQL 17 + Valkey)

Tokio + Axum + SeaORM(PostgreSQL) + Valkey 기반의 고성능 비동기 Rust 백엔드 플레이그라운드입니다.

레이어드 workspace 구조(`application` → `infrastructure` / `api` → `server`·`batch`)로 crate가 분리되어 있으며, Docker Compose로 PostgreSQL 17과 Valkey 8을 로컬에서 띄울 수 있습니다.

---

## 🛠️ 기술 스택

- **비동기 런타임**: [Tokio](https://tokio.rs/)
- **웹 프레임워크**: [Axum v0.7](https://github.com/tokio-rs/axum)
- **ORM**: [SeaORM v1.1](https://www.sea-ql.org/SeaORM/) + **PostgreSQL 17**
- **캐시**: **Valkey 8** (`redis` 크레이트 호환)
- **로깅**: [Tracing](https://github.com/tokio-rs/tracing)
- **직렬화**: [Serde](https://serde.rs/)

---

## 📋 사전 요구 사항

| 용도 | 필요 도구 |
|:---|:---|
| 로컬 Rust 실행 | [Rust](https://rustup.rs/) (stable), Docker (DB/캐시용) |
| 컨테이너 전체 실행 | Docker, Docker Compose v2 |

---

## 🚀 로컬에서 실행 (권장 개발 흐름)

### 1. 환경 변수 준비

```bash
cp .env.example .env
```

`.env` 기본값은 Docker Compose로 띄운 PostgreSQL(`127.0.0.1:5433`)·Valkey(`127.0.0.1:6379`)에 맞춰져 있습니다.

### 2. DB & 캐시만 컨테이너로 기동

```bash
docker compose up -d
```

| 서비스 | 접속 |
|:---|:---|
| PostgreSQL 17 | `127.0.0.1:5433` — user `postgres`, password `postgrespassword`, DB `playground` |
| Valkey 8 | `127.0.0.1:6379` |

### 3. Rust 서버 실행

워크스페이스 루트에서:

```bash
cargo run
# 또는 릴리스 빌드
cargo run --release
```

기동 시 `posts` 테이블이 없으면 자동 생성됩니다.

```text
INFO Starting Rust Backend Playground...
INFO Connecting to database...
INFO Database connected and schema initialized successfully!
INFO Listening on http://127.0.0.1:3000
```

브라우저: [http://127.0.0.1:3000](http://127.0.0.1:3000)

### 4. 배치 워커 실행 (선택)

DB/캐시가 떠 있는 상태에서, 주기적으로 포스트 개수를 집계하고 `VALKEY_URL`이 있으면 `playground:posts:count` 키에 동기화합니다.

```bash
cargo run -p batch
```

### 5. 인프라 중지

```bash
docker compose down
# 볼륨까지 삭제: docker compose down -v
```

---

## 🐳 컨테이너로 전체 스택 실행

PostgreSQL, Valkey, **Rust API 서버**, **배치 워커**를 한 번에 띄웁니다.

```bash
docker compose --profile app up -d --build
```

| 서비스 | 접속 |
|:---|:---|
| API | [http://127.0.0.1:3000](http://127.0.0.1:3000) |
| Batch | 백그라운드 워커 (30초마다 포스트 수 → Valkey) |
| PostgreSQL | 호스트 `127.0.0.1:5433` (컨테이너 내부에서는 `postgres:5432`) |
| Valkey | 호스트 `127.0.0.1:6379` (컨테이너 내부에서는 `valkey:6379`) |

로그 확인:

```bash
docker compose logs -f app
docker compose logs -f batch
```

중지:

```bash
docker compose --profile app down
```

> **참고**: `app` 서비스는 `profiles: ["app"]`로 분리되어 있어, `docker compose up -d`만 실행하면 DB/캐시만 올라갑니다(로컬 `cargo run` 개발용).

---

## ⚡ Valkey(Redis) 연동

`.env` / `.env.example`:

```env
VALKEY_URL=redis://127.0.0.1:6379
```

컨테이너 내부(`app` 서비스)에서는 `redis://valkey:6379`를 사용합니다.

---

## 📡 API 엔드포인트

| HTTP 메서드 | 엔드포인트 | 설명 | 요청 본문 (JSON) |
|:---|:---|:---|:---|
| **GET** | `/` | 웰컴 대시보드 | (없음) |
| **GET** | `/posts` | 전체 포스트 조회 (최신순) | (없음) |
| **POST** | `/posts` | 새 포스트 작성 | `{ "title": "제목", "content": "내용" }` |
| **GET** | `/posts/:id` | 특정 포스트 조회 | (없음) |
| **PUT** | `/posts/:id` | 포스트 수정 | `{ "title": "...", "content": "..." }` |
| **DELETE** | `/posts/:id` | 포스트 삭제 | (없음) |

### cURL 예시

```bash
curl -X POST http://127.0.0.1:3000/posts \
  -H "Content-Type: application/json" \
  -d '{"title": "테스트", "content": "PostgreSQL + Axum"}'

curl http://127.0.0.1:3000/posts
```

---

## 📂 프로젝트 구조

```
.
├── docker-compose.yml   # postgres, valkey, (profile) app
├── Dockerfile           # API 서버 이미지 빌드
├── .env.example         # 로컬 개발용 환경 변수 템플릿
└── crates/
    ├── application/     # 유스케이스, DTO, 입력 검증
    ├── infrastructure/  # SeaORM 엔티티, 리포지토리, DB 연결
    ├── api/             # HTTP 핸들러, 라우팅
    ├── server/          # API 진입점 (HTTP)
    └── batch/           # 배치 진입점 (주기적 작업)
```

---

## 🔧 환경 변수

| 변수 | 기본값 (미설정 시) | 설명 |
|:---|:---|:---|
| `DATABASE_URL` | `postgres://...@127.0.0.1:5433/playground` | SeaORM 연결 문자열 |
| `VALKEY_URL` | (없음) | 캐시 연동 시 사용 |
| `HOST` | `127.0.0.1` | 바인드 주소 (`0.0.0.0` in Docker) |
| `PORT` | `3000` | HTTP 포트 |
| `RUST_LOG` | main.rs 내 기본 필터 | tracing 로그 레벨 |
| `BATCH_INTERVAL_SECS` | `60` | 배치 작업 주기(초), `batch` 크레이트 |
