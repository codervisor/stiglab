//! Shared logic for processing AgentMessage events (used by both the WebSocket
//! handler and the built-in runner).

use sqlx::AnyPool;
use stiglab_core::{AgentMessage, SessionState};

use crate::db;

/// Process an `AgentMessage` by applying the corresponding DB mutations.
/// `node_id` identifies the agent node (used only for heartbeat updates).
pub async fn handle_agent_message(pool: &AnyPool, node_id: &str, msg: AgentMessage) {
    match msg {
        AgentMessage::Heartbeat { active_sessions } => {
            if let Err(e) = db::update_node_heartbeat(pool, node_id, active_sessions).await {
                tracing::warn!(
                    node_id = %node_id,
                    active_sessions,
                    error = ?e,
                    "failed to update heartbeat"
                );
            }
        }
        AgentMessage::SessionStateChanged { session_id, state } => {
            if let Err(e) = db::update_session_state(pool, &session_id, state).await {
                tracing::error!("failed to update session state: {e}");
            }
        }
        AgentMessage::SessionOutput { session_id, chunk } => {
            if let Err(e) = db::append_session_log(pool, &session_id, &chunk, "stdout").await {
                tracing::error!("failed to append session log: {e}");
            }
        }
        AgentMessage::SessionCompleted { session_id, output } => {
            if let Err(e) = db::update_session_state(pool, &session_id, SessionState::Done).await {
                tracing::error!("failed to update session state to done: {e}");
            }
            if !output.is_empty() {
                if let Err(e) = db::append_session_log(pool, &session_id, &output, "stdout").await {
                    tracing::error!("failed to append session log: {e}");
                }
            }
        }
        AgentMessage::SessionFailed { session_id, error } => {
            if let Err(e) = db::update_session_state(pool, &session_id, SessionState::Failed).await
            {
                tracing::error!("failed to update session state to failed: {e}");
            }
            if let Err(e) = db::append_session_log(pool, &session_id, &error, "stderr").await {
                tracing::error!("failed to append session log: {e}");
            }
        }
        AgentMessage::Register(_) => {
            // Registration is handled separately (node creation + WS setup)
        }
    }
}
