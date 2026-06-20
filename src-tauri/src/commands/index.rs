use crate::{
    db::{FileEmbedding, FileType},
    errors::AppError,
    frontend::{IndexingState, clear_index_status, send_index_status, send_indexing_error},
    models::{ModelManifest, VisualSearchModel},
    state::AppState,
};
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, atomic::Ordering};
use std::time::{Duration, Instant, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State, command};
use walkdir::WalkDir;

#[derive(Serialize)]
pub struct IndexingResult {
    processed: usize,
    total: usize,
    elapsed_secs: u64,
    was_paused: bool,
    errors: Vec<(String, AppError)>,
}

#[cfg(not(feature = "heif"))]
static IMAGE_EXTENSIONS: [&str; 8] = ["jpg", "jpeg", "png", "bmp", "gif", "webp", "tiff", "avif"];
#[cfg(feature = "heif")]
static IMAGE_EXTENSIONS: [&str; 10] = [
    "jpg", "jpeg", "png", "bmp", "gif", "webp", "tiff", "avif", "heic", "heif",
];

#[cfg(feature = "video")]
static VIDEO_EXTENSIONS: [&str; 8] = ["mp4", "avi", "mov", "mkv", "flv", "wmv", "webm", "mpeg"];

pub struct File {
    id: i64,
    path: PathBuf,
}

struct FileList {
    files: Vec<File>,
    batch_size: u32,
    file_type: FileType,
}

fn collect_files_from_dir(dir_path: &std::path::Path) -> Vec<(PathBuf, FileType)> {
    let mut files = vec![];

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
            Some(e) if IMAGE_EXTENSIONS.contains(&e) => {
                files.push((path.to_path_buf(), FileType::IMG))
            }
            #[cfg(feature = "video")]
            Some(e) if VIDEO_EXTENSIONS.contains(&e) => {
                files.push((path.to_path_buf(), FileType::VID))
            }
            _ => {}
        }
    }

    files
}

#[command]
pub async fn index_directories(app_handle: AppHandle) -> Result<IndexingResult, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        let selected_model_manifest = state.selected_model;

        if state.is_indexing.load(Ordering::Relaxed) {
            return Err(AppError::AlreadyIndexing);
        }

        let selected_visual_model =
            Arc::clone(&state.model_manager.visual_search_models[selected_model_manifest]);
        // let selected_kind = EmbeddingKind::Vision; // Currently only vision (CLIP) model supported

        if !selected_visual_model
            .read()
            .unwrap()
            .is_vision_encoder_loaded()
        {
            clear_index_status(&app_handle);
            return Err(AppError::ModelNotReady);
        }

        let dirs = state.db.get_dirs().unwrap_or_default();
        if dirs.is_empty() {
            clear_index_status(&app_handle);
            return Err(AppError::NoIndex);
        }

        let cumulative = state.indexing_elapsed_secs.load(Ordering::Relaxed);
        let start_time = Instant::now();

        state.is_indexing.store(true, Ordering::Relaxed);
        state.indexing_paused.store(false, Ordering::Relaxed);

        let mut processed = state.indexing_processed.load(Ordering::Relaxed) as usize;
        let mut total = 0usize;
        let mut errors: Vec<(String, AppError)> = vec![];

        let result = indexing(
            &app_handle,
            &dirs,
            state.deref(),
            &selected_model_manifest,
            selected_visual_model,
            &mut total,
            &mut processed,
            &mut errors,
        );

        let was_paused = state.indexing_paused.load(Ordering::Relaxed);
        let this_elapsed = start_time.elapsed().as_secs();
        let elapsed_secs = cumulative + this_elapsed;
        if was_paused {
            state
                .indexing_elapsed_secs
                .store(elapsed_secs, Ordering::Relaxed);
            state
                .indexing_processed
                .store(processed as u64, Ordering::Relaxed);
        } else {
            state.indexing_elapsed_secs.store(0, Ordering::Relaxed);
            state.indexing_processed.store(0, Ordering::Relaxed);
        }

        state.is_indexing.store(false, Ordering::Relaxed);
        state.indexing_paused.store(false, Ordering::Relaxed);

        if let Err(e) = result {
            state.indexing_elapsed_secs.store(0, Ordering::Relaxed);
            state.indexing_processed.store(0, Ordering::Relaxed);
            send_index_status(&app_handle, IndexingState::FatalError, 0, 0, 0);
            return Err(e);
        }

        Ok(IndexingResult {
            processed,
            total,
            elapsed_secs,
            was_paused,
            errors,
        })
    })
    .await
    .unwrap()
}

