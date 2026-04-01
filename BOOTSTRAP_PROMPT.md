# Stiglab — Bootstrap Prompt

You are scaffolding **Stiglab**, a distributed AI agent session orchestration platform. Stiglab manages multiple AI coding agent sessions (Claude Code, Codex, Gemini CLI, etc.) across distributed machines from a unified control plane.

## Project Context

- **Author**: Marvin Zhang (creator of [Crawlab](https://github.com/crawlab-team/crawlab), an open-source distributed web scraping platform)
- **Architecture lineage**: Same distributed pattern as Crawlab (Control Plane + Node Agent + Dashboard), rebuilt from scratch in Rust + TypeScript for AI-native use cases
- **Development philosophy**: Specification-Driven Development (SDD) — write specs first, then code

## Architecture

```
┌─────────────────────────────────────────┐
│              Dashboard / UI              │
│  (task list / session state / log stream)│
└────────────────┬────────────────────────┘
                 │ SSE
┌────────────────▼────────────────────────┐
│           Control Plane (API)            │
│  task dispatch / session lifecycle /     │
│  node routing / state persistence        │
└──────┬──────────────────────┬───────────┘
       │ WebSocket             │ WebSocket
┌──────▼──────┐        ┌──────▼──────┐
│   Node A    │        │   Node B    │
│ agent proc  │        │ agent proc  │
│ 1..N sessions│       │ 1..N sessions│  ...
│ hook→report │        │ hook→report │
└─────────────┘        └─────────────┘
```

## Core Design Insight

AI coding agent sessions have a state dimension that traditional task runners lack: **WAITING_INPUT**. An agent may pause at any time waiting for human input. The system must support bidirectional communication, not just one-way log streaming.

### Session State Machine

```
PENDING → DISPATCHED → RUNNING ⇄ WAITING_INPUT
                         │
                    ┌────┴────┐
                    ▼         ▼
                  DONE      FAILED
```

## Tech Stack

|Layer        |Technology                                                        |
|-------------|------------------------------------------------------------------|
|Control Plane|Rust + Axum                                                       |
|Node Agent   |Rust, single binary, zero external deps                           |
|Transport    |WebSocket (heartbeat + event stream)                              |
|Dashboard    |TypeScript + React + Vite + shadcn/ui + Tailwind CSS              |
|State Storage|PostgreSQL (production) / SQLite (fallback, zero-config local dev)|
|Session Exec |`claude --print` subprocess + stdin/stdout                        |

## Monorepo Structure

```
stiglab/
├── crates/
│   ├── stiglab-core/        # Shared types, state machine, error handling
│   ├── stiglab-server/      # Control Plane (Axum HTTP + WS)
│   └── stiglab-agent/       # Node Agent binary
├── packages/
│   └── stiglab-ui/          # React + shadcn/ui Dashboard
├── docker/
│   ├── server.Dockerfile    # Multi-stage build for stiglab-server + UI
│   └── agent.Dockerfile     # Multi-stage build for stiglab-agent
├── .github/
│   └── workflows/
│       ├── ci.yml           # Lint, test, typecheck on PR
│       ├── release.yml      # Build + push Docker images to GHCR on tag
│       └── docker.yml       # Build + push on main branch (latest tag)
├── specs/                   # Architecture specification docs
├── docker-compose.yml       # Local dev: server + agent + UI
├── pnpm-workspace.yaml      # pnpm workspace config
├── package.json             # Root scripts (dev, build, lint, test)
├── railway.toml             # Railway deployment config
├── Cargo.toml               # Workspace root
├── README.md
└── LICENSE                  # AGPL-3.0
```

## Phase 0 — Execution Plan

Execute the following steps in order. Commit after each phase.

### Phase 1: Initialize Workspace

1. Create `Cargo.toml` workspace with members: `crates/stiglab-core`, `crates/stiglab-server`, `crates/stiglab-agent`
1. Initialize each crate with `cargo init --lib` (core) or `cargo init` (server, agent)
1. Add shared dependencies to workspace `[workspace.dependencies]`:
- `serde` + `serde_json` (serialization)
- `tokio` (async runtime, full features)
- `axum` (HTTP framework, server only)
- `tokio-tungstenite` (WebSocket)
- `sqlx` with `postgres`, `sqlite`, and `runtime-tokio` features (dual DB support)
- `tracing` + `tracing-subscriber` (logging)
- `chrono` (timestamps)
- `uuid` (identifiers)
- `thiserror` (error types)
1. Create `specs/` directory with a `000-architecture.md` stub referencing this prompt
1. Create `.gitignore` (Rust + Node standard ignores)
1. `git init && git add . && git commit -m "chore: initialize stiglab workspace"`

### Phase 2: Define Core Types (`stiglab-core`)

Define the shared domain model in `stiglab-core/src/`:

```
src/
├── lib.rs          # Re-exports
├── node.rs         # Node, NodeStatus, NodeInfo
├── session.rs      # Session, SessionState (state machine)
├── task.rs         # Task, TaskRequest, TaskStatus
├── event.rs        # Event enum (agent→server, server→dashboard)
└── protocol.rs     # WebSocket message types (AgentMessage, ServerMessage)
```

**Key types:**

```rust
// node.rs
pub struct Node {
    pub id: String,
    pub name: String,
    pub hostname: String,
    pub status: NodeStatus,
    pub max_sessions: u32,
    pub active_sessions: u32,
    pub last_heartbeat: DateTime<Utc>,
    pub registered_at: DateTime<Utc>,
}

pub enum NodeStatus {
    Online,
    Offline,
    Draining,  // accepting no new tasks, finishing existing
}

// session.rs
pub struct Session {
    pub id: String,
    pub task_id: String,
    pub node_id: String,
    pub state: SessionState,
    pub prompt: String,
    pub output: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum SessionState {
    Pending,
    Dispatched,
    Running,
    WaitingInput,
    Done,
    Failed,
}

// task.rs
pub struct TaskRequest {
    pub prompt: String,
    pub node_id: Option<String>,       // None = auto-assign
    pub working_dir: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_turns: Option<u32>,
}

// protocol.rs — WebSocket messages
pub enum AgentMessage {
    Register(NodeInfo),
    Heartbeat { active_sessions: u32 },
    SessionStateChanged { session_id: String, state: SessionState },
    SessionOutput { session_id: String, chunk: String },
    SessionCompleted { session_id: String, output: String },
    SessionFailed { session_id: String, error: String },
}

pub enum ServerMessage {
    Registered { node_id: String },
    DispatchTask(Task),
    CancelSession { session_id: String },
    SendInput { session_id: String, input: String },
}
```

Implement `serde::Serialize` and `serde::Deserialize` for all types. Implement `Display` for state enums. Add state transition validation on `SessionState` (e.g., cannot go from `Done` to `Running`).

Commit: `feat(core): define shared domain types and protocol`

### Phase 3: Scaffold Server (`stiglab-server`)

```
src/
├── main.rs         # Axum app bootstrap, bind HTTP + WS
├── config.rs       # Server config (port, db path)
├── db.rs           # Database abstraction (PostgreSQL or SQLite), migrations, CRUD
├── routes/
│   ├── mod.rs
│   ├── nodes.rs    # GET /api/nodes
│   ├── tasks.rs    # POST /api/tasks
│   └── sessions.rs # GET /api/sessions, GET /api/sessions/:id/logs (SSE)
├── ws/
│   ├── mod.rs
│   └── agent.rs    # WS /agent/ws — handle agent connections
└── state.rs        # AppState (db pool, connected agents map)
```

**API endpoints:**

```
GET  /api/nodes              — list registered nodes
POST /api/tasks              — dispatch task to node
GET  /api/sessions           — list sessions with state
GET  /api/sessions/:id       — session detail
GET  /api/sessions/:id/logs  — SSE log stream
WS   /agent/ws               — agent connection endpoint
```

**AppState** holds:

- `AnyPool` (sqlx) — connects to PostgreSQL or SQLite based on `DATABASE_URL` scheme
- `HashMap<String, AgentConnection>` behind `Arc<RwLock<>>` — maps node_id to WS sender

**WebSocket handler** (`/agent/ws`):

1. Agent connects → sends `AgentMessage::Register`
1. Server assigns/confirms node_id → sends `ServerMessage::Registered`
1. Agent sends `Heartbeat` every 30s
1. Server sends `DispatchTask` when a new task targets this node
1. Agent streams `SessionOutput` / `SessionStateChanged` back

**Database selection logic:**

```rust
// db.rs
// If DATABASE_URL starts with "postgres://" → use PostgreSQL
// If DATABASE_URL starts with "sqlite://" or is absent → use SQLite at ./data/stiglab.db
// Use sqlx::any::AnyPool for runtime-polymorphic queries
```

Default (no env var): `sqlite://./data/stiglab.db` — zero-config local development.

**Schema** (create via sqlx migrations, compatible with both PostgreSQL and SQLite — use TEXT for timestamps for portability):

```sql
CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    hostname TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'online',
    max_sessions INTEGER NOT NULL DEFAULT 4,
    active_sessions INTEGER NOT NULL DEFAULT 0,
    last_heartbeat TEXT NOT NULL,
    registered_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'pending',
    prompt TEXT NOT NULL,
    output TEXT,
    working_dir TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

For MVP, implement proper task dispatch logic:

- If `node_id` is specified, send to that node (fail if offline)
- If `node_id` is null, auto-assign: pick the online node with lowest `active_sessions / max_sessions` ratio
- Reject dispatch if all nodes are at capacity or offline
- Support multiple concurrent agent connections — this is the core differentiator

Commit: `feat(server): scaffold control plane with HTTP + WS endpoints`

### Phase 4: Scaffold Agent (`stiglab-agent`)

```
src/
├── main.rs         # CLI entry, parse args, connect to server
├── config.rs       # Agent config (server_url, node_name, max_sessions)
├── connection.rs   # WebSocket client, reconnect logic
├── session/
│   ├── mod.rs
│   ├── manager.rs  # SessionManager — spawn/track/kill sessions
│   └── process.rs  # Subprocess wrapper for `claude --print`
└── reporter.rs     # Collect session events, send to server
```

**Agent lifecycle:**

1. Read config (server URL, node name, max concurrent sessions)
1. Connect to server via WebSocket
1. Send `Register` message
1. Start heartbeat loop (30s interval)
1. Listen for `DispatchTask` → spawn session subprocess
1. Stream subprocess stdout as `SessionOutput` events
1. Detect state transitions (running/waiting/done/failed) and report

**Subprocess management:**

- Spawn `claude --print "<prompt>"` (or configurable command) as a child process
- Capture stdout line-by-line via `tokio::process::Command`
- Detect `WAITING_INPUT` state by watching for input prompt patterns in stdout
- Support `SendInput` from server → write to subprocess stdin
- On process exit, send `SessionCompleted` or `SessionFailed`

For MVP, use a simple `Vec<ChildSession>` to track active sessions, with `max_sessions` cap.

Commit: `feat(agent): scaffold node agent with subprocess management`

### Phase 5: Initialize Dashboard (`stiglab-ui`)

```bash
cd packages/
pnpm create vite@latest stiglab-ui --template react-ts
cd stiglab-ui
pnpm install
# Install Tailwind CSS
pnpm add -D tailwindcss @tailwindcss/vite
# Install shadcn/ui dependencies
pnpx shadcn@latest init
# Install core components
pnpx shadcn@latest add badge button card data-table dropdown-menu input separator sidebar table tabs toast scroll-area
# Install additional dependencies
pnpm add react-router-dom @tanstack/react-query lucide-react next-themes
```

**Design Requirements — match Crawlab’s professional quality:**

The dashboard must look and feel like a production management platform, not a hackathon prototype. Reference Crawlab’s UI patterns:

- **Sidebar navigation**: collapsible sidebar with icon + label items (Nodes, Sessions, Settings), using `shadcn/ui Sidebar` component
- **Data tables**: sortable, filterable tables using `shadcn/ui DataTable` with column visibility toggle, pagination, and row actions
- **Status indicators**: colored `Badge` components for node/session states (green=Online/Running, yellow=WaitingInput, red=Failed, gray=Offline/Pending)
- **Real-time updates**: live-updating session counts, heartbeat indicators with relative timestamps (“3s ago”)
- **Consistent spacing and typography**: use Tailwind’s design tokens, `inter` or `geist` font, proper heading hierarchy

**Pages and layout:**

```
src/
├── main.tsx
├── App.tsx                    # Router + QueryClientProvider + layout
├── components/
│   ├── layout/
│   │   ├── AppSidebar.tsx     # Sidebar with nav items + branding
│   │   ├── AppLayout.tsx      # Sidebar + main content area
│   │   └── ThemeToggle.tsx    # Light / Dark / System toggle
│   ├── providers/
│   │   └── ThemeProvider.tsx   # next-themes wrapper
│   ├── nodes/
│   │   ├── NodeTable.tsx      # DataTable with columns: name, status, sessions, heartbeat, actions
│   │   ├── NodeStatusBadge.tsx
│   │   └── NodeDetailSheet.tsx
│   ├── sessions/
│   │   ├── SessionTable.tsx   # DataTable with columns: id, node, state, prompt, timestamps
│   │   ├── SessionStateBadge.tsx
│   │   └── SessionLogStream.tsx  # SSE-powered real-time log viewer with auto-scroll
│   └── dashboard/
│       └── Overview.tsx       # Summary cards: total nodes, active sessions, waiting input count
├── pages/
│   ├── DashboardPage.tsx      # Overview with stat cards + recent activity
│   ├── NodesPage.tsx          # Node management
│   ├── SessionsPage.tsx       # Session list
│   └── SessionDetailPage.tsx  # Full prompt + live log stream + state timeline
├── lib/
│   ├── api.ts                 # Typed API client (fetch wrapper)
│   └── sse.ts                 # EventSource hook for log streaming
└── hooks/
    ├── useNodes.ts            # react-query hook for nodes
    └── useSessions.ts         # react-query hook for sessions
```

**Key UI components:**

1. **DashboardPage** — top-level overview with `Card` stat widgets: total nodes (online/offline), active sessions, sessions in `WAITING_INPUT` state (highlighted — this is the critical signal). Recent session activity feed below.
1. **NodesPage** — `DataTable` showing all registered nodes. Each row: name, hostname, status badge, `active/max` sessions bar, last heartbeat as relative time. Row click → `Sheet` panel with node detail. Actions dropdown: drain, remove.
1. **SessionsPage** — `DataTable` with filterable state column (multi-select badges). Columns: session ID (truncated), node name, state badge, prompt preview (first 80 chars), created time. Click → navigate to detail page.
1. **SessionDetailPage** — split layout. Top: session metadata card (full prompt, node, state, timestamps). Bottom: `ScrollArea` with monospace log output, auto-scrolling, connected via `EventSource` to `/api/sessions/:id/logs`. State timeline showing transitions with timestamps.

**Color scheme**: dark mode by default (zinc/slate palette from shadcn), with a theme toggle (light / dark / system) in the sidebar footer. Accent color: blue-500 for primary actions.

**Theme switching implementation:**

- Install `next-themes` (works with plain React + Vite too): `pnpm add next-themes`
- Add a `ThemeProvider` wrapping the app that reads/writes `localStorage` and applies `class="dark"` to `<html>`
- Add a `ThemeToggle` component using shadcn `DropdownMenu` with three options: Light (Sun icon), Dark (Moon icon), System (Monitor icon) — place it in the sidebar footer next to user/settings
- Tailwind config: set `darkMode: "class"` so all `dark:` variants work via class toggle
- All shadcn components support dark mode out of the box when using CSS variables from the shadcn theme system (`--background`, `--foreground`, `--card`, etc.)
- Persist preference in `localStorage` under key `stiglab-theme`

```tsx
// components/ThemeToggle.tsx
import { Moon, Sun, Monitor } from "lucide-react"
import { useTheme } from "next-themes"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"

export function ThemeToggle() {
  const { setTheme } = useTheme()
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon">
          <Sun className="h-4 w-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
          <Moon className="absolute h-4 w-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onClick={() => setTheme("light")}>
          <Sun className="mr-2 h-4 w-4" /> Light
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme("dark")}>
          <Moon className="mr-2 h-4 w-4" /> Dark
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme("system")}>
          <Monitor className="mr-2 h-4 w-4" /> System
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
```

Commit: `feat(ui): initialize React + shadcn/ui dashboard with professional layout`

### Phase 6: Wire End-to-End

1. Initialize pnpm workspace at repo root:

**`pnpm-workspace.yaml`**:

```yaml
packages:
  - "packages/*"
```

**Root `package.json`**:

```json
{
  "name": "stiglab",
  "private": true,
  "scripts": {
    "dev": "concurrently -n server,agent,ui -c blue,green,cyan \"pnpm dev:server\" \"pnpm dev:agent\" \"pnpm dev:ui\"",
    "dev:server": "cargo run -p stiglab-server",
    "dev:agent": "cargo run -p stiglab-agent -- --server ws://localhost:3000/agent/ws --name dev-node",
    "dev:ui": "pnpm --filter stiglab-ui dev",
    "build": "cargo build --release && pnpm --filter stiglab-ui build",
    "build:server": "cargo build --release -p stiglab-server",
    "build:agent": "cargo build --release -p stiglab-agent",
    "build:ui": "pnpm --filter stiglab-ui build",
    "lint": "cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && pnpm --filter stiglab-ui lint",
    "test": "cargo test --all",
    "docker:up": "docker compose up -d",
    "docker:down": "docker compose down"
  },
  "devDependencies": {
    "concurrently": "^9.0.0"
  }
}
```

1. Run `pnpm install` at repo root
1. Verify `pnpm dev` starts all three services concurrently with labeled output
1. Test distributed flow:
- Start server
- Start agent-1 (`--name node-a`), verify registration in dashboard
- Start agent-2 in another terminal (`--name node-b`), verify both nodes visible
- POST task with `node_id: "node-a"` → verify dispatched to node-a only
- POST task without `node_id` → verify auto-assigned to least-loaded node
- Verify dashboard shows both nodes, all sessions, and live log streaming

Commit: `chore: wire end-to-end MVP with pnpm workspace scripts`

### Phase 7: Dockerize (`docker/`)

Create multi-stage Dockerfiles optimized for small image size and layer caching.

**`docker/server.Dockerfile`** — builds both the Rust server binary and the React dashboard, serves UI as static assets from the server:

```dockerfile
# ---- Stage 1: Build UI ----
FROM node:20-alpine AS ui-builder
RUN corepack enable && corepack prepare pnpm@latest --activate
WORKDIR /app/packages/stiglab-ui
COPY packages/stiglab-ui/package.json packages/stiglab-ui/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY packages/stiglab-ui/ ./
RUN pnpm build

# ---- Stage 2: Build Rust server ----
FROM rust:1.75-bookworm AS rust-builder
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
# DATABASE_URL: set to postgres://... for production, or omit for SQLite fallback
# ENV DATABASE_URL=postgres://stiglab:stiglab@localhost:5432/stiglab
ENV STIGLAB_HOST=0.0.0.0
ENV STIGLAB_PORT=3000
EXPOSE 3000
CMD ["/app/stiglab-server"]
```

**`docker/agent.Dockerfile`** — minimal agent binary:

```dockerfile
# ---- Build ----
FROM rust:1.75-bookworm AS builder
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
```

**`docker-compose.yml`** — local development and demo:

```yaml
version: "3.9"

services:
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: stiglab
      POSTGRES_PASSWORD: stiglab
      POSTGRES_DB: stiglab
    ports:
      - "5432:5432"
    volumes:
      - stiglab-db:/var/lib/postgresql/data

  server:
    build:
      context: .
      dockerfile: docker/server.Dockerfile
    ports:
      - "3000:3000"
    depends_on:
      - db
    environment:
      STIGLAB_HOST: "0.0.0.0"
      STIGLAB_PORT: "3000"
      DATABASE_URL: "postgres://stiglab:stiglab@db:5432/stiglab"

  agent:
    build:
      context: .
      dockerfile: docker/agent.Dockerfile
    depends_on:
      - server
    environment:
      STIGLAB_SERVER_URL: "ws://server:3000/agent/ws"
      STIGLAB_NODE_NAME: "agent-1"
      STIGLAB_MAX_SESSIONS: "4"

volumes:
  stiglab-db:
```

**Server must serve static UI files**: update `stiglab-server/src/main.rs` to serve the built React app from `STIGLAB_STATIC_DIR` using `tower_http::services::ServeDir`. All routes not matching `/api/*` or `/agent/*` should fall back to `index.html` (SPA routing).

Add to server dependencies:

```toml
tower-http = { version = "0.5", features = ["fs", "cors"] }
```

Commit: `feat(docker): add multi-stage Dockerfiles and docker-compose`

### Phase 8: GitHub Actions CI/CD (`.github/workflows/`)

**`.github/workflows/ci.yml`** — runs on every PR and push to main:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: Check & Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Test
        run: cargo test --all

  ui:
    name: UI Lint & Build
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: packages/stiglab-ui
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
          cache-dependency-path: packages/stiglab-ui/pnpm-lock.yaml

      - run: pnpm install --frozen-lockfile
      - run: pnpm lint
      - run: pnpm build
```

**`.github/workflows/release.yml`** — builds and pushes Docker images on version tags:

```yaml
name: Release

on:
  push:
    tags: ["v*"]

env:
  REGISTRY: ghcr.io
  SERVER_IMAGE: ghcr.io/${{ github.repository }}/server
  AGENT_IMAGE: ghcr.io/${{ github.repository }}/agent

permissions:
  contents: read
  packages: write

jobs:
  build-and-push:
    name: Build & Push Images
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
          - target: server
            dockerfile: docker/server.Dockerfile
          - target: agent
            dockerfile: docker/agent.Dockerfile
    steps:
      - uses: actions/checkout@v4

      - name: Log in to GHCR
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract version from tag
        id: version
        run: echo "version=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ${{ matrix.dockerfile }}
          push: true
          tags: |
            ghcr.io/${{ github.repository }}/${{ matrix.target }}:${{ steps.version.outputs.version }}
            ghcr.io/${{ github.repository }}/${{ matrix.target }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
          platforms: linux/amd64,linux/arm64
```

**`.github/workflows/docker.yml`** — builds and pushes `latest` on every push to main (for continuous deployment):

```yaml
name: Docker Latest

on:
  push:
    branches: [main]
    paths:
      - "crates/**"
      - "packages/stiglab-ui/**"
      - "docker/**"
      - "Cargo.toml"
      - "Cargo.lock"

env:
  REGISTRY: ghcr.io

permissions:
  contents: read
  packages: write

jobs:
  build-and-push:
    name: Build & Push Latest
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
          - target: server
            dockerfile: docker/server.Dockerfile
          - target: agent
            dockerfile: docker/agent.Dockerfile
    steps:
      - uses: actions/checkout@v4

      - name: Log in to GHCR
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ${{ matrix.dockerfile }}
          push: true
          tags: ghcr.io/${{ github.repository }}/${{ matrix.target }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

Commit: `ci: add GitHub Actions workflows for CI, release, and Docker builds`

### Phase 9: Railway Deployment

**`railway.toml`** — Railway deployment configuration for the server (control plane + dashboard):

```toml
[build]
builder = "dockerfile"
dockerfilePath = "docker/server.Dockerfile"

[deploy]
startCommand = "/app/stiglab-server"
healthcheckPath = "/api/health"
healthcheckTimeout = 30
restartPolicyType = "on_failure"
restartPolicyMaxRetries = 3

[[services]]
name = "stiglab-server"
```

**Add health check endpoint** to the server (`/api/health`):

```rust
// routes/health.rs
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
```

Register in router: `.route("/api/health", get(health))`

**Railway environment variables** (set in Railway dashboard):

```
STIGLAB_HOST=0.0.0.0
STIGLAB_PORT=$PORT          # Railway injects $PORT
DATABASE_URL=$DATABASE_URL  # Railway injects when you add a PostgreSQL plugin
```

**Railway PostgreSQL**: add the PostgreSQL plugin in Railway dashboard. Railway auto-injects `DATABASE_URL`. The server detects `postgres://` scheme and uses PostgreSQL automatically.

**Agent deployment note**: agents run on your own machines (not on Railway). They connect outbound to the Railway-hosted server via WebSocket. The agent Docker image is pulled from GHCR and run locally:

```bash
docker run -d \
  --name stiglab-agent \
  -e STIGLAB_SERVER_URL=wss://your-railway-domain.up.railway.app/agent/ws \
  -e STIGLAB_NODE_NAME=my-machine \
  -e STIGLAB_MAX_SESSIONS=4 \
  ghcr.io/your-org/stiglab/agent:latest
```

Commit: `feat(deploy): add Railway config and health check endpoint`

## Constraints

- **Spec first**: before writing implementation code, create a brief doc in `specs/` describing the component’s contract
- **Distributed from day one**: MVP must support multiple agents on different machines connecting to a single control plane. Single-machine-only is not an MVP — it’s the whole point of Stiglab. Keep infrastructure simple (single server process, PostgreSQL or SQLite, no message queues, no Kubernetes), but the agent→server→dashboard flow must work across network boundaries
- **Error handling**: use `thiserror` for domain errors, `anyhow` only in binary crates. No `.unwrap()` in library code
- **Naming convention**: all crate names prefixed with `stiglab-`, all API routes under `/api/`, WebSocket under `/agent/`
- **Testing**: add at least integration tests for core state machine transitions and protocol serialization
