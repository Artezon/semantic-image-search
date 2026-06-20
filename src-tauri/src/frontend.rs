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
        let _ = app_handle.emit("message", self).unwrap();
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

#[derive(Clone, Serialize, Default)]
pub struct ModelStatusPayload {
    pub status: ModelStatus,
    pub status_key: String,
    pub device_text: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub params: HashMap<String, serde_json::Value>,
}

impl ModelStatusPayload {
    pub fn new(status: ModelStatus, status_key: &str) -> Self {
        Self {
            status,
            status_key: status_key.to_string(),
            device_text: String::new(),
            params: HashMap::new(),
        }
    }

    pub fn device(mut self, text: &str) -> Self {
        self.device_text = text.to_string();
        self
    }

    pub fn param(mut self, k: &str, v: serde_json::Value) -> Self {
        self.params.insert(k.to_string(), v);
        self
    }

    pub fn emit(&self, app_handle: &AppHandle) {
        let _ = app_handle.emit("model-status", self).unwrap();
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IndexingState {
    #[default]
    Idle,
    Indexing,
    FatalError,
}

#[derive(Clone, Serialize, Default)]
pub struct IndexingStatusPayload {
    pub state: IndexingState,
    pub processed: usize,
    pub total: usize,
    pub errors: usize,
}

pub fn update_index_status(
    app_handle: &AppHandle,
    processed: usize,
    total: usize,
    errors: usize,
    state: IndexingState,
) {
    let _ = app_handle
        .emit(
            "index-status",
            &IndexingStatusPayload {
                state,
                processed,
                total,
                errors,
            },
        )
        .unwrap();
}

pub fn clear_index_status(app_handle: &AppHandle) {
    update_index_status(app_handle, 0, 0, 0, IndexingState::Idle);
}

// pub fn clear_results(app_handle: &AppHandle) {
//     let _ = app_handle.emit("clear-results", ()).unwrap();
// }
