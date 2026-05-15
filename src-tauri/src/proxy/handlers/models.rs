use crate::protocol::openai_types::{ModelInfo, ModelsListResponse};
use crate::proxy::router::ProxyState;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn handle_list_models(State(state): State<ProxyState>) -> Response {
    let db_guard = state.db.lock().await;

    let config = match crate::storage::config_repo::get_proxy_config(&db_guard) {
        Ok(c) => c,
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

    let data: Vec<ModelInfo> = if let Some(shadow_id) = config.shadow_mapping_id {
        // Return only the shadow model
        match crate::storage::model_repo::get_model(&db_guard, &shadow_id) {
            Ok(Some(_m)) => vec![ModelInfo {
                id: config.shadow_model_name.clone(),
                object: "model".to_string(),
                created: now,
                owned_by: "oh-my-llm".to_string(),
            }],
            _ => {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Shadow model mapping not found".to_string(),
                )
                    .into_response();
            }
        }
    } else {
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

        models
            .into_iter()
            .map(|m| ModelInfo {
                id: m.exposed_name,
                object: "model".to_string(),
                created: now,
                owned_by: "oh-my-llm".to_string(),
            })
            .collect()
    };

    let response = ModelsListResponse {
        object: "list".to_string(),
        data,
    };

    Json(response).into_response()
}
