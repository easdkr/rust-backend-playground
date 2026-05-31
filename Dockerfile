# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p server -p batch

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-backend-playground /usr/local/bin/rust-backend-playground
COPY --from=builder /app/target/release/rust-backend-playground-batch /usr/local/bin/rust-backend-playground-batch

EXPOSE 3000

CMD ["rust-backend-playground"]
