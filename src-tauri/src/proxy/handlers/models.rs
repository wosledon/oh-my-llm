use crate::protocol::openai_types::{ModelInfo, ModelsListResponse};
use crate::proxy::router::ProxyState;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn handle_list_models(State(state): State<ProxyState>) -> Response {
    let db_guard = state.db.lock().await;

    let models = match crate::storage::model_repo::list_models(&db_guard, None) {
        Ok(list) => list,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
                .into_response();
        }
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let data: Vec<ModelInfo> = models
        .into_iter()
        .map(|m| ModelInfo {
            id: m.exposed_name,
            object: "model".to_string(),
            created: now,
            owned_by: "oh-my-llm".to_string(),
        })
        .collect();

    let response = ModelsListResponse {
        object: "list".to_string(),
        data,
    };

    Json(response).into_response()
}
