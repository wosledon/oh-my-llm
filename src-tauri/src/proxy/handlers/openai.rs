use crate::protocol::openai_types::{ApiError, ChatCompletionRequest, ErrorResponse};
use crate::proxy::router::{resolve_model_route, resolve_shadow_route, select_client, ProxyState};
use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Instant;

// ── Think tag filter (cross-chunk stateful) ──────────────────────────────
// All OpenAI-compatible responses are filtered for `<think>...</think>` tags.
// Models that use a different format should be handled upstream before reaching
// this handler.

const THINK_START: &str = "<think>";
const THINK_END: &str = "</think>";
const THINK_START_LEN: usize = 7;
const THINK_END_LEN: usize = 8;

pub struct ThinkFilter {
    pending: String,
    /// State machine:
    ///   false  → still looking for `<think>` at the head of the response
    ///   true   → inside `<think>...</think>`, buffering reasoning
    ///   done   → the first `</think>` has been closed; everything after is
    ///            normal content and passes through untouched.
    in_think: bool,
    done: bool,
    sse_buffer: String,
    /// Accumulated output characters (reasoning + visible content) for
    /// token estimation.  Only meaningful in streaming mode.
    output_chars: usize,
    /// Shared counter for streaming completion-token estimate.
    /// Written when [DONE] is processed.
    completion_estimate: Option<Arc<AtomicI64>>,
}

impl ThinkFilter {
    pub fn new() -> Self {
        Self {
            pending: String::new(),
            in_think: false,
            done: false,
            sse_buffer: String::new(),
            output_chars: 0,
            completion_estimate: None,
        }
    }

    fn with_completion_counter(counter: Arc<AtomicI64>) -> Self {
        let mut f = Self::new();
        f.completion_estimate = Some(counter);
        f
    }

    /// Convert accumulated output characters to a token estimate and write it
    /// to the shared counter (if one was provided).
    fn write_completion_estimate(&self) {
        if let Some(counter) = &self.completion_estimate {
            // chars / 2 is the same heuristic used by estimate_tokens().
            let tokens = (self.output_chars / 2).max(1) as i64;
            counter.store(tokens, Ordering::Relaxed);
        }
    }

    /// Process incoming text and return (reasoning_content, visible_content).
    /// After the first `</think>` is seen the filter becomes a no-op.
    pub fn process(&mut self, text: &str) -> (String, String) {
        if self.done {
            self.output_chars += text.chars().count();
            return (String::new(), text.to_string());
        }

        // Aggressive fast path: when we are not inside a think block and have
        // no pending tail, look at the *start* of `text`.
        if !self.in_think && self.pending.is_empty() {
            let trimmed = text.trim_start();
            if !trimmed.starts_with('<') {
                // Definitely not a think tag — pass through untouched.
                self.output_chars += text.chars().count();
                return (String::new(), text.to_string());
            }
            // Starts with '<'.  If the text is long enough to rule out
            // `<think>` and it is NOT `<think>`, pass through.
            if trimmed.len() >= THINK_START_LEN && !trimmed.starts_with(THINK_START) {
                self.output_chars += text.chars().count();
                return (String::new(), text.to_string());
            }
            // If the text is short (< 7 chars) and NOT a prefix of `<think>`,
            // it can never become a think tag — pass through.
            if trimmed.len() < THINK_START_LEN && !THINK_START.starts_with(trimmed) {
                self.output_chars += text.chars().count();
                return (String::new(), text.to_string());
            }
            // Otherwise it might be `<think>` (or its prefix). Fall through to
            // the state machine.
        }

        self.pending.push_str(text);
        let mut reasoning = String::new();
        let mut content = String::new();

        loop {
            if self.in_think {
                if let Some(pos) = self.pending.find(THINK_END) {
                    reasoning.push_str(&self.pending[..pos]);
                    self.pending.drain(..pos + THINK_END_LEN);
                    self.in_think = false;
                    self.done = true; // think block closed → done forever
                                      // Anything left in pending after </think> is visible content
                                      // that belongs to the *same* SSE delta.
                    content.push_str(&self.pending);
                    self.output_chars += reasoning.chars().count() + content.chars().count();
                    self.pending.clear();
                    break;
                } else {
                    // Streaming: emit everything except trailing bytes that
                    // could be part of the end tag.
                    if self.pending.len() > THINK_END_LEN {
                        let split = self.pending.len() - THINK_END_LEN;
                        let boundary = char_boundary(&self.pending, split);
                        reasoning.push_str(&self.pending[..boundary]);
                        self.pending.drain(..boundary);
                    }
                    break;
                }
            } else {
                if let Some(pos) = self.pending.find(THINK_START) {
                    content.push_str(&self.pending[..pos]);
                    self.pending.drain(..pos + THINK_START_LEN);
                    self.in_think = true;
                } else {
                    // Keep at most 6 trailing chars (<think> is 7).
                    // If no '<' in the tail, safe to emit everything.
                    if self.pending.len() > 6 {
                        let split = self.pending.len() - 6;
                        let boundary = char_boundary(&self.pending, split);
                        if self.pending[boundary..].contains('<') {
                            content.push_str(&self.pending[..boundary]);
                            self.pending.drain(..boundary);
                        } else {
                            content.push_str(&self.pending);
                            self.pending.clear();
                        }
                    }
                    break;
                }
            }
        }

        self.output_chars += reasoning.chars().count() + content.chars().count();
        (reasoning, content)
    }

