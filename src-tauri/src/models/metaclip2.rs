#[cfg(feature = "video")]
use crate::models::common::clip_prepare_rgb;
use crate::models::common::{build_session, clip_prepare_image};
use crate::models::{
    EmbeddingKind, Model, ModelError, ModelManifest, TextEncoder, VisualEncoder, VisualSearchModel,
};
use crate::utils::l2_normalize;
use anyhow::Result;
use ndarray::{Array1, Array2, Array4, ArrayView2, Axis};
use ort::{inputs, session::Session, value::TensorRef};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};
use tokenizers::tokenizer::Tokenizer;

pub static MANIFEST: ModelManifest = ModelManifest {
    name: "metaclip2_worldwide_b16_384",
    dim: 512,
    files: &[
        "metaclip2/text_model.onnx",
        "metaclip2/tokenizer.json",
        "metaclip2/vision_model.onnx",
    ],
    capabilities: &[EmbeddingKind::Visual],
};

const MEAN: [f32; 3] = [0.48145466f32, 0.4578275, 0.40821073];
const STD: [f32; 3] = [0.26862954f32, 0.26130258, 0.27577711];

pub struct MetaCLIP2B16_384Model {
    models_base_dir: PathBuf,
    input_img_w_and_h: u16,
    text_session: Option<Session>,
    text_session_device: String,
    vision_session: Option<Session>,
    tokenizer: Option<Tokenizer>,
    vision_session_device: String,
}

impl Model for MetaCLIP2B16_384Model {
    fn new(models_base_dir: &Path) -> Self {
        Self {
            models_base_dir: models_base_dir.to_path_buf(),
            input_img_w_and_h: 384,
            text_session: None,
            text_session_device: String::new(),
            vision_session: None,
            vision_session_device: String::new(),
            tokenizer: None,
        }
    }

    fn manifest(&self) -> &'static ModelManifest {
        &MANIFEST
    }

    fn device_string(&self) -> String {
        let text = self
            .text_session
            .as_ref()
            .map(|_| self.text_session_device.as_str());
        let vision = self
            .vision_session
            .as_ref()
            .map(|_| self.vision_session_device.as_str());

        match (text, vision) {
            (None, None) => String::new(),
            (Some(d), None) | (None, Some(d)) => d.to_owned(),
            (Some(a), Some(b)) if a == b => a.to_owned(),
            (Some(a), Some(b)) => format!("{a} and {b}"),
        }
    }
}

impl VisualSearchModel for MetaCLIP2B16_384Model {}

impl TextEncoder for MetaCLIP2B16_384Model {
    fn load_text_encoder(&mut self) -> Result<(), ModelError> {
        if self.is_text_encoder_loaded() {
            return Ok(());
        }

        let text_model_path = self.models_base_dir.join("metaclip2/text_model.onnx");
        let tok_path = self.models_base_dir.join("metaclip2/tokenizer.json");

        let (text_session, device_str) = build_session(text_model_path.to_str().unwrap(), false)
            .map_err(|e| {
                ModelError::LoadFailed(format!("MetaCLIP 2 text model loading failed: {}", e))
            })?;
        let tokenizer = Tokenizer::from_file(tok_path.to_str().unwrap()).map_err(|e| {
            ModelError::LoadFailed(format!("MetaCLIP 2 tokenizer loading failed: {}", e))
        })?;

        self.text_session = Some(text_session);
        self.tokenizer = Some(tokenizer);
        self.text_session_device = device_str.to_string();
        Ok(())
    }

    fn unload_text_encoder(&mut self) {
        self.text_session = None;
        self.tokenizer = None;
        self.text_session_device = String::new();
    }

    fn is_text_encoder_loaded(&self) -> bool {
        self.text_session.is_some() && self.tokenizer.is_some()
    }

    fn embed_text(&mut self, text: &str) -> Result<Array1<f32>, ModelError> {
        let encoding = self
            .tokenizer
            .as_ref()
            .ok_or(ModelError::NotLoaded)?
            .encode(text, true)
            .map_err(|e| ModelError::InferenceFailed(format!("Failed tokenization: {}", e)))?;

        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .collect();

        let seq_len = input_ids.len();

        let input_ids_arr = Array2::from_shape_vec((1, seq_len), input_ids)?;
        let attention_mask_arr = Array2::from_shape_vec((1, seq_len), attention_mask)?;

        let outputs = self.text_session.as_mut().unwrap().run(inputs![
            "input_ids"     => TensorRef::from_array_view(input_ids_arr.view())?,
            "attention_mask"=> TensorRef::from_array_view(attention_mask_arr.view())?,
        ])?;

        let embed_output = &outputs["text_embeds"];

        let (shape, raw_data) = embed_output
            .try_extract_tensor::<f32>()
            .map_err(|e| ModelError::InferenceFailed(format!("Failed inference: {}", e)))?;
        let embed = ArrayView2::from_shape((shape[0] as usize, shape[1] as usize), raw_data)?;

        let flat = embed.index_axis(Axis(0), 0).to_owned();
        Ok(l2_normalize(flat))
    }
}

