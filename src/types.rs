use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Tier1 = 1,
    Tier2 = 2,
    Tier3 = 3,
    Tier4 = 4,
    Tier5 = 5,
}

impl From<Tier> for u8 {
    fn from(t: Tier) -> u8 {
        t as u8
    }
}

/// T5 arithmetic constants used for deterministic proof verification (D-13-02).
/// `proof = ((seed + a) * b) % modulus`, where `seed = u32::from_str_radix(&nonce[0..8], 16)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct T5Formula {
    pub a: u32,
    pub b: u32,
    pub modulus: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingLocation {
    HtmlComment,
    MetaTag,
    InvisibleElement,
    JsonLd,
    SemanticProse,
}

impl fmt::Display for EmbeddingLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmbeddingLocation::HtmlComment => write!(f, "html_comment"),
            EmbeddingLocation::MetaTag => write!(f, "meta_tag"),
            EmbeddingLocation::InvisibleElement => write!(f, "invisible_element"),
            EmbeddingLocation::JsonLd => write!(f, "json_ld"),
            EmbeddingLocation::SemanticProse => write!(f, "semantic_prose"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub id: String,
    pub tier: Tier,
    pub embedding_location: EmbeddingLocation,
    pub instruction: String,
    /// Phase 13 (D-13-12): T5 arithmetic constants; `Some` only for tier-5 payloads.
    #[serde(default)]
    pub t5_formula: Option<T5Formula>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceMapping {
    pub nonce: String,
    pub tier: Tier,
    pub payload_id: String,
    pub embedding_location: EmbeddingLocation,
    pub callback_url: String,
}

/// Agent classification per D-06: three-tier system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentClass {
    KnownCrawler { provider: String },
    KnownAgent { provider: String },
    Unknown,
}

/// Fingerprint extracted from an HTTP callback request (SRV-03)
#[derive(Debug, Clone)]
pub struct AgentFingerprint {
    pub source_ip: std::net::IpAddr,
    pub user_agent: String,
    pub headers: std::collections::HashMap<String, String>,
    pub received_at: u64,
}

/// Raw callback event assembled in the Axum handler before broker processing
#[derive(Debug, Clone)]
pub struct RawCallbackEvent {
    pub nonce: String,
    pub tier: u8,
    pub payload_id: String,
    pub embedding_loc: String,
    pub fingerprint: AgentFingerprint,
    pub classification: AgentClass,
    pub received_at: u64,
    /// Phase 13 (T4): sanitized capability string decoded from the /cb/v4/ path.
    /// `Some` only for tier-4 callbacks that passed the decode+sanitize pipeline.
    pub t4_capability: Option<String>,
    /// Phase 13 (T5): raw 3-digit proof string submitted on the /cb/v5/ path.
    /// `Some` only for tier-5 callbacks that passed the 3-ASCII-digit format check.
    pub t5_proof: Option<String>,
    /// Phase 13 (T5): whether `t5_proof` matched the expected value derived from
    /// `nonce::derive_seed` and the payload's `T5Formula`.
    pub t5_proof_valid: Option<bool>,
    /// Phase 14: server-verified T5 formula constants (`(seed + a) * b % modulus`),
    /// propagated from the payload catalog so the Monitor detail pane can render
    /// `formula=(seed+A)*B % M`. `None` for non-T5 events and for attach-mode
    /// reads of legacy DBs with no catalog context.
    pub t5_formula: Option<T5Formula>,
}

/// Enriched event after broker processing (with session info, replay detection)
#[derive(Debug, Clone)]
pub struct AppEvent {
    pub nonce: String,
    pub tier: u8,
    pub payload_id: String,
    pub embedding_loc: String,
    pub fingerprint: AgentFingerprint,
    pub classification: AgentClass,
    pub session_id: String,
    pub is_replay: bool,
    pub fire_count: u32,
    pub received_at: u64,
    /// Phase 13 (T4): propagated from `RawCallbackEvent.t4_capability`.
    pub t4_capability: Option<String>,
    /// Phase 13 (T5): propagated from `RawCallbackEvent.t5_proof`.
    pub t5_proof: Option<String>,
    /// Phase 13 (T5): propagated from `RawCallbackEvent.t5_proof_valid`.
    pub t5_proof_valid: Option<bool>,
    /// Phase 14: server-verified T5 formula constants (`(seed + a) * b % modulus`),
    /// propagated from the payload catalog so the Monitor detail pane can render
    /// `formula=(seed+A)*B % M`. `None` for non-T5 events and for attach-mode
    /// reads of legacy DBs with no catalog context.
    pub t5_formula: Option<T5Formula>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::net::IpAddr;

