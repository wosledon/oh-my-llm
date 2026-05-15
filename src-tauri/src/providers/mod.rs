pub mod openai;
pub mod compatible;
pub mod anthropic;

use crate::storage::Provider;
use axum::body::Body;
use axum::response::Response;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProviderContext {
    pub provider: Provider,
    pub upstream_model: String,
}

#[async_trait::async_trait]
pub trait ProviderClient: Send + Sync {
    async fn chat_completion(
        &self,
        ctx: ProviderContext,
        request: crate::protocol::openai_types::ChatCompletionRequest,
    ) -> Result<Response<Body>, String>;
}

pub type DynProviderClient = Arc<dyn ProviderClient>;
