use serde::Deserialize;
use std::path::PathBuf;

const DEFAULT_SYSTEM_PROMPT: &str = "You are a lossless text compressor for developer content. \
Output ONLY the compressed text, nothing else. \
Rules: \
1) Keep all error codes, file paths, line numbers and column numbers. \
2) Keep all variable names, type names and function names exactly. \
3) Remove decorative characters like arrows, pipes, carets and repeated dashes used only for visual formatting. \
4) Remove lines that only contain whitespace or visual separators. \
5) Keep the error message, the affected lines of code, and the location references. \
No explanations, no preamble.";

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "defaults::lm_studio_url")]
    pub lm_studio_url: String,

    #[serde(default = "defaults::model")]
    pub model: String,

    #[serde(default = "defaults::hotkey")]
    pub hotkey: String,

    #[serde(default = "defaults::auto_trigger")]
    pub auto_trigger: bool,

    #[serde(default = "defaults::auto_threshold_chars")]
    pub auto_threshold_chars: usize,

    #[serde(default = "defaults::system_prompt")]
    pub system_prompt: String,

    #[serde(default = "defaults::max_tokens")]
    pub max_tokens: u32,

    #[serde(default = "defaults::log_to_file")]
    pub log_to_file: bool,
}

mod defaults {
    use super::DEFAULT_SYSTEM_PROMPT;

    pub fn lm_studio_url() -> String {
        "http://localhost:1234".into()
    }
    pub fn model() -> String {
        "meta-llama-3-8b-instruct".into()
    }
    pub fn hotkey() -> String {
        "F14".into()
    }
    pub fn auto_trigger() -> bool {
        false
    }
    pub fn auto_threshold_chars() -> usize {
        2000
    }
    pub fn system_prompt() -> String {
        DEFAULT_SYSTEM_PROMPT.into()
    }
    pub fn max_tokens() -> u32 {
        512
    }
    pub fn log_to_file() -> bool {
        false
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            lm_studio_url: defaults::lm_studio_url(),
            model: defaults::model(),
            hotkey: defaults::hotkey(),
            auto_trigger: defaults::auto_trigger(),
            auto_threshold_chars: defaults::auto_threshold_chars(),
            system_prompt: defaults::system_prompt(),
            max_tokens: defaults::max_tokens(),
            log_to_file: defaults::log_to_file(),
        }
    }
}

impl Config {
    /// Load config.toml from the same directory as the running binary.
    /// Falls back to defaults if the file is absent.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        if !config_path.exists() {
            log::warn!(
                "config.toml not found at {:?} — using defaults",
                config_path
            );
            return Ok(Config::default());
        }
        let raw = std::fs::read_to_string(&config_path)?;
        let cfg: Config = toml::from_str(&raw)?;
        Ok(cfg)
    }

    fn config_path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("config.toml")))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_lm_studio_url() {
        assert_eq!(Config::default().lm_studio_url, "http://localhost:1234");
    }

    #[test]
    fn default_model() {
        assert_eq!(Config::default().model, "meta-llama-3-8b-instruct");
    }

    #[test]
    fn default_hotkey() {
        assert_eq!(Config::default().hotkey, "F14");
    }

    #[test]
    fn default_auto_trigger_disabled() {
        assert!(!Config::default().auto_trigger);
    }

    #[test]
    fn default_auto_threshold() {
        assert_eq!(Config::default().auto_threshold_chars, 2000);
    }

    #[test]
    fn default_max_tokens() {
        assert_eq!(Config::default().max_tokens, 512);
    }

    #[test]
    fn default_log_to_file_disabled() {
        assert!(!Config::default().log_to_file);
    }

    #[test]
    fn default_system_prompt_non_empty() {
        assert!(!Config::default().system_prompt.is_empty());
    }

    #[test]
    fn deserialize_partial_toml_fills_defaults() {
        let toml = r#"model = "my-custom-model""#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.model, "my-custom-model");
        assert_eq!(cfg.hotkey, "F14");
        assert_eq!(cfg.max_tokens, 512);
    }
}
