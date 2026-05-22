use crate::{
    db::{FileEmbResult, FileType},
    frontend::{
        MessageKind, clear_index_status, set_is_indexing, show_message,
        update_index_processing_status, update_index_status,
    },
    models::{ModelManifest, VisualSearchModel},
    state::AppState,
    utils::format_seconds,
};
use std::{
    fmt::Write,
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock, atomic::Ordering},
    time::{Duration, Instant, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager, State, command};
use walkdir::WalkDir;

#[cfg(not(feature = "heif"))]
static IMAGE_EXTENSIONS: [&str; 8] = ["jpg", "jpeg", "png", "bmp", "gif", "webp", "tiff", "avif"];
#[cfg(feature = "heif")]
static IMAGE_EXTENSIONS: [&str; 10] = [
    "jpg", "jpeg", "png", "bmp", "gif", "webp", "tiff", "avif", "heic", "heif",
];

#[cfg(feature = "video")]
static VIDEO_EXTENSIONS: [&str; 8] = ["mp4", "avi", "mov", "mkv", "flv", "wmv", "webm", "mpeg"];

struct FileList {
    paths: Vec<PathBuf>,
    batch_size: u32,
    file_type: FileType,
}

#[command]
pub async fn index_directory(app_handle: AppHandle, dir: String, batch_size: u32) {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        let selected_model_manifest = state.selected_model;
        let selected_visual_model =
            Arc::clone(&state.model_manager.visual_search_models[selected_model_manifest]);
        // let selected_kind = EmbeddingKind::Visual; // Currently only visual supported

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
    total: &mut usize,
    processed: &mut usize,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let emb_type_ids = state
        .db
        .add_model_to_db(selected_model_manifest)
        .map_err(|e| e.to_string())?;
    let emb_type_id = emb_type_ids[0];

    let mut images: Vec<PathBuf> = vec![];
    #[cfg(feature = "video")]
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
            #[cfg(feature = "video")]
            Some(e) if VIDEO_EXTENSIONS.contains(&e) => videos.push(path.to_path_buf()),
            _ => {}
        }
    }

    #[cfg(feature = "video")]
    {
        *total = images.len() + videos.len();
    }
    #[cfg(not(feature = "video"))]
    {
        *total = images.len();
    }

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
        #[cfg(feature = "video")]
        FileList {
            paths: videos,
            batch_size: 1,
            file_type: FileType::VID,
        },
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
                .map_err(|e| e.to_string())?
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis() as i64;
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

        for chunk in needs_indexing.chunks(batch_size as usize) {
            if state.indexing_stopped.load(Ordering::Relaxed) {
                return Ok(());
            }

            let result = match file_type {
                FileType::IMG => selected_visual_model.write().unwrap().embed_images(chunk),
                #[cfg(feature = "video")]
                FileType::VID => {
                    let path = chunk[0].clone();
                    let num_frames = state.config.read().unwrap().video_frames;
                    let emb = selected_visual_model
                        .write()
                        .unwrap()
                        .embed_video(&path, num_frames);
                    match emb {
                        Ok(emb) => Ok(vec![(path, Ok(emb))]),
                        Err(e) => Ok(vec![(path, Err(e.to_string()))]),
                    }
                }
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
                        .insert_files_and_embeddings(paths_and_embeddings, &file_type, emb_type_id)
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
