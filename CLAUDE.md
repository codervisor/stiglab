# CLAUDE.md

## Project

Stiglab is a distributed AI agent session orchestration platform. Rust backend (Cargo workspace), React frontend (pnpm), Docker for deployment.

## Build & Test

```bash
pnpm lint          # cargo fmt + clippy + eslint
pnpm test          # cargo test --all
pnpm build         # full release build (Rust + UI)
```

## Railway Deployment

Both services are deployed to Railway from the same repo. Configuration is done entirely via the Railway dashboard (not `railway.toml`) because Railway's config-as-code only supports one service per file and overrides dashboard settings — making it incompatible with multi-service monorepos sharing a root directory.

### stiglab-server

| Setting | Value |
|---------|-------|
| Builder | Dockerfile |
| Dockerfile path | `docker/server/Dockerfile` |
| Start command | `/app/stiglab-server` |
| Healthcheck path | `/api/health` |
| Healthcheck timeout | 30s |
| Restart policy | On failure (max 3) |

Environment variables:

| Variable | Value |
|----------|-------|
| `STIGLAB_HOST` | `0.0.0.0` |
| `DATABASE_URL` | `${{Postgres.DATABASE_URL}}` |

`PORT` is auto-injected by Railway. The server reads `PORT` first, then `STIGLAB_PORT`, then defaults to `3000`.

### stiglab-agent

| Setting | Value |
|---------|-------|
| Builder | Dockerfile |
| Dockerfile path | `docker/agent/Dockerfile` |
| Start command | `/app/stiglab-agent` |
| Restart policy | On failure (max 3) |

Environment variables:

| Variable | Value |
|----------|-------|
| `STIGLAB_SERVER_URL` | `wss://${{stiglab-server.RAILWAY_PUBLIC_DOMAIN}}/agent/ws` |
| `STIGLAB_NODE_NAME` | `railway-agent` |
| `STIGLAB_MAX_SESSIONS` | `4` |

## Lessons

### CI must validate what gets deployed, not just source code

The Rust CI jobs use `dtolnay/rust-toolchain@stable` while the Dockerfiles pin a specific Rust version. These are two separate build environments. If CI only checks source code compilation but not the Docker build, version drift between the two goes undetected — code passes CI but the actual deployable artifact is broken.

**Principle:** If the deployment artifact is a Docker image, the Docker build is a first-class CI check. Any build path that reaches production must be exercised in CI.

### When fixing version mismatches, anchor to the known-working reference

When a build fails due to a version being too old, don't increment and hope. Find the version that is already proven to work (e.g., the CI stable toolchain) and align to that directly. Iterating upward from a broken version wastes CI cycles and teaches you nothing — the failure mode is the same each time until you happen to cross the threshold.

**Principle:** Fix version problems with evidence, not guesses. If stable works, use stable. If you must pin, pin to what's tested.

### Railway config-as-code is per-service, not per-repo

Railway's `railway.toml` defines configuration for a single service. It overrides dashboard settings and cannot be scoped to a specific service — every service sharing the same repo root reads the same file. For multi-service monorepos, this means `railway.toml` would force the same Dockerfile path, start command, etc. on all services. The fix is to skip `railway.toml` entirely and configure all services through the Railway dashboard.

**Principle:** Read the platform docs before adopting config-as-code. A config file that silently overrides UI settings and can't distinguish between services is worse than no config file at all.
