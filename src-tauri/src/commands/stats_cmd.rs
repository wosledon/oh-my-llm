use crate::stats::aggregator::{get_daily_usage, get_usage_summary, DailyUsage, UsageSummary};
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct UsageQueryParams {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize)]
pub struct UsageQueryResult {
    pub daily: Vec<DailyUsage>,
    pub summary: UsageSummary,
}

#[tauri::command]
pub async fn get_usage(
    state: State<'_, AppState>,
    params: UsageQueryParams,
) -> Result<UsageQueryResult, String> {
    let db = state.db.lock().await;
    let daily = get_daily_usage(&db, &params.start_date, &params.end_date).map_err(|e| e)?;
    let summary = get_usage_summary(&db, &params.start_date, &params.end_date).map_err(|e| e)?;
    Ok(UsageQueryResult { daily, summary })
}
