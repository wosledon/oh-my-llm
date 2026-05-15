use crate::protocol::openai_types::{ApiError, ChatCompletionRequest, ErrorResponse};
use crate::proxy::router::{resolve_model_route, resolve_shadow_route, select_client, ProxyState};
use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use std::time::Instant;

// ── Think tag filter (cross-chunk stateful) ──────────────────────────────

#[derive(Clone)]
struct ThinkTagConfig {
    start: String,
    end: String,
}

impl ThinkTagConfig {
    fn think() -> Self {
        Self {
            start: "<think>".to_string(),
            end: "</think>".to_string(),
        }
    }
}

/// Return a config if the upstream model is known to wrap reasoning in think tags.
/// Models are matched by name heuristics.  Unmatched models pass through untouched.
fn needs_think_filter(model_name: &str) -> Option<ThinkTagConfig> {
    let name = model_name.to_lowercase();

    // DeepSeek reasoning models (R1, Reasoner, V4)
    if name.contains("deepseek")
        && (name.contains("reasoner") || name.contains("r1") || name.contains("v4"))
    {
        return Some(ThinkTagConfig::think());
    }

    // Qwen QwQ series
    if name.contains("qwq") || name.contains("qwen-qwq") {
        return Some(ThinkTagConfig::think());
    }

    // Kimi / Moonshot K1 / reasoning variants
    if name.contains("kimi") && (name.contains("k1") || name.contains("reasoning")) {
        return Some(ThinkTagConfig::think());
    }

    // Xiaomi MiMo (GCMP reference)
    if name.starts_with("mimo-") || name.contains("mimo-v") || name.contains("xiaomimimo") {
        return Some(ThinkTagConfig::think());
    }

    // Generic fallback: any model id containing "reasoning" or "think"
    if name.contains("reasoning") || name.contains("-think-") {
        return Some(ThinkTagConfig::think());
    }

    None
}

struct ThinkFilter {
    pending: String,
    in_think: bool,
    sse_buffer: String,
    config: ThinkTagConfig,
}

/// Move prefix bytes from `s` to a new string, leaving at most `tail_len` bytes.
/// Split point is guaranteed to be on a UTF-8 character boundary.
fn drain_prefix(s: &mut String, tail_len: usize) -> String {
    if s.len() <= tail_len {
        return String::new();
    }
    let mut split = s.len() - tail_len;
    while split > 0 && !s.is_char_boundary(split) {
        split -= 1;
    }
    let prefix = s[..split].to_string();
    *s = s[split..].to_string();
    prefix
}

/// Return the tail of `s` with at most `tail_len` bytes, starting on a char boundary.
fn safe_tail(s: &str, tail_len: usize) -> &str {
    if s.len() <= tail_len {
        return s;
    }
    let mut split = s.len() - tail_len;
    while split > 0 && !s.is_char_boundary(split) {
        split -= 1;
    }
    &s[split..]
}

impl ThinkFilter {
    fn new(config: ThinkTagConfig) -> Self {
        Self {
            pending: String::new(),
            in_think: false,
            sse_buffer: String::new(),
            config,
        }
    }

    /// Process incoming text and return (reasoning_content, visible_content).
    /// Handles think tags that may be split across multiple process() calls.
    fn process(&mut self, text: &str) -> (String, String) {
        self.pending.push_str(text);
        let mut reasoning = String::new();
        let mut content = String::new();
        let start_len = self.config.start.len();
        let end_len = self.config.end.len();

        loop {
            if self.in_think {
                if let Some(pos) = self.pending.find(&self.config.end) {
                    reasoning.push_str(&self.pending[..pos]);
                    self.pending = self.pending[pos + end_len..].to_string();
                    self.in_think = false;
                } else {
                    // Streaming: emit everything except the trailing bytes
                    // that could be part of the end tag.  This keeps
                    // reasoning_content flowing in real-time instead of
                    // buffering until the tag is closed.
                    if self.pending.len() > end_len {
                        reasoning.push_str(&drain_prefix(&mut self.pending, end_len));
                    }
                    break;
                }
            } else {
                if let Some(pos) = self.pending.find(&self.config.start) {
                    content.push_str(&self.pending[..pos]);
                    self.pending = self.pending[pos + start_len..].to_string();
                    self.in_think = true;
                } else {
                    // Keep at most (start_len - 1) trailing chars.
                    // If the first char of start tag is not in the tail,
                    // it's safe to emit everything.
                    let keep = start_len.saturating_sub(1);
                    if self.pending.len() > keep {
                        let first_char = self.config.start.chars().next().unwrap_or('<');
                        if safe_tail(&self.pending, keep).contains(first_char) {
                            content.push_str(&drain_prefix(&mut self.pending, keep));
                        } else {
                            content.push_str(&self.pending);
                            self.pending.clear();
                        }
                    }
                    break;
                }
            }
        }

        (reasoning, content)
    }

