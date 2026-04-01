use chrono::Utc;
use sqlx::AnyPool;
use stiglab_core::{Node, NodeStatus, Session, SessionState};

pub async fn init_pool(database_url: &str) -> anyhow::Result<AnyPool> {
    // Ensure the SQLite data directory exists if needed
    if database_url.starts_with("sqlite://") {
        let path = database_url.trim_start_matches("sqlite://");
        if let Some(parent) = std::path::Path::new(path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    // Install drivers
    sqlx::any::install_default_drivers();

    let pool = AnyPool::connect(database_url).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

async fn run_migrations(pool: &AnyPool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            hostname TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'online',
            max_sessions INTEGER NOT NULL DEFAULT 4,
            active_sessions INTEGER NOT NULL DEFAULT 0,
            last_heartbeat TEXT NOT NULL,
            registered_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            state TEXT NOT NULL DEFAULT 'pending',
            prompt TEXT NOT NULL,
            output TEXT,
            working_dir TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS session_logs (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            seq INTEGER NOT NULL,
            chunk TEXT NOT NULL,
            stream TEXT NOT NULL DEFAULT 'stdout',
            created_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_session_logs_session_id ON session_logs (session_id, seq)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ── Node CRUD ──

pub async fn upsert_node(pool: &AnyPool, node: &Node) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO nodes (id, name, hostname, status, max_sessions, active_sessions, last_heartbeat, registered_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT(id) DO UPDATE SET
            name = $2, hostname = $3, status = $4, max_sessions = $5,
            active_sessions = $6, last_heartbeat = $7",
    )
    .bind(&node.id)
    .bind(&node.name)
    .bind(&node.hostname)
    .bind(node.status.to_string())
    .bind(node.max_sessions as i32)
    .bind(node.active_sessions as i32)
    .bind(node.last_heartbeat.to_rfc3339())
    .bind(node.registered_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_nodes(pool: &AnyPool) -> anyhow::Result<Vec<Node>> {
    let rows = sqlx::query_as::<_, NodeRow>("SELECT id, name, hostname, status, max_sessions, active_sessions, last_heartbeat, registered_at FROM nodes")
        .fetch_all(pool)
        .await?;
    rows.into_iter().map(|r| r.try_into()).collect()
}

pub async fn update_node_heartbeat(
    pool: &AnyPool,
    node_id: &str,
    active_sessions: u32,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE nodes SET last_heartbeat = $1, active_sessions = $2 WHERE id = $3")
        .bind(Utc::now().to_rfc3339())
        .bind(active_sessions as i32)
        .bind(node_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_node_status(
    pool: &AnyPool,
    node_id: &str,
    status: NodeStatus,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE nodes SET status = $1 WHERE id = $2")
        .bind(status.to_string())
        .bind(node_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn find_least_loaded_node(pool: &AnyPool) -> anyhow::Result<Option<Node>> {
    let row = sqlx::query_as::<_, NodeRow>(
        "SELECT id, name, hostname, status, max_sessions, active_sessions, last_heartbeat, registered_at
         FROM nodes
         WHERE status = 'online' AND active_sessions < max_sessions
         ORDER BY CAST(active_sessions AS REAL) / CAST(max_sessions AS REAL) ASC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    row.map(|r| r.try_into()).transpose()
}

pub async fn find_node_by_name(pool: &AnyPool, name: &str) -> anyhow::Result<Option<Node>> {
    let row = sqlx::query_as::<_, NodeRow>(
        "SELECT id, name, hostname, status, max_sessions, active_sessions, last_heartbeat, registered_at FROM nodes WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;
    row.map(|r| r.try_into()).transpose()
}

pub async fn get_node(pool: &AnyPool, node_id: &str) -> anyhow::Result<Option<Node>> {
    let row = sqlx::query_as::<_, NodeRow>(
        "SELECT id, name, hostname, status, max_sessions, active_sessions, last_heartbeat, registered_at FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_optional(pool)
    .await?;
    row.map(|r| r.try_into()).transpose()
}

// ── Session CRUD ──

pub async fn insert_session(pool: &AnyPool, session: &Session) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO sessions (id, task_id, node_id, state, prompt, output, working_dir, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(&session.id)
    .bind(&session.task_id)
    .bind(&session.node_id)
    .bind(session.state.to_string())
    .bind(&session.prompt)
    .bind(&session.output)
    .bind(&session.working_dir)
    .bind(session.created_at.to_rfc3339())
    .bind(session.updated_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_sessions(pool: &AnyPool) -> anyhow::Result<Vec<Session>> {
    let rows = sqlx::query_as::<_, SessionRow>(
        "SELECT id, task_id, node_id, state, prompt, output, working_dir, created_at, updated_at FROM sessions ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(|r| r.try_into()).collect()
}

pub async fn get_session(pool: &AnyPool, session_id: &str) -> anyhow::Result<Option<Session>> {
    let row = sqlx::query_as::<_, SessionRow>(
        "SELECT id, task_id, node_id, state, prompt, output, working_dir, created_at, updated_at FROM sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;
    row.map(|r| r.try_into()).transpose()
}

pub async fn update_session_state(
    pool: &AnyPool,
    session_id: &str,
    state: SessionState,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE sessions SET state = $1, updated_at = $2 WHERE id = $3")
        .bind(state.to_string())
        .bind(Utc::now().to_rfc3339())
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ── Session Logs (append-only) ──

pub async fn append_session_log(
    pool: &AnyPool,
    session_id: &str,
    chunk: &str,
    stream: &str,
) -> anyhow::Result<()> {
    let id = uuid::Uuid::new_v4().to_string();
    // Use a subquery to get the next sequence number for this session
    sqlx::query(
        "INSERT INTO session_logs (id, session_id, seq, chunk, stream, created_at)
         VALUES ($1, $2, COALESCE((SELECT MAX(seq) FROM session_logs WHERE session_id = $2), 0) + 1, $3, $4, $5)",
    )
    .bind(&id)
    .bind(session_id)
    .bind(chunk)
    .bind(stream)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

/// Get all log chunks for a session, ordered by sequence number.
pub async fn get_session_logs(pool: &AnyPool, session_id: &str) -> anyhow::Result<Vec<LogChunk>> {
    let rows = sqlx::query_as::<_, LogChunkRow>(
        "SELECT chunk, stream, created_at FROM session_logs WHERE session_id = $1 ORDER BY seq ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

/// Get log chunks added after a given sequence number (for incremental SSE).
pub async fn get_session_logs_after(
    pool: &AnyPool,
    session_id: &str,
    after_seq: i64,
) -> anyhow::Result<Vec<LogChunkWithSeq>> {
    let rows = sqlx::query_as::<_, LogChunkWithSeqRow>(
        "SELECT seq, chunk, stream, created_at FROM session_logs WHERE session_id = $1 AND seq > $2 ORDER BY seq ASC",
    )
    .bind(session_id)
    .bind(after_seq)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

#[allow(dead_code)]
pub struct LogChunk {
    pub chunk: String,
    pub stream: String,
    pub created_at: String,
}

#[allow(dead_code)]
pub struct LogChunkWithSeq {
    pub seq: i64,
    pub chunk: String,
    pub stream: String,
    pub created_at: String,
}

#[derive(sqlx::FromRow)]
struct LogChunkRow {
    chunk: String,
    stream: String,
    created_at: String,
}

impl From<LogChunkRow> for LogChunk {
    fn from(row: LogChunkRow) -> Self {
        LogChunk {
            chunk: row.chunk,
            stream: row.stream,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LogChunkWithSeqRow {
    seq: i32,
    chunk: String,
    stream: String,
    created_at: String,
}

impl From<LogChunkWithSeqRow> for LogChunkWithSeq {
    fn from(row: LogChunkWithSeqRow) -> Self {
        LogChunkWithSeq {
            seq: row.seq as i64,
            chunk: row.chunk,
            stream: row.stream,
            created_at: row.created_at,
        }
    }
}

// ── Row types for sqlx ──

#[derive(sqlx::FromRow)]
struct NodeRow {
    id: String,
    name: String,
    hostname: String,
    status: String,
    max_sessions: i32,
    active_sessions: i32,
    last_heartbeat: String,
    registered_at: String,
}

impl TryFrom<NodeRow> for Node {
    type Error = anyhow::Error;

    fn try_from(row: NodeRow) -> anyhow::Result<Self> {
        Ok(Node {
            id: row.id,
            name: row.name,
            hostname: row.hostname,
            status: row
                .status
                .parse()
                .map_err(|e: stiglab_core::StiglabError| anyhow::anyhow!(e))?,
            max_sessions: row.max_sessions as u32,
            active_sessions: row.active_sessions as u32,
            last_heartbeat: chrono::DateTime::parse_from_rfc3339(&row.last_heartbeat)?
                .with_timezone(&Utc),
            registered_at: chrono::DateTime::parse_from_rfc3339(&row.registered_at)?
                .with_timezone(&Utc),
        })
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    task_id: String,
    node_id: String,
    state: String,
    prompt: String,
    output: Option<String>,
    working_dir: Option<String>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<SessionRow> for Session {
    type Error = anyhow::Error;

    fn try_from(row: SessionRow) -> anyhow::Result<Self> {
        Ok(Session {
            id: row.id,
            task_id: row.task_id,
            node_id: row.node_id,
            state: row
                .state
                .parse()
                .map_err(|e: stiglab_core::StiglabError| anyhow::anyhow!(e))?,
            prompt: row.prompt,
            output: row.output,
            working_dir: row.working_dir,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)?.with_timezone(&Utc),
        })
    }
}
