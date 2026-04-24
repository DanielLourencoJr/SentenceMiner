# Repository Guidelines

## Project Structure & Module Organization
- `src-tauri/`: Rust backend (Tauri v2). Main entry: `src-tauri/src/main.rs`.
- `src-tauri/src/`: backend modules (`config.rs`, `capture/`, `api/`, `anki/`).
- `ui/`: frontend HTML/CSS/JS (`index.html`, `style.css`, `main.js`).
- `dev_server.py`: no-cache dev server for UI during development.
- `legacy-root/`: old root Rust project kept for reference (not used by Tauri).
- Spec reference: `SentenceMiner_Spec.md`.

## Build, Test, and Development Commands
- Run UI dev server: `python3 dev_server.py` (serves at `http://localhost:1420`)
- Run app: `cargo tauri dev` (from repo root)
- If `/tmp` is small, set:
  `export TMPDIR=/media/<disk>/tmp`
  `export CARGO_TARGET_DIR=/media/<disk>/sentenceminer-target`

## Coding Style & Naming Conventions
- Rust 2021 edition.
- Prefer explicit, simple code; avoid `unwrap()` in production paths.
- Use `Result` and propagate errors with `?` where possible.
- Frontend is vanilla HTML/CSS/JS (no frameworks).
- Naming: snake_case for Rust functions/modules; kebab-case IDs in HTML are acceptable and already used.

## Testing Guidelines
- Run tests: `cargo test` (from `src-tauri/` directory)
- Tests in: `src-tauri/tests/config_tests.rs`
- Manual checks:
  - Selection capture and OCR.
  - AnkiConnect addNote.
  - API “Gerar verso”.

## Commit & Pull Request Guidelines
- No formal conventions found in history yet.
- For future changes:
  - Use short, imperative commit messages (e.g., “Add OCR flow”).
  - PRs should describe user-visible changes and include screenshots for UI updates.

## Configuration & Runtime Notes
- User config: `~/.config/sentenceminer/config.toml`.
- AnkiConnect must be running locally (`http://localhost:8765`).
- OCR relies on Tesseract/Leptonica system libs.
- System deps (Ubuntu): `tesseract-ocr libtesseract-dev libleptonica-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev`

## Platform Quirks
- Global hotkey uses `ashpd` (xdg-desktop-portal). Requires GNOME 48+ or KDE Plasma.
- On GNOME 46 (Ubuntu 24.04): emits warning, hotkey unavailable. Use UI buttons instead.
- Gdk/pen display can crash in release mode on Wayland (see bug.md).
