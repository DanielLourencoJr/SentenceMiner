mod api;
mod anki;
mod capture;
mod config;

use tauri::Manager;

#[tauri::command]
fn capture_selection() -> Result<String, String> {
    capture::selection::read_primary_selection()
}

fn main() {
    let config = match config::load_or_create() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Config error: {err}");
            config::Config::default()
        }
    };

    tauri::Builder::default()
        .manage(config)
        .invoke_handler(tauri::generate_handler![capture_selection])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| eprintln!("Tauri error: {e}"));
}
