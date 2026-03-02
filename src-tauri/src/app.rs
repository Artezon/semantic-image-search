use crate::{
    db::FileEmbResult,
    models::{EmbeddingKind, ModelManifest, VisualSearchModel},
    state::AppState,
    utils::format_seconds,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Write,
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock, atomic::Ordering},
    time::Duration,
    time::Instant,
};
use tauri::{AppHandle, Emitter, Manager, State, command};
use walkdir::WalkDir;

static IMAGE_EXTENSIONS: [&str; 8] = ["jpg", "jpeg", "png", "bmp", "gif", "webp", "tiff", "heic"];
static VIDEO_EXTENSIONS: [&str; 8] = ["mp4", "avi", "mov", "mkv", "flv", "wmv", "webm", "mpeg"];

#[derive(PartialEq)]
pub enum FileType {
    IMG,
    // VID,
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IMG => "IMG",
            // Self::VID => "VID",
        }
    }
}

struct FileList {
    paths: Vec<PathBuf>,
    batch_size: u32,
    file_type: FileType,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum MessageKind {
    #[default]
    Info,
    Error,
    Warning,
}

#[derive(Clone, Serialize, Default)]
struct MessagePayload {
    title: String,
    msg: String,
    kind: MessageKind,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum ModelStatus {
    #[default]
    Neutral,
    Success,
    Error,
}

#[derive(Clone, Serialize, Default)]
struct ModelStatusPayload {
    status: ModelStatus,
    status_text: String,
    device_text: String,
}

#[derive(Clone, Serialize, Default)]
struct IndexStatusPayload {
    progress: Option<f32>,
    text: String,
}

fn show_message(app_handle: &AppHandle, title: &str, msg: &str, kind: MessageKind) {
    let _ = app_handle
        .emit(
            "message",
            &MessagePayload {
                title: title.to_string(),
                msg: msg.to_string(),
                kind,
            },
        )
        .unwrap();
}

fn update_model_status(
    app_handle: &AppHandle,
    status: ModelStatus,
    status_text: &str,
    device_text: &str,
) {
    let _ = app_handle
        .emit(
            "model-status",
            &ModelStatusPayload {
                status,
                status_text: status_text.to_string(),
                device_text: device_text.to_string(),
            },
        )
        .unwrap();
}

fn update_model_status_from_payload(app_handle: &AppHandle, payload: &ModelStatusPayload) {
    let _ = app_handle.emit("model-status", payload).unwrap();
}

fn update_index_status(app_handle: &AppHandle, progress: Option<f32>, text: &str) {
    let _ = app_handle
        .emit(
            "index-status",
            &IndexStatusPayload {
                progress,
                text: text.to_string(),
            },
        )
        .unwrap();
}

fn update_index_processing_status(
    app_handle: &AppHandle,
    processed: usize,
    total: usize,
    errors: usize,
) {
    let mut msg = format!("Processed {processed} / {total} files...");
    if errors > 0 {
        msg += &format!(" ({errors} errors)");
    }
    update_index_status(&app_handle, Some(processed as f32 / total as f32), &msg);
}

fn clear_index_status(app_handle: &AppHandle) {
    update_index_status(&app_handle, Some(1.0), "")
}

fn set_is_indexing(app_handle: &AppHandle, is_indexing: bool) {
    let _ = app_handle.emit("is-indexing", is_indexing).unwrap();
}

#[command]
pub async fn get_index_status(app_handle: AppHandle) {
    let state = app_handle.state::<AppState>();

    update_index_status(
        &app_handle,
        None,
        &format!("{} indexed files", state.db.count_indexed_files().unwrap()),
    );
}

#[command]
pub async fn get_model_status(app_handle: AppHandle) {
    let state = app_handle.state::<AppState>();
    let selected_model_manifest = state.selected_model;

    update_model_status(&app_handle, ModelStatus::Neutral, "Loading model...", "");

    let model = Arc::clone(&state.model_manager.visual_search_models[selected_model_manifest]);

    let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut model_context = model.write().unwrap();
        model_context.load_text_encoder()?;
        model_context.load_vision_encoder()
    })) {
        Ok(Ok(())) => ModelStatusPayload {
            status: ModelStatus::Success,
            status_text: "Model loaded successfully!".to_string(),
            device_text: model.read().unwrap().device_string(),
        },
        Ok(Err(e)) => ModelStatusPayload {
            status: ModelStatus::Error,
            status_text: format!("Error loading model: {:?}", e),
            device_text: "".to_string(),
        },
        Err(panic_info) => {
            let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown error".to_string()
            };
            ModelStatusPayload {
                status: ModelStatus::Error,
                status_text: format!("Fatal error: {}", msg),
                device_text: "".to_string(),
            }
        }
    };

    update_model_status_from_payload(&app_handle, &result);
}

