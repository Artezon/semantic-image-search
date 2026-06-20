use crate::errors::AppError;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::FfmpegEvent;
use image::RgbImage;
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

static FFMPEG: OnceLock<PathBuf> = OnceLock::new();

pub fn set_ffmpeg_path(path: PathBuf) {
    FFMPEG.set(path).ok();
}

fn ffmpeg() -> FfmpegCommand {
    match FFMPEG.get() {
        Some(path) => FfmpegCommand::new_with_path(path),
        None => FfmpegCommand::new(),
    }
}

pub fn extract_video_frames(path: &Path, num_frames: u32) -> Result<Vec<RgbImage>, AppError> {
    let num_frames = num_frames.max(1);

    // Probe duration
    let duration = {
        let mut dur = None;
        let events = ffmpeg()
            .input(path.to_string_lossy())
            .spawn()
            .map_err(|e| AppError::VideoReadFailed {
                detail: e.to_string(),
            })?
            .iter()
            .map_err(|e| AppError::VideoReadFailed {
                detail: e.to_string(),
            })?;
        for event in events {
            if let FfmpegEvent::ParsedDuration(d) = event {
                dur = Some(d.duration);
                break;
            }
        }
        dur.ok_or(AppError::VideoReadFailed {
            detail: "duration_error".to_string(),
        })?
    };

    if duration <= 0.0 {
        return Err(AppError::VideoReadFailed {
            detail: "duration_error".to_string(),
        });
    }

    // Build and run extraction command
    let mut cmd = ffmpeg();
    cmd.input(path.to_string_lossy());

    if num_frames > 1 && duration >= 0.1 {
        cmd.args(["-vf", &format!("fps={}/{}", num_frames - 1, duration)]);
    }

    cmd.args(["-vframes", &num_frames.to_string()]).rawvideo();
    let mut child = cmd.spawn().map_err(|e| AppError::VideoReadFailed {
        detail: e.to_string(),
    })?;
    let frames: Vec<RgbImage> = child
        .iter()
        .map_err(|e| AppError::VideoReadFailed {
            detail: e.to_string(),
        })?
        .filter_map(|e| {
            if let FfmpegEvent::OutputFrame(frame) = e {
                RgbImage::from_raw(frame.width, frame.height, frame.data)
            } else {
                None
            }
        })
        .collect();

    child.kill().ok();

    if frames.is_empty() {
        return Err(AppError::VideoReadFailed {
            detail: "error.video_format_unsupported".to_string(),
        });
    }

    Ok(frames)
}
