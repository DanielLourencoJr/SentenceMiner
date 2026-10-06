use std::collections::HashSet;
use std::sync::Mutex;
use tempfile::tempdir;
use wiremock::{
    matchers::{body_string_contains, method, path},
    Mock, MockServer, ResponseTemplate,
};

use app_lib::infer::vocab;

static HOME_LOCK: Mutex<()> = Mutex::new(());

struct TempHome {
    _dir: tempfile::TempDir,
    _guard: std::sync::MutexGuard<'static, ()>,
    old_home: Option<String>,
}

impl TempHome {
    fn set() -> Self {
        let guard = HOME_LOCK.lock().unwrap();
        let dir = tempdir().expect("create temp dir");
        let home = dir.path().to_str().expect("valid utf-8").to_string();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home);
        Self {
            _dir: dir,
            _guard: guard,
            old_home,
        }
    }
}

impl Drop for TempHome {
    fn drop(&mut self) {
        match self.old_home.take() {
            Some(val) => std::env::set_var("HOME", val),
            None => std::env::remove_var("HOME"),
        }
    }
}

#[tokio::test]
async fn refresh_builds_vocab_from_deck_notes() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("findNotes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": [11, 22],
            "error": null
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("notesInfo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": [
                {
                    "noteId": 11,
                    "modelName": "Basic",
                    "tags": ["sentenceminer"],
                    "fields": {
                        "Front": {"value": "She looked <b>inscrutable</b>", "order": 0},
                        "Back": {"value": "She looked. inscrutable", "order": 1}
                    }
                },
                {
                    "noteId": 22,
                    "modelName": "Basic",
                    "tags": [],
                    "fields": {
                        "Front": {"value": "Breathtaking views", "order": 0},
                        "Back": {"value": "Something", "order": 1}
                    }
                }
            ],
            "error": null
        })))
        .mount(&mock_server)
        .await;

    let _home = TempHome::set();
    let count = vocab::refresh("localhost", mock_server.address().port(), "Default")
        .await
        .expect("refresh works");
    assert!(count > 0, "expected words, got {count}");

    let known = vocab::load_known_words();
    let expected: HashSet<String> = [
        "she",
        "looked",
        "inscrutable",
        "breathtaking",
        "views",
        "something",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert!(
        expected.iter().all(|w| known.contains(w)),
        "missing words in {known:?}"
    );
    // Markup internals must not leak into the vocabulary.
    assert!(!known.contains("b"));
    assert!(!known.contains("color"));
}

#[tokio::test]
async fn refresh_fails_cleanly_without_anki() {
    // Port 1 refuses connections: must be an Err, never a panic.
    let _home = TempHome::set();
    let result = vocab::refresh("localhost", 1, "Default").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn remember_texts_merges_into_existing_cache() {
    let _home = TempHome::set();
    // No cache yet: skipped, never poisons.
    vocab::remember_texts(&["Floccinaucinihilipilification everywhere"]);
    assert!(vocab::load_known_words().is_empty());

    // Seed a cache, then merge.
    std::fs::create_dir_all(vocab::cache_path().expect("path").parent().expect("parent"))
        .expect("mkdir");
    std::fs::write(
        vocab::cache_path().expect("path"),
        r#"{"words": ["known"], "updated_at": 0}"#,
    )
    .expect("seed");
    vocab::remember_texts(&["Brand new WORDS here"]);
    let known = vocab::load_known_words();
    assert!(known.contains("known"));
    assert!(known.contains("brand"));
    assert!(known.contains("words"));
}
