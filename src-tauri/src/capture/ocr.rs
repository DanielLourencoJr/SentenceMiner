use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn ocr_last_screenshot(lang: &str) -> Result<String, String> {
    let dir = screenshots_dir()?;
    let latest = latest_image_file(&dir)
        .ok_or_else(|| "Nenhum screenshot encontrado em ~/Pictures/Screenshots".to_string())?;

    let mut lt = leptess::LepTess::new(None, lang).map_err(|e| e.to_string())?;
    if !lt.set_image(latest.to_string_lossy().as_ref()) {
        return Err("Falha ao carregar imagem para OCR.".to_string());
    }

    let text = lt.get_utf8_text().map_err(|e| e.to_string())?;
    let cleaned = text.trim().to_string();
    if cleaned.is_empty() {
        return Err("OCR nao retornou texto.".to_string());
    }
    Ok(cleaned)
}

fn screenshots_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    Ok(Path::new(&home).join("Pictures/Screenshots"))
}

fn latest_image_file(dir: &Path) -> Option<PathBuf> {
    let mut latest: Option<(SystemTime, PathBuf)> = None;
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !is_image_file(&path) {
            continue;
        }
        let meta = entry.metadata().ok()?;
        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        match &latest {
            Some((t, _)) if *t >= modified => {}
            _ => latest = Some((modified, path)),
        }
    }
    latest.map(|(_, p)| p)
}

fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(OsStr::to_str) {
        Some(ext) => matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg"),
        None => false,
    }
}
