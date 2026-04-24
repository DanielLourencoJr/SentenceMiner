use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
use ashpd::WindowIdentifier;
use futures_lite::StreamExt;
use tauri::{AppHandle, Emitter};

use crate::capture::selection::read_primary_selection;
use crate::config::Config;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureResultPayload {
    pub text: Option<String>,
    pub error: Option<String>,
}

pub async fn init_shortcuts(app_handle: AppHandle, config: &Config) -> Result<(), String> {
    let hotkey_id = "sentenceminer_capture";
    let shortcut_id = config.capture.hotkey.clone();

    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();

    if !is_wayland {
        let _ = app_handle.emit(
            "hotkey_warning",
            "Ashpd requer Wayland nativo. Executando sem atalho global (X11/XWayland).",
        );
        return Err("Not running on native Wayland".to_string());
    }

    let proxy = GlobalShortcuts::new()
        .await
        .map_err(|e| format!("Failed to create ashpd proxy: {}", e))?;

    let shortcut = NewShortcut::new(hotkey_id, "Capturar texto selecionado")
        .preferred_trigger(Some(shortcut_id.as_str()));

    let shortcuts = vec![shortcut];

    let session = proxy
        .create_session()
        .await
        .map_err(|e| format!("Failed to create session: {}", e))?;

    let window = WindowIdentifier::from_xid(0);
    proxy
        .bind_shortcuts(&session, &shortcuts, Some(&window))
        .await
        .map_err(|e| format!("Failed to bind shortcuts: {}", e))?;

    let _ = app_handle.emit("hotkey_registered", &shortcut_id);

    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        let mut activations = match proxy.receive_activated().await {
            Ok(stream) => stream,
            Err(e) => {
                let _ = app_handle_clone.emit(
                    "hotkey_error",
                    format!("Falha ao ouvir atalhos: {}", e),
                );
                return;
            }
        };

        while let Some(activation) = activations.next().await {
            if activation.shortcut_id() == hotkey_id {
                let _ = app_handle_clone.emit("hotkey_triggered", "Pressed");
                let _ = app_handle_clone.emit("capture_selection_started", ());

                let handle = app_handle_clone.clone();
                tokio::spawn(async move {
                    let text = tokio::task::spawn_blocking(|| read_primary_selection())
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
                    let _ = handle.emit("capture_selection_result", payload);
                });
            }
        }
    });

    Ok(())
}