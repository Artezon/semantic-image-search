use crate::{
    dylib::preload_libs,
    frontend::{
        ModelStatus, ModelStatusPayload, update_model_status, update_model_status_from_payload,
    },
    state::AppState,
};
use std::sync::Arc;
use tauri::{AppHandle, Manager, command};

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

    update_model_status(
        &app_handle,
        ModelStatus::Neutral,
        "Loading libraries...",
        "",
    );

    if let Err(e) = preload_libs(&state.data_path.join("lib")) {
        update_model_status(&app_handle, ModelStatus::Error, &e, "");
        return;
    }

    update_model_status(&app_handle, ModelStatus::Neutral, "Loading model...", "");

    let load_result = (|| {
        let mut model_context = model.write().unwrap();
        model_context.load_text_encoder()?;
        model_context.load_vision_encoder()
    })();

    let result = match load_result {
        Ok(()) => ModelStatusPayload {
            status: ModelStatus::Success,
            status_text: "Model loaded successfully!".to_string(),
            device_text: model.read().unwrap().device_string(),
        },
        Err(e) => ModelStatusPayload {
            status: ModelStatus::Error,
            status_text: format!("Error loading model: {:?}", e),
            device_text: "".to_string(),
        },
    };

    update_model_status_from_payload(&app_handle, &result);
}
