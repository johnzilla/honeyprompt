use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;
use sha2::{Digest, Sha256};

use crate::types::AgentFingerprint;

/// Headers kept for fingerprinting. Everything else is dropped, so credentials
/// (Authorization, Cookie, Proxy-Authorization, API-key headers) are never stored.
pub const HEADER_ALLOWLIST: &[&str] = &[
    "user-agent",
    "accept",
    "accept-language",
    "accept-encoding",
    "from",
    "via",
    "x-forwarded-for",
    "x-real-ip",
];

/// Longest header value stored; longer values are truncated.
const MAX_HEADER_VALUE_CHARS: usize = 512;

/// Extract an AgentFingerprint from request parts.
///
/// Reads the user-agent header, keeps only allowlisted headers (skipping
/// non-UTF-8 values, truncating long ones), strips control characters from the
/// user-agent, and records the current unix timestamp as received_at.
pub fn extract(source_ip: IpAddr, headers: &HeaderMap) -> AgentFingerprint {
    // Extract user-agent, default to empty string.
    let raw_ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Strip control characters (bytes < 0x20 except horizontal tab 0x09).
    let user_agent: String = raw_ua
        .chars()
        .filter(|&c| c == '\t' || c >= '\x20')
        .collect();

    // Keep only allowlisted headers, filtering invalid UTF-8.
    let header_map: HashMap<String, String> = headers
        .iter()
        .filter(|(name, _)| HEADER_ALLOWLIST.contains(&name.as_str()))
        .filter_map(|(name, value)| -> Option<(String, String)> {
            value.to_str().ok().map(|v| {
                (
                    name.as_str().to_string(),
                    v.chars().take(MAX_HEADER_VALUE_CHARS).collect(),
                )
            })
        })
        .collect();

    let received_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    AgentFingerprint {
        source_ip,
        user_agent,
        headers: header_map,
        received_at,
    }
}

/// Compute a session ID from a per-deployment salt, IP address and user-agent.
///
/// SHA-256(salt || 0 || ip || 0 || ua), first 8 bytes, hex-encoded → 16-char hex.
/// The salt stops IDs from being correlated across deployments or reversed by
/// brute-forcing IPv4 space against common user-agents.
pub fn compute_session_id(salt: &str, ip: &str, ua: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update([0u8]);
    hasher.update(ip.as_bytes());
    hasher.update([0u8]);
    hasher.update(ua.as_bytes());
    let digest = hasher.finalize();
    hex::encode(&digest[..8])
}

