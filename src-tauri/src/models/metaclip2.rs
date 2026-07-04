use crate::models::common::clip_prepare_rgb;
use crate::models::{
    EmbeddingKind, Model, ModelManifest, TextEncoder, TokenizedText, VisualEncoder,
    VisualSearchModel,
    common::{build_session, build_tokenizer},
};
use crate::utils::l2_normalize;
use crate::{errors::AppError, models::EmbedResult};
use ndarray::{Array2, Array4, ArrayView2, Axis};
use ort::{
    inputs,
    session::{NoSelectedOutputs, RunOptions, Session},
    value::TensorRef,
};
use std::path::{Path, PathBuf};
use tokenizers::tokenizer::Tokenizer;

pub static MANIFEST: ModelManifest = ModelManifest {
    name: "metaclip2_worldwide_b16_384",
    dim: 512,
    files: &[
        "metaclip2/tokenizer.json",
        "metaclip2/text_model.onnx",
        "metaclip2/vision_model.onnx",
    ],
    capabilities: &[EmbeddingKind::Vision],
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
    fn load_text_encoder(&mut self) -> Result<(), AppError> {
        if self.is_text_encoder_loaded() {
            return Ok(());
        }

        let (text_session, device_str) = build_session(&self.models_base_dir, &MANIFEST, 1, false)?;
        let tokenizer = build_tokenizer(&self.models_base_dir, &MANIFEST, 0)?;

        self.text_session = Some(text_session);
        self.tokenizer = Some(tokenizer);
        self.text_session_device = device_str.to_string();
        Ok(())
    }

    // fn unload_text_encoder(&mut self) {
    //     self.text_session = None;
    //     self.tokenizer = None;
    //     self.text_session_device = String::new();
    // }

    fn is_text_encoder_loaded(&self) -> bool {
        self.text_session.is_some() && self.tokenizer.is_some()
    }

    fn tokenize_text(&self, text: &str) -> Result<TokenizedText, AppError> {
        let encoding = self
            .tokenizer
            .as_ref()
            .ok_or(AppError::ModelNotReady)?
            .encode(text, true)?;

        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&x| x as i64)
            .collect();

        let seq_len = input_ids.len();

        Ok(TokenizedText {
            input_ids: Array2::from_shape_vec((1, seq_len), input_ids)?,
            attention_mask: Array2::from_shape_vec((1, seq_len), attention_mask)?,
        })
    }

    fn infer_text_embeddings(
        &mut self,
        tokens: TokenizedText,
        run_options: Option<&RunOptions<NoSelectedOutputs>>,
    ) -> EmbedResult {
        let input_values = inputs![
            "input_ids" => TensorRef::from_array_view(tokens.input_ids.view())?,
            "attention_mask" => TensorRef::from_array_view(tokens.attention_mask.view())?,
        ];
        let session = self.text_session.as_mut().ok_or(AppError::ModelNotReady)?;
        let outputs = if let Some(opts) = run_options {
            session.run_with_options(input_values, opts)?
        } else {
            session.run(input_values)?
        };
        let embed_output = &outputs["text_embeds"];
        let (shape, raw_data) = embed_output.try_extract_tensor::<f32>()?;
        let embed = ArrayView2::from_shape((shape[0] as usize, shape[1] as usize), raw_data)?;
        let flat = embed.index_axis(Axis(0), 0).to_owned();
        Ok(l2_normalize(flat))
    }
}

impl VisualEncoder for MetaCLIP2B16_384Model {
    fn load_vision_encoder(&mut self) -> Result<(), AppError> {
        if self.is_vision_encoder_loaded() {
            return Ok(());
        }

        let (vision_session, device_str) =
            build_session(&self.models_base_dir, &MANIFEST, 2, false)?;

        self.vision_session = Some(vision_session);
        self.vision_session_device = device_str.to_string();
        Ok(())
    }

    // fn unload_vision_encoder(&mut self) {
    //     self.vision_session = None;
    //     self.vision_session_device = String::new();
    // }

    fn is_vision_encoder_loaded(&self) -> bool {
        self.vision_session.is_some()
    }

    fn preprocess_image(&self, img: &image::RgbImage) -> Vec<f32> {
        let size = self.input_img_w_and_h as usize;
        clip_prepare_rgb(img, size, size, MEAN, STD)
    }

    #[cfg(feature = "video")]
    fn preprocess_video_frame(&self, frame: &image::RgbImage) -> Vec<f32> {
        let size = self.input_img_w_and_h as usize;
        clip_prepare_rgb(frame, size, size, MEAN, STD)
    }

    fn infer_embeddings(
        &mut self,
        pixel_batch: Vec<Vec<f32>>,
        run_options: Option<&RunOptions<NoSelectedOutputs>>,
    ) -> Result<Array2<f32>, AppError> {
        let size = self.input_img_w_and_h as usize;
        let batch_size = pixel_batch.len();
        let flat: Vec<f32> = pixel_batch.into_iter().flatten().collect();
        let pixel_tensor = Array4::from_shape_vec((batch_size, 3, size, size), flat)?;

        let input_values = inputs![
            "pixel_values" => TensorRef::from_array_view(pixel_tensor.view())?,
        ];
        let session = self
            .vision_session
            .as_mut()
            .ok_or(AppError::ModelNotReady)?;
        let outputs = if let Some(opts) = run_options {
            session.run_with_options(input_values, opts)?
        } else {
            session.run(input_values)?
        };
        let embed_output = &outputs["image_embeds"];
        let (shape, raw_data) = embed_output.try_extract_tensor::<f32>()?;
        Ok(ArrayView2::from_shape((shape[0] as usize, shape[1] as usize), raw_data)?.to_owned())
    }
}
