use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize, Default)]
pub struct MessagePayload {
    pub id: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub params: HashMap<String, serde_json::Value>,
}

impl MessagePayload {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            params: HashMap::new(),
        }
    }

    pub fn param(mut self, k: &str, v: serde_json::Value) -> Self {
        self.params.insert(k.to_string(), v);
        self
    }

    pub fn emit(&self, app_handle: &AppHandle) {
        let _ = app_handle.emit(&self.id, self).unwrap();
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    #[default]
    Neutral,
    Success,
    Error,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IndexingState {
    #[default]
    Idle,
    Preparing,
    Indexing,
    FatalError,
}

pub fn send_model_status(app_handle: &AppHandle, status: ModelStatus, status_key: &str) {
    MessagePayload::new("model-status")
        .param("status", serde_json::json!(status))
        .param("status_key", serde_json::json!(status_key))
        .emit(app_handle);
}

pub fn send_index_status(
    app_handle: &AppHandle,
    state: IndexingState,
    processed: usize,
    total: usize,
    errors: usize,
) {
    MessagePayload::new("index-status")
        .param("state", serde_json::json!(state))
        .param("processed", serde_json::json!(processed))
        .param("total", serde_json::json!(total))
        .param("errors", serde_json::json!(errors))
        .emit(app_handle);
}

pub fn clear_index_status(app_handle: &AppHandle) {
    send_index_status(app_handle, IndexingState::Idle, 0, 0, 0);
}

pub fn send_indexing_error(app_handle: &AppHandle, path: &str, detail: &str) {
    MessagePayload::new("indexing-error")
        .param("path", serde_json::json!(path))
        .param("detail", serde_json::json!(detail))
        .emit(app_handle);
}
