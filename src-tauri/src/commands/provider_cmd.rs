use crate::storage::provider_repo;
use crate::storage::{Provider, ProviderInput};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn list_providers(state: State<AppState>) -> Result<Vec<Provider>, String> {
    let conn = state.db.blocking_lock();
    provider_repo::list_providers(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_provider(state: State<AppState>, input: ProviderInput) -> Result<Provider, String> {
    let mut conn = state.db.blocking_lock();
    provider_repo::add_provider(&mut conn, &input)
}

#[tauri::command]
pub fn update_provider(
    state: State<AppState>,
    id: String,
    input: ProviderInput,
) -> Result<Provider, String> {
    let mut conn = state.db.blocking_lock();
    provider_repo::update_provider(&mut conn, &id, &input)
}

#[tauri::command]
pub fn delete_provider(state: State<AppState>, id: String) -> Result<(), String> {
    let mut conn = state.db.blocking_lock();
    provider_repo::delete_provider(&mut conn, &id)
}
