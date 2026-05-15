use crate::protocol::openai_types::{ChatCompletionRequest, ErrorResponse, ApiError};
use crate::proxy::router::{resolve_model_route, select_client, ProxyState};
use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use std::time::Instant;

pub async fn handle_chat_completions(
    State(state): State<ProxyState>,
    req: Request<Body>,
) -> Response {
    let _start = Instant::now();

    let bytes = match to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return build_error_response(
                StatusCode::BAD_REQUEST,
                &format!("Failed to read request body: {}", e),
            );
        }
    };

    let request: ChatCompletionRequest = match serde_json::from_slice(&bytes) {
        Ok(r) => r,
        Err(e) => {
            return build_error_response(
                StatusCode::BAD_REQUEST,
                &format!("Invalid JSON: {}", e),
            );
        }
    };

    let model_name = request.model.clone();

    let db_guard = state.db.lock().await;
    let ctx = match resolve_model_route(&db_guard, &model_name) {
        Ok(c) => c,
        Err(e) => {
            return build_error_response(StatusCode::NOT_FOUND, &e);
        }
    };
    let prov_type = ctx.provider.prov_type.clone();

    let client = match select_client(&state, &prov_type) {
        Ok(c) => c,
        Err(e) => {
            return build_error_response(StatusCode::BAD_REQUEST, &e);
        }
    };
    drop(db_guard);

    match client.chat_completion(ctx, request).await {
        Ok(resp) => resp,
        Err(e) => build_error_response(
            StatusCode::BAD_GATEWAY,
            &format!("Upstream error: {}", e),
        ),
    }
}

fn build_error_response(status: StatusCode, message: &str) -> Response {
    let body = ErrorResponse {
        error: ApiError {
            message: message.to_string(),
            error_type: "proxy_error".to_string(),
            param: None,
            code: Some(status.as_u16().to_string()),
        },
    };
    (status, axum::Json(body)).into_response()
}
