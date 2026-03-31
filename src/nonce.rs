/// Generate a 16-character lowercase hex nonce from 8 bytes of OS CSPRNG.
pub fn generate_nonce() -> String {
    let mut buf = [0u8; 8];
    getrandom::fill(&mut buf).expect("OS CSPRNG unavailable");
    hex::encode(buf)
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
}
