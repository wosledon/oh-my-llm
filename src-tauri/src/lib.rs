use rusqlite::Connection;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

pub mod commands;
pub mod crypto;
pub mod logging;
pub mod protocol;
pub mod providers;
pub mod proxy;
pub mod stats;
pub mod storage;

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub proxy_server: Arc<Mutex<proxy::server::ProxyServer>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let conn = storage::db::init_db(&app.app_handle())?;
            let db = Arc::new(Mutex::new(conn));
            app.manage(AppState {
                db: db.clone(),
                proxy_server: Arc::new(Mutex::new(proxy::server::ProxyServer::new())),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::provider_cmd::list_providers,
            commands::provider_cmd::add_provider,
            commands::provider_cmd::update_provider,
            commands::provider_cmd::delete_provider,
            commands::model_cmd::list_models,
            commands::model_cmd::add_model,
            commands::model_cmd::update_model,
            commands::model_cmd::delete_model,
            commands::proxy_cmd::get_proxy_config,
            commands::proxy_cmd::update_proxy_config,
            commands::proxy_cmd::start_proxy,
            commands::proxy_cmd::stop_proxy,
            commands::proxy_cmd::restart_proxy,
            commands::proxy_cmd::get_proxy_status,
            commands::log_cmd::query_logs,
            commands::log_cmd::get_log_detail,
            commands::stats_cmd::get_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
