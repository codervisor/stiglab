# CLAUDE.md

## Project

Stiglab is a distributed AI agent session orchestration platform. Rust backend (Cargo workspace), React frontend (pnpm), Docker for deployment.

## Build & Test

```bash
pnpm lint          # cargo fmt + clippy + eslint
pnpm test          # cargo test --all
pnpm build         # full release build (Rust + UI)
```

## Lessons

### CI must validate what gets deployed, not just source code

The Rust CI jobs use `dtolnay/rust-toolchain@stable` while the Dockerfiles pin a specific Rust version. These are two separate build environments. If CI only checks source code compilation but not the Docker build, version drift between the two goes undetected — code passes CI but the actual deployable artifact is broken.

**Principle:** If the deployment artifact is a Docker image, the Docker build is a first-class CI check. Any build path that reaches production must be exercised in CI.

### When fixing version mismatches, anchor to the known-working reference

When a build fails due to a version being too old, don't increment and hope. Find the version that is already proven to work (e.g., the CI stable toolchain) and align to that directly. Iterating upward from a broken version wastes CI cycles and teaches you nothing — the failure mode is the same each time until you happen to cross the threshold.

**Principle:** Fix version problems with evidence, not guesses. If stable works, use stable. If you must pin, pin to what's tested.
