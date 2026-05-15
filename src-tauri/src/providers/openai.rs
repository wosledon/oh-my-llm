use crate::protocol::openai_types::ChatCompletionRequest;
use crate::providers::{DynProviderClient, ProviderClient, ProviderContext};
use axum::body::Body;
use axum::response::Response;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::sync::Arc;

pub struct OpenAiClient {
    pub http: reqwest::Client,
}

impl OpenAiClient {
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
impl ProviderClient for OpenAiClient {
    async fn chat_completion(
        &self,
        ctx: ProviderContext,
        mut request: ChatCompletionRequest,
    ) -> Result<Response<Body>, String> {
        let api_key = &ctx.provider.api_key;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| format!("Invalid auth header: {}", e))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(extra) = &ctx.provider.extra_headers {
            if let Ok(map) =
                serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(extra)
            {
                for (k, v) in map {
                    if let Some(val) = v.as_str() {
                        if let Ok(hv) = HeaderValue::from_str(val) {
                            headers.insert(
                                reqwest::header::HeaderName::from_bytes(k.as_bytes())
                                    .unwrap_or_else(|_| {
                                        reqwest::header::HeaderName::from_static("x-custom")
                                    }),
                                hv,
                            );
                        }
                    }
                }
            }
        }

        request.model = ctx.upstream_model;

        let is_stream = request.stream.unwrap_or(false);
        let url = format!(
            "{}/chat/completions",
            ctx.provider.base_url.trim_end_matches('/')
        );

        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

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
    Arc::new(OpenAiClient::new())
}
