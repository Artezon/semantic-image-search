mod common;

pub mod metaclip2;
#[cfg(feature = "video")]
pub mod video;

use crate::errors::AppError;
use ndarray::Array1;
use ort::session::{NoSelectedOutputs, RunOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

type EmbedResult = Result<Array1<f32>, AppError>;
type FileEmbedResult = (PathBuf, EmbedResult);
type FilesEmbedResult = Result<Vec<FileEmbedResult>, AppError>;

#[derive(Hash, Eq, PartialEq)]
pub enum EmbeddingKind {
    // Text,
    Vision,
}

impl EmbeddingKind {
    pub fn as_str(&self) -> &str {
        match self {
            // Self::Text => "text",
            Self::Vision => "vision",
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct ModelManifest {
    pub name: &'static str,
    pub dim: u32,
    pub files: &'static [&'static str],
    pub capabilities: &'static [EmbeddingKind],
}

pub struct ModelManager {
    // pub all_models: HashMap<&'static ModelManifest, Arc<RwLock<dyn Model>>>,
    pub visual_search_models: HashMap<&'static ModelManifest, Arc<RwLock<dyn VisualSearchModel>>>,
}

impl ModelManager {
    pub fn new(models_dir: PathBuf) -> Self {
        let metaclip2 = metaclip2::MetaCLIP2B16_384Model::new(&models_dir);

        let visual_search_models = HashMap::from([(
            metaclip2.manifest(),
            Arc::new(RwLock::new(metaclip2)) as Arc<RwLock<dyn VisualSearchModel>>,
        )]);

        // TODO: Add other model types (DocumentSearchModel and AudioSearchModel)

        Self {
            visual_search_models,
        }
    }
}

pub trait Model: Send + Sync {
    fn new(models_dir: &Path) -> Self
    where
        Self: Sized;
    fn device_string(&self) -> String;
    fn manifest(&self) -> &'static ModelManifest;
}

pub trait TextEncoder: Model {
    fn load_text_encoder(&mut self) -> Result<(), AppError>;
    // fn unload_text_encoder(&mut self);
    fn is_text_encoder_loaded(&self) -> bool;

    fn embed_text(&mut self, text: &str) -> EmbedResult;
}

pub trait VisualEncoder: Model {
    fn load_vision_encoder(&mut self) -> Result<(), AppError>;
    // fn unload_vision_encoder(&mut self);
    fn is_vision_encoder_loaded(&self) -> bool;

    fn embed_images(
        &mut self,
        paths: &[PathBuf],
        run_options: Option<&RunOptions<NoSelectedOutputs>>,
    ) -> FilesEmbedResult;

    #[cfg(feature = "video")]
    fn embed_video(
        &mut self,
        path: &Path,
        num_frames: u32,
        run_options: Option<&RunOptions<NoSelectedOutputs>>,
    ) -> FilesEmbedResult;
}

pub trait VisualSearchModel: TextEncoder + VisualEncoder {}
