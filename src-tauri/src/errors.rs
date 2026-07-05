use serde::Serialize;
use std::fmt;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum AppError {
    ModelNotReady,
    InvalidImagePath,
    InvalidDirectory,
    NoIndex,
    AlreadyIndexing,
    DirectoryNotFound {
        detail: String,
    },
    ImageReadFailed {
        detail: String,
    },
    VideoReadFailed {
        detail: String,
    },
    TokenizerLoadFailed {
        detail: String,
        model_name: &'static str,
    },
    TokenizationFailed {
        detail: String,
    },
    ModelLoadFailed {
        detail: String,
        model_name: &'static str,
    },
    ModelInferenceFailed {
        detail: String,
    },
    LibraryLoadFailed {
        detail: String,
        name: String,
    },
    Generic {
        detail: String,
    },
    Unknown,
}

impl AppError {
    pub fn generic(e: impl fmt::Display) -> Self {
        AppError::Generic {
            detail: e.to_string(),
        }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Generic { detail: s }
    }
}

impl From<ort::Error> for AppError {
    fn from(e: ort::Error) -> Self {
        AppError::ModelInferenceFailed {
            detail: e.to_string(),
        }
    }
}

impl From<ndarray::ShapeError> for AppError {
    fn from(e: ndarray::ShapeError) -> Self {
        AppError::ModelInferenceFailed {
            detail: e.to_string(),
        }
    }
}

impl From<tokenizers::Error> for AppError {
    fn from(e: tokenizers::Error) -> Self {
        AppError::TokenizationFailed {
            detail: e.to_string(),
        }
    }
}

pub fn fatal_error(handle: &tauri::AppHandle, msg: &str, title: &str) -> ! {
    handle
        .dialog()
        .message(msg)
        .kind(MessageDialogKind::Error)
        .title(title)
        .blocking_show();
    std::process::exit(1);
}
