use crate::{errors::AppError, models::ModelManifest};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer, images::Image};
use image::RgbImage;
use ort::{ep, ep::ExecutionProvider, session::Session};
use std::path::Path;
use tokenizers::tokenizer::Tokenizer;

pub fn build_session(
    models_base_dir: &Path,
    manifest: &ModelManifest,
    file_index: usize,
    cpu_only: bool,
) -> Result<(Session, &'static str), AppError> {
    let cuda = ep::CUDA::default();

    let mut builder = Session::builder().map_err(|e| AppError::ModelLoadFailed {
        detail: e.to_string(),
        model_name: manifest.name,
    })?;
    let device = if cpu_only {
        "CPU"
    } else {
        if cuda.register(&mut builder).is_ok() {
            "NVIDIA GPU (CUDA)"
        } else {
            "CPU"
        }
    };

    let session = builder
        .commit_from_file(models_base_dir.join(manifest.files[file_index]))
        .map_err(|e| AppError::ModelLoadFailed {
            detail: e.to_string(),
            model_name: manifest.name,
        })?;
    Ok((session, device))
}

pub fn build_tokenizer(
    models_base_dir: &Path,
    manifest: &ModelManifest,
    file_index: usize,
) -> Result<Tokenizer, AppError> {
    let tokenizer = Tokenizer::from_file(models_base_dir.join(manifest.files[file_index]))
        .map_err(|e| AppError::TokenizerLoadFailed {
            detail: e.to_string(),
            model_name: manifest.name,
        })?;
    Ok(tokenizer)
}

pub fn clip_prepare_rgb(
    img: &RgbImage,
    w: usize,
    h: usize,
    mean: [f32; 3],
    std: [f32; 3],
) -> Vec<f32> {
    let hw = w * h;

    let (w0, h0) = img.dimensions();
    let (w0, h0) = (w0 as usize, h0 as usize);

    let (w1, h1) = if w0 < h0 {
        (w, ((h0 * w) + w0 / 2) / w0)
    } else {
        (((w0 * h) + h0 / 2) / h0, h)
    };

    let mut resizer = Resizer::new();
    let mut resized_img = Image::new(w1 as u32, h1 as u32, PixelType::U8x3);
    resizer
        .resize(
            img,
            &mut resized_img,
            &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::CatmullRom)),
        )
        .unwrap();

    let crop_x = (w1 - w) / 2;
    let crop_y = (h1 - h) / 2;
    let raw = resized_img.into_vec();

    let mut pixel_buf = vec![0f32; 3 * hw];
    for y in 0..h {
        for x in 0..w {
            let src_idx = ((y + crop_y) * w1 + (x + crop_x)) * 3;
            pixel_buf[0 * hw + y * w + x] = (raw[src_idx] as f32 / 255.0 - mean[0]) / std[0];
            pixel_buf[1 * hw + y * w + x] = (raw[src_idx + 1] as f32 / 255.0 - mean[1]) / std[1];
            pixel_buf[2 * hw + y * w + x] = (raw[src_idx + 2] as f32 / 255.0 - mean[2]) / std[2];
        }
    }
    pixel_buf
}
