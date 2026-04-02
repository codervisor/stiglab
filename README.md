# Stiglab

**Distributed AI agent session orchestration platform.**

Stiglab manages multiple AI coding agent sessions (Claude Code, Codex, Gemini CLI) across distributed machines from a unified control plane — the missing infrastructure layer between “SSH into a box and run tmux” and production-grade agent fleet management.

> **Etymology**: from *stigmergy* (Greek στίγμα *stigma* “mark” + ἔργον *ergon* “work”) — the biological principle where agents coordinate indirectly through traces left in their environment, as seen in ant colonies and termite mounds. Not related to DISA STIG.

## Why

Running AI coding agents at scale today means SSH-ing into remote machines, managing tmux sessions by hand, and mentally tracking which pane is doing what. This works for 2-3 sessions. It breaks down at 10+.

**What’s missing:**

- No unified view across machines
- No awareness of which session needs attention (the `WAITING_INPUT` problem)
- Cognitive load scales linearly with session count
- No programmatic dispatch or monitoring API

Stiglab fills this gap.

## Architecture

```
┌─────────────────────────────────────────┐
│          Dashboard (React + shadcn/ui)    │
│   nodes • sessions • real-time logs      │
└────────────────┬────────────────────────┘
                 │ SSE
┌────────────────▼────────────────────────┐
│         Control Plane (Rust/Axum)        │
│   task dispatch • session lifecycle      │
│   node routing  • state persistence      │
└──────┬──────────────────────┬───────────┘
       │ WS                    │ WS
┌──────▼──────┐         ┌─────▼───────┐
│   Node A    │         │   Node B    │
│  stiglab-   │         │  stiglab-   │
│  agent      │         │  agent      │    ...
│  sessions:  │         │  sessions:  │
│   claude ×3 │         │   codex ×2  │
└─────────────┘         └─────────────┘
```

### Session State Machine

AI agent sessions have a unique dimension that traditional task runners lack: agents can pause and wait for human input at any time.

```
PENDING → DISPATCHED → RUNNING ⇄ WAITING_INPUT
                         │
                    ┌────┴────┐
                    ▼         ▼
                  DONE      FAILED
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 20+ with pnpm 9+
- PostgreSQL 16+ (optional — falls back to SQLite for local dev)

### Run with Docker

```bash
# Pull and run the server (control plane + dashboard)
docker run -d \
  --name stiglab-server \
  -p 3000:3000 \
  -v stiglab-data:/app/data \
  ghcr.io/nicedoc/stiglab/server:latest

# Run an agent on any machine
docker run -d \
  --name stiglab-agent \
  -e STIGLAB_SERVER_URL=ws://your-server:3000/agent/ws \
  -e STIGLAB_NODE_NAME=my-machine \
  -e STIGLAB_MAX_SESSIONS=4 \
  ghcr.io/nicedoc/stiglab/agent:latest
```

Or use Docker Compose for local development:

```bash
docker compose up
# or
pnpm docker:up
```

### Run from Source

```bash
# Clone
git clone https://github.com/nicedoc/stiglab.git
cd stiglab

# Install dependencies
pnpm install

# Start all services (server + agent + dashboard)
pnpm dev

# Or start individually
pnpm dev:server    # Control plane on :3000
pnpm dev:agent     # Agent connecting to local server
pnpm dev:ui        # Dashboard dev server on :5173
```

### Dispatch a Task

```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Refactor the auth module to use JWT",
    "node_id": "my-node",
    "working_dir": "/home/user/myproject"
  }'
