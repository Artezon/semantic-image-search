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
    let Some(emb_type_id) = state
        .db
        .get_emb_type_id(&selected_model_manifest, &selected_kind)
        .ok()
        .flatten()
    else {
        return Ok(vec![]);
    };

    let query_embedding = match search_type {
        SearchInputType::Text => {
            let mut model = selected_visual_model.write().unwrap();
            if !model.is_text_encoder_loaded() {
                return Err(AppError::ModelNotReady);
            }

            model.embed_text(&query, None)?
        }
        SearchInputType::Image => {
            let mut model = selected_visual_model.write().unwrap();
            if !model.is_vision_encoder_loaded() {
                return Err(AppError::ModelNotReady);
            }

            let path = PathBuf::from(&query);
            if !path.is_file() {
                return Err(AppError::InvalidImagePath);
            }

            model.embed_images(&[path], None)?.remove(0).embedding?
        }
    }
    .to_vec();

    state
        .db
        .search_embeddings(query_embedding, emb_type_id, max_results, threshold)
        .map_err(AppError::generic)
}
