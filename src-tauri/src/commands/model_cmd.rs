use crate::storage::model_repo;
use crate::storage::{ModelInput, ModelMapping};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn list_models(
    state: State<AppState>,
    provider_id: Option<String>,
) -> Result<Vec<ModelMapping>, String> {
    let conn = state.db.blocking_lock();
    let pid = provider_id.as_deref();
    model_repo::list_models(&conn, pid).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_model(state: State<AppState>, input: ModelInput) -> Result<ModelMapping, String> {
    let mut conn = state.db.blocking_lock();
    model_repo::add_model(&mut conn, &input)
}

#[tauri::command]
pub fn update_model(
    state: State<AppState>,
    id: String,
    input: ModelInput,
) -> Result<ModelMapping, String> {
    let mut conn = state.db.blocking_lock();
    model_repo::update_model(&mut conn, &id, &input)
}

#[tauri::command]
pub fn delete_model(state: State<AppState>, id: String) -> Result<(), String> {
    let mut conn = state.db.blocking_lock();
    model_repo::delete_model(&mut conn, &id)
}
