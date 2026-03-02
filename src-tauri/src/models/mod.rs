mod common;
pub mod errors;
pub mod metaclip2;

use crate::dylib::preload_libs;
use crate::models::errors::ModelError;
use ndarray::Array1;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Hash, Eq, PartialEq)]
pub enum EmbeddingKind {
    // Document,
    Visual,
    // Audio,
}

impl EmbeddingKind {
    pub fn as_str(&self) -> &str {
        match self {
            // Self::Document => "document",
            Self::Visual => "visual",
            // Self::Audio => "audio",
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

        // let mut all_models: HashMap<&'static ModelManifest, Arc<RwLock<dyn Model>>> =
        //     HashMap::new();
        // for (&manifest, arc) in &visual_search_models {
        //     all_models
        //         .entry(manifest)
        //         .or_insert_with(|| arc.clone() as Arc<RwLock<dyn Model>>);
        // }

        Self {
            // all_models,
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
    fn load_text_encoder(&mut self) -> Result<(), ModelError>;
    fn unload_text_encoder(&mut self);
    fn is_text_encoder_loaded(&self) -> bool;

    fn embed_text(&mut self, text: &str) -> Result<Array1<f32>, ModelError>;
}

pub trait VisualEncoder: Model {
    fn load_vision_encoder(&mut self) -> Result<(), ModelError>;
    fn unload_vision_encoder(&mut self);
    fn is_vision_encoder_loaded(&self) -> bool;

    fn embed_images(
        &mut self,
        paths: &[PathBuf],
    ) -> Result<Vec<(PathBuf, Result<Array1<f32>, String>)>, ModelError>;
    // fn embed_video(&mut self, path: &Path) -> Result<Vec<f32>, ModelError>; // TODO
}

pub trait VisualSearchModel: TextEncoder + VisualEncoder {}
