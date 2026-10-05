// Global shortcut to summon the dialog via the
// org.freedesktop.portal.GlobalShortcuts portal (ashpd).
//
// Why the portal instead of Tauri's shortcut plugin? The plugin uses the
// `global-hotkey` crate, which on Linux only works on X11. On Wayland the
// compositor does not let apps capture global keys: the only path is asking
// the portal, which shows a system consent dialog and hands over the
// shortcut. Without a portal (or without consent), the app stays tray-only.

use ashpd::desktop::{
    global_shortcuts::{BindShortcutsOptions, GlobalShortcuts, NewShortcut},
    CreateSessionOptions,
};
use futures_lite::StreamExt;

const SUMMON_SHORTCUT_ID: &str = "summon";

pub fn spawn_summon_shortcut(app: tauri::AppHandle, trigger: String) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_hotkey_loop(app, &trigger).await {
            eprintln!("SentenceMiner: global shortcut unavailable ({e}); use the tray.");
        }
    });
}

async fn run_hotkey_loop(app: tauri::AppHandle, trigger: &str) -> Result<(), ashpd::Error> {
    let proxy = GlobalShortcuts::new().await?;
    let session = proxy
        .create_session(CreateSessionOptions::default())
        .await?;

    let shortcut =
        NewShortcut::new(SUMMON_SHORTCUT_ID, "Summon SentenceMiner").preferred_trigger(trigger);
    let request = proxy
        .bind_shortcuts(&session, &[shortcut], None, BindShortcutsOptions::default())
        .await?;
    let _bound = request.response()?;
    eprintln!("SentenceMiner: global shortcut active ({trigger}).");

    let mut activated = proxy.receive_activated().await?;
    while let Some(signal) = activated.next().await {
        if signal.shortcut_id() == SUMMON_SHORTCUT_ID {
            crate::toggle_main_window(&app);
        }
    }
    Ok(())
}
