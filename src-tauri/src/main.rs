#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

mod commands;

mod maybemusic;
mod music;
mod process_music;
mod utils;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context![])
        .expect("error while running tauri application");
}