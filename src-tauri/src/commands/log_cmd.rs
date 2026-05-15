use crate::logging::recorder::{get_log_detail as recorder_get_log_detail, query_logs as recorder_query_logs};
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct LogListItem {
    pub id: String,
    pub timestamp: i64,
    pub protocol: String,
    pub model: String,
    pub upstream_model: Option<String>,
    pub provider_id: Option<String>,
    pub stream: bool,
    pub latency_ms: i64,
    pub status_code: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub cost: f64,
    pub error_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LogDetail {
    pub entry: LogListItem,
    pub request_body: String,
    pub response_body: String,
}

#[derive(Debug, Deserialize)]
pub struct LogQueryParams {
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub model: Option<String>,
    pub provider_id: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

#[tauri::command]
pub async fn query_logs(
    state: State<'_, AppState>,
    params: LogQueryParams,
) -> Result<Vec<LogListItem>, String> {
    let db = state.db.lock().await;
    let logs = recorder_query_logs(
        &db,
        params.start_time,
        params.end_time,
        params.model.as_deref(),
        params.provider_id.as_deref(),
        params.status.as_deref(),
        params.search.as_deref(),
        params.limit,
        params.offset,
    )
    .map_err(|e| e)?;

    Ok(logs
        .into_iter()
        .map(|l| LogListItem {
            id: l.id,
            timestamp: l.timestamp,
            protocol: l.protocol,
            model: l.model,
            upstream_model: l.upstream_model,
            provider_id: l.provider_id,
            stream: l.stream,
            latency_ms: l.latency_ms,
            status_code: l.status_code,
            prompt_tokens: l.prompt_tokens,
            completion_tokens: l.completion_tokens,
            cost: l.cost,
            error_type: l.error_type,
        })
        .collect())
}

#[tauri::command]
pub async fn get_log_detail(
    state: State<'_, AppState>,
    log_id: String,
) -> Result<Option<LogDetail>, String> {
    let db = state.db.lock().await;
    let result = recorder_get_log_detail(&db, &log_id).map_err(|e| e)?;

    Ok(result.map(|(entry, req_body, resp_body)| LogDetail {
        entry: LogListItem {
            id: entry.id,
            timestamp: entry.timestamp,
            protocol: entry.protocol,
            model: entry.model,
            upstream_model: entry.upstream_model,
            provider_id: entry.provider_id,
            stream: entry.stream,
            latency_ms: entry.latency_ms,
            status_code: entry.status_code,
            prompt_tokens: entry.prompt_tokens,
            completion_tokens: entry.completion_tokens,
            cost: entry.cost,
            error_type: entry.error_type,
        },
        request_body: req_body,
        response_body: resp_body,
    }))
}
