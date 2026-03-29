use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Tier1 = 1,
    Tier2 = 2,
    Tier3 = 3,
}

impl From<Tier> for u8 {
    fn from(t: Tier) -> u8 {
        t as u8
    }
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceMapping {
    pub nonce: String,
    pub tier: Tier,
    pub payload_id: String,
    pub embedding_location: EmbeddingLocation,
    pub callback_url: String,
}
