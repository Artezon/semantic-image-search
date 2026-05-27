use crate::errors::AppError;
use crate::{db::SearchResult, models::EmbeddingKind, state::AppState};
use std::{path::PathBuf, sync::Arc};
use tauri::{AppHandle, Manager, command};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchInputType {
    Text,
    Image,
}

#[command(async)]
pub fn search(
    app_handle: AppHandle,
    search_type: SearchInputType,
    query: String,
    max_results: u32,
    threshold: f32,
) -> Result<Vec<SearchResult>, AppError> {
    let state = app_handle.state::<AppState>();
    let selected_model_manifest = state.selected_model;
    let selected_visual_model =
        Arc::clone(&state.model_manager.visual_search_models[selected_model_manifest]);
    let selected_kind = EmbeddingKind::Vision; // Currently only vision (CLIP) model supported
    let emb_type_id = state
        .db
        .get_emb_type_id(selected_model_manifest, &selected_kind)
        .map_err(|e| AppError::unknown(e))?
        .ok_or(AppError::NoIndex)?;

    let query_embedding = match search_type {
        SearchInputType::Text => {
            if !selected_visual_model
                .read()
                .unwrap()
                .is_text_encoder_loaded()
            {
                return Err(AppError::ModelNotReady);
            }

            selected_visual_model.write().unwrap().embed_text(&query)?
        }
        SearchInputType::Image => {
            if !selected_visual_model
                .read()
                .unwrap()
                .is_vision_encoder_loaded()
            {
                return Err(AppError::ModelNotReady);
            }

            let path = PathBuf::from(&query);
            if !path.is_file() {
                return Err(AppError::InvalidImagePath);
            }

            let mut query_img_embed = selected_visual_model
                .write()
                .unwrap()
                .embed_images(&[path])?;
            if query_img_embed.is_empty() {
                return Err(AppError::InvalidImagePath);
            }
            query_img_embed.remove(0).1?
        }
    }
    .to_vec();

    state
        .db
        .search_embeddings(query_embedding, emb_type_id, max_results, threshold)
        .map_err(|e| AppError::unknown(e))
}
