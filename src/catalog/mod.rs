use anyhow::{anyhow, Context};
use rust_embed::RustEmbed;
use serde::Deserialize;

use crate::types::{EmbeddingLocation, Payload, T5Formula, Tier};

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
    // Phase 13 (D-13-12): flat optional T5 formula fields. All `None` for T1-T4;
    // all three `Some` for T5. Partial presence is an error (see `into_payload`).
    #[serde(default)]
    formula_a: Option<u32>,
    #[serde(default)]
    formula_b: Option<u32>,
    #[serde(default)]
    formula_mod: Option<u32>,
}

impl PayloadDef {
    fn into_payload(self) -> anyhow::Result<Payload> {
        let tier = match self.tier {
            1 => Tier::Tier1,
            2 => Tier::Tier2,
            3 => Tier::Tier3,
            4 => Tier::Tier4,
            5 => Tier::Tier5,
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

        // Phase 13 (D-13-12): derive Option<T5Formula> from the three flat optional fields.
        let t5_formula = match (self.formula_a, self.formula_b, self.formula_mod) {
            (Some(a), Some(b), Some(m)) => Some(T5Formula { a, b, modulus: m }),
            (None, None, None) => None,
            _ => {
                return Err(anyhow!(
                    "payload {} has partial formula fields — all three of formula_a/formula_b/formula_mod must be present together",
                    self.id
                ))
            }
        };

        // Phase 13: enforce tier/formula correspondence.
        // - tier 5 MUST have formula constants
        // - tier 1-4 MUST NOT have formula constants
        match (tier, &t5_formula) {
            (Tier::Tier5, None) => {
                return Err(anyhow!(
                    "tier 5 payload {} missing formula constants (formula_a/formula_b/formula_mod required)",
                    self.id
                ))
            }
            (t, Some(_)) if t != Tier::Tier5 => {
                return Err(anyhow!(
                    "payload {} has formula constants but is tier {} (only tier 5 uses formulas)",
                    self.id,
                    u8::from(t)
                ))
            }
            _ => {}
        }

        Ok(Payload {
            id: self.id,
            tier,
            embedding_location,
            instruction: self.instruction,
            t5_formula,
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

/// Load all curated payloads from the embedded catalog (Tiers 1–5).
pub fn load_catalog() -> anyhow::Result<Vec<Payload>> {
    let mut all = Vec::new();
    for filename in &[
        "tier1.toml",
        "tier2.toml",
        "tier3.toml",
        "tier4.toml",
        "tier5.toml",
    ] {
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
        assert_eq!(
            payloads.len(),
            12,
            "Expected 12 total payloads: 2 T1 + 2 T2 + 2 T3 + 3 T4 + 3 T5"
        );
    }

    #[test]
    fn test_tier1_catalog() {
        let payloads = load_for_tiers(&[1]).expect("tier1 must load");
        assert_eq!(payloads.len(), 2, "Tier 1 must have exactly 2 payloads");
        let has_semantic_prose = payloads
            .iter()
            .any(|p| p.embedding_location == EmbeddingLocation::SemanticProse);
        assert!(
            has_semantic_prose,
            "Tier 1 must have at least one semantic_prose payload"
        );
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
        for tier_num in 1u8..=5 {
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
    fn test_tier4_catalog() {
        let all = load_catalog().expect("catalog must load");
        let t4: Vec<&Payload> = all.iter().filter(|p| p.tier == Tier::Tier4).collect();
        assert_eq!(t4.len(), 3, "tier 4 must ship exactly 3 payloads (D-13-07)");
        // Distinct embedding locations within tier 4 (D-13-07/08)
        let mut locs: Vec<_> = t4.iter().map(|p| p.embedding_location).collect();
        locs.sort_by_key(|l| format!("{:?}", l));
        locs.dedup();
        assert_eq!(
            locs.len(),
            3,
            "tier 4 payloads must use 3 distinct embedding locations"
        );
        // None carry formula constants (D-13-12: formulas are T5-only)
        for p in &t4 {
            assert!(
                p.t5_formula.is_none(),
                "tier 4 payload {} must have no t5_formula",
                p.id
            );
        }
    }

    #[test]
    fn test_tier4_diverse_phrasing() {
        let all = load_catalog().expect("catalog must load");
        let t4: Vec<&Payload> = all.iter().filter(|p| p.tier == Tier::Tier4).collect();
        // Minimum diversity check: no two instructions are substrings of each other.
        for i in 0..t4.len() {
            for j in (i + 1)..t4.len() {
                assert!(
                    !t4[i].instruction.contains(&t4[j].instruction)
                        && !t4[j].instruction.contains(&t4[i].instruction),
                    "T4 payloads {} and {} have overlapping instruction text (D-13-08 requires distinct phrasing)",
                    t4[i].id,
                    t4[j].id
                );
            }
        }
    }

    #[test]
    fn test_tier5_catalog() {
        let all = load_catalog().expect("catalog must load");
        let t5: Vec<&Payload> = all.iter().filter(|p| p.tier == Tier::Tier5).collect();
        assert_eq!(
            t5.len(),
            3,
            "tier 5 must ship exactly 3 payloads (D-13-02/PAYLOAD-03)"
        );
        for p in &t5 {
            let f = p
                .t5_formula
                .as_ref()
                .unwrap_or_else(|| panic!("tier 5 payload {} must have t5_formula", p.id));
            assert_eq!(
                f.modulus, 1000,
                "tier 5 payload {} must use formula_mod = 1000 (D-13-02 3-digit output)",
                p.id
            );
            assert!(
                f.b != 0 && f.modulus != 0,
                "tier 5 payload {} has zero formula_b or formula_mod",
                p.id
            );
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
