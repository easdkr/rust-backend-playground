# 🦀 Rust Backend Playground (PostgreSQL 17 + Valkey)

Tokio + Axum + SeaORM(PostgreSQL) + Valkey 기반의 고성능 비동기 Rust 백엔드 플레이그라운드입니다.

이 프로젝트는 Docker Compose를 활용하여 최신 **PostgreSQL 17** 데이터베이스와 고성능 인메모리 저장소인 **Valkey 8**을 탑재하였습니다. 로컬 SQLite에서 실제 프로덕션 환경에 가까운 멀티 컨테이너 환경으로 업그레이드되어 복잡한 DB 및 캐시 연동을 완벽히 테스트할 수 있습니다.

---

## 🛠️ 기술 스택
- **비동기 런타임 (Async Runtime)**: [Tokio](https://tokio.rs/) (전체 기능 탑재)
- **웹 프레임워크 (Web Framework)**: [Axum v0.7](https://github.com/tokio-rs/axum) (모던 고성능 비동기 웹 프레임워크)
- **ORM & Database**: [SeaORM v1.1](https://www.sea-ql.org/SeaORM/) + **PostgreSQL 17** (비동기 ORM 및 자동 스키마 초기화)
- **인메모리 데이터 저장소**: **Valkey 8** (오픈소스 Redis 완전 대체재, `redis` 크레이트와 100% 호환)
- **로깅 & 트레이싱 (Logging)**: [Tracing](https://github.com/tokio-rs/tracing) (비동기 로깅)
- **직렬화 (Serialization)**: [Serde](https://serde.rs/) (데이터 파싱)

---

## 🚀 빠른 시작 (Quick Start)

### 1. Docker Compose로 DB & 캐시 서버 시작
프로젝트 루트 디렉터리에서 다음 명령어를 실행하여 PostgreSQL 17 및 Valkey 컨테이너를 실행합니다.
```bash
docker compose up -d
```

실행이 완료되면 다음 서비스들이 활성화됩니다:
- **PostgreSQL 17**: `127.0.0.1:5433` (Username: `postgres`, Password: `postgrespassword`, DB: `playground`)
- **Valkey 8**: `127.0.0.1:6379` (인메모리 캐시 저장소)

### 2. Rust 백엔드 애플리케이션 실행
서버가 띄워지면 터미널에서 아래 명령어로 백엔드를 구동합니다.
```bash
cargo run
```

서버 구동 시 자동으로 PostgreSQL 데이터베이스에 테이블 스키마가 없는 경우 `posts` 테이블을 생성합니다.
```text
INFO Starting Rust Backend Playground...
INFO Connecting to database at postgres://postgres:postgrespassword@127.0.0.1:5433/playground...
INFO Database connected and schema initialized successfully!
INFO Listening on http://127.0.0.1:3000
```
웹 브라우저를 열고 `http://127.0.0.1:3000`에 접속하여 플레이그라운드 대시보드를 확인할 수 있습니다.

---

## ⚡ Valkey(Redis) 연동 및 가이드
Valkey는 Redis의 완전한 오픈소스 대체재로 프로토콜이 100% 호환됩니다. 
의존성에 이미 `redis` 크레이트가 추가되어 있어, 비동기 캐싱이나 세션 관리가 필요할 때 바로 활용 가능합니다.

**연동 설정 (.env):**
```env
VALKEY_URL=redis://127.0.0.1:6379
```

**러스트 코드 활용 예시:**
```rust
use redis::AsyncCommands;

async fn get_cache_example(con: &mut redis::aio::Connection) -> redis::RedisResult<String> {
    let _: () = con.set("key", "valkey_data").await?;
    let val: String = con.get("key").await?;
    Ok(val)
}
```

---

## 📡 API 엔드포인트 목록

| HTTP 메서드 | 엔드포인트 | 설명 | 요청 본문 (JSON) |
|:---|:---|:---|:---|
| **GET** | `/` | 프로젝트 웰컴 대시보드 | (없음) |
| **GET** | `/posts` | 전체 포스트 목록 조회 (최신순) | (없음) |
| **POST** | `/posts` | 새 포스트 작성 | `{ "title": "제목", "content": "내용" }` |
| **GET** | `/posts/:id` | 특정 ID의 포스트 조회 | (없음) |
| **PUT** | `/posts/:id` | 특정 ID의 포스트 수정 (선택적 필드) | `{ "title": "수정할 제목", "content": "수정할 내용" }` |
| **DELETE**| `/posts/:id` | 특정 ID의 포스트 삭제 | (없음) |

---

## 💡 테스트 가이드 (cURL 예제)

### 1. 새 포스트 생성 (POST)
```bash
curl -X POST http://127.0.0.1:3000/posts \
  -H "Content-Type: application/json" \
  -d '{"title": "Docker PostgreSQL 테스트", "content": "PostgreSQL 17 컨테이너로 작동 중인 비동기 Rust 서버입니다!"}'
```

### 2. 전체 포스트 조회 (GET)
```bash
curl http://127.0.0.1:3000/posts
```

### 3. 특정 포스트 수정 (PUT)
```bash
curl -X PUT http://127.0.0.1:3000/posts/1 \
  -H "Content-Type: application/json" \
  -d '{"title": "수정된 포스트 제목"}'
```

### 4. 특정 포스트 삭제 (DELETE)
```bash
curl -X DELETE http://127.0.0.1:3000/posts/1
```

---

## 📂 프로젝트 구조
- [docker-compose.yml](file:///Users/june/workspace/personal/rust-backend-playground/docker-compose.yml): PostgreSQL 17 및 Valkey 서비스 구성을 정의한 도커 컴포즈 파일
- [Cargo.toml](file:///Users/june/workspace/personal/rust-backend-playground/Cargo.toml): postgres 지원 및 redis 의존성이 추가된 파일
- [.env](file:///Users/june/workspace/personal/rust-backend-playground/.env): PostgreSQL 및 Valkey 접속 정보를 담은 환경 변수
- [src/main.rs](file:///Users/june/workspace/personal/rust-backend-playground/src/main.rs): 서버 실행, DB 초기화 및 라우팅 설정
- [src/db.rs](file:///Users/june/workspace/personal/rust-backend-playground/src/db.rs): PostgreSQL 연결 및 자동 스키마 제네레이터
- [src/entities/](file:///Users/june/workspace/personal/rust-backend-playground/src/entities/): SeaORM 모델 엔티티 폴더
- [src/handlers.rs](file:///Users/june/workspace/personal/rust-backend-playground/src/handlers.rs): REST API 엔드포인트 비즈니스 로직
- [src/error.rs](file:///Users/june/workspace/personal/rust-backend-playground/src/error.rs): 커스텀 에러 변환기
