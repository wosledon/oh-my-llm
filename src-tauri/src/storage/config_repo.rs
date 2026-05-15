use super::ProxyConfig;
use rusqlite::{params, Connection, Result as SqliteResult};

pub fn get_proxy_config(conn: &Connection) -> SqliteResult<ProxyConfig> {
    conn.query_row(
        "SELECT id, port, openai_enabled, anthropic_enabled, default_model, auto_start, log_requests, log_retention_days, budget_enabled, budget_monthly, budget_warning, max_retries, timeout_secs, shadow_model_name, shadow_mapping_id FROM proxy_config WHERE id = 1",
        [],
        |row| {
            Ok(ProxyConfig {
                id: row.get(0)?,
                port: row.get(1)?,
                openai_enabled: row.get::<_, i32>(2)? != 0,
                anthropic_enabled: row.get::<_, i32>(3)? != 0,
                default_model: row.get(4)?,
                auto_start: row.get::<_, i32>(5)? != 0,
                log_requests: row.get::<_, i32>(6)? != 0,
                log_retention_days: row.get(7)?,
                budget_enabled: row.get::<_, i32>(8)? != 0,
                budget_monthly: row.get(9)?,
                budget_warning: row.get(10)?,
                max_retries: row.get(11)?,
                timeout_secs: row.get(12)?,
                shadow_model_name: row.get(13).unwrap_or_else(|_| "oh-my-llm".to_string()),
                shadow_mapping_id: row.get(14).ok(),
            })
        },
    )
}

pub fn update_proxy_config(
    conn: &mut Connection,
    config: &ProxyConfig,
) -> Result<ProxyConfig, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE proxy_config SET port = ?1, openai_enabled = ?2, anthropic_enabled = ?3, default_model = ?4, auto_start = ?5, log_requests = ?6, log_retention_days = ?7, budget_enabled = ?8, budget_monthly = ?9, budget_warning = ?10, max_retries = ?11, timeout_secs = ?12, shadow_model_name = ?13, shadow_mapping_id = ?14 WHERE id = 1",
        params![
            config.port,
            if config.openai_enabled { 1 } else { 0 },
            if config.anthropic_enabled { 1 } else { 0 },
            config.default_model.as_ref(),
            if config.auto_start { 1 } else { 0 },
            if config.log_requests { 1 } else { 0 },
            config.log_retention_days,
            if config.budget_enabled { 1 } else { 0 },
            config.budget_monthly,
            config.budget_warning,
            config.max_retries,
            config.timeout_secs,
            &config.shadow_model_name,
            config.shadow_mapping_id.as_ref(),
        ],
    ).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(config.clone())
}
