FROM rust:1.77-slim as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY packages/solver-core/Cargo.toml packages/solver-core/Cargo.toml
COPY packages/solver-service/Cargo.toml packages/solver-service/Cargo.toml

RUN mkdir -p packages/solver-core/src packages/solver-service/src && \
    echo "pub fn dummy() {}" > packages/solver-core/src/lib.rs && \
    echo "fn main() {}" > packages/solver-service/src/main.rs

RUN cargo build --release --package solver-service || true

COPY packages/ packages/

RUN cargo build --release --package solver-service

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/solver-service /usr/local/bin/solver-service

EXPOSE 3000

ENV RUST_LOG=info

ENTRYPOINT ["solver-service"]
