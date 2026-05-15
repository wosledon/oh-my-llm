use crate::protocol::anthropic_types::AnthropicRequest;
use crate::protocol::translator::{anthropic_to_openai_request, openai_to_anthropic_response};
use crate::proxy::router::{resolve_model_route, resolve_shadow_route, ProxyState};
use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, HeaderName, CONTENT_TYPE};
use std::time::Instant;

pub async fn handle_anthropic_messages(
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

    let mut request: AnthropicRequest = match serde_json::from_slice(&bytes) {
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
    let api_key = ctx.provider.api_key.clone();
    let base_url = ctx.provider.base_url.clone();
    let extra_headers = ctx.provider.extra_headers.clone();
    drop(db_guard);

    // If upstream is Anthropic, forward directly (bypass ProviderClient to preserve native format)
    if prov_type == "anthropic" {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-api-key"),
            HeaderValue::from_str(&api_key)
                .unwrap_or_else(|_| HeaderValue::from_static("")),
        );
        headers.insert(
            HeaderName::from_static("anthropic-version"),
            HeaderValue::from_static("2023-06-01"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(extra) = extra_headers {
            if let Ok(map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&extra) {
                for (k, v) in map {
                    if let Some(val) = v.as_str() {
                        if let Ok(hv) = HeaderValue::from_str(val) {
                            if let Ok(hn) = HeaderName::from_bytes(k.as_bytes()) {
                                headers.insert(hn, hv);
                            }
                        }
                    }
                }
            }
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client");

        let url = format!("{}/messages", base_url.trim_end_matches('/'));

        let resp = match client.post(&url).headers(headers).json(&request).send().await {
            Ok(r) => r,
            Err(e) => {
                return build_error_response(StatusCode::BAD_GATEWAY, &format!("Upstream error: {}", e));
            }
        };

        let status = resp.status();

        if is_stream {
            let stream = resp.bytes_stream();
            let body_stream = Body::from_stream(stream.map(|result| result.map_err(|e| axum::Error::new(e))));
            let mut response = Response::new(body_stream);
            *response.status_mut() = status;
            response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"));
            return response;
        } else {
            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    return build_error_response(StatusCode::BAD_GATEWAY, &format!("Failed to read upstream response: {}", e));
                }
            };
            let mut response = Response::new(Body::from(bytes));
            *response.status_mut() = status;
            return response;
        }
    }

    // For OpenAI / OpenAI-compatible upstream: translate Anthropic -> OpenAI, send, then translate back
    if is_stream {
        return build_error_response(StatusCode::NOT_IMPLEMENTED, "Streaming is not supported for Anthropic-format requests routed to OpenAI-compatible providers");
    }

    let openai_req = anthropic_to_openai_request(request);
    let client = match crate::proxy::router::select_client(&state, &prov_type) {
        Ok(c) => c,
        Err(e) => return build_error_response(StatusCode::BAD_REQUEST, &e),
    };

    let result = client.chat_completion(ctx, openai_req).await;
    let _latency_ms = start.elapsed().as_millis() as i64;

    match result {
        Ok(resp) => {
            let status_code = resp.status().as_u16() as i64;
            let body_bytes = match axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024).await {
                Ok(b) => b,
                Err(_) => {
                    return build_error_response(StatusCode::BAD_GATEWAY, "Failed to read upstream response");
                }
            };

            let openai_resp: crate::protocol::openai_types::ChatCompletionResponse = match serde_json::from_slice(&body_bytes) {
                Ok(r) => r,
                Err(_) => {
                    return build_error_response(StatusCode::BAD_GATEWAY, "Invalid upstream response format");
                }
            };

            let anthropic_resp = openai_to_anthropic_response(openai_resp);
            let body = match serde_json::to_vec(&anthropic_resp) {
                Ok(b) => b,
                Err(_) => {
                    return build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to serialize response");
                }
            };

            let mut response = Response::new(Body::from(body));
            *response.status_mut() = StatusCode::from_u16(status_code as u16).unwrap_or(StatusCode::OK);
            response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            response
        }
        Err(e) => {
            build_error_response(
                StatusCode::BAD_GATEWAY,
                &format!("Upstream error: {}", e),
            )
        }
    }
}

fn build_error_response(status: StatusCode, message: &str) -> Response {
    let body = crate::protocol::anthropic_types::AnthropicErrorResponse {
        error_type: "api_error".to_string(),
        error: crate::protocol::anthropic_types::AnthropicErrorDetail {
            detail_type: "proxy_error".to_string(),
            message: message.to_string(),
        },
    };
    (status, axum::Json(body)).into_response()
}
