use rusqlite::{params, Connection};

#[derive(Debug, Clone, serde::Serialize)]
pub struct DailyUsage {
    pub date: String,
    pub model: String,
    pub provider_id: String,
    pub request_count: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub cost: f64,
}

pub fn record_usage(
    conn: &mut Connection,
    date: &str,
    model: &str,
    provider_id: &str,
    prompt_tokens: i64,
    completion_tokens: i64,
    cost: f64,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO daily_usage (date, model, provider_id, request_count, prompt_tokens, completion_tokens, cost)
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6)
         ON CONFLICT(date, model, provider_id) DO UPDATE SET
            request_count = request_count + 1,
            prompt_tokens = prompt_tokens + excluded.prompt_tokens,
            completion_tokens = completion_tokens + excluded.completion_tokens,
            cost = cost + excluded.cost",
        params![date, model, provider_id, prompt_tokens, completion_tokens, cost],
    )
    .map_err(|e| format!("Failed to record usage: {}", e))?;
    Ok(())
}

pub fn get_daily_usage(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<DailyUsage>, String> {
    let mut stmt = conn.prepare(
        "SELECT date, model, provider_id, request_count, prompt_tokens, completion_tokens, cost
         FROM daily_usage
         WHERE date >= ?1 AND date <= ?2
         ORDER BY date DESC, model"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![start_date, end_date], |row| {
        Ok(DailyUsage {
            date: row.get(0)?,
            model: row.get(1)?,
            provider_id: row.get(2)?,
            request_count: row.get(3)?,
            prompt_tokens: row.get(4)?,
            completion_tokens: row.get(5)?,
            cost: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| e.to_string())?);
    }
    Ok(results)
}

pub fn get_usage_summary(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<UsageSummary, String> {
    let mut stmt = conn.prepare(
        "SELECT SUM(request_count), SUM(prompt_tokens), SUM(completion_tokens), SUM(cost)
         FROM daily_usage
         WHERE date >= ?1 AND date <= ?2"
    ).map_err(|e| e.to_string())?;

    let row = stmt.query_row(params![start_date, end_date], |row| {
        Ok(UsageSummary {
            total_requests: row.get(0).unwrap_or(0),
            total_prompt_tokens: row.get(1).unwrap_or(0),
            total_completion_tokens: row.get(2).unwrap_or(0),
            total_cost: row.get(3).unwrap_or(0.0),
        })
    }).map_err(|e| e.to_string())?;

    Ok(row)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UsageSummary {
    pub total_requests: i64,
    pub total_prompt_tokens: i64,
    pub total_completion_tokens: i64,
    pub total_cost: f64,
}
