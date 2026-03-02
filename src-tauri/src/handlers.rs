#[macro_export]
macro_rules! get_handlers {
    () => {
        tauri::generate_handler![
            app::get_index_status,
            app::get_model_status,
            app::index_directory,
            app::stop_indexing,
            app::search,
        ]
    };
}
