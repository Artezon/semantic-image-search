use crate::{
    dylib::preload_libs,
    errors::AppError,
    frontend::{MessagePayload, ModelStatus, send_model_status},
    state::{self, AppState, Config},
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State, command};

#[command]
pub async fn get_config(app_handle: AppHandle) -> Config {
    let state = app_handle.state::<AppState>();
    state.config.read().unwrap().clone()
}

#[command(async)]
pub fn get_default_config() -> Config {
    Config::default()
}

#[command]
pub async fn update_config(app_handle: AppHandle, updates: HashMap<String, Value>) {
    let state = app_handle.state::<AppState>();
    state.update_config(updates);
    state.save_config();
}

#[command(async)]
pub fn apply_locale(app_handle: AppHandle) {
    let state = app_handle.state::<AppState>();
    let config = state.config.read().unwrap();
    let resolved_lang = state::resolve_lang(&config.lang);
    let _ = app_handle.emit("update-locale", &resolved_lang);
}

#[command(async)]
pub fn get_indexed_count(app_handle: AppHandle) -> i64 {
    let state = app_handle.state::<AppState>();
    state.db.count_indexed_files().unwrap()
}

#[command]
pub async fn get_dirs(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
    state.db.get_dirs().map_err(AppError::generic)
}

#[command]
pub async fn add_directory(state: State<'_, AppState>, path: String) -> Result<(), AppError> {
    let dir_path = PathBuf::from(&path);
    if !dir_path.is_dir() {
        return Err(AppError::InvalidDirectory);
    }
    state.db.add_directory(&path).map_err(AppError::generic)
}

#[command]
pub async fn remove_directory(state: State<'_, AppState>, path: String) -> Result<(), AppError> {
    state.db.remove_directory(&path).map_err(AppError::generic)
}

#[command]
pub async fn reorder_directories(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<(), AppError> {
    state
        .db
        .reorder_directories(&paths)
        .map_err(AppError::generic)
}

#[command(async)]
pub fn get_model_status(app_handle: AppHandle) {
    let state = app_handle.state::<AppState>();
    let selected_model_manifest = state.selected_model;

    let model = Arc::clone(&state.model_manager.visual_search_models[selected_model_manifest]);

    send_model_status(&app_handle, ModelStatus::Neutral, "loading_libraries");

    if let Err(e) = preload_libs(&state.data_path.join("lib")) {
        MessagePayload::new("model-status")
            .param("status", serde_json::json!(ModelStatus::Error))
            .param("status_key", serde_json::json!("error"))
            .param("error_details", serde_json::json!(e))
            .emit(&app_handle);
        return;
    }

    send_model_status(&app_handle, ModelStatus::Neutral, "loading_model");

    let load_result = (|| {
        let mut model_context = model.write().unwrap();
        model_context.load_text_encoder()?;
        model_context.load_vision_encoder()
    })();

    match load_result {
        Ok(()) => {
            MessagePayload::new("model-status")
                .param("status", serde_json::json!(ModelStatus::Success))
                .param("status_key", serde_json::json!("loaded"))
                .param(
                    "device_text",
                    serde_json::json!(model.read().unwrap().device_string()),
                )
                .emit(&app_handle);
        }
        Err(e) => {
            MessagePayload::new("model-status")
                .param("status", serde_json::json!(ModelStatus::Error))
                .param("status_key", serde_json::json!("error"))
                .param("error_details", serde_json::json!(e))
                .emit(&app_handle);
        }
    }
}
