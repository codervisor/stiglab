use serde::{Deserialize, Serialize};

use crate::node::NodeInfo;
use crate::session::SessionState;
use crate::task::Task;

/// Messages sent from the agent to the server over WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentMessage {
    Register(NodeInfo),
    Heartbeat {
        active_sessions: u32,
    },
    SessionStateChanged {
        session_id: String,
        state: SessionState,
    },
    SessionOutput {
        session_id: String,
        chunk: String,
    },
    SessionCompleted {
        session_id: String,
        output: String,
    },
    SessionFailed {
        session_id: String,
        error: String,
    },
}

/// Messages sent from the server to the agent over WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Registered { node_id: String },
    DispatchTask { task: Task, session_id: String },
    CancelSession { session_id: String },
    SendInput { session_id: String, input: String },
}
