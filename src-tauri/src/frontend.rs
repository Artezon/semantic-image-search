use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MessageKind {
    #[default]
    Info,
    Error,
    Warning,
}

#[derive(Clone, Serialize, Default)]
pub struct MessagePayload {
    title: String,
    msg: String,
    kind: MessageKind,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    #[default]
    Neutral,
    Success,
    Error,
}

#[derive(Clone, Serialize, Default)]
pub struct ModelStatusPayload {
    pub status: ModelStatus,
    pub status_text: String,
    pub device_text: String,
}

#[derive(Clone, Serialize, Default)]
pub struct IndexStatusPayload {
    progress: Option<f32>,
    text: String,
}

pub fn show_message(app_handle: &AppHandle, title: &str, msg: &str, kind: MessageKind) {
    let _ = app_handle
        .emit(
            "message",
            &MessagePayload {
                title: title.to_string(),
                msg: msg.to_string(),
                kind,
            },
        )
        .unwrap();
}

pub fn update_model_status(
    app_handle: &AppHandle,
    status: ModelStatus,
    status_text: &str,
    device_text: &str,
) {
    let _ = app_handle
        .emit(
            "model-status",
            &ModelStatusPayload {
                status,
                status_text: status_text.to_string(),
                device_text: device_text.to_string(),
            },
        )
        .unwrap();
}

pub fn update_model_status_from_payload(app_handle: &AppHandle, payload: &ModelStatusPayload) {
    let _ = app_handle.emit("model-status", payload).unwrap();
}

pub fn update_index_status(app_handle: &AppHandle, progress: Option<f32>, text: &str) {
    let _ = app_handle
        .emit(
            "index-status",
            &IndexStatusPayload {
                progress,
                text: text.to_string(),
            },
        )
        .unwrap();
}

pub fn update_index_processing_status(
    app_handle: &AppHandle,
    processed: usize,
    total: usize,
    errors: usize,
) {
    let mut msg = format!("Processed {processed} / {total} files...");
    if errors > 0 {
        msg += &format!(" ({errors} errors)");
    }
    update_index_status(&app_handle, Some(processed as f32 / total as f32), &msg);
}

pub fn clear_index_status(app_handle: &AppHandle) {
    update_index_status(&app_handle, Some(1.0), "")
}

pub fn set_is_indexing(app_handle: &AppHandle, is_indexing: bool) {
    let _ = app_handle.emit("is-indexing", is_indexing).unwrap();
}

pub fn clear_results(app_handle: &AppHandle) {
    let _ = app_handle.emit("clear-results", ()).unwrap();
}
