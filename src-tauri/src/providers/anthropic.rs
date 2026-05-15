use crate::protocol::translator::openai_to_anthropic;
use crate::providers::{DynProviderClient, ProviderClient, ProviderContext};
use crate::protocol::openai_types::ChatCompletionRequest;
use axum::body::Body;
use axum::response::Response;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, HeaderName};
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
            HeaderValue::from_str(api_key)
                .map_err(|e| format!("Invalid API key header: {}", e))?,
        );
        headers.insert(
            HeaderName::from_static("anthropic-version"),
            HeaderValue::from_static("2023-06-01"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(extra) = &ctx.provider.extra_headers {
            if let Ok(map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(extra) {
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
        let url = format!("{}/messages", ctx.provider.base_url.trim_end_matches('/'));

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
            let stream = resp.bytes_stream();
            let body_stream = Body::from_stream(stream.map(|result| {
                result.map_err(|e| axum::Error::new(e))
            }));
            let mut response = Response::new(body_stream);
            *response.status_mut() = status;
            response.headers_mut().insert(
                CONTENT_TYPE,
                HeaderValue::from_static("text/event-stream"),
            );
            Ok(response)
        } else {
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| format!("Failed to read response body: {}", e))?;
            let mut response = Response::new(Body::from(bytes));
            *response.status_mut() = status;
            Ok(response)
        }
    }
}

pub fn create_client() -> DynProviderClient {
    Arc::new(AnthropicClient::new())
}