    #[test]
    fn test_agent_class_known_crawler_has_provider() {
        let c = AgentClass::KnownCrawler {
            provider: "OpenAI".to_string(),
        };
        match c {
            AgentClass::KnownCrawler { provider } => assert_eq!(provider, "OpenAI"),
            _ => panic!("Expected KnownCrawler"),
        }
    }

    #[test]
    fn test_agent_class_known_agent_has_provider() {
        let a = AgentClass::KnownAgent {
            provider: "Anthropic".to_string(),
        };
        match a {
            AgentClass::KnownAgent { provider } => assert_eq!(provider, "Anthropic"),
            _ => panic!("Expected KnownAgent"),
        }
    }

    #[test]
    fn test_agent_class_unknown_has_no_field() {
        let u = AgentClass::Unknown;
        assert_eq!(u, AgentClass::Unknown);
    }

    #[test]
    fn test_agent_fingerprint_fields() {
        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        let mut headers = HashMap::new();
        headers.insert("accept".to_string(), "text/html".to_string());
        let fp = AgentFingerprint {
            source_ip: ip,
            user_agent: "TestAgent/1.0".to_string(),
            headers,
            received_at: 12345u64,
        };
        assert_eq!(fp.source_ip.to_string(), "1.2.3.4");
        assert_eq!(fp.user_agent, "TestAgent/1.0");
        assert!(!fp.headers.is_empty());
        assert_eq!(fp.received_at, 12345u64);
    }

    #[test]
    fn test_raw_callback_event_fields() {
        let ip: IpAddr = "10.0.0.1".parse().unwrap();
        let fp = AgentFingerprint {
            source_ip: ip,
            user_agent: "Bot/1.0".to_string(),
            headers: HashMap::new(),
            received_at: 100u64,
        };
        let ev = RawCallbackEvent {
            nonce: "abc123".to_string(),
            tier: 1,
            payload_id: "t1-html".to_string(),
            embedding_loc: "html_comment".to_string(),
            fingerprint: fp,
            classification: AgentClass::Unknown,
            received_at: 100u64,
            t4_capability: None,
            t5_proof: None,
            t5_proof_valid: None,
            t5_formula: None,
        };
        assert_eq!(ev.nonce, "abc123");
        assert_eq!(ev.tier, 1u8);
    }

    #[test]
    fn test_app_event_fields() {
        let ip: IpAddr = "10.0.0.2".parse().unwrap();
        let fp = AgentFingerprint {
            source_ip: ip,
            user_agent: "Agent/1.0".to_string(),
            headers: HashMap::new(),
            received_at: 200u64,
        };
        let ev = AppEvent {
            nonce: "def456".to_string(),
            tier: 2,
            payload_id: "t2-meta".to_string(),
            embedding_loc: "meta_tag".to_string(),
            fingerprint: fp,
            classification: AgentClass::KnownCrawler {
                provider: "Google".to_string(),
            },
            session_id: "aabbccdd11223344".to_string(),
            is_replay: false,
            fire_count: 1,
            received_at: 200u64,
            t4_capability: None,
            t5_proof: None,
            t5_proof_valid: None,
            t5_formula: None,
        };
        assert_eq!(ev.session_id, "aabbccdd11223344");
        assert!(!ev.is_replay);
        assert_eq!(ev.fire_count, 1);
    }

    #[test]
    fn test_app_event_t5_formula_accessible() {
        let f = T5Formula {
            a: 7,
            b: 13,
            modulus: 1000,
        };
        let ev = AppEvent {
            nonce: "deadbeefcafebabe".to_string(),
            tier: 5,
            payload_id: "t5-test".to_string(),
            embedding_loc: "html_comment".to_string(),
            fingerprint: AgentFingerprint {
                source_ip: IpAddr::V4(std::net::Ipv4Addr::new(1, 2, 3, 4)),
                user_agent: "ua".to_string(),
                headers: HashMap::new(),
                received_at: 0,
            },
            classification: AgentClass::Unknown,
            session_id: "s".to_string(),
            is_replay: false,
            fire_count: 1,
            received_at: 0,
            t4_capability: None,
            t5_proof: Some("123".to_string()),
            t5_proof_valid: Some(true),
            t5_formula: Some(f),
        };
        assert_eq!(ev.t5_formula, Some(f));
        assert_eq!(ev.t5_formula.unwrap().a, 7);
        assert_eq!(ev.t5_formula.unwrap().b, 13);
        assert_eq!(ev.t5_formula.unwrap().modulus, 1000);
    }
}
