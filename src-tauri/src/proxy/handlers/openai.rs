use crate::protocol::openai_types::{ChatCompletionRequest, ErrorResponse, ApiError};
use crate::proxy::router::{resolve_model_route, resolve_shadow_route, select_client, ProxyState};
use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use std::time::Instant;

pub async fn handle_chat_completions(
    State(state): State<ProxyState>,
    req: Request<Body>,
) -> Response {
    let start = Instant::now();

    let bytes = match to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return build_error_response(
                StatusCode::BAD_REQUEST,
                &format!("Failed to read request body: {}", e),
            );
        }
    };

    let mut request: ChatCompletionRequest = match serde_json::from_slice(&bytes) {
        Ok(r) => r,
        Err(e) => {
            return build_error_response(
                StatusCode::BAD_REQUEST,
                &format!("Invalid JSON: {}", e),
            );
        }
    };

    let original_model = request.model.clone();
    let is_stream = request.stream.unwrap_or(false);

    // Resolve route: shadow model or direct model
    let db_guard = state.db.lock().await;
    let config = match crate::storage::config_repo::get_proxy_config(&db_guard) {
        Ok(c) => c,
        Err(_) => {
            drop(db_guard);
            return build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to read proxy config");
        }
    };

    let ctx = if let Some(shadow_id) = &config.shadow_mapping_id {
        match resolve_shadow_route(&db_guard, shadow_id) {
            Ok(c) => {
                request.model = c.upstream_model.clone();
                c
            }
            Err(e) => {
                drop(db_guard);
                return build_error_response(StatusCode::NOT_FOUND, &e);
            }
        }
    } else {
        match resolve_model_route(&db_guard, &original_model) {
            Ok(c) => c,
            Err(e) => {
                drop(db_guard);
                return build_error_response(StatusCode::NOT_FOUND, &e);
            }
        }
    };

    let prov_type = ctx.provider.prov_type.clone();
    let provider_id = ctx.provider.id.clone();
    let upstream_model = ctx.upstream_model.clone();
    let log_requests = config.log_requests;

    let client = match select_client(&state, &prov_type) {
        Ok(c) => c,
        Err(e) => {
            drop(db_guard);
            return build_error_response(StatusCode::BAD_REQUEST, &e);
        }
    };
    drop(db_guard);

    let result = client.chat_completion(ctx, request).await;
    let latency_ms = start.elapsed().as_millis() as i64;

    match result {
        Ok(resp) => {
            let status_code = resp.status().as_u16() as i64;

            if log_requests {
                let request_body_str = String::from_utf8_lossy(&bytes).to_string();

                if is_stream {
                    let _ = log_request(
                        &state,
                        &original_model,
                        &provider_id,
                        &upstream_model,
                        true,
                        latency_ms,
                        status_code,
                        0,
                        0,
                        0.0,
                        &request_body_str,
                        "[streaming]",
                    );
                    return resp;
                }

                let body_bytes = match axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024).await {
                    Ok(b) => b,
                    Err(_) => {
                        let _ = log_request(
                            &state,
                            &original_model,
                            &provider_id,
                            &upstream_model,
                            false,
                            latency_ms,
                            status_code,
                            0,
                            0,
                            0.0,
                            &request_body_str,
                            "[error reading body]",
                        );
                        return build_error_response(StatusCode::BAD_GATEWAY, "Failed to read upstream response");
                    }
                };
                let response_body_str = String::from_utf8_lossy(&body_bytes).to_string();
                let (prompt_tokens, completion_tokens, cost) =
                    extract_usage(&response_body_str, &provider_id, upstream_model.clone());

                let _ = log_request(
                    &state,
                    &original_model,
                    &provider_id,
                    &upstream_model,
                    false,
                    latency_ms,
                    status_code,
                    prompt_tokens,
                    completion_tokens,
                    cost,
                    &request_body_str,
                    &response_body_str,
                );

                let mut response = Response::new(Body::from(body_bytes));
                *response.status_mut() = StatusCode::from_u16(status_code as u16).unwrap_or(StatusCode::OK);
                return response;
            }

            resp
        }
        Err(e) => {
            if log_requests {
                let request_body_str = String::from_utf8_lossy(&bytes).to_string();
                let _ = log_request(
                    &state,
                    &original_model,
                    &provider_id,
                    &upstream_model,
                    is_stream,
                    latency_ms,
                    502,
                    0,
                    0,
                    0.0,
                    &request_body_str,
                    &e,
                );
            }
            build_error_response(
                StatusCode::BAD_GATEWAY,
                &format!("Upstream error: {}", e),
            )
        }
    }
}

fn get_token(val: &serde_json::Value, keys: &[&str]) -> i64 {
    for key in keys {
        if let Some(v) = val.get(key) {
            if let Some(n) = v.as_i64() {
                return n;
            }
            if let Some(n) = v.as_u64() {
                return n as i64;
            }
            if let Some(n) = v.as_f64() {
                return n as i64;
            }
        }
    }
    0
}

fn extract_usage(response_body: &str, _provider_id: &str, _upstream_model: String) -> (i64, i64, f64) {
    let val = match serde_json::from_str::<serde_json::Value>(response_body) {
        Ok(v) => v,
        Err(_) => return (0, 0, 0.0),
    };

    let usage = val.get("usage").unwrap_or(&serde_json::Value::Null);

    let prompt = get_token(usage, &["prompt_tokens", "input_tokens", "prompt", "input"]);
    let completion = get_token(usage, &["completion_tokens", "output_tokens", "completion", "output"]);

    // If prompt/completion are both 0, try to use total_tokens as a fallback hint
    let total = get_token(usage, &["total_tokens", "total"]);

    let (prompt, completion) = if prompt == 0 && completion == 0 && total > 0 {
        // No breakdown available — assign all to prompt as conservative estimate
        (total, 0)
    } else {
        (prompt, completion)
    };

    // Simple cost estimation: assume $2 / 1M tokens for input, $6 / 1M for output if no price config
    let cost = (prompt as f64 * 2.0 + completion as f64 * 6.0) / 1_000_000.0;
    (prompt, completion, cost)
}

fn log_request(
    state: &ProxyState,
    model: &str,
    provider_id: &str,
    upstream_model: &str,
    stream: bool,
    latency_ms: i64,
    status_code: i64,
    prompt_tokens: i64,
    completion_tokens: i64,
    cost: f64,
    request_body: &str,
    response_body: &str,
) {
    let mut conn = match state.db.try_lock() {
        Ok(g) => g,
        Err(_) => return,
    };

    let entry = crate::logging::recorder::LogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
        protocol: "openai".to_string(),
        model: model.to_string(),
        provider_id: Some(provider_id.to_string()),
        upstream_model: Some(upstream_model.to_string()),
        stream,
        latency_ms,
        status_code,
        prompt_tokens,
        completion_tokens,
        cost,
        error_type: if status_code >= 400 { Some("upstream_error".to_string()) } else { None },
        error_message: if status_code >= 400 { Some(response_body.to_string()) } else { None },
    };

    let _ = crate::logging::recorder::record_request_log(
        &mut conn,
        &entry,
        request_body,
        response_body,
    );

    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let _ = crate::stats::aggregator::record_usage(
        &mut conn,
        &date,
        upstream_model,
        provider_id,
        prompt_tokens,
        completion_tokens,
        cost,
    );
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
