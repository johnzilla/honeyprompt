use anyhow::Result;
use dialoguer::{Input, MultiSelect};

use crate::config::Config;

/// Pure function: construct a Config from user-supplied inputs.
/// Sets callback_base_url to "https://{domain}" and theme to "default".
pub fn build_config_from_inputs(
    domain: &str,
    bind_address: &str,
    tiers: Vec<u8>,
    page_title: &str,
) -> Config {
    Config {
        callback_base_url: format!("https://{}", domain),
        bind_address: bind_address.to_string(),
        tiers,
        page_title: page_title.to_string(),
        theme: "default".to_string(),
    }
}

/// Check whether a domain name resolves via DNS.
///
/// Returns Ok(true) if resolution succeeds, Ok(false) if it fails.
/// Never returns Err for DNS failures — non-blocking warning per SETUP-02.
pub fn check_dns(domain: &str) -> Result<bool> {
    use std::net::ToSocketAddrs;
    match (domain, 443u16).to_socket_addrs() {
        Ok(mut addrs) => Ok(addrs.next().is_some()),
        Err(_) => Ok(false),
    }
}

/// Serialize config to TOML and write it to path.
///
/// On failure, returns Err with a message including the path and OS error (SETUP-03).
pub fn validate_and_write_config(config: &Config, path: &std::path::Path) -> Result<()> {
    let toml_string = toml::to_string_pretty(config)?;
    std::fs::write(path, toml_string)
        .map_err(|e| anyhow::anyhow!("Cannot write config: {}: {}", path.display(), e))?;
    Ok(())
}

/// Interactive setup wizard. Prompts the user for domain, bind address, tiers, and page title,
/// then writes honeyprompt.toml into the given directory.
pub fn run_setup(path: &std::path::Path) -> Result<()> {
    let domain: String = Input::new()
        .with_prompt("Domain")
        .interact_text()?;

    let bind_address: String = Input::new()
        .with_prompt("Bind address")
        .default("0.0.0.0:8080".into())
        .interact_text()?;

    let tier_labels = &["Tier 1 (visible)", "Tier 2 (hidden)", "Tier 3 (meta)"];
    let selected = MultiSelect::new()
        .with_prompt("Tiers to enable")
        .items(tier_labels)
        .defaults(&[true, true, true])
        .interact()?;
    // Map 0-indexed selections to 1-indexed tier numbers
    let tiers: Vec<u8> = selected.iter().map(|&i| (i as u8) + 1).collect();

    let page_title: String = Input::new()
        .with_prompt("Page title")
        .default("Security Research Canary".into())
        .interact_text()?;

    // Non-blocking DNS check (SETUP-02)
    match check_dns(&domain) {
        Ok(false) | Err(_) => {
            eprintln!(
                "Warning: DNS lookup for '{}' failed — callback URLs may not work until DNS is configured",
                domain
            );
        }
        Ok(true) => {}
    }

    let config = build_config_from_inputs(&domain, &bind_address, tiers, &page_title);
    let config_path = path.join("honeyprompt.toml");
    validate_and_write_config(&config, &config_path)?;

    println!(
        "Wrote {}/honeyprompt.toml -- run `honeyprompt generate` next.",
        path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load_config;

    #[test]
    fn test_build_config_from_inputs_basic() {
        let cfg = build_config_from_inputs(
            "example.com",
            "0.0.0.0:8080",
            vec![1, 2, 3],
            "My Canary",
        );
        assert_eq!(cfg.callback_base_url, "https://example.com");
        assert_eq!(cfg.bind_address, "0.0.0.0:8080");
        assert_eq!(cfg.tiers, vec![1u8, 2, 3]);
        assert_eq!(cfg.page_title, "My Canary");
        assert_eq!(cfg.theme, "default");
    }

    #[test]
    fn test_build_config_from_inputs_custom() {
        let cfg = build_config_from_inputs(
            "canary.internal",
            "127.0.0.1:9090",
            vec![1, 3],
            "Custom Canary",
        );
        assert_eq!(cfg.callback_base_url, "https://canary.internal");
        assert_eq!(cfg.bind_address, "127.0.0.1:9090");
        assert_eq!(cfg.tiers, vec![1u8, 3]);
        assert_eq!(cfg.page_title, "Custom Canary");
        assert_eq!(cfg.theme, "default");
    }

    #[test]
    fn test_validate_and_write_config_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir failed");
        let config_path = dir.path().join("honeyprompt.toml");
        let cfg = build_config_from_inputs("test.example.com", "0.0.0.0:8080", vec![1, 2], "Test");

        validate_and_write_config(&cfg, &config_path).expect("write failed");

        let loaded = load_config(&config_path).expect("load failed");
        assert_eq!(loaded, cfg);
    }

    #[test]
    #[cfg(unix)]
    fn test_validate_and_write_config_non_writable() {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("tempdir failed");
        // Make the directory read-only so writes inside it fail
        std::fs::set_permissions(dir.path(), Permissions::from_mode(0o444))
            .expect("set_permissions failed");

        let config_path = dir.path().join("honeyprompt.toml");
        let cfg = build_config_from_inputs("example.com", "0.0.0.0:8080", vec![1], "Test");

        let result = validate_and_write_config(&cfg, &config_path);
        // Restore permissions so tempdir cleanup works
        std::fs::set_permissions(dir.path(), Permissions::from_mode(0o755)).ok();
        assert!(result.is_err(), "Expected error writing to read-only directory");
    }

    #[test]
    fn test_check_dns_localhost_resolves() {
        let result = check_dns("localhost").expect("check_dns should not return Err");
        assert!(result, "localhost should resolve");
    }

    #[test]
    fn test_check_dns_invalid_domain_returns_false() {
        let result =
            check_dns("this-domain-does-not-exist-xyz123.invalid").expect("should not Err");
        assert!(!result, "invalid domain should return false");
    }
}
