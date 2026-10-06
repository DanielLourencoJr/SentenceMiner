//! Known-vocabulary cache built from the user's Anki deck.
//!
//! The inference hot path never touches the network: it only reads this
//! cache file. Refreshes happen opportunistically (app startup) and fail
//! silently when Anki is closed.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::anki::client::AnkiClient;

const CACHE_DIR: &str = ".local/share/sentenceminer";
const CACHE_FILE: &str = "vocab.json";
const STALE_AFTER_SECS: u64 = 24 * 3600;
const PAGE_SIZE: usize = 200;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn home_dir() -> Result<PathBuf, String> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|e| e.to_string())
}

pub fn cache_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(CACHE_DIR).join(CACHE_FILE))
}

pub fn is_stale(updated_at: u64, now: u64) -> bool {
    now.saturating_sub(updated_at) >= STALE_AFTER_SECS
}

/// Anki search for every note in a deck. The name is always quoted so
/// decks with spaces or accents ("Inglês") keep working.
pub fn deck_query(deck: &str) -> String {
    format!("deck:\"{deck}\"")
}

/// Strip HTML tags so card markup never pollutes the vocabulary.
pub fn strip_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut inside_tag = false;
    for c in text.chars() {
        match c {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// Lowercase alphanumeric tokens for vocabulary matching.
/// Contractions are dropped whole ("don't" never becomes "don").
pub fn tokenize_words(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() || c == '\'' {
            current.push(c.to_lowercase().next().unwrap_or(c));
        } else if !current.is_empty() {
            if current.len() >= 2 && !current.contains('\'') {
                tokens.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
        }
    }
    if current.len() >= 2 && !current.contains('\'') {
        tokens.push(current);
    }
    tokens
}

fn load_from(path: &std::path::Path) -> HashSet<String> {
    let contents = std::fs::read_to_string(path).unwrap_or_default();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap_or_default();
    parsed
        .get("words")
        .and_then(|w| w.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn save_to(path: &std::path::Path, words: &HashSet<String>, updated_at: u64) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut sorted: Vec<&String> = words.iter().collect();
    sorted.sort();
    let body = serde_json::json!({ "words": sorted, "updated_at": updated_at });
    let text = serde_json::to_string(&body).map_err(|e| e.to_string())?;
    // Atomic write: readers never see a half-written cache.
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

/// Read the known-words set. Empty when the cache is missing or broken —
/// callers fall back to rarity-only scoring.
pub fn load_known_words() -> HashSet<String> {
    cache_path().map(|p| load_from(&p)).unwrap_or_default()
}

fn cache_updated_at() -> Option<u64> {
    let path = cache_path().ok()?;
    let contents = std::fs::read_to_string(path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&contents).ok()?;
    parsed.get("updated_at")?.as_u64()
}

/// Rebuild the cache from Anki unless it is fresh. Silent on success;
/// errors (Anki closed, deck missing) propagate for the caller to ignore.
pub async fn refresh_if_stale(host: &str, port: u16, deck: &str) -> Result<usize, String> {
    if let Some(updated_at) = cache_updated_at() {
        if !is_stale(updated_at, now_secs()) {
            return Ok(0);
        }
    }
    refresh(host, port, deck).await
}

pub async fn refresh(host: &str, port: u16, deck: &str) -> Result<usize, String> {
    let client = AnkiClient::new(host, port);
    let ids = client.find_notes(&deck_query(deck)).await?;
    let mut words = HashSet::new();
    for chunk in ids.chunks(PAGE_SIZE) {
        for note in client.notes_info(chunk).await? {
            for value in note.fields.values() {
                words.extend(tokenize_words(&strip_html(value)));
            }
        }
    }
    let path = cache_path()?;
    save_to(&path, &words, now_secs())?;
    Ok(words.len())
}

/// Merge just-mined texts into an existing cache. Skipped when there is
/// no cache yet, so a partial set can never poison it (the next startup
/// refresh rebuilds the full one).
pub fn remember_texts(texts: &[&str]) {
    let Ok(path) = cache_path() else {
        return;
    };
    if !path.exists() {
        return;
    }
    let mut words = load_from(&path);
    for text in texts {
        words.extend(tokenize_words(&strip_html(text)));
    }
    let updated_at = cache_updated_at().unwrap_or(0);
    let _ = save_to(&path, &words, updated_at);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_tags_but_keeps_text() {
        assert_eq!(
            strip_html("<b style=\"color: #f59e0b\">inscrutable</b> expression"),
            "inscrutable expression"
        );
        assert_eq!(strip_html("plain text"), "plain text");
        assert_eq!(strip_html("<br>"), "");
    }

    #[test]
    fn tokenizes_lowercase_words() {
        assert_eq!(
            tokenize_words("She looked, inscrutable!"),
            vec!["she", "looked", "inscrutable"]
        );
    }

    #[test]
    fn drops_contractions_whole() {
        assert_eq!(tokenize_words("don't I a"), Vec::<String>::new());
    }

    #[test]
    fn quotes_deck_names_with_spaces_and_accents() {
        assert_eq!(deck_query("Inglês"), "deck:\"Inglês\"");
        assert_eq!(deck_query("Default"), "deck:\"Default\"");
    }

    #[test]
    fn staleness_uses_24h_window() {
        assert!(is_stale(0, STALE_AFTER_SECS));
        assert!(is_stale(100, 100 + STALE_AFTER_SECS));
        assert!(!is_stale(100, 100 + STALE_AFTER_SECS - 1));
    }

    #[test]
    fn cache_round_trip_through_temp_file() {
        let dir = std::env::temp_dir().join("sentenceminer_vocab_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create dir");
        let path = dir.join("vocab.json");

        let mut words = HashSet::new();
        words.insert("inscrutable".to_string());
        save_to(&path, &words, 123).expect("save");
        assert_eq!(load_from(&path), words);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
