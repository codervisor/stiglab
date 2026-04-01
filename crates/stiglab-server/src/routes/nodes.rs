use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::db;
use crate::state::AppState;

pub async fn list_nodes(State(state): State<AppState>) -> impl IntoResponse {
    match db::list_nodes(&state.db).await {
        Ok(nodes) => Json(serde_json::json!({ "nodes": nodes })).into_response(),
        Err(e) => {
            tracing::error!("failed to list nodes: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
