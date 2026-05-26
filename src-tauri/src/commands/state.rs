use crate::{
    dylib::preload_libs,
    frontend::{ModelStatus, ModelStatusPayload},
    state::{self, AppState, Config},
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tauri::{AppHandle, Manager, State, command};

#[command]
pub fn get_config(state: State<'_, AppState>) -> Config {
    state.config.read().unwrap().clone()
}

#[command]
pub fn get_default_config() -> Config {
    Config::default()
}

#[command]
pub fn update_config(state: State<'_, AppState>, updates: HashMap<String, Value>) {
    state.update_config(updates);
    state.save_config();
}

#[command]
pub fn apply_locale(app_handle: AppHandle) {
    let state = app_handle.state::<AppState>();
    let config = state.config.read().unwrap();
    let resolved_lang = state::resolve_lang(&config.lang);
    let _ = app_handle.emit("update-locale", &resolved_lang);
}

#[command]
pub async fn get_indexed_count(app_handle: AppHandle) -> i64 {
    let state = app_handle.state::<AppState>();
    state.db.count_indexed_files().unwrap()
}

#[command]
pub async fn get_model_status(app_handle: AppHandle) {
    let state = app_handle.state::<AppState>();
    let selected_model_manifest = state.selected_model;

    let model = Arc::clone(&state.model_manager.visual_search_models[selected_model_manifest]);

    ModelStatusPayload::new(ModelStatus::Neutral, "loading_libraries").emit(&app_handle);

    if let Err(e) = preload_libs(&state.data_path.join("lib")) {
        ModelStatusPayload::new(ModelStatus::Error, "error")
            .param("error", serde_json::json!(e))
            .emit(&app_handle);
        return;
    }

    ModelStatusPayload::new(ModelStatus::Neutral, "loading_model").emit(&app_handle);

    let load_result = (|| {
        let mut model_context = model.write().unwrap();
        model_context.load_text_encoder()?;
        model_context.load_vision_encoder()
    })();

    let result = match load_result {
        Ok(()) => ModelStatusPayload::new(ModelStatus::Success, "loaded")
            .device(&model.read().unwrap().device_string()),
        Err(e) => ModelStatusPayload::new(ModelStatus::Error, "error")
            .param("error", serde_json::json!(e)),
    };

    result.emit(&app_handle);
}
