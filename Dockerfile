# syntax=docker/dockerfile:1
#
# SeaORM Migrator 참고:
# 마이그레이션은 런타임에 애플리케이션 시작 시 자동 실행됨 (db.rs의 Migrator::up())
# 별도의 마이그레이션 빌드 단계는 필요 없음

FROM rust:1-bookworm AS builder
WORKDIR /app

# migration 크레이트는 infrastructure의 의존성으로 자동 포함됨
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migration ./migration

RUN cargo build --release -p server -p batch

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-backend-playground /usr/local/bin/rust-backend-playground
COPY --from=builder /app/target/release/rust-backend-playground-batch /usr/local/bin/rust-backend-playground-batch

EXPOSE 3000

CMD ["rust-backend-playground"]
