mod api;
mod anki;
mod capture;
mod config;

use serde::Serialize;
use tauri::{Emitter, Manager};
use std::str::FromStr;
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};

#[tauri::command]
async fn capture_selection() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| capture::selection::read_primary_selection())
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn capture_ocr_last_screenshot(
    state: tauri::State<'_, config::Config>,
) -> Result<String, String> {
    let lang = state.capture.ocr_language.clone();
    tauri::async_runtime::spawn_blocking(move || capture::ocr::ocr_last_screenshot(&lang))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn anki_check_connection(
    state: tauri::State<'_, config::Config>,
) -> Result<u16, String> {
    let client = anki::client::AnkiClient::new(&state.anki.host, state.anki.port);
    client.check_connection().await
}

#[tauri::command]
async fn anki_get_deck_names(
    state: tauri::State<'_, config::Config>,
) -> Result<Vec<String>, String> {
    let client = anki::client::AnkiClient::new(&state.anki.host, state.anki.port);
    client.get_deck_names().await
}

#[tauri::command]
async fn anki_get_model_names(
    state: tauri::State<'_, config::Config>,
) -> Result<Vec<String>, String> {
    let client = anki::client::AnkiClient::new(&state.anki.host, state.anki.port);
    client.get_model_names().await
}

#[tauri::command]
async fn anki_get_model_field_names(
    state: tauri::State<'_, config::Config>,
    model: String,
) -> Result<Vec<String>, String> {
    let client = anki::client::AnkiClient::new(&state.anki.host, state.anki.port);
    client.get_model_field_names(&model).await
}

#[tauri::command]
async fn anki_add_note(
    state: tauri::State<'_, config::Config>,
    front: String,
    back: String,
    model: String,
    deck: Option<String>,
) -> Result<i64, String> {
    let client = anki::client::AnkiClient::new(&state.anki.host, state.anki.port);
    let fields = client.get_model_field_names(&model).await?;
    if fields.is_empty() {
        return Err("Modelo nao tem campos.".to_string());
    }
    let first = fields.get(0).cloned().unwrap_or_else(|| "Front".to_string());
    let second = fields.get(1).cloned().unwrap_or_else(|| "Back".to_string());
    let mut map = serde_json::Map::new();
    map.insert(first, serde_json::Value::String(front));
    map.insert(second, serde_json::Value::String(back));
    let deck_name = deck.unwrap_or_else(|| state.anki.deck.clone());
    client
        .add_note(&deck_name, &model, map, &state.anki.tags)
        .await
}

#[tauri::command]
async fn generate_back(
    state: tauri::State<'_, config::Config>,
    sentence: String,
    term: String,
    model: String,
) -> Result<String, String> {
    let cfg = state.inner();
    api::translation::generate_back(
        &cfg.api.base_url,
        &cfg.api.api_key,
        &cfg.api.model,
        &cfg.general.source_language,
        &cfg.general.target_language,
        &sentence,
        &term,
        &model,
        cfg.api.timeout_seconds,
    )
    .await
}

#[derive(Serialize)]
struct UiBootstrap {
    default_model: String,
    default_format_preset: String,
    default_deck: String,
    format_presets: Vec<config::FormatPreset>,
}

#[tauri::command]
fn get_ui_bootstrap(state: tauri::State<'_, config::Config>) -> UiBootstrap {
    UiBootstrap {
        default_model: state.ui.default_model.clone(),
        default_format_preset: state.ui.default_format_preset.clone(),
        default_deck: state.anki.deck.clone(),
        format_presets: state.format_presets.clone(),
    }
}

#[derive(Clone, Serialize)]
struct CaptureResultPayload {
    text: Option<String>,
    error: Option<String>,
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
        .invoke_handler(tauri::generate_handler![
            capture_selection,
            capture_ocr_last_screenshot,
            anki_check_connection,
            anki_get_deck_names,
            anki_get_model_names,
            anki_get_model_field_names,
            anki_add_note,
            generate_back,
            get_ui_bootstrap
        ])
        .setup(|app| {
            let hotkey = app.state::<config::Config>().capture.hotkey.clone();
            let shortcut = Shortcut::from_str(&hotkey).map_err(|e| e.to_string())?;
            let handle = app.handle().clone();
            if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
                if session.to_lowercase() == "wayland" {
                    let _ = handle.emit(
                        "hotkey_warning",
                        "Atalhos globais podem nao funcionar no Wayland. Tente uma sessao X11.",
                    );
                }
            }
            let plugin = tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts([shortcut])?
                .with_handler(|app_handle, _shortcut, event| {
                    let _ = app_handle.emit("hotkey_triggered", format!("{:?}", event.state));
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    let _ = app_handle.emit("hotkey_triggered", "Pressed");
                    let _ = app_handle.emit("capture_selection_started", ());
                    let app_handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let text = tauri::async_runtime::spawn_blocking(|| {
                            capture::selection::read_primary_selection()
                        })
                        .await
                        .map_err(|e| e.to_string())
                        .and_then(|r| r);

                        let payload = match text {
                            Ok(t) if !t.trim().is_empty() => CaptureResultPayload {
                                text: Some(t),
                                error: None,
                            },
                            Ok(_) => CaptureResultPayload {
                                text: None,
                                error: Some("Nenhuma selecao detectada.".to_string()),
                            },
                            Err(err) => CaptureResultPayload {
                                text: None,
                                error: Some(err),
                            },
                        };
                        let _ = app_handle.emit("capture_selection_result", payload);
                    });
                })
                .build();
            handle.plugin(plugin)?;
            let _ = handle.emit("hotkey_registered", hotkey);
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| eprintln!("Tauri error: {e}"));
}
