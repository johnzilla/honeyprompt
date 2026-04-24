/// Generate a 16-character lowercase hex nonce from 8 bytes of OS CSPRNG.
pub fn generate_nonce() -> String {
    let mut buf = [0u8; 8];
    getrandom::fill(&mut buf).expect("OS CSPRNG unavailable");
    hex::encode(buf)
}

/// Derive the verification seed from the first 8 hex chars of a nonce.
/// Returns None if the nonce is shorter than 8 chars or does not parse as hex.
///
/// D-13-04: The verification_seed is `u32::from_str_radix(&nonce[0..8], 16).ok()`.
/// The server re-derives the seed at T5 verification time using this exact function.
/// Using `.ok()` (not `.unwrap()`) at the handler boundary prevents panics on any
/// malformed nonce that somehow reaches this helper — RESEARCH.md Risk 1 mitigation.
pub fn derive_seed(nonce: &str) -> Option<u32> {
    if nonce.len() < 8 {
        return None;
    }
    u32::from_str_radix(&nonce[0..8], 16).ok()
}

/// Validate a nonce has the canonical format: 16 lowercase hex characters.
/// D-13-16: unchanged across tiers (T1-T5 all use the same 16-char hex nonce).
/// Extracted from the inline check in server::callback_handler so T4/T5 handlers
/// in Plan 04 can reuse the same logic without drift.
pub fn is_valid_nonce(nonce: &str) -> bool {
    nonce.len() == 16
        && nonce
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_length() {
        assert_eq!(generate_nonce().len(), 16, "Nonce must be 16 characters");
    }

    #[test]
    fn test_nonce_hex() {
        let nonce = generate_nonce();
        assert!(
            nonce
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "Nonce must be lowercase hex: {}",
            nonce
        );
    }

    #[test]
    fn test_nonce_uniqueness() {
        let mut nonces = std::collections::HashSet::new();
        for _ in 0..100 {
            nonces.insert(generate_nonce());
        }
        assert_eq!(nonces.len(), 100, "All 100 nonces must be unique");
    }

    #[test]
    fn test_derive_seed_valid_nonce() {
        // "abcdef1234567890" — first 8 chars "abcdef12" → 0xabcdef12
        assert_eq!(derive_seed("abcdef1234567890"), Some(0xabcdef12u32));
    }

    #[test]
    fn test_derive_seed_all_zero_prefix() {
        // first 8 chars "00000000" → 0
        assert_eq!(derive_seed("0000000000000000"), Some(0));
    }

    #[test]
    fn test_derive_seed_max_prefix() {
        // first 8 chars "ffffffff" → u32::MAX
        assert_eq!(derive_seed("ffffffffffffffff"), Some(u32::MAX));
    }

    #[test]
    fn test_derive_seed_short_nonce() {
        // Fewer than 8 chars must return None (no panic)
        assert_eq!(derive_seed("abc"), None);
        assert_eq!(derive_seed(""), None);
        assert_eq!(derive_seed("abcdef1"), None); // 7 chars
    }

    #[test]
    fn test_derive_seed_non_hex_prefix() {
        // 8 chars that do not parse as hex → None
        assert_eq!(derive_seed("ghijklmn12345678"), None);
    }

    #[test]
    fn test_is_valid_nonce_canonical() {
        assert!(is_valid_nonce("0123456789abcdef"));
        assert!(is_valid_nonce("ffffffffffffffff"));
        assert!(is_valid_nonce("0000000000000000"));
    }

    #[test]
    fn test_is_valid_nonce_rejects_uppercase() {
        assert!(
            !is_valid_nonce("0123456789ABCDEF"),
            "uppercase hex must be rejected (D-13-16)"
        );
        assert!(
            !is_valid_nonce("0123456789aBcDeF"),
            "mixed-case hex must be rejected"
        );
    }

    #[test]
    fn test_is_valid_nonce_rejects_wrong_length() {
        assert!(
            !is_valid_nonce("0123456789abcde"),
            "15 chars must be rejected"
        );
        assert!(
            !is_valid_nonce("0123456789abcdef0"),
            "17 chars must be rejected"
        );
        assert!(!is_valid_nonce(""), "empty must be rejected");
    }

    #[test]
    fn test_is_valid_nonce_rejects_non_hex() {
        assert!(
            !is_valid_nonce("ghijklmn12345678"),
            "non-hex chars must be rejected"
        );
        assert!(
            !is_valid_nonce("0123456789abcde!"),
            "punctuation must be rejected"
        );
    }
}
