mod common;
pub mod metaclip2;

use crate::errors::AppError;
use crate::utils::{l2_normalize, open_image_as_rgb};
#[cfg(feature = "video")]
use crate::video::extract_video_frames;
#[cfg(feature = "video")]
use ndarray::Axis;
use ndarray::{Array1, Array2};
use ort::session::{NoSelectedOutputs, RunOptions};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

type EmbedResult = Result<Array1<f32>, AppError>;

#[derive(Clone)]
pub struct FileEmbedResult {
    pub path: PathBuf,
    pub embedding: EmbedResult,
}

#[derive(Clone)]
pub struct TokenizedText {
    pub input_ids: Array2<i64>,
    pub attention_mask: Array2<i64>,
}

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

    fn tokenize_text(&self, text: &str) -> Result<TokenizedText, AppError>;
    fn infer_text_embeddings(
        &mut self,
        tokens: TokenizedText,
        run_options: Option<&RunOptions<NoSelectedOutputs>>,
    ) -> EmbedResult;

    fn embed_text(
        &mut self,
        text: &str,
        run_options: Option<&RunOptions<NoSelectedOutputs>>,
    ) -> EmbedResult {
        let tokens = self.tokenize_text(text)?;
        self.infer_text_embeddings(tokens, run_options)
    }
}

pub trait VisualEncoder: Model {
    fn load_vision_encoder(&mut self) -> Result<(), AppError>;
    // fn unload_vision_encoder(&mut self);
    fn is_vision_encoder_loaded(&self) -> bool;

    fn preprocess_image(&self, img: &image::RgbImage) -> Vec<f32>;
    #[cfg(feature = "video")]
    fn preprocess_video_frame(&self, frame: &image::RgbImage) -> Vec<f32>;
    fn infer_embeddings(
        &mut self,
        pixel_batch: Vec<Vec<f32>>,
        run_options: Option<&RunOptions<NoSelectedOutputs>>,
    ) -> Result<Array2<f32>, AppError>;

    fn embed_images(
        &mut self,
        paths: &[PathBuf],
        run_options: Option<&RunOptions<NoSelectedOutputs>>,
    ) -> Result<Vec<FileEmbedResult>, AppError> {
        if !self.is_vision_encoder_loaded() {
            return Err(AppError::ModelNotReady);
        }

        let img_buffers: Vec<Result<Vec<f32>, AppError>> = paths
            .par_iter()
            .map(|path| {
                let img = open_image_as_rgb(path).map_err(|e| AppError::ImageReadFailed {
                    detail: e.to_string(),
                })?;
                Ok(self.preprocess_image(&img))
            })
            .collect();

        let mut batch = vec![];
        let mut errors: Vec<Option<AppError>> = vec![];
        for res in img_buffers {
            match res {
                Ok(vec) => {
                    errors.push(None);
                    batch.push(vec);
                }
                Err(e) => errors.push(Some(e)),
            }
        }

        if batch.is_empty() {
            return Ok(paths
                .iter()
                .zip(errors.iter())
                .map(|(path, err)| FileEmbedResult {
                    path: path.clone(),
                    embedding: Err(err.clone().unwrap_or(AppError::Unknown)),
                })
                .collect());
        }

        let embed_2d = self.infer_embeddings(batch, run_options)?;

        let mut embed_iter = embed_2d.outer_iter();
        let result: Vec<FileEmbedResult> = paths
            .iter()
            .zip(errors.iter())
            .map(|(path, err)| match err {
                None => {
                    let row = embed_iter.next().unwrap();
                    FileEmbedResult {
                        path: path.clone(),
                        embedding: Ok(l2_normalize(row.to_owned())),
                    }
                }
                Some(e) => FileEmbedResult {
                    path: path.clone(),
                    embedding: Err(e.clone()),
                },
            })
            .collect();

        Ok(result)
    }

    #[cfg(feature = "video")]
    fn embed_video(
        &mut self,
        path: &Path,
        num_frames: u32,
        run_options: Option<&RunOptions<NoSelectedOutputs>>,
    ) -> Result<Vec<FileEmbedResult>, AppError> {
        if !self.is_vision_encoder_loaded() {
            return Err(AppError::ModelNotReady);
        }

        let frames = extract_video_frames(path, num_frames)?;

        let frame_buffers: Vec<Vec<f32>> = frames
            .par_iter()
            .map(|frame| self.preprocess_video_frame(frame))
            .collect();

        let embed_2d = self.infer_embeddings(frame_buffers, run_options)?;
        let averaged = embed_2d
            .mean_axis(Axis(0))
            .unwrap_or_else(|| Array1::zeros(embed_2d.shape()[1]));

        Ok(vec![FileEmbedResult {
            path: path.to_path_buf(),
            embedding: Ok(l2_normalize(averaged)),
        }])
    }
}

pub trait VisualSearchModel: TextEncoder + VisualEncoder {}
