use crate::protocol::anthropic_types::AnthropicRequest;
use crate::protocol::translator::{anthropic_to_openai_request, openai_to_anthropic_response};
use crate::proxy::router::{resolve_model_route, resolve_shadow_route, ProxyState};
use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use std::time::Instant;

pub async fn handle_anthropic_messages(
    State(state): State<ProxyState>,
    req: Request<Body>,
) -> Response {
    let start = Instant::now();

    let bytes = match to_bytes(req.into_body(), 50 * 1024 * 1024).await {
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
            return build_error_response(StatusCode::BAD_REQUEST, &format!("Invalid JSON: {}", e));
        }
    };

    let original_model = request.model.clone();
    let is_stream = request.stream.unwrap_or(false);

    let db_guard = state.db.lock().await;
    let config = match crate::storage::config_repo::get_proxy_config(&db_guard) {
        Ok(c) => c,
        Err(_) => {
            drop(db_guard);
            return build_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read proxy config",
            );
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
            HeaderValue::from_str(&api_key).unwrap_or_else(|_| HeaderValue::from_static("")),
        );
        headers.insert(
            HeaderName::from_static("anthropic-version"),
            HeaderValue::from_static("2023-06-01"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(extra) = extra_headers {
            if let Ok(map) =
                serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&extra)
            {
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

        let base = base_url.trim_end_matches('/');
        let url = if base.ends_with("/v1") {
            format!("{}/messages", base)
        } else {
            format!("{}/v1/messages", base)
        };

        let resp = match client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return build_error_response(
                    StatusCode::BAD_GATEWAY,
                    &format!("Upstream error: {}", e),
                );
            }
        };

        let status = resp.status();

        if is_stream {
            let stream = resp.bytes_stream();
            let body_stream =
                Body::from_stream(stream.map(|result| result.map_err(|e| axum::Error::new(e))));
            let mut response = Response::new(body_stream);
            *response.status_mut() = status;
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"));
            return response;
        } else {
            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    return build_error_response(
                        StatusCode::BAD_GATEWAY,
                        &format!("Failed to read upstream response: {}", e),
                    );
                }
            };
            let mut response = Response::new(Body::from(bytes));
            *response.status_mut() = status;
            return response;
        }
    }

    // For OpenAI / OpenAI-compatible upstream: translate Anthropic -> OpenAI, send, then translate back
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

            if is_stream {
                // Convert OpenAI SSE stream to Anthropic SSE stream
                let model = original_model.clone();
                let (tx, rx) = tokio::sync::mpsc::channel::<Result<bytes::Bytes, axum::Error>>(32);

                tokio::spawn(async move {
                    let mut stream = resp.into_body().into_data_stream();
                    let mut buffer = String::new();
                    let mut sent_start = false;
                    let mut id = String::new();
                    let mut think_filter = crate::proxy::handlers::openai::ThinkFilter::new();
                    let mut pending_tool_calls: Vec<crate::protocol::openai_types::ToolCall> =
                        Vec::new();
                    let mut tool_call_block_index: usize = 1; // text block is index 0

                    while let Some(chunk) = stream.next().await {
                        match chunk {
                            Ok(data_bytes) => {
                                buffer.push_str(&String::from_utf8_lossy(&data_bytes));
                                loop {
                                    if let Some(line_end) = buffer.find('\n') {
                                        let line = buffer[..line_end].trim().to_string();
                                        buffer = buffer[line_end + 1..].to_string();

                                        if line.starts_with("data: ") {
                                            let data = line[6..].trim();
                                            if data == "[DONE]" {
                                                let _ = tx.send(Ok(bytes::Bytes::from("event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n"))).await;
                                                return;
                                            }

                                            if let Ok(json) =
                                                serde_json::from_str::<serde_json::Value>(data)
                                            {
                                                let chunk_id = json
                                                    .get("id")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                                    .to_string();
                                                if !chunk_id.is_empty() {
                                                    id = chunk_id;
                                                }

                                                if let Some(choices) =
                                                    json.get("choices").and_then(|c| c.as_array())
                                                {
                                                    if let Some(choice) = choices.first() {
                                                        let delta = choice.get("delta");
                                                        let finish_reason = choice
                                                            .get("finish_reason")
                                                            .and_then(|f| f.as_str());

                                                        if !sent_start {
                                                            let start = serde_json::json!({
                                                                "type": "message_start",
                                                                "message": {
                                                                    "id": id.clone(),
                                                                    "type": "message",
                                                                    "role": "assistant",
                                                                    "model": model,
                                                                    "content": [],
                                                                    "stop_reason": null,
                                                                    "stop_sequence": null,
                                                                    "usage": { "input_tokens": 0, "output_tokens": 0 }
                                                                }
                                                            });
                                                            let _ = tx.send(Ok(bytes::Bytes::from(format!("event: message_start\ndata: {}\n\n", start)))).await;

                                                            let block_start = serde_json::json!({
                                                                "type": "content_block_start",
                                                                "index": 0,
                                                                "content_block": { "type": "text", "text": "" }
                                                            });
                                                            let _ = tx.send(Ok(bytes::Bytes::from(format!("event: content_block_start\ndata: {}\n\n", block_start)))).await;

                                                            let ping =
                                                                serde_json::json!({"type": "ping"});
                                                            let _ = tx
                                                                .send(Ok(bytes::Bytes::from(
                                                                    format!(
                                                                        "event: ping\ndata: {}\n\n",
                                                                        ping
                                                                    ),
                                                                )))
                                                                .await;

                                                            sent_start = true;
                                                        }

                                                        if let Some(delta_obj) = delta {
                                                            // content delta
                                                            if let Some(content) = delta_obj
                                                                .get("content")
                                                                .and_then(|c| c.as_str())
                                                            {
                                                                let (_reasoning, visible) =
                                                                    think_filter.process(content);
                                                                if !visible.is_empty() {
                                                                    let delta = serde_json::json!({
                                                                        "type": "content_block_delta",
                                                                        "index": 0,
                                                                        "delta": { "type": "text_delta", "text": visible }
                                                                    });
                                                                    let _ = tx.send(Ok(bytes::Bytes::from(format!("event: content_block_delta\ndata: {}\n\n", delta)))).await;
                                                                }
                                                            }

                                                            // tool_calls delta — accumulate
                                                            if let Some(tc_array) = delta_obj
                                                                .get("tool_calls")
                                                                .and_then(|c| c.as_array())
                                                            {
                                                                for tc in tc_array {
                                                                    if let Ok(tool_call) = serde_json::from_value::<crate::protocol::openai_types::ToolCall>(tc.clone()) {
                                                                        if let Some(existing) = pending_tool_calls.iter_mut().find(|t| t.id == tool_call.id) {
                                                                            if !tool_call.function.name.is_empty() {
                                                                                existing.function.name = tool_call.function.name.clone();
                                                                            }
                                                                            existing.function.arguments.push_str(&tool_call.function.arguments);
                                                                        } else {
                                                                            pending_tool_calls.push(tool_call);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        if let Some(fr) = finish_reason {
                                                            // close text block
                                                            let block_stop = serde_json::json!({
                                                                "type": "content_block_stop",
                                                                "index": 0
                                                            });
                                                            let _ = tx.send(Ok(bytes::Bytes::from(format!("event: content_block_stop\ndata: {}\n\n", block_stop)))).await;

                                                            // emit accumulated tool_calls as tool_use blocks
                                                            for call in &pending_tool_calls {
                                                                let _input = serde_json::from_str(
                                                                    &call.function.arguments,
                                                                )
                                                                .unwrap_or(serde_json::Value::Null);
                                                                let t_start = serde_json::json!({
                                                                    "type": "content_block_start",
                                                                    "index": tool_call_block_index,
                                                                    "content_block": {
                                                                        "type": "tool_use",
                                                                        "id": call.id,
                                                                        "name": call.function.name,
                                                                        "input": {}
                                                                    }
                                                                });
                                                                let _ = tx.send(Ok(bytes::Bytes::from(format!("event: content_block_start\ndata: {}\n\n", t_start)))).await;
                                                                let t_delta = serde_json::json!({
                                                                    "type": "content_block_delta",
                                                                    "index": tool_call_block_index,
                                                                    "delta": {
                                                                        "type": "input_json_delta",
                                                                        "partial_json": call.function.arguments
                                                                    }
                                                                });
                                                                let _ = tx.send(Ok(bytes::Bytes::from(format!("event: content_block_delta\ndata: {}\n\n", t_delta)))).await;
                                                                let t_stop = serde_json::json!({
                                                                    "type": "content_block_stop",
                                                                    "index": tool_call_block_index
                                                                });
                                                                let _ = tx.send(Ok(bytes::Bytes::from(format!("event: content_block_stop\ndata: {}\n\n", t_stop)))).await;
                                                                tool_call_block_index += 1;
                                                            }

                                                            let stop_reason = if fr == "stop" {
                                                                "end_turn"
                                                            } else {
                                                                fr
                                                            };
                                                            let msg_delta = serde_json::json!({
                                                                "type": "message_delta",
                                                                "delta": {
                                                                    "stop_reason": stop_reason,
                                                                    "stop_sequence": null
                                                                },
                                                                "usage": { "output_tokens": 0 }
                                                            });
                                                            let _ = tx.send(Ok(bytes::Bytes::from(format!("event: message_delta\ndata: {}\n\n", msg_delta)))).await;

                                                            let _ = tx.send(Ok(bytes::Bytes::from("event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n"))).await;
                                                            return;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(e)).await;
                                break;
                            }
                        }
                    }
                });

                let body_stream =
                    Body::from_stream(tokio_stream::wrappers::ReceiverStream::new(rx));
                let mut response = Response::new(body_stream);
                *response.status_mut() =
                    StatusCode::from_u16(status_code as u16).unwrap_or(StatusCode::OK);
                response
                    .headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"));
                return response;
            }

            let body_bytes = match axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024).await {
                Ok(b) => b,
                Err(_) => {
                    return build_error_response(
                        StatusCode::BAD_GATEWAY,
                        "Failed to read upstream response",
                    );
                }
            };

            let body_str = String::from_utf8_lossy(&body_bytes);
            let filtered_body = crate::proxy::handlers::openai::strip_think_tags(&body_str);
            let openai_resp: crate::protocol::openai_types::ChatCompletionResponse =
                match serde_json::from_str(&filtered_body) {
                    Ok(r) => r,
                    Err(_) => {
                        return build_error_response(
                            StatusCode::BAD_GATEWAY,
                            "Invalid upstream response format",
                        );
                    }
                };

            let anthropic_resp = openai_to_anthropic_response(openai_resp);
            let body = match serde_json::to_vec(&anthropic_resp) {
                Ok(b) => b,
                Err(_) => {
                    return build_error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to serialize response",
                    );
                }
            };

            let mut response = Response::new(Body::from(body));
            *response.status_mut() =
                StatusCode::from_u16(status_code as u16).unwrap_or(StatusCode::OK);
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            response
        }
        Err(e) => build_error_response(StatusCode::BAD_GATEWAY, &format!("Upstream error: {}", e)),
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