    fn flush(self) -> (String, String) {
        if self.in_think {
            (self.pending, String::new())
        } else {
            (String::new(), self.pending)
        }
    }

    /// Consume one raw SSE chunk, parse complete events, filter think tags,
    /// and return the transformed bytes. Incomplete events are kept in `sse_buffer`.
    fn process_sse_chunk(&mut self, chunk: &bytes::Bytes) -> bytes::Bytes {
        let text = String::from_utf8_lossy(chunk);
        self.sse_buffer.push_str(&text);
        let mut output = Vec::new();

        while let Some(pos) = self.sse_buffer.find("\n\n") {
            let event = self.sse_buffer[..pos].to_string();
            self.sse_buffer = self.sse_buffer[pos + 2..].to_string();

            let mut data_line: Option<String> = None;
            let mut other_lines: Vec<String> = Vec::new();
            for line in event.lines() {
                if line.starts_with("data: ") {
                    data_line = Some(line[6..].to_string());
                } else {
                    other_lines.push(line.to_string());
                }
            }

            if let Some(data) = data_line {
                if data == "[DONE]" {
                    // flush any remaining reasoning before DONE
                    let (final_reasoning, final_content) = {
                        let f = std::mem::replace(self, ThinkFilter::new(self.config.clone()));
                        f.flush()
                    };
                    if !final_reasoning.is_empty() || !final_content.is_empty() {
                        let mut flush_json = serde_json::json!({
                            "choices": [{"index":0,"delta":{},"finish_reason":null}]
                        });
                        if !final_reasoning.is_empty() {
                            flush_json["choices"][0]["delta"]["reasoning_content"] =
                                serde_json::Value::String(final_reasoning);
                        }
                        if !final_content.is_empty() {
                            flush_json["choices"][0]["delta"]["content"] =
                                serde_json::Value::String(final_content);
                        }
                        output.extend_from_slice(
                            format!(
                                "data: {}\n",
                                serde_json::to_string(&flush_json).unwrap_or_default()
                            )
                            .as_bytes(),
                        );
                        for line in &other_lines {
                            output.extend_from_slice(format!("\n{}", line).as_bytes());
                        }
                        output.extend_from_slice(b"\n\n");
                    }
                    output.extend_from_slice(b"data: [DONE]\n\n");
                    continue;
                }

                if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(content) = json
                        .pointer("/choices/0/delta/content")
                        .and_then(|c| c.as_str())
                    {
                        let (reasoning, new_content) = self.process(content);
                        if !reasoning.is_empty() {
                            json["choices"][0]["delta"]["reasoning_content"] =
                                serde_json::Value::String(reasoning);
                        }
                        if new_content.is_empty() {
                            json["choices"][0]["delta"].as_object_mut().map(|m| {
                                m.remove("content");
                            });
                        } else {
                            json["choices"][0]["delta"]["content"] =
                                serde_json::Value::String(new_content);
                        }
                    }
                    output.extend_from_slice(
                        format!("data: {}", serde_json::to_string(&json).unwrap_or_default())
                            .as_bytes(),
                    );
                    for line in &other_lines {
                        output.extend_from_slice(format!("\n{}", line).as_bytes());
                    }
                    output.extend_from_slice(b"\n\n");
                } else {
                    output.extend_from_slice(event.as_bytes());
                    output.extend_from_slice(b"\n\n");
                }
            } else {
                output.extend_from_slice(event.as_bytes());
                output.extend_from_slice(b"\n\n");
            }
        }

        bytes::Bytes::from(output)
    }
}

