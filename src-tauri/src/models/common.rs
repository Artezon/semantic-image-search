use crate::utils::open_image_as_rgb;
use anyhow::Result;
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer, images::Image};
use image::RgbImage;
use ort::{ep, ep::ExecutionProvider, session::Session};
use std::path::Path;

pub fn build_session(model_path: &str, cpu_only: bool) -> Result<(Session, &'static str)> {
    let cuda = ep::CUDA::default();

    let mut builder = Session::builder()?;
    let device = if cpu_only {
        "CPU"
    } else {
        if cuda.register(&mut builder).is_ok() {
            "NVIDIA GPU (CUDA)"
        } else {
            "CPU"
        }
    };

    let session = builder.commit_from_file(model_path)?;
    Ok((session, device))
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

pub fn clip_prepare_image(
    path: &Path,
    w: usize,
    h: usize,
    mean: [f32; 3],
    std: [f32; 3],
) -> Result<Vec<f32>, String> {
    let img = open_image_as_rgb(path).map_err(|e| e.to_string())?;
    Ok(clip_prepare_rgb(&img, w, h, mean, std))
}
