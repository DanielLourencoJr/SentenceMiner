use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub anki: AnkiConfig,
    pub api: ApiConfig,
    pub capture: CaptureConfig,
    pub ui: UiConfig,
    #[serde(default)]
    pub format_presets: Vec<FormatPreset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub source_language: String,
    pub target_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnkiConfig {
    pub host: String,
    pub port: u16,
    pub deck: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub hotkey: String,
    pub ocr_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub default_model: String,
    pub default_format_preset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatPreset {
    pub name: String,
    pub template: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                source_language: "English".to_string(),
                target_language: "Brazilian Portuguese".to_string(),
            },
            anki: AnkiConfig {
                host: "localhost".to_string(),
                port: 8765,
                deck: "Default".to_string(),
                tags: vec!["sentenceminer".to_string()],
            },
            api: ApiConfig {
                base_url: "https://api.groq.com/openai/v1".to_string(),
                api_key: "".to_string(),
                model: "llama3-70b-8192".to_string(),
                timeout_seconds: 15,
            },
            capture: CaptureConfig {
                hotkey: "ctrl+shift+s".to_string(),
                ocr_language: "eng".to_string(),
            },
            ui: UiConfig {
                default_model: "intermediario".to_string(),
                default_format_preset: "negrito".to_string(),
            },
            format_presets: vec![
                FormatPreset {
                    name: "negrito".to_string(),
                    template: "<b>{term}</b>".to_string(),
                },
                FormatPreset {
                    name: "laranja".to_string(),
                    template: "<span style=\"color: #e05c00\">{term}</span>".to_string(),
                },
                FormatPreset {
                    name: "sublinhado".to_string(),
                    template: "<u>{term}</u>".to_string(),
                },
            ],
        }
    }
}

pub fn load_or_create() -> Result<Config, String> {
    let path = config_path()?;
    if path.exists() {
        let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let config: Config = toml::from_str(&contents).map_err(|e| e.to_string())?;
        Ok(config)
    } else {
        let config = Config::default();
        write_config(&path, &config)?;
        Ok(config)
    }
}

pub fn save(config: &Config) -> Result<(), String> {
    let path = config_path()?;
    write_config(&path, config)
}

fn write_config(path: &Path, config: &Config) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let contents = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, contents).map_err(|e| e.to_string())?;
    Ok(())
}

fn config_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    Ok(PathBuf::from(home).join(".config/sentenceminer/config.toml"))
}
