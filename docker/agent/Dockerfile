# ---- Build ----
FROM rust:1.93-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/stiglab-core/Cargo.toml crates/stiglab-core/Cargo.toml
COPY crates/stiglab-server/Cargo.toml crates/stiglab-server/Cargo.toml
COPY crates/stiglab-agent/Cargo.toml crates/stiglab-agent/Cargo.toml
RUN mkdir -p crates/stiglab-core/src crates/stiglab-server/src crates/stiglab-agent/src \
    && echo "fn main() {}" > crates/stiglab-server/src/main.rs \
    && echo "fn main() {}" > crates/stiglab-agent/src/main.rs \
    && touch crates/stiglab-core/src/lib.rs \
    && cargo build --release -p stiglab-agent 2>/dev/null || true
COPY crates/ crates/
RUN cargo build --release -p stiglab-agent

# ---- Runtime ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/stiglab-agent /app/stiglab-agent
ENV STIGLAB_SERVER_URL=ws://localhost:3000/agent/ws
ENV STIGLAB_NODE_NAME=default
ENV STIGLAB_MAX_SESSIONS=4
ENTRYPOINT ["/app/stiglab-agent"]
