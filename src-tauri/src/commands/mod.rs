pub mod indexing;
pub mod search;
pub mod status;
pub mod thumbnail;

#[macro_export]
macro_rules! get_handlers {
    () => {
        tauri::generate_handler![
            commands::status::get_indexed_count,
            commands::status::get_model_status,
            commands::indexing::index_directory,
            commands::indexing::stop_indexing,
            commands::search::search,
            commands::thumbnail::get_thumbnail,
        ]
    };
}
