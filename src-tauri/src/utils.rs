use image::{DynamicImage, ImageDecoder, ImageReader, ImageResult, RgbImage};
use ndarray::Array1;
use std::path::Path;

#[inline]
pub fn l2_normalize(vec: Array1<f32>) -> Array1<f32> {
    let norm = vec.dot(&vec).sqrt().max(f32::EPSILON);
    vec / norm
}

#[inline]
pub fn open_image_as_rgb(path: &Path) -> ImageResult<RgbImage> {
    let mut decoder = ImageReader::open(path)?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation()?;
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    Ok(img.to_rgb8())
}
