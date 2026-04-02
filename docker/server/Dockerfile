# ---- Stage 1: Build UI ----
FROM node:20-alpine AS ui-builder
RUN corepack enable && corepack prepare pnpm@10.29.3 --activate
WORKDIR /app/packages/stiglab-ui
COPY packages/stiglab-ui/package.json packages/stiglab-ui/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY packages/stiglab-ui/ ./
RUN pnpm build

# ---- Stage 2: Build Rust server ----
FROM rust:1.93-bookworm AS rust-builder
WORKDIR /app
# Cache dependencies: copy manifests first
COPY Cargo.toml Cargo.lock ./
COPY crates/stiglab-core/Cargo.toml crates/stiglab-core/Cargo.toml
COPY crates/stiglab-server/Cargo.toml crates/stiglab-server/Cargo.toml
COPY crates/stiglab-agent/Cargo.toml crates/stiglab-agent/Cargo.toml
# Create dummy source files for dependency caching
RUN mkdir -p crates/stiglab-core/src crates/stiglab-server/src crates/stiglab-agent/src \
    && echo "fn main() {}" > crates/stiglab-server/src/main.rs \
    && echo "fn main() {}" > crates/stiglab-agent/src/main.rs \
    && touch crates/stiglab-core/src/lib.rs \
    && cargo build --release -p stiglab-server 2>/dev/null || true
# Copy actual source and rebuild
COPY crates/ crates/
RUN cargo build --release -p stiglab-server

# ---- Stage 3: Runtime ----
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust-builder /app/target/release/stiglab-server /app/stiglab-server
COPY --from=ui-builder /app/packages/stiglab-ui/dist /app/static
ENV STIGLAB_STATIC_DIR=/app/static
ENV STIGLAB_HOST=0.0.0.0
ENV STIGLAB_PORT=3000
EXPOSE 3000
CMD ["/app/stiglab-server"]