/// Strip think tags from non-stream JSON response and move it to `reasoning_content`.
fn strip_think_tags(body: &str, config: &ThinkTagConfig) -> String {
    let mut val: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return body.to_string(),
    };

    let start_len = config.start.len();
    let end_len = config.end.len();

    if let Some(choices) = val.get_mut("choices").and_then(|c| c.as_array_mut()) {
        for choice in choices {
            if let Some(msg) = choice.get_mut("message") {
                let content_str: Option<String> = msg
                    .get("content")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());
                if let Some(content_str) = content_str {
                    if let Some(start) = content_str.find(&config.start) {
                        if let Some(end) = content_str.find(&config.end) {
                            let before = content_str[..start].trim_end();
                            let think = content_str[start + start_len..end].trim();
                            let after = content_str[end + end_len..].trim_start();
                            let new_content = format!("{}{}", before, after);
                            msg["content"] = serde_json::Value::String(new_content);
                            if !think.is_empty() {
                                msg["reasoning_content"] =
                                    serde_json::Value::String(think.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    serde_json::to_string(&val).unwrap_or_else(|_| body.to_string())
}

// ── Handler ──────────────────────────────────────────────────────────────

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
            return build_error_response(StatusCode::BAD_REQUEST, &format!("Invalid JSON: {}", e));
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
    let provider_id = ctx.provider.id.clone();
    let upstream_model = ctx.upstream_model.clone();
    let log_requests = config.log_requests;

    // Query model prices before dropping db guard
    let models =
        crate::storage::model_repo::list_models(&db_guard, Some(&provider_id)).unwrap_or_default();
    let model_info = models.iter().find(|m| m.upstream_name == upstream_model);
    let input_price = model_info.map(|m| m.input_price).unwrap_or(2.0);
    let output_price = model_info.map(|m| m.output_price).unwrap_or(6.0);

    let client = match select_client(&state, &prov_type) {
        Ok(c) => c,
        Err(e) => {
            drop(db_guard);
            return build_error_response(StatusCode::BAD_REQUEST, &e);
        }
    };
    drop(db_guard);

    // Pre-compute prompt token estimate before request is moved
    let estimated_prompt = estimate_prompt_tokens(&request);

    let result = client.chat_completion(ctx, request).await;
    let latency_ms = start.elapsed().as_millis() as i64;

    match result {
        Ok(resp) => {
            let status_code = resp.status().as_u16() as i64;
            let request_body_str = String::from_utf8_lossy(&bytes).to_string();

            if is_stream {
                // ── Streaming: transform think tags in real-time ──
                let status = resp.status();
                let headers = resp.headers().clone();

                if let Some(config) = needs_think_filter(&upstream_model) {
                    let body = resp.into_body();
                    let stream = body.into_data_stream();
                    let mut filter = ThinkFilter::new(config);

                    let transformed = stream.map(move |result| match result {
                        Ok(chunk) => {
                            let out = filter.process_sse_chunk(&chunk);
                            Ok(out)
                        }
                        Err(e) => Err(e),
                    });

                    let new_body = Body::from_stream(transformed);
                    let mut response = Response::new(new_body);
                    *response.status_mut() = status;
                    *response.headers_mut() = headers;

                    if log_requests {
                        let _ = log_request(
                            &state,
                            &original_model,
                            &provider_id,
                            &upstream_model,
                            true,
                            latency_ms,
                            status_code,
                            estimated_prompt,
                            0,
                            0.0,
                            &request_body_str,
                            "[streaming]",
                        );
                    }
                    return response;
                }

                // No think-filter needed: pass through untouched
                let mut response = Response::new(resp.into_body());
                *response.status_mut() = status;
                *response.headers_mut() = headers;

                if log_requests {
                    let _ = log_request(
                        &state,
                        &original_model,
                        &provider_id,
                        &upstream_model,
                        true,
                        latency_ms,
                        status_code,
                        estimated_prompt,
                        0,
                        0.0,
                        &request_body_str,
                        "[streaming]",
                    );
                }
                return response;
            }

            // ── Non-streaming: strip think tags ──
            let body_bytes = match axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024).await {
                Ok(b) => b,
                Err(_) => {
                    if log_requests {
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
                    }
                    return build_error_response(
                        StatusCode::BAD_GATEWAY,
                        "Failed to read upstream response",
                    );
                }
            };

            let response_body_str = String::from_utf8_lossy(&body_bytes).to_string();
            let processed_body = match needs_think_filter(&upstream_model) {
                Some(ref config) => strip_think_tags(&response_body_str, config),
                None => response_body_str,
            };
            let (prompt_tokens, completion_tokens, cost) =
                extract_usage(&processed_body, input_price, output_price, estimated_prompt);

            if log_requests {
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
                    &processed_body,
                );
            }

            let mut response = Response::new(Body::from(processed_body.into_bytes()));
            *response.status_mut() =
                StatusCode::from_u16(status_code as u16).unwrap_or(StatusCode::OK);
            response
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
            build_error_response(StatusCode::BAD_GATEWAY, &format!("Upstream error: {}", e))
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

// ── Token estimation (heuristic, no external tokenizer) ─────────────────

/// Estimate token count from raw text.
/// Heuristic: mixed CJK/English ≈ chars / 2  (conservative).
/// GCMP uses @microsoft/tiktokenizer (o200k_base).  When a Rust tiktoken
/// equivalent is added, replace this with the real encoder.
fn estimate_tokens(text: &str) -> i64 {
    if text.is_empty() {
        return 0;
    }
    let chars = text.chars().count() as i64;
    // CJK chars ≈ 1–2 tokens, ASCII ≈ 0.25 tokens/char.
    // chars/2 is a safe mixed-language lower bound.
    (chars / 2).max(1)
}

/// Estimate prompt tokens from the request body.
fn estimate_prompt_tokens(req: &ChatCompletionRequest) -> i64 {
    let mut total = 0i64;
    for msg in &req.messages {
        // role overhead (≈ 3 tokens per message in ChatML-like formats)
        total += 3;
        if let Some(content) = &msg.content {
            match content {
                crate::protocol::openai_types::ChatContent::Text(t) => {
                    total += estimate_tokens(t);
                }
                crate::protocol::openai_types::ChatContent::Parts(parts) => {
                    for part in parts {
                        match part {
                            crate::protocol::openai_types::ContentPart::Text { text } => {
                                total += estimate_tokens(text);
                            }
                            crate::protocol::openai_types::ContentPart::ImageUrl { .. } => {
                                // Image placeholder — rough fixed cost
                                total += 85;
                            }
                        }
                    }
                }
            }
        }
        if let Some(name) = &msg.name {
            total += estimate_tokens(name);
        }
        if let Some(tool_calls) = &msg.tool_calls {
            for tc in tool_calls {
                total += estimate_tokens(&tc.function.name);
                total += estimate_tokens(&tc.function.arguments);
            }
        }
        if let Some(tool_call_id) = &msg.tool_call_id {
            total += estimate_tokens(tool_call_id);
        }
    }
    total
}

/// Estimate completion tokens from a non-streaming response body.
fn estimate_completion_tokens(response_body: &str) -> i64 {
    let val = match serde_json::from_str::<serde_json::Value>(response_body) {
        Ok(v) => v,
        Err(_) => return 0,
    };

    // Try to sum content from all choices
    let mut total = 0i64;
    if let Some(choices) = val.get("choices").and_then(|c| c.as_array()) {
        for choice in choices {
            let content = choice
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            total += estimate_tokens(content);

            // Also count reasoning_content if present
            let reasoning = choice
                .get("message")
                .and_then(|m| m.get("reasoning_content"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            total += estimate_tokens(reasoning);
        }
    }
    total
}

fn extract_usage(
    response_body: &str,
    input_price: f64,
    output_price: f64,
    estimated_prompt: i64,
) -> (i64, i64, f64) {
    let val = match serde_json::from_str::<serde_json::Value>(response_body) {
        Ok(v) => v,
        Err(_) => {
            // JSON parse failed — fall back to text estimation
            let completion = estimate_completion_tokens(response_body);
            let cost = (estimated_prompt as f64 * input_price + completion as f64 * output_price)
                / 1_000_000.0;
            return (estimated_prompt, completion, cost);
        }
    };

    let usage = val.get("usage").unwrap_or(&serde_json::Value::Null);

    let mut prompt = get_token(usage, &["prompt_tokens", "input_tokens", "prompt", "input"]);
    let mut completion = get_token(
        usage,
        &["completion_tokens", "output_tokens", "completion", "output"],
    );

    // If upstream omitted usage entirely, fall back to estimation
    if prompt == 0 && completion == 0 {
        prompt = estimated_prompt;
        completion = estimate_completion_tokens(response_body);
    }

    // If prompt/completion are both 0, try to use total_tokens as a fallback hint
    let total = get_token(usage, &["total_tokens", "total"]);

    let (prompt, completion) = if prompt == 0 && completion == 0 && total > 0 {
        (total, 0)
    } else {
        (prompt, completion)
    };

    // Cost estimation using configured model prices (USD per 1M tokens)
    let cost = (prompt as f64 * input_price + completion as f64 * output_price) / 1_000_000.0;
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
        error_type: if status_code >= 400 {
            Some("upstream_error".to_string())
        } else {
            None
        },
        error_message: if status_code >= 400 {
            Some(response_body.to_string())
        } else {
            None
        },
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
