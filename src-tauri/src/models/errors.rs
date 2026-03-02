use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum ModelError {
    NotLoaded,
    LoadFailed(String),
    InferenceFailed(String),
}

impl Display for ModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ModelError::NotLoaded => write!(f, "Model not loaded"),
            ModelError::LoadFailed(e) => write!(f, "Model load failed: {e}"),
            ModelError::InferenceFailed(e) => write!(f, "Inference failed: {e}"),
        }
    }
}

impl From<ModelError> for String {
    fn from(e: ModelError) -> Self {
        e.to_string()
    }
}

impl From<ort::Error> for ModelError {
    fn from(e: ort::Error) -> Self {
        ModelError::InferenceFailed(format!("Failed inference: {}", e))
    }
}

impl From<ndarray::ShapeError> for ModelError {
    fn from(e: ndarray::ShapeError) -> Self {
        ModelError::InferenceFailed(format!("Failed inference: {}", e))
    }
}