impl VisualEncoder for MetaCLIP2B16_384Model {
    fn load_vision_encoder(&mut self) -> Result<(), ModelError> {
        if self.is_vision_encoder_loaded() {
            return Ok(());
        }

        let vision_model_path = self.models_base_dir.join("metaclip2/vision_model.onnx");

        let (vision_session, device_str) =
            build_session(vision_model_path.to_str().unwrap(), false).map_err(|e| {
                ModelError::LoadFailed(format!("MetaCLIP 2 vision model loading failed: {}", e))
            })?;

        self.vision_session = Some(vision_session);
        self.vision_session_device = device_str.to_string();
        Ok(())
    }

    fn unload_vision_encoder(&mut self) {
        self.vision_session = None;
        self.vision_session_device = String::new();
    }

    fn is_vision_encoder_loaded(&self) -> bool {
        self.vision_session.is_some()
    }

    fn embed_images(
        &mut self,
        paths: &[PathBuf],
    ) -> Result<Vec<(PathBuf, Result<Array1<f32>, String>)>, ModelError> {
        if self.vision_session.is_none() {
            return Err(ModelError::NotLoaded);
        }

        let w = self.input_img_w_and_h as usize;
        let h = self.input_img_w_and_h as usize;

        let imgs: Vec<Result<Vec<f32>, String>> = paths
            .par_iter()
            .map(|path| clip_prepare_image(path, w, h, MEAN, STD))
            .collect();

        let mut batch = Vec::new();
        let mut errors: Vec<Option<String>> = Vec::new();
        for res in imgs {
            match res {
                Ok(vec) => {
                    errors.push(None);
                    batch.push(vec);
                }
                Err(e) => {
                    errors.push(Some(e));
                }
            }
        }

        let batch_size = batch.len();
        if batch_size == 0 {
            return Ok(vec![]);
        }

        // Assemble successful images into a contiguous batch tensor
        let flat: Vec<f32> = batch.into_iter().flatten().collect();
        let pixel_tensor = Array4::from_shape_vec((batch_size, 3, h, w), flat)?;

        let outputs = self.vision_session.as_mut().unwrap().run(inputs![
            "pixel_values"  => TensorRef::from_array_view(pixel_tensor.view())?,
        ])?;

        let embed_output = &outputs["image_embeds"];
        let (shape, raw_data) = embed_output.try_extract_tensor::<f32>()?;
        let embed_2d = ArrayView2::from_shape((shape[0] as usize, shape[1] as usize), raw_data)?;

        let mut path_iter = paths.iter();
        let mut embed_iter = embed_2d.outer_iter();
        let mut result: Vec<(PathBuf, Result<Array1<f32>, String>)> = Vec::new();
        for err in errors {
            let path = path_iter.next().unwrap();
            match err {
                None => {
                    let row = embed_iter.next().unwrap();
                    let row = l2_normalize(row.to_owned());
                    result.push((path.clone(), Ok(row)));
                }
                Some(err_str) => result.push((path.clone(), Err(err_str))),
            }
        }

        Ok(result)
    }

    #[cfg(feature = "video")]
    fn embed_video(&mut self, path: &Path, num_frames: u32) -> Result<Array1<f32>, ModelError> {
        if self.vision_session.is_none() {
            return Err(ModelError::NotLoaded);
        }

        let frames = crate::models::video::extract_video_frames(path, num_frames)
            .map_err(|e| ModelError::InferenceFailed(e))?;

        let w = self.input_img_w_and_h as usize;
        let h = self.input_img_w_and_h as usize;

        let frame_tensors: Vec<Vec<f32>> = frames
            .iter()
            .map(|frame| clip_prepare_rgb(frame, w, h, MEAN, STD))
            .collect();

        if frame_tensors.is_empty() {
            return Err(ModelError::InferenceFailed(
                "No frames extracted from video".to_string(),
            ));
        }

        let batch_size = frame_tensors.len();
        let flat: Vec<f32> = frame_tensors.into_iter().flatten().collect();
        let pixel_tensor = Array4::from_shape_vec((batch_size, 3, h, w), flat)
            .map_err(|e| ModelError::InferenceFailed(e.to_string()))?;

        let outputs = self.vision_session.as_mut().unwrap().run(inputs![
            "pixel_values" => TensorRef::from_array_view(pixel_tensor.view())?,
        ])?;

        let embed_output = &outputs["image_embeds"];
        let (shape, raw_data) = embed_output.try_extract_tensor::<f32>()?;
        let embed_2d = ArrayView2::from_shape((shape[0] as usize, shape[1] as usize), raw_data)?;

        let averaged = embed_2d
            .mean_axis(Axis(0))
            .unwrap_or_else(|| Array1::zeros(shape[1] as usize));

        Ok(l2_normalize(averaged))
    }
}
