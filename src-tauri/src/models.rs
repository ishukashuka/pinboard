use serde::{Deserialize, Serialize};

fn default_board_id() -> String {
    "default".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub pan_x: f64,
    pub pan_y: f64,
    pub zoom: f64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardObject {
    pub id: String,
    #[serde(default = "default_board_id")]
    pub board_id: String,
    pub kind: String, // "text" | "image" | "url"
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub z_index: i32,
    pub title: Option<String>,
    pub content: String,
    pub meta: Option<String>, // JSON string
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipboardResult {
    pub kind: String, // "text" | "image" | "empty"
    pub text: Option<String>,
    pub image_data: Option<String>, // base64 data uri or asset ID
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub filename: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoardData {
    pub board: Board,
    pub objects: Vec<BoardObject>,
}
