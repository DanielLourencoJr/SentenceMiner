# SentenceMiner

SentenceMiner is a Tauri desktop app that captures a sentence, generates a flashcard back with an OpenAI-compatible API, and adds the note to Anki via AnkiConnect.

The current focus is reducing friction in the manual sentence-mining flow. The program does not try to automate everything from the original specification; it implements a simpler flow that already solves most of the problem.

## What The Program Does Today

The current flow is:

1. Summon a spotlight-style dialog with the global hotkey (`Super+J`, configurable) or the tray icon.
2. The dialog auto-captures the selected text using the Linux/X11 PRIMARY selection.
3. Step 1 shows the captured sentence for review (multiline supported).
4. Step 2 asks for the unknown term, with a live preview of the card front using the configured format preset.
5. Step 3 generates the card back via API in three modes:
   - `beginner`
   - `intermediate`
   - `advanced`
6. The last Enter sends the note to Anki using the default deck and note type.

In other words: today the app is a capture + generate + send-to-Anki assistant, with manual term entry (term inference is planned).

Each step advances with `Ctrl+Enter` (or its action button); `Esc` cancels. The rest of the screen is dimmed while the dialog is open.

## What It Does Not Do Today

These ideas may appear in older notes, but they do not reflect current code behavior:

- clickable tokenization of the sentence;
- term selection by clicking tokens;
- edit mode with retokenization;
- live region-based OCR capture;
- automatic OCR fallback when there is no selection;
- known-vocabulary management.

## Architecture

### Backend

The backend lives in `src-tauri/` and exposes Tauri commands to:

- capture text from the PRIMARY selection;
- OCR the latest screenshot;
- summon/hide the dialog and fit it to the monitor;
- register the global hotkey via the portal;
- list Anki decks and note types;
- generate the card back over HTTP;
- add the note to Anki;
- provide presets and defaults to the UI.

Main modules:

- `src-tauri/src/main.rs`: command registration, tray icon, summon window.
- `src-tauri/src/hotkey.rs`: global shortcut via `org.freedesktop.portal.GlobalShortcuts`.
- `src-tauri/src/config.rs`: reads/writes `~/.config/sentenceminer/config.toml`.
- `src-tauri/src/capture/selection.rs`: primary selection capture via `arboard`.
- `src-tauri/src/capture/ocr.rs`: OCR of the latest screenshot with `leptess`.
- `src-tauri/src/api/translation.rs`: `/chat/completions` call.
- `src-tauri/src/anki/client.rs`: AnkiConnect integration.

### Frontend

The frontend lives in `ui/` and is vanilla HTML/CSS/JS. It is embedded into the binary (custom-protocol, no dev server).

The summon dialog has:

- one field per step (sentence, term, back);
- live front preview with the format preset applied;
- per-step action button;
- status line with actionable error messages.

## Current Usage Flow

1. The user selects a sentence in another app and presses the global hotkey (or tray → Show).
2. The sentence is already in step 1 for review.
3. The user types the unknown term in step 2 (front preview updates live).
4. The generated text appears in step 3, still editable.
5. The last Enter sends the note to the default deck with the first available note type.

When sending, the frontend applies the configured HTML preset to the first literal occurrence of the term in the sentence.

## Generation Models

The backend builds different prompts per selected model:

- `beginner`: natural translation of the sentence plus the term equivalent in Portuguese.
- `intermediate`: short definition in Portuguese plus up to three English synonyms.
- `advanced`: short definition in the source language.

The text returned by the API goes straight into the `Back` field.

## Anki Integration

The app uses AnkiConnect at `http://localhost:8765`.

Today it:

- checks version/connection;
- lists decks;
- lists note types;
- lists the selected model's fields;
- sends the note filling the model's first two fields with `front` and `back`.

If no deck is configured, it uses the default deck from the config file.

## Configuration

On first run the app creates:

`~/.config/sentenceminer/config.toml`

The file contains:

- source and target languages;
- Anki host, port, deck and tags;
- API base URL, key, model and timeout;
- global hotkey (`[capture].hotkey`, GTK accelerator notation, e.g. `"<Super>j"`);
- OCR language;
- default UI model;
- default format preset;
- HTML preset list.

Current default presets:

- `bold`
- `orange`
- `underline`

## Development

### Requirements

- Rust 2021
- Tauri v2
- Anki with AnkiConnect
- Tesseract and Leptonica
- Linux environment with working PRIMARY selection

Expected system dependencies on Ubuntu:

```bash
sudo apt install \
  tesseract-ocr \
  tesseract-ocr-eng \
  libtesseract-dev \
  libleptonica-dev \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev
```

On Arch (see AGENTS.md for the exact list), the equivalents come from pacman, plus a tray host (e.g. GNOME AppIndicator extension) and the GlobalShortcuts portal for the hotkey.

### Running In Development

One command (the UI is embedded in the binary, no server):

```bash
cd src-tauri
cargo tauri dev
```

If `/tmp` is small:

```bash
export TMPDIR=/media/<disk>/tmp
export CARGO_TARGET_DIR=/media/<disk>/sentenceminer-target
```

### Testing The Global Hotkey In Dev

`cargo tauri dev` has no app-id, so the portal refuses to bind. Launch the debug binary from the app grid (needs `~/.local/share/applications/com.daniel.sentenceminer.desktop` installed) and watch for the system consent dialog.

## Limits And Notes

- The global hotkey needs the `GlobalShortcuts` portal (GNOME 48+/KDE). Without it, use the tray.
- Current text capture depends on the PRIMARY selection; it does not use the regular clipboard as fallback.
- Current OCR does not capture a screen region live. It only reads the latest screenshot from `~/Pictures/Screenshots`.
- Front term formatting depends on simple literal matching with `indexOf`, using the first occurrence found.
## Repository Structure

- `src-tauri/`: Rust/Tauri backend.
- `ui/`: vanilla HTML/CSS/JS frontend.
- `__tests__/`: JS tests (vitest, run from the repo root).
- `legacy-root/`: old project kept for reference only.
