#[cfg(test)]
mod config_tests {
    use app_lib::config::{self, Config};
    use std::fs;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static HOME_LOCK: Mutex<()> = Mutex::new(());

    // ─── Default value tests ──────────────────────────────────────────

    #[test]
    fn default_source_language_is_english() {
        let config = Config::default();
        assert_eq!(config.general.source_language, "English");
    }

    #[test]
    fn default_target_language_is_brazilian_portuguese() {
        let config = Config::default();
        assert_eq!(config.general.target_language, "Brazilian Portuguese");
    }

    #[test]
    fn default_api_base_url_is_groq() {
        let config = Config::default();
        assert_eq!(config.api.base_url, "https://api.groq.com/openai/v1");
    }

    #[test]
    fn default_api_model_is_llama3_70b() {
        let config = Config::default();
        assert_eq!(config.api.model, "openai/gpt-oss-120b");
    }

    #[test]
    fn default_api_timeout_is_15_seconds() {
        let config = Config::default();
        assert_eq!(config.api.timeout_seconds, 15);
    }

    #[test]
    fn default_hotkey_is_super_j() {
        let config = Config::default();
        assert_eq!(config.capture.hotkey, "<Super>j");
    }

    #[test]
    fn legacy_config_without_hotkey_gets_default() {
        let toml = r#"
[general]
source_language = "English"
target_language = "Brazilian Portuguese"

[anki]
host = "localhost"
port = 8765
deck = "Default"

[api]
base_url = "https://api.groq.com/openai/v1"
api_key = ""
model = "openai/gpt-oss-120b"
timeout_seconds = 15

[capture]
ocr_language = "eng"

[ui]
default_model = "intermediate"
default_format_preset = "bold"
theme = "light"
"#;
        let config: Config = toml::from_str(toml).expect("legacy config parses");
        assert_eq!(config.capture.hotkey, "<Super>j");
    }

    #[test]
    fn default_anki_host_is_localhost() {
        let config = Config::default();
        assert_eq!(config.anki.host, "localhost");
    }

    #[test]
    fn default_anki_port_is_8765() {
        let config = Config::default();
        assert_eq!(config.anki.port, 8765);
    }

    #[test]
    fn default_anki_deck_is_default() {
        let config = Config::default();
        assert_eq!(config.anki.deck, "Default");
    }

    #[test]
    fn default_anki_tags_includes_sentenceminer() {
        let config = Config::default();
        assert!(config.anki.tags.contains(&"sentenceminer".to_string()));
    }

    #[test]
    fn default_ui_model_is_intermediate() {
        let config = Config::default();
        assert_eq!(config.ui.default_model, "intermediate");
    }

    #[test]
    fn default_ui_preset_is_bold() {
        let config = Config::default();
        assert_eq!(config.ui.default_format_preset, "bold");
    }

    #[test]
    fn default_format_presets_has_three_presets() {
        let config = Config::default();
        assert_eq!(config.format_presets.len(), 3);
    }

    #[test]
    fn default_format_preset_bold_contains_term_placeholder() {
        let config = Config::default();
        let preset = config.format_presets.iter().find(|p| p.name == "bold");
        assert!(preset.is_some());
        assert!(preset.unwrap().template.contains("{term}"));
    }

    #[test]
    fn default_format_preset_orange_has_orange_color() {
        let config = Config::default();
        let preset = config.format_presets.iter().find(|p| p.name == "orange");
        assert!(preset.is_some());
        assert!(preset.unwrap().template.contains("color: #f59e0b"));
    }

    #[test]
    fn default_format_preset_underline_uses_u_tag() {
        let config = Config::default();
        let preset = config.format_presets.iter().find(|p| p.name == "underline");
        assert!(preset.is_some());
        assert!(preset.unwrap().template.contains("<u>"));
    }

    #[test]
    fn config_roundtrip_preserves_all_fields() {
        let original = Config::default();
        let serialized = toml::to_string_pretty(&original).expect("serialize");
        let deserialized: Config = toml::from_str(&serialized).expect("deserialize");

        assert_eq!(
            original.general.source_language,
            deserialized.general.source_language
        );
        assert_eq!(original.anki.host, deserialized.anki.host);
        assert_eq!(original.anki.port, deserialized.anki.port);
        assert_eq!(original.ui.default_model, deserialized.ui.default_model);
    }

