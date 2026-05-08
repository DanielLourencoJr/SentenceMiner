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

#[cfg(test)]
mod tests {
    use super::{is_image_file, latest_image_file, screenshots_dir};
    use std::path::Path;
    use std::sync::Mutex;

    static HOME_LOCK: Mutex<()> = Mutex::new(());

    // ─── is_image_file ────────────────────────────────────────────────

    #[test]
    fn detects_png_extension() {
        assert!(is_image_file(Path::new("screenshot.png")));
    }

    #[test]
    fn detects_uppercase_png() {
        assert!(is_image_file(Path::new("screenshot.PNG")));
    }

    #[test]
    fn detects_jpg_extension() {
        assert!(is_image_file(Path::new("photo.jpg")));
    }

    #[test]
    fn detects_uppercase_jpg() {
        assert!(is_image_file(Path::new("photo.JPG")));
    }

    #[test]
    fn detects_jpeg_extension() {
        assert!(is_image_file(Path::new("image.jpeg")));
    }

    #[test]
    fn detects_uppercase_jpeg() {
        assert!(is_image_file(Path::new("image.JPEG")));
    }

    #[test]
    fn rejects_txt_extension() {
        assert!(!is_image_file(Path::new("readme.txt")));
    }

    #[test]
    fn rejects_path_without_extension() {
        assert!(!is_image_file(Path::new("README")));
    }

    #[test]
    fn rejects_empty_path() {
        assert!(!is_image_file(Path::new("")));
    }

    #[test]
    fn detects_mixed_case_extension() {
        assert!(is_image_file(Path::new("image.PnG")));
    }

    // ─── screenshots_dir ──────────────────────────────────────────────

    #[test]
    fn screenshots_dir_uses_home() {
        let _guard = HOME_LOCK.lock().unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", "/tmp/test-home");
        let dir = screenshots_dir().expect("screenshots dir");
        assert_eq!(dir, Path::new("/tmp/test-home/Pictures/Screenshots"));
        match old_home {
            Some(val) => std::env::set_var("HOME", val),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn screenshots_dir_fails_without_home() {
        let _guard = HOME_LOCK.lock().unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::remove_var("HOME");
        let err = screenshots_dir().expect_err("should fail");
        assert!(err.contains("not found"));
        match old_home {
            Some(val) => std::env::set_var("HOME", val),
            None => std::env::remove_var("HOME"),
        }
    }

    // ─── latest_image_file ────────────────────────────────────────────

    #[test]
    fn latest_image_file_returns_most_recent_image() {
        let dir = std::env::temp_dir().join("test_screenshots");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create dir");

        // Create files with different extensions
        std::fs::write(dir.join("old.txt"), "text").ok();
        std::fs::write(dir.join("newer.png"), "png").ok();

        // Sleep to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
        std::fs::write(dir.join("latest.jpg"), "jpg").ok();

        let result = latest_image_file(&dir);
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.ends_with("latest.jpg"), "got: {:?}", path);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn latest_image_file_returns_none_for_empty_dir() {
        let dir = std::env::temp_dir().join("test_empty_screenshots");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create dir");

        let result = latest_image_file(&dir);
        assert!(result.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn latest_image_file_skips_non_image_files() {
        let dir = std::env::temp_dir().join("test_mixed_screenshots");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create dir");

        std::fs::write(dir.join("readme.txt"), "text").ok();
        std::fs::write(dir.join("data.csv"), "csv").ok();

        let result = latest_image_file(&dir);
        assert!(result.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn latest_image_file_returns_none_for_nonexistent_dir() {
        let dir = Path::new("/tmp/nonexistent_test_dir_12345");
        let result = latest_image_file(dir);
        assert!(result.is_none());
    }
}