#[command]
pub async fn index_directory(app_handle: AppHandle, dir: String, batch_size: u32) {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        let selected_model_manifest = state.selected_model;
        let selected_visual_model =
            Arc::clone(&state.model_manager.visual_search_models[selected_model_manifest]);
        let selected_kind = EmbeddingKind::Visual; // Currently only visual supported

        if !selected_visual_model
            .read()
            .unwrap()
            .is_vision_encoder_loaded()
        {
            show_message(
                &app_handle,
                "Model not ready",
                "Please wait for the model to load.",
                MessageKind::Info,
            );
            clear_index_status(&app_handle);
            return;
        }

        let dir_path = PathBuf::from(dir);

        if !dir_path.is_dir() {
            show_message(
                &app_handle,
                "Invalid directory",
                "Please select a valid directory.",
                MessageKind::Warning,
            );
            clear_index_status(&app_handle);
            return;
        }

        let start_time = Instant::now();

        *state.is_indexing.lock().unwrap() = true;
        set_is_indexing(&app_handle, true);
        state.indexing_stopped.store(false, Ordering::Relaxed);

        let mut total = 0usize;
        let mut processed = 0usize;
        let mut errors: Vec<String> = vec![];

        let indexing_result = dir_indexing(
            &app_handle,
            &dir_path,
            batch_size,
            state.deref(),
            &selected_model_manifest,
            selected_visual_model,
            &selected_kind,
            &mut total,
            &mut processed,
            &mut errors,
        );

        if let Err(e) = indexing_result {
            update_index_status(&app_handle, Some(1.0), "Indexing error occurred");
            show_message(&app_handle, "Indexing error", &e, MessageKind::Error);
            return;
        }

        let elapsed_secs = start_time.elapsed().as_secs();
        let mut summary = format!(
            "Processed {processed} / {total} files.\n\nElapsed time: {}",
            format_seconds(elapsed_secs)
        );
        let err_count = errors.len();
        if err_count > 0 {
            write!(
                summary,
                "\n\nErrors ({}):\n\n{}",
                err_count,
                errors[..err_count.min(5)].join("\n")
            )
            .unwrap();
            if err_count == 6 {
                write!(summary, "\n... and 1 more error.").unwrap();
            } else if err_count > 6 {
                write!(summary, "\n... and {} more errors.", err_count - 5).unwrap();
            }
        }

        update_index_status(
            &app_handle,
            Some(1.0),
            &if state.indexing_stopped.load(Ordering::Relaxed) {
                format!("Stopped! Processed {processed} / {total} files.")
            } else {
                format!("Completed! Processed {processed} / {total} files.")
            },
        );
        show_message(
            &app_handle,
            if state.indexing_stopped.load(Ordering::Relaxed) {
                "Processing stopped"
            } else {
                "Processing complete"
            },
            &summary,
            MessageKind::Info,
        );

        *state.is_indexing.lock().unwrap() = false;
        set_is_indexing(&app_handle, false);
        state.indexing_stopped.store(false, Ordering::Relaxed);
    })
    .await
    .unwrap();
}