    #[test]
    fn api_config_has_valid_timeout() {
        let config = Config::default();
        assert!(config.api.timeout_seconds > 0);
        assert!(config.api.timeout_seconds <= 300);
    }

    #[test]
    fn anki_port_is_valid() {
        let config = Config::default();
        assert!(config.anki.port > 0);
        assert_eq!(config.anki.port, 8765);
    }

    // ─── Filesystem integration tests ─────────────────────────────────
    // These tests modify the HOME env var and must run sequentially.
    // Call with: cargo test --test config_tests -- --test-threads=1

    fn with_temp_home<F>(f: F)
    where
        F: FnOnce(&std::path::Path),
    {
        let _guard = HOME_LOCK.lock().unwrap();
        let dir = tempdir().expect("create temp dir");
        let home = dir.path().to_str().expect("valid utf-8").to_string();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", &home);
        f(dir.path());
        match old_home {
            Some(val) => std::env::set_var("HOME", val),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn load_or_create_creates_default_config_when_missing() {
        with_temp_home(|_| {
            let config = config::load_or_create().expect("load or create");
            assert_eq!(config.general.source_language, "English");

            let config_path = std::env::var("HOME").unwrap() + "/.config/sentenceminer/config.toml";
            assert!(
                std::path::Path::new(&config_path).exists(),
                "config file should be created"
            );
        });
    }

    #[test]
    fn load_or_create_loads_existing_config() {
        with_temp_home(|dir| {
            let config_path = dir.join(".config/sentenceminer/config.toml");
            fs::create_dir_all(config_path.parent().unwrap()).expect("create dirs");

            let custom_toml = r#"
[general]
source_language = "Japanese"
target_language = "Brazilian Portuguese"

[anki]
host = "localhost"
port = 8765
deck = "JLPT"

[api]
base_url = "https://custom.api.com"
api_key = "test-key"
model = "custom-model"
timeout_seconds = 30

[capture]
ocr_language = "jpn"

[ui]
default_model = "advanced"
default_format_preset = "orange"
theme = "dark"
"#;
            fs::write(&config_path, custom_toml).expect("write config");

            let config = config::load_or_create().expect("load");
            assert_eq!(config.general.source_language, "Japanese");
            assert_eq!(config.anki.deck, "JLPT");
            assert_eq!(config.api.base_url, "https://custom.api.com");
            assert_eq!(config.api.api_key, "test-key");
            assert_eq!(config.api.model, "custom-model");
            assert_eq!(config.api.timeout_seconds, 30);
            assert_eq!(config.capture.ocr_language, "jpn");
            assert_eq!(config.ui.default_model, "advanced");
            assert_eq!(config.ui.default_format_preset, "orange");
            assert_eq!(config.ui.theme, "dark");
        });
    }

    #[test]
    fn load_or_create_returns_default_when_no_config_file() {
        with_temp_home(|_| {
            let config = config::load_or_create().expect("load or create");
            assert_eq!(config.general.source_language, "English");
            assert_eq!(config.ui.default_model, "intermediate");
        });
    }

    #[test]
    fn save_writes_config_and_loads_back() {
        with_temp_home(|_| {
            let original = Config::default();
            config::save(&original).expect("save");

            let loaded = config::load_or_create().expect("load");
            assert_eq!(
                loaded.general.source_language,
                original.general.source_language
            );
            assert_eq!(loaded.api.base_url, original.api.base_url);
            assert_eq!(loaded.anki.deck, original.anki.deck);
        });
    }

    #[test]
    fn save_preserves_custom_values_round_trip() {
        with_temp_home(|_| {
            let mut config = Config::default();
            config.anki.deck = "MyCustomDeck".to_string();
            config.ui.theme = "dark".to_string();
            config::save(&config).expect("save");

            let loaded = config::load_or_create().expect("load");
            assert_eq!(loaded.anki.deck, "MyCustomDeck");
            assert_eq!(loaded.ui.theme, "dark");
        });
    }
}
