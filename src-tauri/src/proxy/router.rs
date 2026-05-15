use crate::providers::{DynProviderClient, ProviderContext};
use crate::storage::Provider;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ProxyState {
    pub db: Arc<Mutex<Connection>>,
    pub openai_client: DynProviderClient,
    pub compatible_client: DynProviderClient,
    pub anthropic_client: DynProviderClient,
}

impl ProxyState {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self {
            db,
            openai_client: crate::providers::openai::create_client(),
            compatible_client: crate::providers::compatible::create_client(),
            anthropic_client: crate::providers::anthropic::create_client(),
        }
    }
}

pub async fn route_request(State(state): State<ProxyState>, req: Request<Body>) -> Response {
    let path = req.uri().path();

    match path {
        "/v1/chat/completions" => {
            crate::proxy::handlers::openai::handle_chat_completions(State(state), req).await
        }
        "/v1/models" => crate::proxy::handlers::models::handle_list_models(State(state)).await,
        "/health" => axum::response::Json(serde_json::json!({"status": "ok"})).into_response(),
        _ => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

pub fn resolve_model_route(db: &Connection, exposed_name: &str) -> Result<ProviderContext, String> {
    let mut stmt = db.prepare(
        "SELECT m.provider_id, m.upstream_name, p.name, p.prov_type, p.base_url, p.api_key, p.extra_headers, p.created_at, p.updated_at
         FROM model_mappings m
         JOIN providers p ON m.provider_id = p.id
         WHERE m.exposed_name = ?1 AND m.enabled = 1"
    ).map_err(|e| format!("DB prepare error: {}", e))?;

    let row = stmt.query_row([exposed_name], |row| {
        let api_key_encrypted: Vec<u8> = row.get(5)?;
        let api_key = crate::crypto::decrypt(&api_key_encrypted).unwrap_or_default();
        let provider = Provider {
            id: row.get(0)?,
            name: row.get(2)?,
            prov_type: row.get(3)?,
            base_url: row.get(4)?,
            api_key,
            extra_headers: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        };
        let upstream_name: String = row.get(1)?;
        Ok((provider, upstream_name))
    });

    match row {
        Ok((provider, upstream_model)) => {
            let ctx = ProviderContext {
                provider,
                upstream_model,
            };
            Ok(ctx)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Err(format!("Model '{}' not found or not enabled", exposed_name))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

pub fn resolve_shadow_route(db: &Connection, mapping_id: &str) -> Result<ProviderContext, String> {
    let mut stmt = db.prepare(
        "SELECT m.provider_id, m.upstream_name, p.name, p.prov_type, p.base_url, p.api_key, p.extra_headers, p.created_at, p.updated_at
         FROM model_mappings m
         JOIN providers p ON m.provider_id = p.id
         WHERE m.id = ?1 AND m.enabled = 1"
    ).map_err(|e| format!("DB prepare error: {}", e))?;

    let row = stmt.query_row([mapping_id], |row| {
        let api_key_encrypted: Vec<u8> = row.get(5)?;
        let api_key = crate::crypto::decrypt(&api_key_encrypted).unwrap_or_default();
        let provider = Provider {
            id: row.get(0)?,
            name: row.get(2)?,
            prov_type: row.get(3)?,
            base_url: row.get(4)?,
            api_key,
            extra_headers: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        };
        let upstream_name: String = row.get(1)?;
        Ok((provider, upstream_name))
    });

    match row {
        Ok((provider, upstream_model)) => {
            let ctx = ProviderContext {
                provider,
                upstream_model,
            };
            Ok(ctx)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Err(format!("Shadow model mapping '{}' not found or not enabled", mapping_id))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

pub fn select_client(state: &ProxyState, prov_type: &str) -> Result<DynProviderClient, String> {
    match prov_type {
        "openai" => Ok(state.openai_client.clone()),
        "openai_compatible" => Ok(state.compatible_client.clone()),
        "anthropic" => Ok(state.anthropic_client.clone()),
        _ => Err(format!("Unknown provider type: {}", prov_type)),
    }
}
