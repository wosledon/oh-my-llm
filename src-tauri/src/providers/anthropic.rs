use crate::protocol::openai_types::ChatCompletionRequest;
use crate::protocol::translator::{anthropic_to_openai, openai_to_anthropic};
use crate::providers::{DynProviderClient, ProviderClient, ProviderContext};
use axum::body::Body;
use axum::response::Response;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use std::sync::Arc;

pub struct AnthropicClient {
    pub http: reqwest::Client,
}

impl AnthropicClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }
}

#[async_trait::async_trait]
impl ProviderClient for AnthropicClient {
    async fn chat_completion(
        &self,
        ctx: ProviderContext,
        request: ChatCompletionRequest,
    ) -> Result<Response<Body>, String> {
        let api_key = &ctx.provider.api_key;

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-api-key"),
            HeaderValue::from_str(api_key).map_err(|e| format!("Invalid API key header: {}", e))?,
        );
        headers.insert(
            HeaderName::from_static("anthropic-version"),
            HeaderValue::from_static("2023-06-01"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(extra) = &ctx.provider.extra_headers {
            if let Ok(map) =
                serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(extra)
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

        let anthropic_req = openai_to_anthropic(request);
        let is_stream = anthropic_req.stream.unwrap_or(false);
        let base = ctx.provider.base_url.trim_end_matches('/');
        let url = if base.ends_with("/v1") {
            format!("{}/messages", base)
        } else {
            format!("{}/v1/messages", base)
        };

        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .json(&anthropic_req)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = resp.status();

        if is_stream {
            let id = uuid::Uuid::new_v4().to_string();
            let model = anthropic_req.model.clone();
            let created = chrono::Utc::now().timestamp();
            let (tx, rx) = tokio::sync::mpsc::channel::<Result<bytes::Bytes, axum::Error>>(32);

            tokio::spawn(async move {
                let mut stream = resp.bytes_stream();
                let mut buffer = String::new();
                let mut sent_role = false;
                let mut current_block_is_tool = false;
                let mut current_tool_index: usize = 0;
                let mut tool_use_acc: Vec<(String, String, String)> = Vec::new(); // (id, name, args)

                loop {
                    let got_chunk = match stream.next().await {
                        Some(Ok(bytes)) => {
                            buffer.push_str(&String::from_utf8_lossy(&bytes));
                            true
                        }
                        Some(Err(e)) => {
                            let _ = tx.send(Err(axum::Error::new(e))).await;
                            break;
                        }
                        None => {
                            if !buffer.is_empty() {
                                buffer.push_str("\n\n");
                            }
                            false
                        }
                    };

                    loop {
                        if let Some(event_end) = buffer.find("\n\n") {
                            let event_text = buffer[..event_end].to_string();
                            buffer = buffer[event_end + 2..].to_string();

                            let mut event_type = String::new();
                            let mut data = String::new();
                            for line in event_text.lines() {
                                if line.starts_with("event: ") {
                                    event_type = line[7..].to_string();
                                } else if line.starts_with("data: ") {
                                    data = line[6..].to_string();
                                }
                            }

                            match event_type.as_str() {
                                "message_start" => {
                                    if !sent_role {
                                        let chunk = serde_json::json!({
                                            "id": id,
                                            "object": "chat.completion.chunk",
                                            "created": created,
                                            "model": model,
                                            "choices": [{"index":0,"delta":{"role":"assistant"},"finish_reason":null}]
                                        });
                                        let _ = tx
                                            .send(Ok(bytes::Bytes::from(format!(
                                                "data: {}\n\n",
                                                chunk
                                            ))))
                                            .await;
                                        sent_role = true;
                                    }
                                }
                                "content_block_start" => {
                                    if let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(&data)
                                    {
                                        if let Some(block_type) = json
                                            .get("content_block")
                                            .and_then(|b| b.get("type"))
                                            .and_then(|t| t.as_str())
                                        {
                                            current_block_is_tool = block_type == "tool_use";
                                            if current_block_is_tool {
                                                let tc_id = json
                                                    .get("content_block")
                                                    .and_then(|b| b.get("id"))
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                                    .to_string();
                                                let tc_name = json
                                                    .get("content_block")
                                                    .and_then(|b| b.get("name"))
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                                    .to_string();
                                                tool_use_acc.push((tc_id, tc_name, String::new()));
                                                current_tool_index =
                                                    tool_use_acc.len().saturating_sub(1);
                                            }
                                        }
                                    }
                                }
                                "content_block_delta" => {
                                    if let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(&data)
                                    {
                                        // text_delta
                                        if let Some(text) = json
                                            .get("delta")
                                            .and_then(|d| d.get("text"))
                                            .and_then(|t| t.as_str())
                                        {
                                            let chunk = serde_json::json!({
                                                "id": id,
                                                "object": "chat.completion.chunk",
                                                "created": created,
                                                "model": model,
                                                "choices": [{"index":0,"delta":{"content":text},"finish_reason":null}]
                                            });
                                            let _ = tx
                                                .send(Ok(bytes::Bytes::from(format!(
                                                    "data: {}\n\n",
                                                    chunk
                                                ))))
                                                .await;
                                            sent_role = true;
                                        }
                                        // input_json_delta (tool_use)
                                        if current_block_is_tool {
                                            if let Some(partial) = json
                                                .get("delta")
                                                .and_then(|d| d.get("partial_json"))
                                                .and_then(|v| v.as_str())
                                            {
                                                if let Some(entry) =
                                                    tool_use_acc.get_mut(current_tool_index)
                                                {
                                                    entry.2.push_str(partial);
                                                }
                                            }
                                        }
                                    }
                                }
                                "content_block_stop" => {
                                    if current_block_is_tool {
                                        if let Some((tc_id, tc_name, tc_args)) = tool_use_acc.pop()
                                        {
                                            let chunk = serde_json::json!({
                                                "id": id,
                                                "object": "chat.completion.chunk",
                                                "created": created,
                                                "model": model,
                                                "choices": [{"index":0,"delta":{"tool_calls":[{"index":current_tool_index,"id":tc_id,"type":"function","function":{"name":tc_name,"arguments":tc_args}}]},"finish_reason":null}]
                                            });
                                            let _ = tx
                                                .send(Ok(bytes::Bytes::from(format!(
                                                    "data: {}\n\n",
                                                    chunk
                                                ))))
                                                .await;
                                        }
                                        current_block_is_tool = false;
                                    }
                                }
                                "message_delta" => {
                                    if let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(&data)
                                    {
                                        let finish_reason = json
                                            .get("delta")
                                            .and_then(|d| d.get("stop_reason"))
                                            .and_then(|s| s.as_str())
                                            .unwrap_or("stop");
                                        let chunk = serde_json::json!({
                                            "id": id,
                                            "object": "chat.completion.chunk",
                                            "created": created,
                                            "model": model,
                                            "choices": [{"index":0,"delta":{},"finish_reason":finish_reason}]
                                        });
                                        let _ = tx
                                            .send(Ok(bytes::Bytes::from(format!(
                                                "data: {}\n\n",
                                                chunk
                                            ))))
                                            .await;
                                    }
                                }
                                "message_stop" => {
                                    let _ =
                                        tx.send(Ok(bytes::Bytes::from("data: [DONE]\n\n"))).await;
                                }
                                _ => {}
                            }
                        } else {
                            break;
                        }
                    }

                    if !got_chunk {
                        break;
                    }
                }
            });

            let body_stream = Body::from_stream(tokio_stream::wrappers::ReceiverStream::new(rx));
            let mut response = Response::new(body_stream);
            *response.status_mut() = status;
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"));
            Ok(response)
        } else {
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| format!("Failed to read response body: {}", e))?;

            let anthropic_resp: crate::protocol::anthropic_types::AnthropicResponse =
                serde_json::from_slice(&bytes)
                    .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

            let openai_resp = anthropic_to_openai(anthropic_resp);
            let body = serde_json::to_vec(&openai_resp)
                .map_err(|e| format!("Failed to serialize OpenAI response: {}", e))?;

            let mut response = Response::new(Body::from(body));
            *response.status_mut() = status;
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            Ok(response)
        }
    }
}

pub fn create_client() -> DynProviderClient {
    Arc::new(AnthropicClient::new())
}
