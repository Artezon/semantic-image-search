use anyhow::Result;
use image::{DynamicImage, ImageDecoder, ImageReader, RgbImage};
use ndarray::Array1;
use std::path::Path;

pub fn format_seconds(seconds: u64) -> String {
    let h = seconds / 3600;
    let rem = seconds % 3600;
    let m = rem / 60;
    let s = rem % 60;

    let mut parts: Vec<String> = Vec::with_capacity(3);
    if h > 0 {
        parts.push(format!("{}h", h));
    }
    if m > 0 {
        parts.push(format!("{}m", m));
    }
    parts.push(format!("{}s", s));

    parts.join(" ")
}

#[inline]
pub fn l2_normalize(vec: Array1<f32>) -> Array1<f32> {
    let norm = vec.dot(&vec).sqrt().max(f32::EPSILON);
    vec / norm
}

#[inline]
pub fn open_image_as_rgb(path: &Path) -> Result<RgbImage> {
    let mut decoder = ImageReader::open(path)?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation()?;
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    Ok(img.to_rgb8())
}
