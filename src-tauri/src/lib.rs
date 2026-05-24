// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod dylib;
mod errors;
mod frontend;
mod models;
mod state;
mod utils;

#[cfg(feature = "heif")]
use libheif_rs::integration::image::register_all_decoding_hooks;
use tauri::{Builder, Manager, WebviewUrl, WebviewWindowBuilder, webview, window};
use tauri_plugin_window_state::StateFlags;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(feature = "heif")]
    register_all_decoding_hooks();

    Builder::default()
        .setup(|app| {
            let app_state = state::AppState::new(app.handle());

            #[cfg(feature = "video")]
            models::video::set_ffmpeg_path(app_state.data_path.join("lib/ffmpeg.exe"));

            app.manage(app_state);

            #[cfg(desktop)]
            app.handle().plugin(
                tauri_plugin_window_state::Builder::new()
                    .with_state_flags(StateFlags::all() & !StateFlags::VISIBLE)
                    .build(),
            )?;

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
        .invoke_handler(get_handlers!())
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                app_handle.state::<state::AppState>().db.close();
            }
        });
}
