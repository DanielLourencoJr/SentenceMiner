# Repository Guidelines

## Project Structure
- `src-tauri/`: Rust backend (Tauri v2). Entry: `src-tauri/src/main.rs`. Lib: `src/lib.rs`.
- `ui/`: frontend HTML/CSS/JS vanilla (no frameworks). Dev server: `python3 dev_server.py` (serves `ui/` at `http://localhost:1420`).
- `SentenceMiner_Spec.md`: full spec reference for card models, API format, config schema.
- `legacy-root/`: old project kept for reference, not used by Tauri build.

## Build and Run
```bash
# Production binary
cd src-tauri && cargo tauri build

# Development with hot-reload webview (dev server on :1420 auto-starts via beforeDevCommand)
cargo tauri dev

# Global hotkey (portal) in dev: `cargo tauri dev` has no app-id, so the
# portal refuses to bind. Test the shortcut by launching the debug binary
# from the app grid (needs
# ~/.local/share/applications/com.daniel.sentenceminer.desktop installed).
# The debug binary always loads the devUrl, so keep `python3 dev_server.py`
# running from the repo root alongside the grid launch.
```

If `/tmp` is small:
```bash
export TMPDIR=/media/<disk>/tmp
export CARGO_TARGET_DIR=/media/<disk>/sentenceminer-target
```

## Test Commands
```bash
# Rust: 190 tests (unit + integration)
cd src-tauri && cargo test
cd src-tauri && cargo test --lib           # unit only

# JS: 61 tests (vitest, from repo root)
npm test
```

## Testing Quirks
- **Config/OCR tests that modify `$HOME`** use a `HOME_LOCK: Mutex<()>` to prevent env var races. If adding new tests that set `$HOME`, acquire `HOME_LOCK` and save/restore the old value.
- Config tests with `with_temp_home` helper create a temp dir and set `$HOME` to it — already behind the mutex.

## Build Quirks
- Tauri `frontendDist` = `"../ui"` (points to the `ui/` directory). `node_modules/` must **never** exist inside `ui/` or `cargo tauri build` fails.
- JS test infra (`package.json`, `vitest.config.js`, `__tests__/`) lives at project root to keep `ui/` clean.

## Configuration & Runtime Dependencies
- User config: `~/.config/sentenceminer/config.toml` (TOML, auto-created with defaults).
- AnkiConnect must run at `http://localhost:8765`.
- API defaults to Groq (`openai/gpt-oss-120b`) at `https://api.groq.com/openai/v1`.
- System deps (Ubuntu):
  ```bash
  apt install tesseract-ocr libtesseract-dev libleptonica-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
  ```

## Pull Request Checklist

Work on feature branches, merge into `main` via PR. CI (`ci.yml`) runs
fmt, clippy (`-D warnings`), Rust tests, JS tests and the Linux build —
all must be green. Before opening a PR, run locally from repo root:

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd .. && npm test
```

Plus: `node_modules/` must never exist inside `ui/`; no secrets in
commits (`api_key` lives only in `~/.config/sentenceminer/config.toml`).

## Platform Quirks
- Global hotkey (`Ctrl+Shift+S`) requires `ashpd` (xdg-desktop-portal). Works on GNOME 48+ / KDE Plasma. On GNOME 46 (Ubuntu 24.04) the hotkey is unavailable; use UI capture buttons instead.
- Gdk/pen display can crash in release mode on Wayland.
