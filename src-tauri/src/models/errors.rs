#[derive(Debug)]
pub enum ModelError {
    NotLoaded,
    LoadFailed(String),
    InferenceFailed(String),
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
