use rusqlite::Connection;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const CURRENT_SCHEMA_VERSION: i32 = 2;

pub fn get_db_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(app_data_dir.join("oh-my-llm.db"))
}

pub fn init_db(app_handle: &AppHandle) -> Result<Connection, String> {
    let db_path = get_db_path(app_handle)?;
    let mut conn = Connection::open(&db_path).map_err(|e| format!("Failed to open db: {}", e))?;
    run_migrations(&mut conn)?;
    Ok(conn)
}

fn run_migrations(conn: &mut Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        )",
        [],
    )
    .map_err(|e| format!("Failed to create schema_version table: {}", e))?;

    let current_version: Option<i32> = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get(0)
        })
        .ok();

    let version = current_version.unwrap_or(0);

    if version < 1 {
        migration_v1(conn)?;
    }
    if version < 2 {
        migration_v2(conn)?;
    }

    conn.execute(
        "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
        [CURRENT_SCHEMA_VERSION],
    )
    .map_err(|e| format!("Failed to update schema version: {}", e))?;

    Ok(())
}

fn migration_v1(conn: &mut Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS providers (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            prov_type   TEXT NOT NULL,
            base_url    TEXT NOT NULL,
            api_key     BLOB NOT NULL,
            extra_headers TEXT,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS model_mappings (
            id              TEXT PRIMARY KEY,
            provider_id     TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
            exposed_name    TEXT NOT NULL,
            upstream_name   TEXT NOT NULL,
            enabled         INTEGER NOT NULL DEFAULT 1,
            input_price     REAL NOT NULL DEFAULT 0,
            output_price    REAL NOT NULL DEFAULT 0,
            UNIQUE(exposed_name, provider_id)
        );

        CREATE TABLE IF NOT EXISTS proxy_config (
            id                  INTEGER PRIMARY KEY DEFAULT 1,
            port                INTEGER NOT NULL DEFAULT 11888,
            openai_enabled      INTEGER NOT NULL DEFAULT 1,
            anthropic_enabled   INTEGER NOT NULL DEFAULT 1,
            default_model       TEXT,
            auto_start          INTEGER NOT NULL DEFAULT 0,
            log_requests        INTEGER NOT NULL DEFAULT 1,
            log_retention_days  INTEGER NOT NULL DEFAULT 30,
            budget_enabled      INTEGER NOT NULL DEFAULT 0,
            budget_monthly      REAL NOT NULL DEFAULT 0,
            budget_warning      REAL NOT NULL DEFAULT 0.8,
            max_retries         INTEGER NOT NULL DEFAULT 3,
            timeout_secs        INTEGER NOT NULL DEFAULT 120
        );

        CREATE TABLE IF NOT EXISTS request_logs (
            id              TEXT PRIMARY KEY,
            timestamp       INTEGER NOT NULL,
            protocol        TEXT NOT NULL,
            model           TEXT NOT NULL,
            provider_id     TEXT,
            upstream_model  TEXT,
            stream          INTEGER NOT NULL DEFAULT 0,
            latency_ms      INTEGER,
            status_code     INTEGER,
            prompt_tokens   INTEGER,
            completion_tokens INTEGER,
            cost            REAL,
            error_type      TEXT,
            error_message   TEXT,
            created_at      INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_req_logs_ts ON request_logs(timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_req_logs_model ON request_logs(model);

        CREATE TABLE IF NOT EXISTS request_contents (
            id              TEXT PRIMARY KEY,
            log_id          TEXT NOT NULL UNIQUE REFERENCES request_logs(id) ON DELETE CASCADE,
            request_body    TEXT NOT NULL,
            extracted_text  TEXT,
            created_at      INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS response_contents (
            id              TEXT PRIMARY KEY,
            log_id          TEXT NOT NULL UNIQUE REFERENCES request_logs(id) ON DELETE CASCADE,
            response_body   TEXT NOT NULL,
            extracted_text  TEXT,
            is_truncated    INTEGER NOT NULL DEFAULT 0,
            created_at      INTEGER NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS logs_fts USING fts5(
            log_id UNINDEXED,
            model,
            request_text,
            response_text,
            content='request_contents',
            content_rowid='rowid'
        );

        CREATE TABLE IF NOT EXISTS daily_usage (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            date            TEXT NOT NULL,
            model           TEXT NOT NULL,
            provider_id     TEXT NOT NULL,
            request_count   INTEGER NOT NULL DEFAULT 0,
            prompt_tokens   INTEGER NOT NULL DEFAULT 0,
            completion_tokens INTEGER NOT NULL DEFAULT 0,
            cost            REAL NOT NULL DEFAULT 0,
            UNIQUE(date, model, provider_id)
        );
        CREATE INDEX IF NOT EXISTS idx_daily_usage_date ON daily_usage(date DESC);
        CREATE INDEX IF NOT EXISTS idx_daily_usage_model ON daily_usage(model);

        CREATE TABLE IF NOT EXISTS budget_config (
            id                  INTEGER PRIMARY KEY DEFAULT 1,
            monthly_budget      REAL NOT NULL DEFAULT 0,
            warning_threshold   REAL NOT NULL DEFAULT 0.8,
            enabled             INTEGER NOT NULL DEFAULT 0,
            last_reset_date     TEXT
        );

        -- Insert default proxy config if not exists
        INSERT OR IGNORE INTO proxy_config (id) VALUES (1);

        -- Insert default budget config if not exists
        INSERT OR IGNORE INTO budget_config (id) VALUES (1);
        "#,
    )
    .map_err(|e| format!("Migration v1 failed: {}", e))?;

    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
            rusqlite::params![table, column],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to check column existence: {}", e))?;
    Ok(count > 0)
}

fn migration_v2(conn: &mut Connection) -> Result<(), String> {
    if !column_exists(conn, "proxy_config", "shadow_model_name")? {
        conn.execute(
            "ALTER TABLE proxy_config ADD COLUMN shadow_model_name TEXT NOT NULL DEFAULT 'oh-my-llm'",
            [],
        )
        .map_err(|e| format!("Migration v2 failed adding shadow_model_name: {}", e))?;
    }
    if !column_exists(conn, "proxy_config", "shadow_mapping_id")? {
        conn.execute(
            "ALTER TABLE proxy_config ADD COLUMN shadow_mapping_id TEXT",
            [],
        )
        .map_err(|e| format!("Migration v2 failed adding shadow_mapping_id: {}", e))?;
    }
    Ok(())
}
