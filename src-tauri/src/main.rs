// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Prefer Wayland backend when running under a Wayland compositor (e.g. Hyprland)
    // to prevent blurry XWayland fractional-scaling rasterization.
    if std::env::var("WAYLAND_DISPLAY").is_ok() && std::env::var("GDK_BACKEND").is_err() {
        std::env::set_var("GDK_BACKEND", "wayland,x11");
    }
    pinboard_lib::run();
}
