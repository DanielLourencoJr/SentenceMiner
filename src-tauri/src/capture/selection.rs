pub fn read_primary_selection() -> Result<String, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    #[cfg(target_os = "linux")]
    {
        use arboard::{GetExtLinux, LinuxClipboardKind};
        let primary = clipboard
            .get()
            .clipboard(LinuxClipboardKind::Primary)
            .text()
            .map_err(|e| e.to_string());
        let primary = primary.unwrap_or_default();

        // Heuristica: em alguns ambientes (ex. Wayland) PRIMARY pode espelhar o CLIPBOARD.
        // Se PRIMARY == CLIPBOARD, tratamos como "sem selecao" para evitar usar clipboard.
        let clipboard_text = clipboard
            .get()
            .clipboard(LinuxClipboardKind::Clipboard)
            .text()
            .unwrap_or_default();
        if !primary.is_empty() && primary == clipboard_text {
            return Ok(String::new());
        }
        return Ok(primary);
    }

    #[cfg(not(target_os = "linux"))]
    {
        clipboard.get_text().map_err(|e| e.to_string())
    }
}
