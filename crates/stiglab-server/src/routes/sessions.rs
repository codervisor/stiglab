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
        Ok(Some(session)) => Json(serde_json::json!({ "session": session })).into_response(),
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

    // Return SSE stream that polls for session updates
    let sse_stream = stream::unfold(
        (state, session_id),
        |(state, session_id)| async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            match db::get_session(&state.db, &session_id).await {
                Ok(Some(session)) => {
                    let event = Event::default()
                        .json_data(serde_json::json!({
                            "state": session.state,
                            "output": session.output,
                        }))
                        .ok()?;
                    Some((Ok::<_, Infallible>(event), (state, session_id)))
                }
                _ => None,
            }
        },
    );

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}
