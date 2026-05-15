use crate::storage::config_repo;
use crate::storage::ProxyConfig;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_proxy_config(state: State<AppState>) -> Result<ProxyConfig, String> {
    let conn = state.db.blocking_lock();
    config_repo::get_proxy_config(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_proxy_config(
    state: State<AppState>,
    config: ProxyConfig,
) -> Result<ProxyConfig, String> {
    let mut conn = state.db.blocking_lock();
    config_repo::update_proxy_config(&mut conn, &config)
}

#[tauri::command]
pub async fn start_proxy(state: State<'_, AppState>) -> Result<(), String> {
    let config = {
        let conn = state.db.lock().await;
        config_repo::get_proxy_config(&conn).map_err(|e| e.to_string())?
    };

    let mut proxy = state.proxy_server.lock().await;
    if proxy.is_running() {
        return Err("Proxy already running".to_string());
    }

    proxy.start(state.db.clone(), config.port as u16).await
}

#[tauri::command]
pub async fn stop_proxy(state: State<'_, AppState>) -> Result<(), String> {
    let mut proxy = state.proxy_server.lock().await;
    proxy.stop()
}

#[tauri::command]
pub async fn restart_proxy(state: State<'_, AppState>) -> Result<(), String> {
    let config = {
        let conn = state.db.lock().await;
        config_repo::get_proxy_config(&conn).map_err(|e| e.to_string())?
    };

    let mut proxy = state.proxy_server.lock().await;
    if proxy.is_running() {
        proxy.stop()?;
    }
    proxy.start(state.db.clone(), config.port as u16).await
}

#[tauri::command]
pub async fn get_proxy_status(state: State<'_, AppState>) -> Result<bool, String> {
    let proxy = state.proxy_server.lock().await;
    Ok(proxy.is_running())
}
