# syntax=docker/dockerfile:1

# ============================================================
# Builder stage
# ============================================================
FROM rust:1-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migration ./migration

# Build all server binaries
RUN cargo build --release -p server -p auth-server -p websocket-server

# ============================================================
# API Server runtime
# ============================================================
FROM debian:bookworm-slim AS api-server

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-backend-playground /usr/local/bin/server

EXPOSE 3000

CMD ["server"]

# ============================================================
# Auth Server runtime
# ============================================================
FROM debian:bookworm-slim AS auth-server

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/auth-server /usr/local/bin/auth-server

EXPOSE 3002

CMD ["auth-server"]

# ============================================================
# WebSocket Server runtime
# ============================================================
FROM debian:bookworm-slim AS websocket-server

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/websocket-server /usr/local/bin/websocket-server

EXPOSE 3001

CMD ["websocket-server"]