fn dir_indexing(
    app_handle: &AppHandle,
    dir_path: &PathBuf,
    batch_size: u32,
    state: &AppState,
    selected_model_manifest: &ModelManifest,
    selected_visual_model: Arc<RwLock<dyn VisualSearchModel>>,
    selected_kind: &EmbeddingKind,
    total: &mut usize,
    processed: &mut usize,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let _ = state
        .db
        .add_model_to_db(selected_model_manifest)
        .map_err(|e| e.to_string())?;
    let emb_type_id = state
        .db
        .get_emb_type_id(selected_model_manifest, &selected_kind)
        .map_err(|e| e.to_string())?;

    let mut images: Vec<PathBuf> = vec![];
    let mut videos: Vec<PathBuf> = vec![];

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match ext.as_deref() {
            Some(e) if IMAGE_EXTENSIONS.contains(&e) => images.push(path.to_path_buf()),
            Some(e) if VIDEO_EXTENSIONS.contains(&e) => videos.push(path.to_path_buf()),
            _ => {}
        }
    }

    *total = images.len() + videos.len();

    if *total == 0 {
        update_index_status(
            &app_handle,
            Some(1.0),
            "No media files found in the directory.",
        );
        return Ok(());
    }

    let progress_update_interval = Duration::from_millis(100);
    let mut last_progress_update = Instant::now();

    let file_lists = [
        FileList {
            paths: images,
            batch_size,
            file_type: FileType::IMG,
        },
        // FileList {
        //     paths: videos,
        //     batch_size: 1,
        //     file_type: FileType::VID, // TODO: video embedding
        // },
    ];

    for FileList {
        paths,
        batch_size,
        file_type,
    } in file_lists
    {
        let mut needs_indexing: Vec<PathBuf> = vec![];

        for path in paths {
            if state.indexing_stopped.load(Ordering::Relaxed) {
                return Ok(());
            }

            let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
            let mtime = metadata
                .modified()
                .unwrap()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs_f64();
            let size = metadata.len();

            let file_in_db = state
                .db
                .get_emb_by_file_path(&path, Some(emb_type_id))
                .unwrap();
            match file_in_db {
                // File is not in DB
                None => needs_indexing.push(path),
                Some(FileEmbResult { ref embeddings, .. }) => match embeddings.get(0) {
                    // File is in DB but has no embedding of this type yet
                    None => needs_indexing.push(path),
                    // Embedding exists but file has changed since it was indexed
                    Some(emb)
                        if mtime != emb.last_file_mtime || size != emb.last_file_size as u64 =>
                    {
                        needs_indexing.push(path);
                    }
                    // Embedding is up to date
                    Some(_) => {
                        *processed += 1;
                        if last_progress_update.elapsed() > progress_update_interval {
                            update_index_processing_status(
                                &app_handle,
                                *processed,
                                *total,
                                errors.len(),
                            );
                            last_progress_update = Instant::now();
                        }
                    }
                },
            }
        }

        update_index_processing_status(&app_handle, *processed, *total, errors.len());

        let model_id = selected_model_manifest.name;

        for chunk in needs_indexing.chunks(batch_size as usize) {
            if state.indexing_stopped.load(Ordering::Relaxed) {
                return Ok(());
            }

            let result = match file_type {
                FileType::IMG => selected_visual_model.write().unwrap().embed_images(chunk),
                // FileType::VID => selected_model.write().unwrap().embed_video(chunk[0]),
            };

            match result {
                Ok(vec) => {
                    let mut paths_and_embeddings: Vec<(PathBuf, Vec<f32>)> = vec![];
                    for (path, emb_res) in vec {
                        match emb_res {
                            Ok(emb) => {
                                paths_and_embeddings.push((path, emb.to_vec()));
                            }
                            Err(e) => {
                                errors.push(format!("Skipped {}\n{}\n", path.display(), e));
                            }
                        }
                    }

                    *processed += paths_and_embeddings.len();
                    state
                        .db
                        .insert_files_and_embeddings(
                            paths_and_embeddings,
                            &file_type,
                            model_id,
                            emb_type_id,
                        )
                        .map_err(|e| e.to_string())?;
                }
                Err(e) => {
                    for path in chunk {
                        errors.push(format!("Skipped {}\n{:?}\n", path.display(), e));
                    }
                }
            }
            if last_progress_update.elapsed() > progress_update_interval {
                update_index_processing_status(&app_handle, *processed, *total, errors.len());
                last_progress_update = Instant::now();
            }
        }
    }

    Ok(())
}

#[command]
pub async fn stop_indexing(state: State<'_, AppState>) -> Result<(), ()> {
    state.indexing_stopped.store(true, Ordering::Relaxed);
    Ok(())
}
