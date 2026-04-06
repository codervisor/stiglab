# CLAUDE.md — codervisor project standards

This file defines shared conventions enforced across all codervisor repositories.
It is synced from the [codervisor meta-repo](https://github.com/codervisor/codervisor) into each child project.

## Repository overview

codervisor is a suite of AI-agent infrastructure projects:

| Repo | Stack | Purpose |
|------|-------|---------|
| stiglab | Rust + TypeScript | Distributed AI agent session orchestration |
| synodic | TypeScript | AI harness governance framework |
| telegramable | TypeScript | Telegram-first AI agent proxy |
| ising | Rust | Code graph analysis engine |
| skills | TOML / Markdown | Shared Claude Code skills |
| lean-spec | Lean 4 | Formal specification framework |

## Coding conventions

### General

- Write clear, self-documenting code. Add comments only where intent is non-obvious.
- Prefer small, focused commits with descriptive messages (imperative mood, <72 chars).
- Every PR must pass CI before merge. No force-pushing to `main`.
- Keep dependencies minimal. Justify new crates / packages in the PR description.

### Rust repos (stiglab, ising)

- Edition: 2021 or later.
- Format with `rustfmt` (default config unless `rustfmt.toml` is present).
- Lint with `clippy` — treat warnings as errors in CI (`-D warnings`).
- Use `thiserror` for library errors, `anyhow` for binary/application errors.
- Prefer `#[must_use]` on functions returning values that should not be silently dropped.
- Tests live next to the code in `#[cfg(test)]` modules; integration tests in `tests/`.

### TypeScript repos (synodic, telegramable)

- Target: ES2022+ / Node 20+.
- Strict mode: `"strict": true` in `tsconfig.json`.
- Lint with Biome (preferred) or ESLint. Format with Biome or Prettier.
- Use named exports. Avoid `default` exports except for framework conventions.
- Prefer `async/await` over raw Promises. Avoid `.then()` chains.
- Tests use Vitest (preferred) or Jest. Co-locate test files as `*.test.ts`.

### Lean repos (lean-spec)

- Follow Mathlib conventions for naming and style.
- Keep proofs tactic-mode where possible for readability.

## Commit messages

```
<type>: <short summary in imperative mood>

Optional body explaining *why*, not *what*.
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `ci`, `perf`.

## Branch naming

```
<type>/<short-description>
```

Examples: `feat/session-pool`, `fix/auth-timeout`, `chore/update-deps`.

## PR standards

- Title follows commit message format.
- Description includes a **Summary** (what and why) and **Test plan** (how it was verified).
- Keep PRs small and focused. Split large changes into stacked PRs.
- Request review from at least one team member.

## Security

- Never commit secrets, tokens, or credentials. Use environment variables.
- Validate all external input at system boundaries.
- Follow OWASP top-10 awareness for any web-facing code.
- Pin CI action versions to full SHA, not tags.

## Dependencies

- Rust: pin exact versions in `Cargo.toml` for binaries; use semver ranges for libraries.
- TypeScript: use a lockfile (`package-lock.json` or `pnpm-lock.yaml`). Commit it.
- Review dependency updates via Dependabot or Renovate PRs — don't auto-merge.

---

# Stiglab — project-specific

## Architecture

Single unified binary (`stiglab`) with `server` and `agent` subcommands. Rust backend (Cargo workspace), React frontend (pnpm), Docker for deployment.

Single binary, two modes:
- `stiglab server` — serves UI, API, WebSocket endpoint, and runs a built-in task runner (enabled by default). A single Railway service handles everything.
- `stiglab agent` — connects to a server via WebSocket and executes tasks. Used for horizontal scaling by adding worker nodes.

The built-in runner registers itself as a regular node in the agents map. The server's dispatch logic doesn't distinguish between built-in and remote agents — they look identical.

## Build & Test

```bash
pnpm lint          # cargo fmt + clippy + eslint
pnpm test          # cargo test --all
pnpm build         # full release build (Rust + UI)
```

## Running Locally

```bash
pnpm dev           # starts server (with built-in runner) + UI dev server
pnpm dev:agent     # start an additional agent node
```

## Railway Deployment

Single service deployed to Railway. Configuration is done via the Railway dashboard.

### stiglab (server mode)

| Setting | Value |
|---------|-------|
| Builder | Dockerfile |
| Dockerfile path | `Dockerfile` |
| Start command | `/app/stiglab server` |
| Healthcheck path | `/api/health` |
| Healthcheck timeout | 30s |
| Restart policy | On failure (max 3) |

Environment variables:

| Variable | Value |
|----------|-------|
| `STIGLAB_HOST` | `0.0.0.0` |
| `DATABASE_URL` | `${{Postgres.DATABASE_URL}}` |
| `STIGLAB_MAX_SESSIONS` | `4` |
| `STIGLAB_NODE_NAME` | `railway` |

`PORT` is auto-injected by Railway. The server reads `PORT` first, then `STIGLAB_PORT`, then defaults to `3000`.

To disable the built-in runner and require external agents, set `STIGLAB_NO_RUNNER=true`.

### Additional agent nodes (optional, for scaling)

| Setting | Value |
|---------|-------|
| Start command | `/app/stiglab agent` |

| Variable | Value |
|----------|-------|
| `STIGLAB_SERVER_URL` | `wss://${{stiglab.RAILWAY_PUBLIC_DOMAIN}}/agent/ws` |
| `STIGLAB_NODE_NAME` | `railway-agent-N` |
| `STIGLAB_MAX_SESSIONS` | `4` |

## Lessons

### CI must validate what gets deployed, not just source code

The Rust CI jobs use `dtolnay/rust-toolchain@stable` while the Dockerfile pins a specific Rust version. These are two separate build environments. If CI only checks source code compilation but not the Docker build, version drift between the two goes undetected — code passes CI but the actual deployable artifact is broken.

**Principle:** If the deployment artifact is a Docker image, the Docker build is a first-class CI check. Any build path that reaches production must be exercised in CI.

### When fixing version mismatches, anchor to the known-working reference

When a build fails due to a version being too old, don't increment and hope. Find the version that is already proven to work (e.g., the CI stable toolchain) and align to that directly. Iterating upward from a broken version wastes CI cycles and teaches you nothing — the failure mode is the same each time until you happen to cross the threshold.

**Principle:** Fix version problems with evidence, not guesses. If stable works, use stable. If you must pin, pin to what's tested.

### Railway config-as-code is per-service, not per-repo

Railway's `railway.toml` defines configuration for a single service. It overrides dashboard settings and cannot be scoped to a specific service — every service sharing the same repo root reads the same file. For multi-service monorepos, this means `railway.toml` would force the same Dockerfile path, start command, etc. on all services. The fix is to skip `railway.toml` entirely and configure all services through the Railway dashboard.

**Principle:** Read the platform docs before adopting config-as-code. A config file that silently overrides UI settings and can't distinguish between services is worse than no config file at all.

### Prefer single-binary architectures for simple deployments

A distributed system that requires two services even for single-node deployment adds unnecessary operational complexity. Embedding the worker capability into the server (with opt-out) means one container, one Railway service, one thing to monitor. External agents remain available for horizontal scaling.

**Principle:** Default to the simplest deployment topology. Make distribution opt-in, not mandatory.