fn indexing(
    app_handle: &AppHandle,
    dirs: &[String],
    state: &AppState,
    selected_model_manifest: &ModelManifest,
    selected_visual_model: Arc<RwLock<dyn VisualSearchModel>>,
    total: &mut usize,
    processed: &mut usize,
    errors: &mut Vec<(String, AppError)>,
) -> Result<(), AppError> {
    let batch_size = state.config.read().unwrap().batch_size;
    let num_frames = state.config.read().unwrap().video_frames;

    let progress_update_interval = Duration::from_millis(100);
    let mut last_progress_update = Instant::now();

    let emb_type_ids = state
        .db
        .add_model_to_db(selected_model_manifest)
        .map_err(|e| e.to_string())?;
    let emb_type_id = emb_type_ids[0];

    for dir_str in dirs {
        if state.indexing_paused.load(Ordering::Relaxed) {
            return Ok(());
        }

        let dir_path = PathBuf::from(dir_str);
        if !dir_path.is_dir() {
            continue;
        }

        let files = collect_files_from_dir(&dir_path);
        match state.db.update_directory(dir_str, files) {
            Err(e) if e.downcast_ref() == Some(&rusqlite::Error::QueryReturnedNoRows) => {
                return Ok(());
            }
            other => other.map_err(|e| e.to_string())?,
        }
    }

    let all_files_db = state
        .db
        .get_all_files_with_emb_status(emb_type_id)
        .map_err(|e| e.to_string())?;

    let mut images_to_index: Vec<File> = vec![];
    #[cfg(feature = "video")]
    let mut videos_to_index: Vec<File> = vec![];

    for file in all_files_db {
        if state.indexing_paused.load(Ordering::Relaxed) {
            return Ok(());
        }

        let Ok(metadata) = std::fs::metadata(&file.path) else {
            continue;
        };
        let mtime = metadata
            .modified()
            .map_err(|e| e.to_string())?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis() as i64;
        let size = metadata.len() as i64;

        let needs_index = match (file.last_file_mtime, file.last_file_size) {
            (None, None) => true,
            (Some(mt), Some(sz)) => mtime != mt || size != sz,
            _ => true,
        };

        if needs_index {
            match file.file_type.as_str() {
                "IMG" => images_to_index.push(File {
                    id: file.id,
                    path: file.path,
                }),
                #[cfg(feature = "video")]
                "VID" => videos_to_index.push(File {
                    id: file.id,
                    path: file.path,
                }),
                _ => {}
            }
        }

        *total = *processed + images_to_index.len();
        #[cfg(feature = "video")]
        {
            *total += videos_to_index.len();
        }

        if *processed == 0 && last_progress_update.elapsed() > progress_update_interval {
            send_index_status(
                app_handle,
                IndexingState::Preparing,
                *processed,
                *total,
                errors.len(),
            );
            last_progress_update = Instant::now();
        }
    }

    if *total == 0 {
        return Ok(());
    }

    send_index_status(
        app_handle,
        IndexingState::Indexing,
        *processed,
        *total,
        errors.len(),
    );

    let file_lists = [
        FileList {
            files: images_to_index,
            batch_size,
            file_type: FileType::IMG,
        },
        #[cfg(feature = "video")]
        FileList {
            files: videos_to_index,
            batch_size: 1,
            file_type: FileType::VID,
        },
    ];

    for FileList {
        files,
        batch_size,
        file_type,
    } in file_lists
    {
        for chunk in files.chunks(batch_size as usize) {
            if state.indexing_paused.load(Ordering::Relaxed) {
                return Ok(());
            }

            let path_to_id: HashMap<&PathBuf, i64> =
                chunk.iter().map(|f| (&f.path, f.id)).collect();
            let paths: Vec<PathBuf> = chunk.iter().map(|f| f.path.clone()).collect();
            let result = match file_type {
                FileType::IMG => selected_visual_model.write().unwrap().embed_images(&paths),
                #[cfg(feature = "video")]
                FileType::VID => selected_visual_model
                    .write()
                    .unwrap()
                    .embed_video(&chunk[0].path, num_frames),
            };

            if state.indexing_paused.load(Ordering::Relaxed) {
                return Ok(());
            }

            match result {
                Ok(emb_results) => {
                    let mut embeddings: Vec<FileEmbedding> = vec![];
                    for (path, emb_res) in emb_results {
                        let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
                        let modified_at = metadata
                            .modified()
                            .map_err(|e| e.to_string())?
                            .duration_since(std::time::UNIX_EPOCH)
                            .map_err(|e| e.to_string())?
                            .as_millis() as i64;
                        let size = metadata.len() as i64;

                        match emb_res {
                            Ok(emb) => {
                                embeddings.push(FileEmbedding {
                                    file_id: *path_to_id.get(&path).unwrap(),
                                    file_mtime: modified_at,
                                    file_size: size,
                                    embedding: emb.to_vec(),
                                });
                            }
                            Err(e) => {
                                let err_value = serde_json::to_value(&e).unwrap_or_default();
                                let detail = err_value
                                    .get("detail")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("");
                                let path_str = path.display().to_string();
                                send_indexing_error(app_handle, &path_str, detail);
                                errors.push((path_str, e));
                            }
                        }
                    }

                    *processed += embeddings.len();
                    state
                        .db
                        .insert_embeddings(emb_type_id, embeddings)
                        .map_err(|e| e.to_string())?;
                }
                Err(e) => {
                    let err_value = serde_json::to_value(&e).unwrap_or_default();
                    let detail = err_value
                        .get("detail")
                        .and_then(|m| m.as_str())
                        .unwrap_or("");
                    for file in chunk {
                        let path_str = file.path.display().to_string();
                        send_indexing_error(app_handle, &path_str, detail);
                        errors.push((path_str, e.clone()));
                    }
                }
            }
            if last_progress_update.elapsed() > progress_update_interval {
                send_index_status(
                    &app_handle,
                    IndexingState::Indexing,
                    *processed,
                    *total,
                    errors.len(),
                );
                last_progress_update = Instant::now();
            }
        }
    }

    Ok(())
}

#[command]
pub async fn pause_indexing(state: State<'_, AppState>) -> Result<(), ()> {
    state.indexing_paused.store(true, Ordering::Relaxed);
    Ok(())
}
