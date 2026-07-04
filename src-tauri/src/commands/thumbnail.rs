use crate::errors::AppError;
use crate::utils::open_image_as_rgb;
use crate::video::extract_video_frames;
use image::DynamicImage;
use image::codecs::jpeg::JpegEncoder;
use std::path::PathBuf;
use tauri::{async_runtime, command};

const THUMBNAIL_MAX_PX: u32 = 512;
const THUMBNAIL_JPEG_QUALITY: u8 = 85;

#[derive(serde::Serialize)]
pub struct ThumbnailResult {
    pub bytes: Option<Vec<u8>>,
    pub mime: Option<String>,
}

impl ThumbnailResult {
    fn jpeg(bytes: Vec<u8>) -> Self {
        Self {
            bytes: Some(bytes),
            mime: Some("image/jpeg".to_string()),
        }
    }

    fn empty() -> Self {
        Self {
            bytes: None,
            mime: None,
        }
    }
}

#[command]
pub async fn get_thumbnail(path: String, file_type: String) -> Result<ThumbnailResult, AppError> {
    let path = PathBuf::from(&path);

    async_runtime::spawn_blocking(move || match file_type.as_str() {
        "IMG" => generate_image_thumbnail(&path),
        "VID" => generate_video_thumbnail(&path),
        _ => Ok(ThumbnailResult::empty()),
    })
    .await
    .map_err(AppError::generic)?
}

#[cfg(feature = "video")]
fn generate_video_thumbnail(path: &PathBuf) -> Result<ThumbnailResult, AppError> {
    let frame = extract_video_frames(path, 1)?.swap_remove(0);
    encode_jpeg(&DynamicImage::ImageRgb8(frame))
}

#[cfg(not(feature = "video"))]
fn generate_video_thumbnail(_path: &PathBuf) -> Result<ThumbnailResult, AppError> {
    Err(AppError::VideoReadFailed {
        msg: "video_support_disabled".to_string(),
    })
}

fn generate_image_thumbnail(path: &PathBuf) -> Result<ThumbnailResult, AppError> {
    let img = open_image_as_rgb(path).map_err(AppError::generic)?;
    encode_jpeg(&DynamicImage::ImageRgb8(img))
}

fn encode_jpeg(img: &DynamicImage) -> Result<ThumbnailResult, AppError> {
    let img = img.thumbnail(THUMBNAIL_MAX_PX, THUMBNAIL_MAX_PX);
    let mut buf = vec![];
    JpegEncoder::new_with_quality(&mut buf, THUMBNAIL_JPEG_QUALITY)
        .encode_image(&img)
        .map_err(AppError::generic)?;
    Ok(ThumbnailResult::jpeg(buf))
}
