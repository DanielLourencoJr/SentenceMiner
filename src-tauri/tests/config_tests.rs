#[cfg(test)]
mod config_tests {
    use app_lib::config::Config;

    #[test]
    fn default_hotkey_is_ctrl_shift_s() {
        let config = Config::default();
        assert_eq!(config.capture.hotkey, "ctrl+shift+s");
    }

    #[test]
    fn default_ocr_language_is_eng() {
        let config = Config::default();
        assert_eq!(config.capture.ocr_language, "eng");
    }

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
        assert_eq!(config.api.model, "llama3-70b-8192");
    }

    #[test]
    fn default_api_timeout_is_15_seconds() {
        let config = Config::default();
        assert_eq!(config.api.timeout_seconds, 15);
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
    fn default_ui_model_is_intermediario() {
        let config = Config::default();
        assert_eq!(config.ui.default_model, "intermediario");
    }

    #[test]
    fn default_ui_preset_is_negrito() {
        let config = Config::default();
        assert_eq!(config.ui.default_format_preset, "negrito");
    }

    #[test]
    fn default_format_presets_has_three_presets() {
        let config = Config::default();
        assert_eq!(config.format_presets.len(), 3);
    }

    #[test]
    fn default_format_preset_negrito_contains_term_placeholder() {
        let config = Config::default();
        let preset = config.format_presets.iter().find(|p| p.name == "negrito");
        assert!(preset.is_some());
        assert!(preset.unwrap().template.contains("{term}"));
    }

    #[test]
    fn default_format_preset_laranja_has_orange_color() {
        let config = Config::default();
        let preset = config.format_presets.iter().find(|p| p.name == "laranja");
        assert!(preset.is_some());
        assert!(preset.unwrap().template.contains("color: #f59e0b"));
    }

    #[test]
    fn default_format_preset_sublinhado_uses_u_tag() {
        let config = Config::default();
        let preset = config.format_presets.iter().find(|p| p.name == "sublinhado");
        assert!(preset.is_some());
        assert!(preset.unwrap().template.contains("<u>"));
    }

    #[test]
    fn config_serialize_with_hotkey() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).expect("should serialize");
        assert!(toml_str.contains("hotkey"));
    }

    #[test]
    fn config_deserialize_with_custom_hotkey() {
        let toml_str = r#"
[general]
source_language = "English"
target_language = "Portuguese"

[anki]
host = "192.168.1.100"
port = 8765
deck = "MyDeck"
tags = ["test"]

[api]
base_url = "https://api.example.com"
api_key = "secret"
model = "gpt-4"
timeout_seconds = 30

[capture]
hotkey = "ctrl+shift+x"
ocr_language = "por"

[ui]
default_model = "avancado"
default_format_preset = "laranja"

[[format_presets]]
name = "custom"
template = "<mark>{term}</mark>"
"#;
        let config: Config = toml::from_str(toml_str).expect("should deserialize");
        assert_eq!(config.capture.hotkey, "ctrl+shift+x");
        assert_eq!(config.capture.ocr_language, "por");
        assert_eq!(config.ui.default_model, "avancado");
    }

    #[test]
    fn config_deserialize_with_empty_hotkey_uses_default() {
        let toml_str = r#"
[general]
source_language = "English"
target_language = "Portuguese"

[anki]
host = "localhost"
port = 8765
deck = "Default"
tags = []

[api]
base_url = "https://api.example.com"
api_key = ""
model = "model"
timeout_seconds = 10

[capture]
hotkey = ""
ocr_language = "eng"

[ui]
default_model = "iniciante"
default_format_preset = "negrito"

[[format_presets]]
name = "test"
template = "{term}"
"#;
        let config: Config = toml::from_str(toml_str).expect("should deserialize");
        assert_eq!(config.capture.hotkey, "");
    }

    #[test]
    fn config_roundtrip_preserves_all_fields() {
        let original = Config::default();
        let serialized = toml::to_string_pretty(&original).expect("serialize");
        let deserialized: Config = toml::from_str(&serialized).expect("deserialize");
        
        assert_eq!(original.capture.hotkey, deserialized.capture.hotkey);
        assert_eq!(original.capture.ocr_language, deserialized.capture.ocr_language);
        assert_eq!(original.general.source_language, deserialized.general.source_language);
        assert_eq!(original.anki.host, deserialized.anki.host);
        assert_eq!(original.anki.port, deserialized.anki.port);
    }

    #[test]
    fn capture_config_implements_default() {
        let config = Config::default();
        assert!(!config.capture.hotkey.is_empty());
        assert!(!config.capture.ocr_language.is_empty());
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
        assert!(config.anki.port <= 65535);
    }
}