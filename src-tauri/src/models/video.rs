use ffmpeg_next::{
    codec::{context, decoder},
    format::{Pixel, context::Input, input},
    media::Type,
    software::scaling::{context::Context, flag::Flags},
    util::frame::video::Video,
};
use image::RgbImage;
use std::path::Path;

pub fn extract_video_frames(path: &Path, num_frames: u32) -> Result<Vec<RgbImage>, String> {
    let num_frames = num_frames.max(1);

    ffmpeg_next::init().map_err(|e| format!("ffmpeg init failed: {e}"))?;

    let mut input_context = input(&path).map_err(|e| e.to_string())?;
    let video_stream = input_context
        .streams()
        .best(Type::Video)
        .ok_or("No video stream found")?;
    let stream_index = video_stream.index();
    let parameters = video_stream.parameters();
    drop(video_stream);

    let duration = input_context.duration();
    if duration <= 0 {
        return Err("Video duration is too small".to_string());
    }

    let mut dec = {
        let ctx =
            context::Context::from_parameters(parameters.clone()).map_err(|e| e.to_string())?;
        ctx.decoder().video().map_err(|e| e.to_string())?
    };
    let pix_fmt = dec.format();
    let w = dec.width();
    let h = dec.height();

    let mut scaler = Context::get(pix_fmt, w, h, Pixel::RGB24, w, h, Flags::BILINEAR)
        .map_err(|e| e.to_string())?;

    let mut frames: Vec<RgbImage> = Vec::with_capacity(num_frames as usize);

    for i in 1..=num_frames {
        let timestamp = (duration as u64 * i as u64 / (num_frames as u64 + 1)) as i64;
        input_context.seek(timestamp, ..timestamp).ok();
        dec.flush();

        if let Some(img) = decode_one_frame(&mut input_context, stream_index, &mut dec, &mut scaler)
        {
            frames.push(img);
        }
    }

    if frames.is_empty() {
        return Err("No frames decoded".to_string());
    }

    Ok(frames)
}

fn decode_one_frame(
    input_context: &mut Input,
    stream_index: usize,
    decoder: &mut decoder::Video,
    scaler: &mut Context,
) -> Option<RgbImage> {
    for (stream, packet) in input_context.packets() {
        // If not main video stream
        if stream.index() != stream_index {
            continue;
        }
        decoder.send_packet(&packet).ok();
        let mut decoded = Video::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = Video::empty();
            if scaler.run(&decoded, &mut rgb_frame).is_ok() {
                let data = rgb_frame.data(0).to_vec();
                if let Some(img) = RgbImage::from_raw(rgb_frame.width(), rgb_frame.height(), data) {
                    return Some(img);
                }
            }
        }
    }
    None
}
