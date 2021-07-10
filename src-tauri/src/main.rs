#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod commands;

mod compat;
mod maybemusic;
mod music;
mod process_music;
mod utils;

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler!(
      commands::get_maybemusic_by_csv_path
    ))
    .run(tauri::generate_context![])
    .expect("error while running tauri application");
}
