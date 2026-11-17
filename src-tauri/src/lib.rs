mod models;
mod db;
mod clipboard;
mod commands;

use std::sync::Mutex;
use tauri::Manager;
use commands::AppState;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from(".pinboard_data"));
            let db_path = app_data_dir.join("pinboard.db");
            let assets_dir = app_data_dir.join("assets");

            let conn = db::init_db(&db_path).expect("Failed to initialize SQLite database");
            let clipboard_mgr = clipboard::ClipboardManager::new(assets_dir);

            app.manage(AppState {
                db: Mutex::new(conn),
                clipboard: clipboard_mgr,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_clipboard,
            commands::copy_text,
            commands::copy_image,
            commands::save_image_from_bytes,
            commands::load_board,
            commands::save_board,
            commands::delete_objects,
            commands::set_always_on_top,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
