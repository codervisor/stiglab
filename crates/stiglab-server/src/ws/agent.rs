use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;

use stiglab_core::{AgentMessage, Node, NodeStatus, ServerMessage, SessionState};

use crate::db;
use crate::state::{AgentConnection, AppState};

pub async fn agent_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_agent_connection(socket, state))
}

async fn handle_agent_connection(socket: WebSocket, state: AppState) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // Spawn task to forward messages from channel to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut node_id: Option<String> = None;

    // Process incoming messages
    while let Some(Ok(msg)) = ws_receiver.next().await {
        let Message::Text(text) = msg else {
            continue;
        };

        let Ok(agent_msg) = serde_json::from_str::<AgentMessage>(&text) else {
            tracing::warn!("invalid message from agent: {text}");
            continue;
        };

        match agent_msg {
            AgentMessage::Register(info) => {
                // Reuse existing node ID if a node with this name already exists
                let existing = db::find_node_by_name(&state.db, &info.name)
                    .await
                    .ok()
                    .flatten();
                let id = existing
                    .as_ref()
                    .map(|n| n.id.clone())
                    .unwrap_or_else(|| Uuid::new_v4().to_string());
                let node = Node {
                    id: id.clone(),
                    name: info.name.clone(),
                    hostname: info.hostname,
                    status: NodeStatus::Online,
                    max_sessions: info.max_sessions,
                    active_sessions: 0,
                    last_heartbeat: Utc::now(),
                    registered_at: existing.map(|n| n.registered_at).unwrap_or_else(Utc::now),
                };

                if let Err(e) = db::upsert_node(&state.db, &node).await {
                    tracing::error!("failed to register node: {e}");
                    continue;
                }

                // Store agent connection
                {
                    let mut agents = state.agents.write().await;
                    agents.insert(
                        id.clone(),
                        AgentConnection {
                            node_id: id.clone(),
                            sender: tx.clone(),
                        },
                    );
                }

                node_id = Some(id.clone());
                tracing::info!("node registered: {} ({})", info.name, id);

                // Send confirmation
                let response = ServerMessage::Registered { node_id: id };
                if let Ok(json) = serde_json::to_string(&response) {
                    let _ = tx.send(Message::Text(json));
                }
            }

            AgentMessage::Heartbeat { active_sessions } => {
                if let Some(ref nid) = node_id {
                    if let Err(e) = db::update_node_heartbeat(&state.db, nid, active_sessions).await
                    {
                        tracing::error!("failed to update heartbeat: {e}");
                    }
                }
            }

            AgentMessage::SessionStateChanged {
                session_id,
                state: new_state,
            } => {
                if let Err(e) = db::update_session_state(&state.db, &session_id, new_state).await {
                    tracing::error!("failed to update session state: {e}");
                }
            }

            AgentMessage::SessionOutput { session_id, chunk } => {
                // O(1) append — insert a new log row instead of read-modify-write
                if let Err(e) =
                    db::append_session_log(&state.db, &session_id, &chunk, "stdout").await
                {
                    tracing::error!("failed to append session log: {e}");
                }
            }

            AgentMessage::SessionCompleted { session_id, output } => {
                let _ = db::update_session_state(&state.db, &session_id, SessionState::Done).await;
                if !output.is_empty() {
                    let _ = db::append_session_log(&state.db, &session_id, &output, "stdout").await;
                }
            }

            AgentMessage::SessionFailed { session_id, error } => {
                let _ =
                    db::update_session_state(&state.db, &session_id, SessionState::Failed).await;
                let _ = db::append_session_log(&state.db, &session_id, &error, "stderr").await;
            }
        }
    }

    // Clean up on disconnect
    if let Some(ref nid) = node_id {
        tracing::info!("node disconnected: {nid}");
        let _ = db::update_node_status(&state.db, nid, NodeStatus::Offline).await;
        let mut agents = state.agents.write().await;
        agents.remove(nid);
    }

    send_task.abort();
}
