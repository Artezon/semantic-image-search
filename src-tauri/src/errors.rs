use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum AppError {
    ModelNotReady,
    InvalidImagePath,
    NoIndex,
    VideoReadFailed {
        msg: String,
    },
    TokenizerLoadFailed {
        msg: String,
        model_name: &'static str,
    },
    TokenizationFailed {
        msg: String,
    },
    ModelLoadFailed {
        msg: String,
        model_name: &'static str,
    },
    ModelInferenceFailed {
        msg: String,
    },
    LibraryLoadFailed {
        msg: String,
        name: String,
    },
    Unknown {
        msg: String,
    },
}

impl AppError {
    pub fn unknown(e: impl fmt::Display) -> Self {
        AppError::Unknown { msg: e.to_string() }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Unknown { msg: s }
    }
}

impl From<ort::Error> for AppError {
    fn from(e: ort::Error) -> Self {
        AppError::ModelInferenceFailed { msg: e.to_string() }
    }
}

impl From<ndarray::ShapeError> for AppError {
    fn from(e: ndarray::ShapeError) -> Self {
        AppError::ModelInferenceFailed { msg: e.to_string() }
    }
}

impl From<tokenizers::Error> for AppError {
    fn from(e: tokenizers::Error) -> Self {
        AppError::TokenizationFailed { msg: e.to_string() }
    }
}
