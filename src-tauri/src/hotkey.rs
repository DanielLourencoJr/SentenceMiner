// Atalho global (Ctrl+Shift+S) para invocar o diálogo via portal
// org.freedesktop.portal.GlobalShortcuts (ashpd).
//
// Por que portal e não o plugin de atalho do Tauri? O plugin usa a crate
// `global-hotkey`, que no Linux só funciona em X11. No Wayland o compositor
// não permite que apps capturem teclas globais: o único caminho é pedir ao
// portal, que mostra um diálogo de consentimento do sistema e entrega o
// atalho. Sem portal (ou sem consentimento), o app segue só com o tray.

use ashpd::desktop::{
    global_shortcuts::{BindShortcutsOptions, GlobalShortcuts, NewShortcut},
    CreateSessionOptions,
};
use futures_lite::StreamExt;

const SUMMON_SHORTCUT_ID: &str = "summon";
const SUMMON_TRIGGER: &str = "ctrl+shift+s";

pub fn spawn_summon_shortcut(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_hotkey_loop(app).await {
            eprintln!("SentenceMiner: atalho global indisponível ({e}); use o tray.");
        }
    });
}

async fn run_hotkey_loop(app: tauri::AppHandle) -> Result<(), ashpd::Error> {
    let proxy = GlobalShortcuts::new().await?;
    let session = proxy
        .create_session(CreateSessionOptions::default())
        .await?;

    let shortcut = NewShortcut::new(SUMMON_SHORTCUT_ID, "Invocar SentenceMiner")
        .preferred_trigger(SUMMON_TRIGGER);
    let request = proxy
        .bind_shortcuts(&session, &[shortcut], None, BindShortcutsOptions::default())
        .await?;
    let _bound = request.response()?;
    eprintln!("SentenceMiner: atalho global ativo ({SUMMON_TRIGGER}).");

    let mut activated = proxy.receive_activated().await?;
    while let Some(signal) = activated.next().await {
        if signal.shortcut_id() == SUMMON_SHORTCUT_ID {
            crate::toggle_main_window(&app);
        }
    }
    Ok(())
}
