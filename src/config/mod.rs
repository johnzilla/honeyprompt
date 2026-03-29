use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// User-configurable settings for a honeyprompt project.
///
/// GEN-02 compliance note: The human-visible warning text (fixed top banner and inline notice)
/// is hard-coded in the HTML template and is NOT configurable via this struct.
/// No `warning`, `show_warning`, `warning_text`, or similar fields exist here by design.
/// Hiding or disabling the warning for humans would violate the project's ethical model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Base URL for callback notifications (e.g., "http://localhost:8080")
    pub callback_base_url: String,

    /// Address and port to bind the callback server to (e.g., "0.0.0.0:8080")
    pub bind_address: String,

    /// Which payload tiers to include in the generated honeypot (e.g., [1, 2, 3])
    pub tiers: Vec<u8>,

    /// Title shown in the honeypot page header
    pub page_title: String,

    /// Visual theme for the generated page
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            callback_base_url: "http://localhost:8080".to_string(),
            bind_address: "0.0.0.0:8080".to_string(),
            tiers: vec![1, 2, 3],
            page_title: "Security Research Canary".to_string(),
            theme: "default".to_string(),
        }
    }
}

/// Read and parse a `honeyprompt.toml` config file from the given path.
pub fn load_config(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

/// Write the default config as pretty TOML to the given path.
pub fn write_default_config(path: &Path) -> Result<()> {
    let config = Config::default();
    let toml_string = toml::to_string_pretty(&config)?;
    std::fs::write(path, toml_string)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_roundtrip() {
        let dir = tempfile::tempdir().expect("could not create temp dir");
        let config_path = dir.path().join("honeyprompt.toml");

        write_default_config(&config_path).expect("write_default_config failed");

        let loaded = load_config(&config_path).expect("load_config failed");
        let default = Config::default();

        assert_eq!(loaded.callback_base_url, default.callback_base_url);
        assert_eq!(loaded.bind_address, default.bind_address);
        assert_eq!(loaded.tiers, default.tiers);
        assert_eq!(loaded.page_title, default.page_title);
        assert_eq!(loaded.theme, default.theme);
    }

    /// GEN-02 compliance: verify no warning-related fields exist on Config.
    /// The human warning is hard-coded in the template, never in the config struct.
    #[test]
    fn test_config_no_warning_field() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).expect("serialize failed");
        assert!(
            !toml_str.contains("warning"),
            "Config must not contain a 'warning' field (GEN-02 compliance)"
        );
        assert!(
            !toml_str.contains("show_warning"),
            "Config must not contain a 'show_warning' field (GEN-02 compliance)"
        );
    }
}
