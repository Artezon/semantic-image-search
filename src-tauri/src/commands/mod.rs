pub mod indexing;
pub mod search;
pub mod state;
pub mod thumbnail;

#[macro_export]
macro_rules! get_handlers {
    () => {
        tauri::generate_handler![
            commands::state::get_config,
            commands::state::get_default_config,
            commands::state::apply_locale,
            commands::state::update_config,
            commands::state::get_indexed_count,
            commands::state::get_model_status,
            commands::state::get_dirs,
            commands::state::add_directory,
            commands::state::remove_directory,
            commands::state::reorder_directories,
            commands::indexing::index_directories,
            commands::indexing::pause_indexing,
            commands::search::search,
            commands::thumbnail::get_thumbnail,
        ]
    };
}
