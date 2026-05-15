use rusqlite::{params, Connection, ToSql};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: i64,
    pub protocol: String,
    pub model: String,
    pub provider_id: Option<String>,
    pub upstream_model: Option<String>,
    pub stream: bool,
    pub latency_ms: i64,
    pub status_code: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub cost: f64,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
}

pub fn record_request_log(
    conn: &mut Connection,
    entry: &LogEntry,
    request_body: &str,
    response_body: &str,
) -> Result<(), String> {
    let now = now_secs();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO request_logs (
            id, timestamp, protocol, model, provider_id, upstream_model,
            stream, latency_ms, status_code, prompt_tokens, completion_tokens,
            cost, error_type, error_message, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            entry.id,
            entry.timestamp,
            entry.protocol,
            entry.model,
            entry.provider_id,
            entry.upstream_model,
            entry.stream as i64,
            entry.latency_ms,
            entry.status_code,
            entry.prompt_tokens,
            entry.completion_tokens,
            entry.cost,
            entry.error_type,
            entry.error_message,
            now,
        ],
    )
    .map_err(|e| format!("Failed to insert request log: {}", e))?;

    let req_extracted = extract_text_from_json(request_body);
    tx.execute(
        "INSERT INTO request_contents (id, log_id, request_body, extracted_text, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            Uuid::new_v4().to_string(),
            entry.id,
            request_body,
            req_extracted,
            now,
        ],
    )
    .map_err(|e| format!("Failed to insert request content: {}", e))?;

    let resp_extracted = extract_text_from_json(response_body);
    let is_truncated = if response_body.len() > 100000 { 1 } else { 0 };
    tx.execute(
        "INSERT INTO response_contents (id, log_id, response_body, extracted_text, is_truncated, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            Uuid::new_v4().to_string(),
            entry.id,
            response_body,
            resp_extracted,
            is_truncated,
            now,
        ],
    )
    .map_err(|e| format!("Failed to insert response content: {}", e))?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn query_logs(
    conn: &Connection,
    start_time: Option<i64>,
    end_time: Option<i64>,
    model: Option<&str>,
    provider_id: Option<&str>,
    status: Option<&str>,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<LogEntry>, String> {
    let mut sql = String::from(
        "SELECT l.id, l.timestamp, l.protocol, l.model, l.provider_id, l.upstream_model,
                l.stream, l.latency_ms, l.status_code, l.prompt_tokens, l.completion_tokens,
                l.cost, l.error_type, l.error_message
         FROM request_logs l WHERE 1=1"
    );
    let mut params_list: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(s) = start_time {
        sql.push_str(" AND l.timestamp >= ?");
        params_list.push(Box::new(s));
    }
    if let Some(e) = end_time {
        sql.push_str(" AND l.timestamp <= ?");
        params_list.push(Box::new(e));
    }
    if let Some(m) = model {
        sql.push_str(" AND l.model = ?");
        params_list.push(Box::new(m.to_string()));
    }
    if let Some(p) = provider_id {
        sql.push_str(" AND l.provider_id = ?");
        params_list.push(Box::new(p.to_string()));
    }
    if let Some(st) = status {
        if st == "success" {
            sql.push_str(" AND l.status_code >= 200 AND l.status_code < 300");
        } else if st == "error" {
            sql.push_str(" AND (l.status_code >= 400 OR l.error_type IS NOT NULL)");
        }
    }
    if let Some(q) = search {
        sql.push_str(
            " AND l.id IN (SELECT log_id FROM logs_fts WHERE logs_fts MATCH ?)"
        );
        params_list.push(Box::new(q.to_string()));
    }

    sql.push_str(" ORDER BY l.timestamp DESC LIMIT ? OFFSET ?");
    params_list.push(Box::new(limit));
    params_list.push(Box::new(offset));

    let params_refs: Vec<&dyn ToSql> = params_list.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(LogEntry {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            protocol: row.get(2)?,
            model: row.get(3)?,
            provider_id: row.get(4)?,
            upstream_model: row.get(5)?,
            stream: row.get::<_, i64>(6)? != 0,
            latency_ms: row.get(7)?,
            status_code: row.get(8)?,
            prompt_tokens: row.get(9)?,
            completion_tokens: row.get(10)?,
            cost: row.get(11)?,
            error_type: row.get(12)?,
            error_message: row.get(13)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut logs = Vec::new();
    for row in rows {
        logs.push(row.map_err(|e| e.to_string())?);
    }
    Ok(logs)
}

pub fn get_log_detail(
    conn: &Connection,
    log_id: &str,
) -> Result<Option<(LogEntry, String, String)>, String> {
    let mut stmt = conn.prepare(
        "SELECT l.id, l.timestamp, l.protocol, l.model, l.provider_id, l.upstream_model,
                l.stream, l.latency_ms, l.status_code, l.prompt_tokens, l.completion_tokens,
                l.cost, l.error_type, l.error_message,
                rc.request_body, resp.response_body
         FROM request_logs l
         LEFT JOIN request_contents rc ON l.id = rc.log_id
         LEFT JOIN response_contents resp ON l.id = resp.log_id
         WHERE l.id = ?1"
    ).map_err(|e| e.to_string())?;

    let result = stmt.query_row([log_id], |row| {
        Ok((
            LogEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                protocol: row.get(2)?,
                model: row.get(3)?,
                provider_id: row.get(4)?,
                upstream_model: row.get(5)?,
                stream: row.get::<_, i64>(6)? != 0,
                latency_ms: row.get(7)?,
                status_code: row.get(8)?,
                prompt_tokens: row.get(9)?,
                completion_tokens: row.get(10)?,
                cost: row.get(11)?,
                error_type: row.get(12)?,
                error_message: row.get(13)?,
            },
            row.get::<_, String>(14).unwrap_or_default(),
            row.get::<_, String>(15).unwrap_or_default(),
        ))
    });

    match result {
        Ok(entry) => Ok(Some(entry)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn cleanup_old_logs(conn: &mut Connection, retention_days: i64) -> Result<i64, String> {
    let cutoff = now_secs() - retention_days * 86400;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let deleted = tx.execute(
        "DELETE FROM request_logs WHERE timestamp < ?1",
        [cutoff],
    ).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(deleted as i64)
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn extract_text_from_json(body: &str) -> Option<String> {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
        let mut texts = Vec::new();
        if let Some(messages) = val.get("messages").and_then(|m| m.as_array()) {
            for msg in messages {
                if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                    texts.push(content.to_string());
                }
            }
        }
        if let Some(choices) = val.get("choices").and_then(|c| c.as_array()) {
            for choice in choices {
                if let Some(content) = choice
                    .get("message")
                    .or_else(|| choice.get("delta"))
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                {
                    texts.push(content.to_string());
                }
            }
        }
        if texts.is_empty() {
            None
        } else {
            Some(texts.join("\n"))
        }
    } else {
        None
    }
}
