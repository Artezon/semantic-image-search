use crate::{db::SearchResult, models::EmbeddingKind, state::AppState};
use std::{path::PathBuf, sync::Arc};
use tauri::{AppHandle, Manager, command};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchInputType {
    Text,
    Image,
}

#[command]
pub async fn search(
    app_handle: AppHandle,
    search_type: SearchInputType,
    query: String,
    max_results: u32,
    threshold: f32,
) -> Result<Vec<SearchResult>, String> {
    let state = app_handle.state::<AppState>();
    let selected_model_manifest = state.selected_model;
    let selected_visual_model =
        Arc::clone(&state.model_manager.visual_search_models[selected_model_manifest]);
    let model_id = selected_model_manifest.name;
    let selected_kind = EmbeddingKind::Visual; // Currently only visual supported
    let emb_type_id = state
        .db
        .get_emb_type_id(selected_model_manifest, &selected_kind)
        .map_err(|e| e.to_string())?;

    let query_embedding = match search_type {
        SearchInputType::Text => {
            if !selected_visual_model
                .read()
                .unwrap()
                .is_text_encoder_loaded()
            {
                return Err("Model not ready".to_string());
            }

            selected_visual_model
                .write()
                .unwrap()
                .embed_text(&query)
                .map_err(|e| e.to_string())
        }
        SearchInputType::Image => {
            if !selected_visual_model
                .read()
                .unwrap()
                .is_vision_encoder_loaded()
            {
                return Err("Model not ready".to_string());
            }

            let path = PathBuf::from(&query);
            if !path.is_file() {
                return Err("Please select a valid image as search query".to_string());
            }

            selected_visual_model
                .write()
                .unwrap()
                .embed_images(&[path])?
                .remove(0)
                .1
                .map_err(|e| format!("Can't open query image: {}", e))
        }
    }?
    .to_vec();

    state
        .db
        .search_embeddings(
            query_embedding,
            model_id,
            emb_type_id,
            max_results,
            threshold,
        )
        .map_err(|e| e.to_string())
}
