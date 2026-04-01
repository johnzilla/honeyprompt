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

/// Apply CLI flag overrides to a base config.
///
/// Precedence: flag value (Some) > base config value (when flag is None).
/// When domain is provided, sets callback_base_url to `https://{domain}`,
/// defaults bind_address to `0.0.0.0:8080`, and defaults tiers to `[1, 2, 3]`.
/// Explicit `bind` or `tiers` flags are applied after domain defaults, taking
/// highest precedence (SERVE-03).
pub fn config_with_overrides(
    base: &Config,
    domain: Option<&str>,
    bind: Option<&str>,
    tiers: Option<Vec<u8>>,
) -> Config {
    let mut cfg = base.clone();

    if let Some(d) = domain {
        cfg.callback_base_url = format!("https://{}", d);
        // Domain implies these defaults unless explicitly overridden below
        cfg.bind_address = "0.0.0.0:8080".to_string();
        cfg.tiers = vec![1, 2, 3];
    }

    // Explicit flag overrides (applied after domain defaults — highest precedence)
    if let Some(b) = bind {
        cfg.bind_address = b.to_string();
    }
    if let Some(t) = tiers {
        cfg.tiers = t;
    }

    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- config_with_overrides precedence tests (SERVE-02, SERVE-03) ----

    #[test]
    fn test_config_with_overrides_domain_sets_url_bind_tiers() {
        let base = Config::default();
        let cfg = config_with_overrides(&base, Some("example.com"), None, None);
        assert_eq!(cfg.callback_base_url, "https://example.com");
        assert_eq!(cfg.bind_address, "0.0.0.0:8080");
        assert_eq!(cfg.tiers, vec![1, 2, 3]);
    }

    #[test]
    fn test_config_with_overrides_flags_override_domain_defaults() {
        let base = Config::default();
        let cfg = config_with_overrides(
            &base,
            Some("example.com"),
            Some("127.0.0.1:9090"),
            Some(vec![1, 2]),
        );
        assert_eq!(cfg.callback_base_url, "https://example.com");
        assert_eq!(cfg.bind_address, "127.0.0.1:9090");
        assert_eq!(cfg.tiers, vec![1, 2]);
    }

    #[test]
    fn test_config_with_overrides_no_flags_preserves_base() {
        let loaded = Config {
            callback_base_url: "https://my.server.io".to_string(),
            bind_address: "10.0.0.1:9000".to_string(),
            tiers: vec![2, 3],
            page_title: "Custom Title".to_string(),
            theme: "dark".to_string(),
        };
        let cfg = config_with_overrides(&loaded, None, None, None);
        assert_eq!(cfg, loaded);
    }

    #[test]
    fn test_config_with_overrides_partial_bind_only() {
        let base = Config {
            bind_address: "127.0.0.1:8080".to_string(),
            ..Config::default()
        };
        let cfg = config_with_overrides(&base, None, Some("0.0.0.0:9090"), None);
        assert_eq!(cfg.bind_address, "0.0.0.0:9090");
        // callback_base_url and tiers stay as base
        assert_eq!(cfg.callback_base_url, base.callback_base_url);
        assert_eq!(cfg.tiers, base.tiers);
    }

    // ---- existing tests ----

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
