use crate::utils::open_image_as_rgb;
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
pub async fn get_thumbnail(path: String, file_type: String) -> Result<ThumbnailResult, String> {
    let path = PathBuf::from(&path);

    async_runtime::spawn_blocking(move || match file_type.as_str() {
        "VID" => generate_video_thumbnail(&path),
        "IMG" => generate_image_thumbnail(&path),
        _ => Ok(ThumbnailResult::empty()),
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(feature = "video")]
fn generate_video_thumbnail(path: &PathBuf) -> Result<ThumbnailResult, String> {
    let frame = crate::models::video::extract_video_frames(path, 1)
        .map_err(|e| e.to_string())?
        .into_iter()
        .next()
        .ok_or_else(|| "No frame extracted".to_string())?;

    encode_jpeg(DynamicImage::ImageRgb8(frame))
}

#[cfg(not(feature = "video"))]
fn generate_video_thumbnail(_path: &PathBuf) -> Result<ThumbnailResult, String> {
    Err("Video support is disabled".to_string())
}

fn generate_image_thumbnail(path: &PathBuf) -> Result<ThumbnailResult, String> {
    let img = open_image_as_rgb(path).map_err(|e| e.to_string())?;
    encode_jpeg(DynamicImage::ImageRgb8(img))
}

fn encode_jpeg(img: DynamicImage) -> Result<ThumbnailResult, String> {
    let img = img.thumbnail(THUMBNAIL_MAX_PX, THUMBNAIL_MAX_PX);
    let mut buf = Vec::new();
    JpegEncoder::new_with_quality(&mut buf, THUMBNAIL_JPEG_QUALITY)
        .encode_image(&img)
        .map_err(|e| e.to_string())?;
    Ok(ThumbnailResult::jpeg(buf))
}