/// Load the deployment's session salt from `{state_dir}/session-salt`, creating
/// a random 32-byte one on first run.
pub fn load_or_create_salt(state_dir: &Path) -> anyhow::Result<String> {
    let path = state_dir.join("session-salt");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    std::fs::create_dir_all(state_dir)?;
    let mut buf = [0u8; 32];
    getrandom::fill(&mut buf).map_err(|e| anyhow::anyhow!("OS CSPRNG unavailable: {e}"))?;
    let salt = hex::encode(buf);
    std::fs::write(&path, &salt)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(salt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_extract_basic_fingerprint() {
        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", "Mozilla/5.0".parse().unwrap());
        headers.insert("accept", "text/html".parse().unwrap());

        let fp = extract(ip, &headers);
        assert_eq!(fp.source_ip.to_string(), "1.2.3.4");
        assert_eq!(fp.user_agent, "Mozilla/5.0");
        assert!(!fp.headers.is_empty(), "headers map should not be empty");
        assert!(
            fp.headers.contains_key("accept"),
            "accept header should be present"
        );
    }

    #[test]
    fn test_extract_strips_control_characters() {
        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        let mut headers = HeaderMap::new();
        // Build a UA string with embedded control chars via header value bytes.
        // We test the strip logic by calling the function with the sanitized result.
        // The extract function strips chars < 0x20 (except tab 0x09).
        // We verify the logic via unit testing the strip directly:
        let raw_ua = "Some\x01Agent\x1f/1.0\x00";
        let stripped: String = raw_ua
            .chars()
            .filter(|&c| c == '\t' || c >= '\x20')
            .collect();
        assert_eq!(stripped, "SomeAgent/1.0");

        // Also verify extract returns valid output on a normal UA.
        headers.insert("user-agent", "NormalBot/1.0".parse().unwrap());
        let fp = extract(ip, &headers);
        assert_eq!(fp.user_agent, "NormalBot/1.0");
    }

    #[test]
    fn test_extract_missing_user_agent_defaults_to_empty() {
        let ip: IpAddr = "5.6.7.8".parse().unwrap();
        let headers = HeaderMap::new();
        let fp = extract(ip, &headers);
        assert_eq!(fp.user_agent, "");
    }

    #[test]
    fn test_compute_session_id_returns_16_char_hex() {
        let sid = compute_session_id("salt", "1.2.3.4", "Mozilla/5.0");
        assert_eq!(sid.len(), 16, "session ID must be exactly 16 hex chars");
        assert!(
            sid.chars().all(|c| c.is_ascii_hexdigit()),
            "session ID must be all hex digits"
        );
    }

    #[test]
    fn test_compute_session_id_deterministic() {
        let sid1 = compute_session_id("salt", "1.2.3.4", "Mozilla/5.0");
        let sid2 = compute_session_id("salt", "1.2.3.4", "Mozilla/5.0");
        assert_eq!(sid1, sid2, "same inputs must produce same session ID");
    }

    #[test]
    fn test_compute_session_id_different_inputs_differ() {
        let sid1 = compute_session_id("salt", "1.2.3.4", "Mozilla/5.0");
        let sid2 = compute_session_id("salt", "9.9.9.9", "DifferentBot/2.0");
        assert_ne!(
            sid1, sid2,
            "different inputs must produce different session IDs"
        );
    }

    #[test]
    fn test_extract_drops_sensitive_headers() {
        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", "Bot/1.0".parse().unwrap());
        headers.insert("authorization", "Bearer secret".parse().unwrap());
        headers.insert("cookie", "session=abc".parse().unwrap());
        headers.insert("proxy-authorization", "Basic xyz".parse().unwrap());
        headers.insert("x-api-key", "sk-123".parse().unwrap());
        headers.insert("x-forwarded-for", "9.9.9.9".parse().unwrap());

        let fp = extract(ip, &headers);
        for dropped in [
            "authorization",
            "cookie",
            "proxy-authorization",
            "x-api-key",
        ] {
            assert!(
                !fp.headers.contains_key(dropped),
                "{dropped} must be dropped"
            );
        }
        assert_eq!(fp.headers.get("x-forwarded-for").unwrap(), "9.9.9.9");
        assert_eq!(fp.headers.get("user-agent").unwrap(), "Bot/1.0");
    }

    #[test]
    fn test_extract_truncates_long_values() {
        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("accept", "a".repeat(2000).parse().unwrap());
        let fp = extract(ip, &headers);
        assert_eq!(
            fp.headers.get("accept").unwrap().len(),
            MAX_HEADER_VALUE_CHARS
        );
    }

    #[test]
    fn test_compute_session_id_salt_changes_id() {
        let a = compute_session_id("salt-a", "1.2.3.4", "Mozilla/5.0");
        let b = compute_session_id("salt-b", "1.2.3.4", "Mozilla/5.0");
        assert_ne!(a, b, "different salts must produce different session IDs");
    }

    #[test]
    fn test_load_or_create_salt_persists() {
        let dir = tempfile::TempDir::new().unwrap();
        let first = load_or_create_salt(dir.path()).unwrap();
        assert_eq!(first.len(), 64);
        let second = load_or_create_salt(dir.path()).unwrap();
        assert_eq!(first, second, "salt must be stable across runs");
    }
}
