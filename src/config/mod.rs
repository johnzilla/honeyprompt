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

    /// Which payload tiers to include in the generated honeypot (default: all served tiers, 1-5)
    pub tiers: Vec<u8>,

    /// Title shown in the honeypot page header
    pub page_title: String,

    /// Visual theme for the generated page
    pub theme: String,

    /// Reverse proxies whose `X-Forwarded-For` is trusted (IPs or CIDRs, e.g.
    /// ["172.16.0.0/12"]). Empty = always use the TCP peer address.
    #[serde(default)]
    pub trusted_proxies: Vec<String>,

    /// Outbound alerts (`[webhook]`). Fields can be overridden with
    /// `HONEYPROMPT_WEBHOOK_*` environment variables.
    #[serde(
        default,
        skip_serializing_if = "crate::webhook::WebhookConfig::is_default"
    )]
    pub webhook: crate::webhook::WebhookConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            callback_base_url: "http://localhost:8080".to_string(),
            bind_address: "0.0.0.0:8080".to_string(),
            tiers: vec![1, 2, 3, 4, 5],
            page_title: "Security Research Canary".to_string(),
            theme: "default".to_string(),
            trusted_proxies: Vec::new(),
            webhook: Default::default(),
        }
    }
}

/// Read and parse a `honeyprompt.toml` config file from the given path.
pub fn load_config(path: &Path) -> Result<Config> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => anyhow::bail!(
            "no honeyprompt.toml at {} — run `honeyprompt init`, or pass --domain your-host.example to `serve` to create one",
            path.display()
        ),
        Err(e) => anyhow::bail!("cannot read {}: {e}", path.display()),
    };
    let mut config: Config = toml::from_str(&contents)?;
    config.callback_base_url = validate_base_url(
        &config.callback_base_url,
        &format!("callback_base_url in {}", path.display()),
        true,
    )?;
    Ok(config)
}

/// Write the default config as pretty TOML to the given path.
pub fn write_default_config(path: &Path) -> Result<()> {
    let config = Config::default();
    let toml_string = toml::to_string_pretty(&config)?;
    std::fs::write(path, toml_string)?;
    Ok(())
}

/// Validate a base URL and return it without a trailing slash.
///
/// http(s), a host, optional port, no credentials, query or fragment, and only
/// `[A-Za-z0-9:/.\-\[\]]` (plus `_ ~ %` in a path) — so it is safe to embed in
/// generated HTML and in shell commands. `allow_path` permits a path prefix
/// (a config's `callback_base_url` behind a proxy); flags take host[:port] only.
/// `what` names the setting in error messages.
pub fn validate_base_url(raw: &str, what: &str, allow_path: bool) -> Result<String> {
    let trimmed = raw.trim().trim_end_matches('/');
    let url = reqwest::Url::parse(trimmed)
        .map_err(|e| anyhow::anyhow!("{what} {raw:?} is not a URL: {e}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        anyhow::bail!("{what} must be http:// or https://");
    }
    let path_ok = url.path() == "/" || (allow_path && !url.path().is_empty());
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !path_ok
    {
        anyhow::bail!(
            "{what} must be just scheme://host[:port]{}, e.g. https://agents.example.com (got {raw:?})",
            if allow_path { "[/path]" } else { "" }
        );
    }
    let extra: &[char] = if allow_path { &['_', '~', '%'] } else { &[] };
    let ok = trimmed.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(c, ':' | '/' | '.' | '-' | '[' | ']')
            || extra.contains(&c)
    });
    if !ok {
        anyhow::bail!("{what} {raw:?} contains unsupported characters");
    }
    Ok(trimmed.to_string())
}

/// clap value parser for `--domain`: a bare host with optional port.
pub fn parse_domain(s: &str) -> std::result::Result<String, String> {
    validate_base_url(&format!("https://{s}"), "--domain", false)
        .map(|_| s.trim().to_string())
        .map_err(|_| {
            format!("{s:?} is not a host name: use host or host:port, e.g. honeypot.example.com")
        })
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

    // --domain sets only the callback URL. A new project gets the defaults'
    // bind and tiers from `Config::default()`; an existing config keeps its own
    // (it used to be reset to tiers 1-3, overriding the file).
    if let Some(d) = domain {
        cfg.callback_base_url = format!("https://{}", d);
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

    #[test]
    fn domain_and_base_url_validation() {
        for ok in [
            "honeyprompt.sh",
            "localhost",
            "127.0.0.1:8443",
            "a-b.example.com",
        ] {
            assert_eq!(parse_domain(ok).unwrap(), ok);
        }
        for bad in [
            "x/evil",
            "x?y",
            "x#y",
            "user@x",
            "x\"><script>",
            "https://x.example",
            "",
            "x y",
        ] {
            assert!(parse_domain(bad).is_err(), "{bad}");
        }
        assert_eq!(
            validate_base_url("https://x.example/hp/", "u", true).unwrap(),
            "https://x.example/hp"
        );
        assert!(validate_base_url("https://x.example/hp", "u", false).is_err());
        assert!(validate_base_url("https://x.example/a\"b", "u", true).is_err());
        assert!(validate_base_url("javascript:alert(1)", "u", true).is_err());
    }

    #[test]
    fn load_config_rejects_a_malformed_callback_base_url() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("honeyprompt.toml");
        let cfg = Config {
            callback_base_url: "https://x.example/?q=<b>".into(),
            ..Config::default()
        };
        std::fs::write(&p, toml::to_string_pretty(&cfg).unwrap()).unwrap();
        let err = load_config(&p).unwrap_err().to_string();
        assert!(err.contains("callback_base_url"), "{err}");
        let good = Config {
            callback_base_url: "https://x.example/".into(),
            ..Config::default()
        };
        std::fs::write(&p, toml::to_string_pretty(&good).unwrap()).unwrap();
        assert_eq!(
            load_config(&p).unwrap().callback_base_url,
            "https://x.example"
        );
    }

    // ---- config_with_overrides precedence tests (SERVE-02, SERVE-03) ----

    #[test]
    fn test_config_with_overrides_domain_sets_url_bind_tiers() {
        let base = Config::default();
        let cfg = config_with_overrides(&base, Some("example.com"), None, None);
        assert_eq!(cfg.callback_base_url, "https://example.com");
        assert_eq!(cfg.bind_address, "0.0.0.0:8080");
        assert_eq!(cfg.tiers, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_domain_keeps_an_existing_configs_tiers_and_bind() {
        let loaded = Config {
            bind_address: "127.0.0.1:9000".into(),
            tiers: vec![1, 2, 3, 4, 5],
            ..Config::default()
        };
        let cfg = config_with_overrides(&loaded, Some("example.com"), None, None);
        assert_eq!(cfg.callback_base_url, "https://example.com");
        assert_eq!(cfg.bind_address, "127.0.0.1:9000");
        assert_eq!(cfg.tiers, vec![1, 2, 3, 4, 5]);
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
            trusted_proxies: Vec::new(),
            webhook: Default::default(),
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