    fn flush(self) -> (String, String, usize) {
        let chars = self.pending.chars().count();
        if self.in_think {
            (self.pending, String::new(), chars)
        } else {
            (String::new(), self.pending, chars)
        }
    }

    /// Consume one raw SSE chunk, parse complete events, filter think tags,
    /// and return the transformed bytes. Incomplete events are kept in `sse_buffer`.
    fn process_sse_chunk(&mut self, chunk: &bytes::Bytes) -> bytes::Bytes {
        // Once filtering is finished, avoid all SSE parsing and forward raw.
        // Still count characters for token estimation.
        if self.done && self.sse_buffer.is_empty() {
            let text = String::from_utf8_lossy(chunk);
            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data != "[DONE]" {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(content) = json
                                .pointer("/choices/0/delta/content")
                                .and_then(|c| c.as_str())
                            {
                                self.output_chars += content.chars().count();
                            }
                            if let Some(reasoning) = json
                                .pointer("/choices/0/delta/reasoning_content")
                                .and_then(|c| c.as_str())
                            {
                                self.output_chars += reasoning.chars().count();
                            }
                        }
                    }
                }
            }
            return chunk.clone();
        }

        let text = String::from_utf8_lossy(chunk);
        self.sse_buffer.push_str(&text);
        let mut output = Vec::new();

        while let Some(pos) = self.sse_buffer.find("\n\n") {
            let event = self.sse_buffer[..pos].to_string();
            self.sse_buffer = self.sse_buffer[pos + 2..].to_string();

            let mut data_line: Option<&str> = None;
            let mut other_lines: Vec<&str> = Vec::new();
            for line in event.lines() {
                if line.starts_with("data: ") {
                    data_line = Some(&line[6..]);
                } else {
                    other_lines.push(line);
                }
            }

            if let Some(data) = data_line {
                if data == "[DONE]" {
                    // flush any remaining reasoning before DONE
                    let prev_output_chars = self.output_chars;
                    let (final_reasoning, final_content, remaining_chars) = {
                        let f = std::mem::replace(self, ThinkFilter::new());
                        f.flush()
                    };
                    self.output_chars = prev_output_chars + remaining_chars;
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
                        output.extend_from_slice(b"data: ");
                        output.extend_from_slice(
                            serde_json::to_string(&flush_json)
                                .unwrap_or_default()
                                .as_bytes(),
                        );
                        output.extend_from_slice(b"\n");
                        for line in &other_lines {
                            output.extend_from_slice(line.as_bytes());
                            output.push(b'\n');
                        }
                        output.extend_from_slice(b"\n");
                    }
                    self.write_completion_estimate();
                    output.extend_from_slice(b"data: [DONE]\n\n");
                    continue;
                }

                // Fast path: filter already finished → forward raw data line
                // Still parse JSON briefly to count chars for token estimation.
                if self.done && data != "[DONE]" {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json
                            .pointer("/choices/0/delta/content")
                            .and_then(|c| c.as_str())
                        {
                            self.output_chars += content.chars().count();
                        }
                        if let Some(reasoning) = json
                            .pointer("/choices/0/delta/reasoning_content")
                            .and_then(|c| c.as_str())
                        {
                            self.output_chars += reasoning.chars().count();
                        }
                    }
                    output.extend_from_slice(b"data: ");
                    output.extend_from_slice(data.as_bytes());
                    for line in &other_lines {
                        output.push(b'\n');
                        output.extend_from_slice(line.as_bytes());
                    }
                    output.extend_from_slice(b"\n\n");
                    continue;
                }

                if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(data) {
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
                    output.extend_from_slice(b"data: ");
                    output.extend_from_slice(
                        serde_json::to_string(&json).unwrap_or_default().as_bytes(),
                    );
                    for line in &other_lines {
                        output.push(b'\n');
                        output.extend_from_slice(line.as_bytes());
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

        // If filter is done and buffer is empty, remaining raw bytes can be forwarded.
        if self.done && self.sse_buffer.is_empty() {
            let extra = chunk.len().saturating_sub(output.len());
            if extra > 0 {
                // This shouldn't normally happen because we process complete events,
                // but as a safety net append any trailing raw bytes.
                output.extend_from_slice(&chunk[chunk.len() - extra..]);
            }
        }

        bytes::Bytes::from(output)
    }

    /// Flush any remaining SSE data in `sse_buffer` by appending `\n\n` and
    /// re-processing.  Should be called once when the upstream stream ends.
    fn flush_sse_buffer(&mut self) -> bytes::Bytes {
        if self.sse_buffer.is_empty() {
            return bytes::Bytes::new();
        }
        let mut buf = std::mem::take(&mut self.sse_buffer);
        buf.push_str("\n\n");
        self.process_sse_chunk(&bytes::Bytes::from(buf))
    }
}

