use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};
use uuid::Uuid;
use crate::models::ClipboardResult;

pub struct ClipboardManager {
    assets_dir: PathBuf,
}

impl ClipboardManager {
    pub fn new(assets_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&assets_dir);
        Self { assets_dir }
    }

    /// Read current clipboard content (image or text)
    pub fn read_clipboard(&self) -> Result<ClipboardResult, String> {
        #[cfg(target_os = "linux")]
        {
            // 1. Try Wayland wl-paste first if available
            if let Ok(result) = self.read_from_wl_paste() {
                if result.kind != "empty" {
                    return Ok(result);
                }
            }

            // 2. Try arboard cross-platform clipboard
            if let Ok(result) = self.read_from_arboard() {
                if result.kind != "empty" {
                    return Ok(result);
                }
            }

            // 3. Try X11 xclip
            if let Ok(result) = self.read_from_xclip() {
                if result.kind != "empty" {
                    return Ok(result);
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Windows / macOS: Use arboard native OS clipboard directly
            if let Ok(result) = self.read_from_arboard() {
                if result.kind != "empty" {
                    return Ok(result);
                }
            }
        }

        Ok(ClipboardResult {
            kind: "empty".to_string(),
            text: None,
            image_data: None,
            width: None,
            height: None,
            filename: None,
        })
    }

    #[cfg(target_os = "linux")]
    fn read_from_wl_paste(&self) -> Result<ClipboardResult, String> {
        // Query types
        let output = Command::new("wl-paste")
            .arg("--list-types")
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err("wl-paste --list-types failed".to_string());
        }

        let types_str = String::from_utf8_lossy(&output.stdout);
        let types: Vec<&str> = types_str.lines().map(|l| l.trim()).collect();

        // Check if an image format is present in clipboard
        let has_png = types.iter().any(|&t| t == "image/png");
        let has_jpeg = types.iter().any(|&t| t == "image/jpeg");
        let has_bmp = types.iter().any(|&t| t == "image/bmp");
        let has_image = has_png || has_jpeg || has_bmp || types.iter().any(|&t| t.starts_with("image/"));

        if has_image {
            let img_type = if has_png { "image/png" } else if has_jpeg { "image/jpeg" } else { "image/png" };
            let img_output = Command::new("wl-paste")
                .args(["--type", img_type])
                .output()
                .map_err(|e| e.to_string())?;

            if img_output.status.success() && !img_output.stdout.is_empty() {
                return self.save_image_bytes(&img_output.stdout, None);
            }
        }

        // Check text format
        let has_text = types.iter().any(|&t| {
            t == "text/plain;charset=utf-8" || t == "text/plain" || t == "UTF8_STRING" || t == "STRING" || t == "TEXT"
        });

        if has_text || !types.is_empty() {
            let text_output = Command::new("wl-paste")
                .arg("--no-newline")
                .output()
                .map_err(|e| e.to_string())?;

            if text_output.status.success() {
                let text = String::from_utf8_lossy(&text_output.stdout).to_string();
                if !text.trim().is_empty() {
                    return Ok(ClipboardResult {
                        kind: "text".to_string(),
                        text: Some(text),
                        image_data: None,
                        width: None,
                        height: None,
                        filename: None,
                    });
                }
            }
        }

        Ok(ClipboardResult {
            kind: "empty".to_string(),
            text: None,
            image_data: None,
            width: None,
            height: None,
            filename: None,
        })
    }

    fn read_from_arboard(&self) -> Result<ClipboardResult, String> {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

        // Try getting image first
        if let Ok(img) = clipboard.get_image() {
            let width = img.width as u32;
            let height = img.height as u32;
            if width > 0 && height > 0 {
                if let Some(rgba) = image::RgbaImage::from_raw(width, height, img.bytes.into_owned()) {
                    let mut png_bytes = Vec::new();
                    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
                    use image::ImageEncoder;
                    if encoder.write_image(&rgba, width, height, image::ExtendedColorType::Rgba8).is_ok() {
                        return self.save_image_bytes(&png_bytes, None);
                    }
                }
            }
        }

        // Try getting text
        if let Ok(text) = clipboard.get_text() {
            if !text.trim().is_empty() {
                return Ok(ClipboardResult {
                    kind: "text".to_string(),
                    text: Some(text),
                    image_data: None,
                    width: None,
                    height: None,
                    filename: None,
                });
            }
        }

        Ok(ClipboardResult {
            kind: "empty".to_string(),
            text: None,
            image_data: None,
            width: None,
            height: None,
            filename: None,
        })
    }

    #[cfg(target_os = "linux")]
    fn read_from_xclip(&self) -> Result<ClipboardResult, String> {
        // Try getting image via xclip
        let img_output = Command::new("xclip")
            .args(["-selection", "clipboard", "-t", "image/png", "-o"])
            .output();

        if let Ok(out) = img_output {
            if out.status.success() && !out.stdout.is_empty() {
                return self.save_image_bytes(&out.stdout, None);
            }
        }

        // Try getting text via xclip
        let text_output = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output();

        if let Ok(out) = text_output {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout).to_string();
                if !text.trim().is_empty() {
                    return Ok(ClipboardResult {
                        kind: "text".to_string(),
                        text: Some(text),
                        image_data: None,
                        width: None,
                        height: None,
                        filename: None,
                    });
                }
            }
        }

        Ok(ClipboardResult {
            kind: "empty".to_string(),
            text: None,
            image_data: None,
            width: None,
            height: None,
            filename: None,
        })
    }

    /// Save image bytes into managed assets directory and return metadata
    pub fn save_image_bytes(&self, bytes: &[u8], original_name: Option<String>) -> Result<ClipboardResult, String> {
        let dyn_img = image::load_from_memory(bytes).map_err(|e| format!("Failed to decode image: {}", e))?;
        let width = dyn_img.width();
        let height = dyn_img.height();

        let filename = format!("{}.png", Uuid::new_v4());
        let file_path = self.assets_dir.join(&filename);

        std::fs::write(&file_path, bytes).map_err(|e| format!("Failed to write asset file: {}", e))?;

        // Also create a data URI for instant webview display without any protocol hurdles
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
        let data_uri = format!("data:image/png;base64,{}", b64);

        Ok(ClipboardResult {
            kind: "image".to_string(),
            text: None,
            image_data: Some(data_uri),
            width: Some(width),
            height: Some(height),
            filename: Some(original_name.unwrap_or(filename)),
        })
    }

    /// Copy text back to system clipboard
    pub fn write_text(&self, text: &str) -> Result<(), String> {
        let mut written = false;

        #[cfg(target_os = "linux")]
        {
            // Wayland wl-copy
            if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                let _ = child.wait();
                written = true;
            }

            // X11 xclip
            if !written {
                if let Ok(mut child) = Command::new("xclip")
                    .args(["-selection", "clipboard"])
                    .stdin(Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    let _ = child.wait();
                    written = true;
                }
            }
        }

        // arboard cross-platform (native Win32 on Windows, fallback on Linux)
        if !written {
            if let Ok(mut cb) = arboard::Clipboard::new() {
                if cb.set_text(text.to_string()).is_ok() {
                    written = true;
                }
            }
        }

        if written {
            Ok(())
        } else {
            Err("Failed to write to clipboard".to_string())
        }
    }

    /// Copy image back to system clipboard
    pub fn write_image(&self, image_data: &str) -> Result<(), String> {
        // image_data can be a filename in assets, or a data URI (data:image/...;base64,...)
        let bytes: Vec<u8> = if image_data.starts_with("data:image/") {
            if let Some(idx) = image_data.find(";base64,") {
                let b64_str = &image_data[idx + 8..];
                use base64::Engine;
                base64::engine::general_purpose::STANDARD
                    .decode(b64_str)
                    .map_err(|e| format!("Invalid base64 image data: {}", e))?
            } else {
                return Err("Unsupported data URI format".to_string());
            }
        } else {
            // Treat as asset path or relative asset filename
            let path = if Path::new(image_data).is_absolute() {
                PathBuf::from(image_data)
            } else {
                self.assets_dir.join(image_data)
            };
            std::fs::read(&path).map_err(|e| format!("Could not read image file {}: {}", path.display(), e))?
        };

        let mut written = false;

        #[cfg(target_os = "linux")]
        {
            // 1. Wayland wl-copy
            if let Ok(mut child) = Command::new("wl-copy")
                .args(["--type", "image/png"])
                .stdin(Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(&bytes);
                }
                let _ = child.wait();
                written = true;
            }

            // 2. X11 xclip
            if !written {
                if let Ok(mut child) = Command::new("xclip")
                    .args(["-selection", "clipboard", "-t", "image/png", "-i"])
                    .stdin(Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(&bytes);
                    }
                    let _ = child.wait();
                    written = true;
                }
            }
        }

        // 3. Arboard (native Win32 on Windows, fallback on Linux)
        if !written {
            if let Ok(dyn_img) = image::load_from_memory(&bytes) {
                let rgba = dyn_img.to_rgba8();
                let (w, h) = rgba.dimensions();
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let img_data = arboard::ImageData {
                        width: w as usize,
                        height: h as usize,
                        bytes: std::borrow::Cow::Owned(rgba.into_raw()),
                    };
                    if cb.set_image(img_data).is_ok() {
                        written = true;
                    }
                }
            }
        }

        if written {
            Ok(())
        } else {
            Err("Failed to copy image to clipboard".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_image_bytes() {
        let temp_dir = std::env::temp_dir().join(format!("pinboard_test_{}", uuid::Uuid::new_v4()));
        let mgr = ClipboardManager::new(temp_dir.clone());

        // Create a 1x1 red PNG using the image crate
        let img = image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]));
        let mut png_bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        use image::ImageEncoder;
        encoder.write_image(&img, 1, 1, image::ExtendedColorType::Rgba8).unwrap();

        let result = mgr.save_image_bytes(&png_bytes, Some("test_dot.png".to_string())).unwrap();
        assert_eq!(result.kind, "image");
        assert_eq!(result.width, Some(1));
        assert_eq!(result.height, Some(1));
        assert_eq!(result.filename.as_deref(), Some("test_dot.png"));
        assert!(result.image_data.unwrap().starts_with("data:image/png;base64,"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
