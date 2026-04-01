use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use futures_util::stream;
use std::convert::Infallible;
use std::time::Duration;

use crate::db;
use crate::state::AppState;

pub async fn list_sessions(State(state): State<AppState>) -> impl IntoResponse {
    match db::list_sessions(&state.db).await {
        Ok(sessions) => Json(serde_json::json!({ "sessions": sessions })).into_response(),
        Err(e) => {
            tracing::error!("failed to list sessions: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match db::get_session(&state.db, &session_id).await {
        Ok(Some(session)) => {
            // Aggregate output from log chunks
            let output = match db::get_session_logs(&state.db, &session.id).await {
                Ok(chunks) => {
                    if chunks.is_empty() {
                        session.output
                    } else {
                        Some(chunks.into_iter().map(|c| c.chunk).collect::<String>())
                    }
                }
                Err(_) => session.output,
            };
            Json(serde_json::json!({
                "session": {
                    "id": session.id,
                    "task_id": session.task_id,
                    "node_id": session.node_id,
                    "state": session.state,
                    "prompt": session.prompt,
                    "output": output,
                    "working_dir": session.working_dir,
                    "created_at": session.created_at,
                    "updated_at": session.updated_at,
                }
            }))
            .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "session not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("failed to get session: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn session_logs(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    // Verify session exists
    match db::get_session(&state.db, &session_id).await {
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "session not found" })),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ));
        }
        Ok(Some(_)) => {}
    }

    // SSE stream: send only new chunks since last poll (cursor-based)
    let initial_state = (state, session_id, 0i64);
    let sse_stream = stream::unfold(initial_state, |(state, session_id, last_seq)| async move {
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Get session state
        let session = db::get_session(&state.db, &session_id).await.ok()??;

        // Get only new log chunks since last_seq
        let new_chunks = db::get_session_logs_after(&state.db, &session_id, last_seq)
            .await
            .ok()?;

        let new_last_seq = new_chunks.last().map(|c| c.seq).unwrap_or(last_seq);
        let chunks_data: Vec<serde_json::Value> = new_chunks
            .iter()
            .map(|c| {
                serde_json::json!({
                    "chunk": c.chunk,
                    "stream": c.stream,
                })
            })
            .collect();

        let event = Event::default()
            .json_data(serde_json::json!({
                "state": session.state,
                "chunks": chunks_data,
            }))
            .ok()?;

        // Stop streaming if session is in a terminal state and no new chunks
        let is_terminal = matches!(
            session.state,
            stiglab_core::SessionState::Done | stiglab_core::SessionState::Failed
        );
        if is_terminal && chunks_data.is_empty() {
            return None;
        }

        Some((
            Ok::<_, Infallible>(event),
            (state, session_id, new_last_seq),
        ))
    });

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}
