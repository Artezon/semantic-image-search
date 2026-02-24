// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod models;
mod state;

use tauri::{webview, window, Builder, Manager, WebviewUrl, WebviewWindowBuilder};

fn main() {
    Builder::default()
        .setup(|app| {
            app.manage(state::AppState::new(app.handle()));

            WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("Media Search")
                .inner_size(1100.0, 650.0)
                .min_inner_size(800.0, 600.0)
                .decorations(false)
                .visible(false)
                .center()
                .incognito(true)
                .background_color(window::Color(0, 0, 0, 255))
                .scroll_bar_style(webview::ScrollBarStyle::FluentOverlay)
                .data_directory(app.state::<state::AppState>().data_path.join("webview"))
                .build()?;

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        // .invoke_handler(tauri::generate_handler![greet, func2])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