```

### API

```
GET  /api/nodes              — registered nodes
POST /api/tasks              — dispatch a task
GET  /api/sessions           — list sessions + state
GET  /api/sessions/:id       — session detail
GET  /api/sessions/:id/logs  — SSE real-time log stream
WS   /agent/ws               — agent connection endpoint
```

## Tech Stack

|Layer        |Technology                                          |
|-------------|----------------------------------------------------|
|Control Plane|Rust + Axum + PostgreSQL / SQLite                   |
|Node Agent   |Rust, single static binary                          |
|Transport    |WebSocket (heartbeat + event stream)                |
|Dashboard    |TypeScript + React + Vite + shadcn/ui + Tailwind CSS|
|Session Exec |Configurable subprocess (default: `claude --print`) |
|Container    |Docker multi-stage builds, GHCR                     |
|CI/CD        |GitHub Actions (lint, test, build, push)            |
|Hosting      |Railway (server) + self-hosted (agents)             |

## Project Structure

```
stiglab/
├── crates/
│   ├── stiglab-core/        # Shared types, state machine, protocol
│   ├── stiglab-server/      # Control Plane
│   └── stiglab-agent/       # Node Agent
├── packages/
│   └── stiglab-ui/          # React + shadcn/ui Dashboard
├── docker/
│   ├── server.Dockerfile    # Server + UI multi-stage build
│   └── agent.Dockerfile     # Agent multi-stage build
├── .github/workflows/
│   ├── ci.yml               # Lint, test, typecheck on PR
│   ├── release.yml          # Build + push to GHCR on version tag
│   └── docker.yml           # Push latest on main
├── specs/                   # Architecture specs (SDD)
├── docker-compose.yml       # Local dev stack
├── pnpm-workspace.yaml      # pnpm workspace config
├── package.json             # Root scripts (dev, build, lint)
├── railway.toml             # Railway deployment config
├── Cargo.toml               # Workspace
└── README.md
```

## Deployment

### Docker Images

Images are published to GitHub Container Registry on every release and main branch push:

```
ghcr.io/nicedoc/stiglab/server:latest    # Control Plane + Dashboard
ghcr.io/nicedoc/stiglab/agent:latest     # Node Agent
```

Multi-arch support: `linux/amd64` and `linux/arm64`.

### Railway (Recommended for Server)

Deploy the control plane to Railway with one click:

1. Connect your GitHub repo in Railway
1. Railway auto-detects `railway.toml` and builds from `docker/server/Dockerfile`
1. Add PostgreSQL plugin — Railway auto-injects `DATABASE_URL`
1. Set environment variable: `STIGLAB_PORT=$PORT`

Agents run on your own machines and connect outbound to the Railway-hosted server:

```bash
docker run -d \
  -e STIGLAB_SERVER_URL=wss://stiglab.up.railway.app/agent/ws \
  -e STIGLAB_NODE_NAME=prod-node-1 \
  ghcr.io/nicedoc/stiglab/agent:latest
```

### Self-Hosted

```bash
# docker-compose.yml runs server + one agent locally
docker compose up -d

# Add more agents on other machines
docker run -d \
  -e STIGLAB_SERVER_URL=ws://your-server-ip:3000/agent/ws \
  -e STIGLAB_NODE_NAME=node-2 \
  ghcr.io/nicedoc/stiglab/agent:latest
```

## Roadmap

- [x] Phase 0 — Distributed MVP (multi-node dispatch, auto-assignment, real-time streaming)
- [ ] Phase 1 — `WAITING_INPUT` detection + bidirectional input relay
- [ ] Phase 2 — Authentication, RBAC, API keys
- [ ] Phase 3 — Notification integrations (Telegram, Slack, webhooks)
- [ ] Phase 4 — Agent-agnostic adapters (Claude Code, Codex, Gemini CLI, custom)
- [ ] Phase 5 — Stiglab Cloud (hosted control plane, commercial offering)

## Related Projects

|Project                                                   |Relationship                                                                                        |
|----------------------------------------------------------|----------------------------------------------------------------------------------------------------|
|[Crawlab](https://github.com/crawlab-team/crawlab)        |Sister project — distributed web scraping platform. Same architectural philosophy, different domain.|
|[Synodic](https://github.com/codervisor/synodic)          |AI harness/governance layer. Can integrate as node-side hooks for session behavior control.         |
|[Telegramable](https://github.com/codervisor/telegramable)|Telegram-first AI agent proxy. Can serve as notification + command input channel for Stiglab.       |

## Contributing

Contributions welcome. Please read `specs/` before submitting PRs — we practice Specification-Driven Development.

## License

AGPL-3.0 — see <LICENSE> for details.

For commercial licensing inquiries, please contact the author.
