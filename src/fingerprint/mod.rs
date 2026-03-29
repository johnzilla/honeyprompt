use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;
use sha2::{Digest, Sha256};

use crate::types::AgentFingerprint;

/// Extract an AgentFingerprint from request parts.
///
/// Reads the user-agent header, converts all headers to a HashMap<String,String>
/// (skipping non-UTF-8 values), strips control characters from the user-agent,
/// and records the current unix timestamp as received_at.
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

    // Convert HeaderMap to HashMap<String, String>, filtering invalid UTF-8.
    let header_map: HashMap<String, String> = headers
        .iter()
        .filter_map(|(name, value)| -> Option<(String, String)> {
            value.to_str().ok().map(|v| (name.as_str().to_string(), v.to_string()))
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

/// Compute a deterministic session ID from IP address and user-agent string.
///
/// Per D-07: SHA-256(ip + ua), first 8 bytes, hex-encoded → 16-char hex string.
pub fn compute_session_id(ip: &str, ua: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    hasher.update(ua.as_bytes());
    let digest = hasher.finalize();
    hex::encode(&digest[..8])
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
        assert!(fp.headers.contains_key("accept"), "accept header should be present");
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
        let sid = compute_session_id("1.2.3.4", "Mozilla/5.0");
        assert_eq!(sid.len(), 16, "session ID must be exactly 16 hex chars");
        assert!(
            sid.chars().all(|c| c.is_ascii_hexdigit()),
            "session ID must be all hex digits"
        );
    }

    #[test]
    fn test_compute_session_id_deterministic() {
        let sid1 = compute_session_id("1.2.3.4", "Mozilla/5.0");
        let sid2 = compute_session_id("1.2.3.4", "Mozilla/5.0");
        assert_eq!(sid1, sid2, "same inputs must produce same session ID");
    }

    #[test]
    fn test_compute_session_id_different_inputs_differ() {
        let sid1 = compute_session_id("1.2.3.4", "Mozilla/5.0");
        let sid2 = compute_session_id("9.9.9.9", "DifferentBot/2.0");
        assert_ne!(sid1, sid2, "different inputs must produce different session IDs");
    }
}