/// Largest index ≤ `byte_idx` that is a char boundary.
#[inline]
fn char_boundary(s: &str, byte_idx: usize) -> usize {
    let mut i = byte_idx;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Strip `<think>...</think>` from non-stream JSON response and move it to `reasoning_content`.
pub fn strip_think_tags(body: &str) -> String {
    // Fast path: no think tags at all
    if !body.contains(THINK_START) {
        return body.to_string();
    }

    let mut val: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return body.to_string(),
    };

    if let Some(choices) = val.get_mut("choices").and_then(|c| c.as_array_mut()) {
        for choice in choices {
            if let Some(msg) = choice.get_mut("message") {
                let content_str: Option<String> = msg
                    .get("content")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());
                if let Some(content_str) = content_str {
                    if let Some(start) = content_str.find(THINK_START) {
                        if let Some(end) = content_str.find(THINK_END) {
                            let before = content_str[..start].trim_end();
                            let think = content_str[start + THINK_START_LEN..end].trim();
                            let after = content_str[end + THINK_END_LEN..].trim_start();
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

    let bytes = match to_bytes(req.into_body(), 50 * 1024 * 1024).await {
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
                // ── Streaming: always filter think tags ──
                let status = resp.status();
                let headers = resp.headers().clone();
                let body = resp.into_body();
                let stream = body.into_data_stream();
                let completion_estimate = Arc::new(AtomicI64::new(0));
                let completion_clone = completion_estimate.clone();
                let filter = ThinkFilter::with_completion_counter(completion_clone);

                let transformed = futures_util::stream::unfold(
                    (stream, filter),
                    |(mut stream, mut filter)| async move {
                        match stream.next().await {
                            Some(Ok(chunk)) => {
                                let out = filter.process_sse_chunk(&chunk);
                                Some((Ok(out), (stream, filter)))
                            }
                            Some(Err(e)) => Some((Err(e), (stream, filter))),
                            None => {
                                let out = filter.flush_sse_buffer();
                                if out.is_empty() {
                                    None
                                } else {
                                    Some((Ok(out), (stream, filter)))
                                }
                            }
                        }
                    },
                );

                let new_body = Body::from_stream(transformed);
                let mut response = Response::new(new_body);
                *response.status_mut() = status;
                *response.headers_mut() = headers;

                if log_requests {
                    // Stream is lazy — log after it finishes via a background task.
                    let state_clone = state.clone();
                    let completion_clone = completion_estimate.clone();
                    let original_model = original_model.clone();
                    let provider_id = provider_id.clone();
                    let upstream_model = upstream_model.clone();
                    let request_body_str = request_body_str.clone();
                    tokio::spawn(async move {
                        // Poll every second for up to 120 s until the stream writes
                        // its completion estimate (or we give up).
                        let mut completion = 0i64;
                        for _ in 0..120 {
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            completion = completion_clone.load(Ordering::Relaxed);
                            if completion > 0 {
                                break;
                            }
                        }
                        let cost = (estimated_prompt as f64 * input_price
                            + completion as f64 * output_price)
                            / 1_000_000.0;
                        let _ = log_request(
                            &state_clone,
                            &original_model,
                            &provider_id,
                            &upstream_model,
                            true,
                            latency_ms,
                            status_code,
                            estimated_prompt,
                            completion,
                            cost,
                            &request_body_str,
                            "[streaming]",
                        );
                    });
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
            let processed_body = strip_think_tags(&response_body_str);
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
