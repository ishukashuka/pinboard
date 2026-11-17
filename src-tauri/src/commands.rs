use std::sync::Mutex;
use tauri::{AppHandle, State, Manager};
use crate::clipboard::ClipboardManager;
use crate::models::{Board, BoardConnector, BoardData, BoardObject, ClipboardResult};

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub clipboard: ClipboardManager,
}

#[tauri::command]
pub fn get_clipboard(state: State<'_, AppState>) -> Result<ClipboardResult, String> {
    state.clipboard.read_clipboard()
}

#[tauri::command]
pub fn copy_text(text: String, state: State<'_, AppState>) -> Result<(), String> {
    state.clipboard.write_text(&text)
}

#[tauri::command]
pub fn copy_image(image_data: String, state: State<'_, AppState>) -> Result<(), String> {
    state.clipboard.write_image(&image_data)
}

#[tauri::command]
pub fn save_image_from_bytes(
    bytes_base64: String,
    original_filename: Option<String>,
    state: State<'_, AppState>,
) -> Result<ClipboardResult, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&bytes_base64)
        .map_err(|e| format!("Invalid base64 data: {}", e))?;

    state.clipboard.save_image_bytes(&bytes, original_filename)
}

#[tauri::command]
pub fn load_board(board_id: Option<String>, state: State<'_, AppState>) -> Result<BoardData, String> {
    let target_id = board_id.unwrap_or_else(|| "default".to_string());
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let (board, objects, connectors) = crate::db::load_board(&db, &target_id).map_err(|e| e.to_string())?;
    Ok(BoardData { board, objects, connectors })
}

#[tauri::command]
pub fn save_board(
    board_id: String,
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
    objects: Vec<BoardObject>,
    connectors: Vec<BoardConnector>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    crate::db::save_board_state(&mut db, &board_id, pan_x, pan_y, zoom, &objects, &connectors).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_objects(ids: Vec<String>, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::db::delete_objects(&db, &ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_boards(state: State<'_, AppState>) -> Result<Vec<Board>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::db::get_boards(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_board(name: String, state: State<'_, AppState>) -> Result<Board, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::db::create_board(&db, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_board(id: String, name: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::db::rename_board(&db, &id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_always_on_top(enabled: bool, app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.set_always_on_top(enabled).map_err(|e| e.to_string())?;
    }
    Ok(())
}
