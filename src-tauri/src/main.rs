mod anki;
mod api;
mod capture;
mod config;
mod hotkey;

use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            _ => {
                let _ = window.show();
                // After show: a visible window always has a monitor
                // (hidden ones may return None from current_monitor).
                fit_to_monitor(app, &window);
                let _ = window.set_focus();
                let _ = app.emit("summon", ());
            }
        }
    }
}

// True fullscreen + transparency are incompatible on Mutter
// (a fullscreen window leaves the compositor). Instead, we size
// a normal window to the exact monitor size: it covers everything,
// including the top bar, and stays composited (alpha works).
fn fit_to_monitor(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    if let Ok(Some(m)) = window.current_monitor() {
        let s = m.size();
        return apply_monitor_size(window, "current", s.width, s.height);
    }
    if let Ok(Some(m)) = app.primary_monitor() {
        let s = m.size();
        return apply_monitor_size(window, "primary", s.width, s.height);
    }
    if let Ok(monitors) = app.available_monitors() {
        if let Some(m) = monitors.into_iter().next() {
            let s = m.size();
            return apply_monitor_size(window, "available", s.width, s.height);
        }
    }
    eprintln!("SentenceMiner: no monitor found, keeping current size");
}

fn apply_monitor_size(
    window: &tauri::WebviewWindow,
    source: &'static str,
    width: u32,
    height: u32,
) {
    eprintln!("SentenceMiner: summon at {width}x{height} (monitor {source})");
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }));
    let _ = window.center();
}

fn load_tray_icon() -> Result<tauri::image::Image<'static>, String> {
    let bytes = include_bytes!("../icons/32x32.png");
    let rgba = image::load_from_memory(bytes)
        .map_err(|e| e.to_string())?
        .to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    Ok(tauri::image::Image::new_owned(
        rgba.into_raw(),
        width,
        height,
    ))
}

fn build_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let toggle_item = MenuItem::with_id(app, "toggle", "Show/Hide", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle_item, &quit_item])?;

    let icon = load_tray_icon()?;

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("SentenceMiner")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => toggle_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

#[tauri::command]
async fn capture_selection() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(capture::selection::read_primary_selection)
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
async fn anki_check_connection(state: tauri::State<'_, config::Config>) -> Result<u16, String> {
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
        return Err("Model has no fields.".to_string());
    }
    let first = fields
        .first()
        .cloned()
        .unwrap_or_else(|| "Front".to_string());
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

#[tauri::command]
fn dismiss(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[derive(Serialize)]
struct UiBootstrap {
    default_model: String,
    default_format_preset: String,
    default_deck: String,
    format_presets: Vec<config::FormatPreset>,
    theme: String,
}

#[tauri::command]
fn get_ui_bootstrap(state: tauri::State<'_, config::Config>) -> UiBootstrap {
    UiBootstrap {
        default_model: state.ui.default_model.clone(),
        default_format_preset: state.ui.default_format_preset.clone(),
        default_deck: state.anki.deck.clone(),
        format_presets: state.format_presets.clone(),
        theme: state.ui.theme.clone(),
    }
}

#[tauri::command]
fn set_theme(theme: String, state: tauri::State<'_, config::Config>) -> Result<(), String> {
    let mut config = state.inner().clone();
    config.ui.theme = theme;
    config::save(&config)
}

fn main() {
    let config = match config::load_or_create() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Config error: {err}");
            config::Config::default()
        }
    };

    let summon_trigger = config.capture.hotkey.clone();

    tauri::Builder::default()
        .manage(config)
        .setup(move |app| {
            if let Err(e) = build_tray(app.handle()) {
                eprintln!("Failed to create tray icon: {e}");
            }
            hotkey::spawn_summon_shortcut(app.handle().clone(), summon_trigger.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            capture_selection,
            capture_ocr_last_screenshot,
            anki_check_connection,
            anki_get_deck_names,
            anki_get_model_names,
            anki_get_model_field_names,
            anki_add_note,
            generate_back,
            get_ui_bootstrap,
            set_theme,
            dismiss
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| eprintln!("Tauri error: {e}"));
}
