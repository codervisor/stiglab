use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use uuid::Uuid;

use stiglab_core::{ServerMessage, Session, SessionState, Task, TaskRequest};

use crate::db;
use crate::state::AppState;

pub async fn create_task(
    State(state): State<AppState>,
    Json(request): Json<TaskRequest>,
) -> impl IntoResponse {
    let task = Task {
        id: Uuid::new_v4().to_string(),
        prompt: request.prompt.clone(),
        node_id: request.node_id.clone(),
        working_dir: request.working_dir.clone(),
        allowed_tools: request.allowed_tools.clone(),
        max_turns: request.max_turns,
        created_at: Utc::now(),
    };

    // Find target node
    let target_node = if let Some(ref node_id) = request.node_id {
        match db::get_node(&state.db, node_id).await {
            Ok(Some(node)) => {
                if node.status != stiglab_core::NodeStatus::Online {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({ "error": format!("node {} is not online", node_id) })),
                    )
                        .into_response();
                }
                if node.active_sessions >= node.max_sessions {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({ "error": format!("node {} is at capacity", node_id) })),
                    )
                        .into_response();
                }
                node
            }
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": format!("node {} not found", node_id) })),
                )
                    .into_response();
            }
            Err(e) => {
                tracing::error!("failed to get node: {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
        }
    } else {
        // Auto-assign to least loaded node
        match db::find_least_loaded_node(&state.db).await {
            Ok(Some(node)) => node,
            Ok(None) => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({ "error": "no available nodes for dispatch" })),
                )
                    .into_response();
            }
            Err(e) => {
                tracing::error!("failed to find node: {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
        }
    };

    // Create session
    let mut session = Session {
        id: Uuid::new_v4().to_string(),
        task_id: task.id.clone(),
        node_id: target_node.id.clone(),
        state: SessionState::Pending,
        prompt: request.prompt.clone(),
        output: None,
        working_dir: request.working_dir.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    if let Err(e) = db::insert_session(&state.db, &session).await {
        tracing::error!("failed to insert session: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // Dispatch to agent via WebSocket
    let agents = state.agents.read().await;
    if let Some(agent) = agents.get(&target_node.id) {
        let msg = ServerMessage::DispatchTask {
            task: task.clone(),
            session_id: session.id.clone(),
        };
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = agent.sender.send(axum::extract::ws::Message::Text(json));
        }
        // Update session state to dispatched
        let _ = db::update_session_state(&state.db, &session.id, SessionState::Dispatched).await;
        session.state = SessionState::Dispatched;
        session.updated_at = Utc::now();
    } else {
        tracing::warn!(
            "agent for node {} not connected, session stays pending",
            target_node.id
        );
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "task": task,
            "session": session,
        })),
    )
        .into_response()
}
