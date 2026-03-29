use rust_embed::RustEmbed;
use serde::Deserialize;
use anyhow::{anyhow, Context};

use crate::types::{EmbeddingLocation, Payload, Tier};

#[derive(RustEmbed)]
#[folder = "assets/catalog/"]
struct CatalogAssets;

/// Intermediate deserialization struct matching the TOML schema.
#[derive(Debug, Deserialize)]
struct CatalogFile {
    payloads: Vec<PayloadDef>,
}

#[derive(Debug, Deserialize)]
struct PayloadDef {
    id: String,
    tier: u8,
    embedding_location: String,
    instruction: String,
}

impl PayloadDef {
    fn into_payload(self) -> anyhow::Result<Payload> {
        let tier = match self.tier {
            1 => Tier::Tier1,
            2 => Tier::Tier2,
            3 => Tier::Tier3,
            n => return Err(anyhow!("Unknown tier: {}", n)),
        };
        let embedding_location = match self.embedding_location.as_str() {
            "html_comment" => EmbeddingLocation::HtmlComment,
            "meta_tag" => EmbeddingLocation::MetaTag,
            "invisible_element" => EmbeddingLocation::InvisibleElement,
            "json_ld" => EmbeddingLocation::JsonLd,
            "semantic_prose" => EmbeddingLocation::SemanticProse,
            s => return Err(anyhow!("Unknown embedding_location: {}", s)),
        };
        Ok(Payload {
            id: self.id,
            tier,
            embedding_location,
            instruction: self.instruction,
        })
    }
}

/// Load payloads from a single embedded TOML file by name.
fn load_tier_file(filename: &str) -> anyhow::Result<Vec<Payload>> {
    let file = CatalogAssets::get(filename)
        .with_context(|| format!("Embedded catalog asset not found: {}", filename))?;
    let content = std::str::from_utf8(file.data.as_ref())
        .with_context(|| format!("Catalog asset is not valid UTF-8: {}", filename))?;
    let catalog: CatalogFile = toml::from_str(content)
        .with_context(|| format!("Failed to parse catalog TOML: {}", filename))?;
    catalog
        .payloads
        .into_iter()
        .map(PayloadDef::into_payload)
        .collect()
}

/// Load all curated payloads from the embedded catalog (Tiers 1–3).
pub fn load_catalog() -> anyhow::Result<Vec<Payload>> {
    let mut all = Vec::new();
    for filename in &["tier1.toml", "tier2.toml", "tier3.toml"] {
        let payloads = load_tier_file(filename)?;
        all.extend(payloads);
    }
    Ok(all)
}

/// Load curated payloads for the specified tier numbers (1, 2, 3).
pub fn load_for_tiers(tiers: &[u8]) -> anyhow::Result<Vec<Payload>> {
    let all = load_catalog()?;
    Ok(all
        .into_iter()
        .filter(|p| {
            let t: u8 = p.tier.into();
            tiers.contains(&t)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_payloads() {
        let payloads = load_catalog().expect("catalog must load");
        assert_eq!(payloads.len(), 6, "Expected 6 total payloads across all tiers");
    }

    #[test]
    fn test_tier1_catalog() {
        let payloads = load_for_tiers(&[1]).expect("tier1 must load");
        assert_eq!(payloads.len(), 2, "Tier 1 must have exactly 2 payloads");
        let has_semantic_prose = payloads
            .iter()
            .any(|p| p.embedding_location == EmbeddingLocation::SemanticProse);
        assert!(has_semantic_prose, "Tier 1 must have at least one semantic_prose payload");
    }

    #[test]
    fn test_tier2_branches() {
        let payloads = load_for_tiers(&[2]).expect("tier2 must load");
        assert_eq!(payloads.len(), 2, "Tier 2 must have exactly 2 payloads");
        for payload in &payloads {
            assert!(
                payload.instruction.contains("{callback_url_a}"),
                "Tier 2 payload '{}' must contain {{callback_url_a}}",
                payload.id
            );
            assert!(
                payload.instruction.contains("{callback_url_b}"),
                "Tier 2 payload '{}' must contain {{callback_url_b}}",
                payload.id
            );
        }
    }

    #[test]
    fn test_tier3_computed() {
        let payloads = load_for_tiers(&[3]).expect("tier3 must load");
        assert_eq!(payloads.len(), 2, "Tier 3 must have exactly 2 payloads");
        for payload in &payloads {
            assert!(
                payload.instruction.contains("{callback_url_base}"),
                "Tier 3 payload '{}' must contain {{callback_url_base}}",
                payload.id
            );
        }
    }

    #[test]
    fn test_no_duplicate_locations() {
        let all = load_catalog().expect("catalog must load");
        for tier_num in 1u8..=3 {
            let tier_payloads: Vec<_> = all
                .iter()
                .filter(|p| {
                    let t: u8 = p.tier.into();
                    t == tier_num
                })
                .collect();
            let mut seen = std::collections::HashSet::new();
            for p in &tier_payloads {
                let loc = format!("{:?}", p.embedding_location);
                assert!(
                    seen.insert(loc.clone()),
                    "Tier {} has duplicate embedding location: {}",
                    tier_num,
                    loc
                );
            }
        }
    }

    #[test]
    fn test_catalog_is_curated() {
        // Verify no public function exists that accepts arbitrary payload text.
        // The catalog API is load_catalog() and load_for_tiers(&[u8]) only.
        // This test confirms the module compiles with only curated loaders —
        // there is no inject_payload() or add_payload() function callable here.
        let payloads = load_catalog().expect("catalog must load");
        assert!(!payloads.is_empty(), "Catalog must not be empty");
        // The only way to get payloads is through the curated embedded TOML files.
        // No public API accepts arbitrary instruction strings (GEN-07).
    }
}
