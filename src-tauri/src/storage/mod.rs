pub mod db;
pub mod provider_repo;
pub mod model_repo;
pub mod config_repo;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub prov_type: String,
    pub base_url: String,
    pub api_key: String,
    pub extra_headers: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInput {
    pub name: String,
    pub prov_type: String,
    pub base_url: String,
    pub api_key: String,
    pub extra_headers: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMapping {
    pub id: String,
    pub provider_id: String,
    pub exposed_name: String,
    pub upstream_name: String,
    pub enabled: bool,
    pub input_price: f64,
    pub output_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInput {
    pub provider_id: String,
    pub exposed_name: String,
    pub upstream_name: String,
    pub enabled: bool,
    pub input_price: f64,
    pub output_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub id: i32,
    pub port: i32,
    pub openai_enabled: bool,
    pub anthropic_enabled: bool,
    pub default_model: Option<String>,
    pub auto_start: bool,
    pub log_requests: bool,
    pub log_retention_days: i32,
    pub budget_enabled: bool,
    pub budget_monthly: f64,
    pub budget_warning: f64,
    pub max_retries: i32,
    pub timeout_secs: i32,
    pub shadow_model_name: String,
    pub shadow_mapping_id: Option<String>,
}
