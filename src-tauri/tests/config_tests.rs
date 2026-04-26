#[cfg(test)]
mod config_tests {
    use app_lib::config::Config;

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
    fn config_roundtrip_preserves_all_fields() {
        let original = Config::default();
        let serialized = toml::to_string_pretty(&original).expect("serialize");
        let deserialized: Config = toml::from_str(&serialized).expect("deserialize");
        
        assert_eq!(original.general.source_language, deserialized.general.source_language);
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
        assert!(config.anki.port <= 65535);
    }
}