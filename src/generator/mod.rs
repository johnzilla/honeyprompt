use anyhow::Context;
use minijinja::{context, Environment};
use rusqlite::Connection;
use rust_embed::RustEmbed;
use serde::Serialize;
use std::path::Path;

use crate::catalog;
use crate::config::Config;
use crate::nonce;
use crate::store;
use crate::types::{NonceMapping, Tier};

/// Embedded Jinja templates from assets/templates/.
#[derive(RustEmbed)]
#[folder = "assets/templates/"]
struct Templates;

/// Payload data passed to the Jinja template context.
#[derive(Debug, Serialize)]
struct RenderedPayload {
    embedding_location: String,
    rendered_instruction: String,
}

/// Load a template by name from embedded assets and render it with the given context.
fn render_template(name: &str, ctx: minijinja::Value) -> anyhow::Result<String> {
    let file =
        Templates::get(name).with_context(|| format!("Embedded template not found: {}", name))?;
    let source = std::str::from_utf8(file.data.as_ref())
        .with_context(|| format!("Template is not valid UTF-8: {}", name))?;

    let mut env = Environment::new();
    env.add_template(name, source)
        .with_context(|| format!("Failed to add template: {}", name))?;
    let tmpl = env
        .get_template(name)
        .with_context(|| format!("Failed to retrieve template: {}", name))?;
    let rendered = tmpl
        .render(ctx)
        .with_context(|| format!("Failed to render template: {}", name))?;
    Ok(rendered)
}

/// Top-level generate function.
///
/// Loads payloads from the catalog, assigns nonces, renders all templates,
/// writes output files to `{project_path}/output/`, and stores nonce mappings in SQLite.
pub fn generate(config: &Config, conn: &Connection, project_path: &Path) -> anyhow::Result<()> {
    let payloads =
        catalog::load_for_tiers(&config.tiers).context("Failed to load payload catalog")?;

    let mut rendered_payloads: Vec<RenderedPayload> = Vec::new();
    let mut nonce_mappings: Vec<NonceMapping> = Vec::new();
    // Phase 13 Plan 03 (D-13-05): accumulate one <script type="application/ld+json">
    // block per active T5 payload. Joined after the loop and rendered via the
    // `seed_scripts_json` template key (| safe). Empty when no T5 active.
    let mut t5_seed_scripts: Vec<String> = Vec::new();

    for payload in &payloads {
        let embedding_loc = payload.embedding_location.to_string();
        let tier_num: u8 = payload.tier.into();

        match payload.tier {
            Tier::Tier1 => {
                let nonce = nonce::generate_nonce();
                let callback_url = format!("{}/cb/v1/{}", config.callback_base_url, nonce);
                let rendered = payload.instruction.replace("{callback_url}", &callback_url);

                store::insert_nonce(conn, &nonce, tier_num, &payload.id, &embedding_loc)
                    .with_context(|| {
                        format!("Failed to insert nonce for payload {}", payload.id)
                    })?;

                nonce_mappings.push(NonceMapping {
                    nonce,
                    tier: payload.tier,
                    payload_id: payload.id.clone(),
                    embedding_location: payload.embedding_location,
                    callback_url,
                });

                rendered_payloads.push(RenderedPayload {
                    embedding_location: embedding_loc,
                    rendered_instruction: rendered,
                });
            }
            Tier::Tier2 => {
                let nonce_a = nonce::generate_nonce();
                let nonce_b = nonce::generate_nonce();
                let callback_url_a = format!("{}/cb/v1/{}", config.callback_base_url, nonce_a);
                let callback_url_b = format!("{}/cb/v1/{}", config.callback_base_url, nonce_b);

                let rendered = payload
                    .instruction
                    .replace("{callback_url_a}", &callback_url_a)
                    .replace("{callback_url_b}", &callback_url_b);

                store::insert_nonce(conn, &nonce_a, tier_num, &payload.id, &embedding_loc)
                    .with_context(|| {
                        format!("Failed to insert nonce_a for payload {}", payload.id)
                    })?;
                store::insert_nonce(conn, &nonce_b, tier_num, &payload.id, &embedding_loc)
                    .with_context(|| {
                        format!("Failed to insert nonce_b for payload {}", payload.id)
                    })?;

                nonce_mappings.push(NonceMapping {
                    nonce: nonce_a,
                    tier: payload.tier,
                    payload_id: payload.id.clone(),
                    embedding_location: payload.embedding_location,
                    callback_url: callback_url_a,
                });
                nonce_mappings.push(NonceMapping {
                    nonce: nonce_b,
                    tier: payload.tier,
                    payload_id: payload.id.clone(),
                    embedding_location: payload.embedding_location,
                    callback_url: callback_url_b,
                });

                rendered_payloads.push(RenderedPayload {
                    embedding_location: embedding_loc,
                    rendered_instruction: rendered,
                });
            }
            Tier::Tier3 => {
                let nonce = nonce::generate_nonce();
                let callback_url_base = format!("{}/cb/v1/{}", config.callback_base_url, nonce);
                let rendered = payload
                    .instruction
                    .replace("{callback_url_base}", &callback_url_base);

                store::insert_nonce(conn, &nonce, tier_num, &payload.id, &embedding_loc)
                    .with_context(|| {
                        format!("Failed to insert nonce for payload {}", payload.id)
                    })?;

                nonce_mappings.push(NonceMapping {
                    nonce,
                    tier: payload.tier,
                    payload_id: payload.id.clone(),
                    embedding_location: payload.embedding_location,
                    callback_url: callback_url_base,
                });

                rendered_payloads.push(RenderedPayload {
                    embedding_location: embedding_loc,
                    rendered_instruction: rendered,
                });
            }
            Tier::Tier4 => {
                // Phase 13 Plan 03: T4 capability introspection.
                // Agent base64-encodes a sanitized capability string and GETs
                // <callback_base>/cb/v4/<nonce>/<b64>. Server decodes + validates.
                let nonce = nonce::generate_nonce();
                let callback_url_b64_base = format!("{}/cb/v4/{}", config.callback_base_url, nonce);
                let rendered = payload
                    .instruction
                    .replace("{callback_url_b64_base}", &callback_url_b64_base);

                store::insert_nonce(conn, &nonce, tier_num, &payload.id, &embedding_loc)
                    .with_context(|| {
                        format!("Failed to insert nonce for payload {}", payload.id)
                    })?;

                nonce_mappings.push(NonceMapping {
                    nonce: nonce.clone(),
                    tier: payload.tier,
                    payload_id: payload.id.clone(),
                    embedding_location: payload.embedding_location,
                    callback_url: callback_url_b64_base,
                });

                rendered_payloads.push(RenderedPayload {
                    embedding_location: embedding_loc,
                    rendered_instruction: rendered,
                });
            }
            Tier::Tier5 => {
                // Phase 13 Plan 03: T5 multi-step compliance chain.
                // Agent extracts verification_seed from a JSON-LD block on the page
                // (emitted below), applies the formula from the catalog, zero-pads
                // the proof to 3 decimal digits, and GETs
                // <callback_base>/cb/v5/<nonce>/<proof>. D-13-04 derives the seed
                // from the nonce; D-13-05 emits one seed JSON-LD block per active
                // T5 payload (self-identifying via the "nonce" field — Q1
                // resolution).
                let nonce = nonce::generate_nonce();
                let callback_url_proof_base =
                    format!("{}/cb/v5/{}", config.callback_base_url, nonce);
                let rendered = payload
                    .instruction
                    .replace("{callback_url_proof_base}", &callback_url_proof_base);

                // generate_nonce() always produces a 16-char hex string, so
                // derive_seed returns Some(_) — `expect` documents this generator
                // invariant (RESEARCH Risk 1: invariant-break panics at build
                // time, preferable to silent wrong-seed emission).
                let seed = nonce::derive_seed(&nonce)
                    .expect("generator-produced nonce is well-formed 16-char hex");
                t5_seed_scripts.push(format!(
                    r#"<script type="application/ld+json">{{"verification_seed":{},"nonce":"{}"}}</script>"#,
                    seed, nonce
                ));

                store::insert_nonce(conn, &nonce, tier_num, &payload.id, &embedding_loc)
                    .with_context(|| {
                        format!("Failed to insert nonce for payload {}", payload.id)
                    })?;

                nonce_mappings.push(NonceMapping {
                    nonce: nonce.clone(),
                    tier: payload.tier,
                    payload_id: payload.id.clone(),
                    embedding_location: payload.embedding_location,
                    callback_url: callback_url_proof_base,
                });

                rendered_payloads.push(RenderedPayload {
                    embedding_location: embedding_loc,
                    rendered_instruction: rendered,
                });
            }
        }
    }

    // Phase 13 Plan 03 (D-13-05): join accumulated T5 seed JSON-LD blocks. When
    // no T5 payloads are active this is the empty string — the template's
    // `{{ seed_scripts_json | safe }}` renders to zero bytes, so no stray
    // <script> tag appears in the HTML.
    let seed_scripts_json = t5_seed_scripts.join("\n");

    // Render templates
    let html = render_template(
        "index.html.jinja",
        context! {
            page_title => &config.page_title,
            payloads => &rendered_payloads,
            seed_scripts_json => &seed_scripts_json,
        },
    )
    .context("Failed to render index.html")?;

    let robots =
        render_template("robots.txt.jinja", context! {}).context("Failed to render robots.txt")?;

    let ai_txt = render_template(
        "ai.txt.jinja",
        context! {
            page_title => &config.page_title,
            callback_base_url => &config.callback_base_url,
        },
    )
    .context("Failed to render ai.txt")?;

    let security_txt = render_template("security.txt.jinja", context! {})
        .context("Failed to render security.txt")?;

    let callback_map_json = serde_json::to_string_pretty(&nonce_mappings)
        .context("Failed to serialize callback-map.json")?;

    // Write output files
    let output_dir = project_path.join("output");
    std::fs::create_dir_all(&output_dir).context("Failed to create output/ directory")?;

    let well_known_dir = output_dir.join(".well-known");
    std::fs::create_dir_all(&well_known_dir)
        .context("Failed to create output/.well-known/ directory")?;

    std::fs::write(output_dir.join("index.html"), html)
        .context("Failed to write output/index.html")?;
    std::fs::write(output_dir.join("robots.txt"), robots)
        .context("Failed to write output/robots.txt")?;
    std::fs::write(output_dir.join("ai.txt"), ai_txt).context("Failed to write output/ai.txt")?;
    std::fs::write(output_dir.join("callback-map.json"), callback_map_json)
        .context("Failed to write output/callback-map.json")?;
    std::fs::write(well_known_dir.join("security.txt"), security_txt)
        .context("Failed to write output/.well-known/security.txt")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::store;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory DB must open");
        store::run_migrations(&conn).expect("migrations must succeed");
        conn
    }

    #[test]
    fn test_render_template_index() {
        let payloads: Vec<RenderedPayload> = vec![];
        let result = render_template(
            "index.html.jinja",
            context! {
                page_title => "Test Page",
                payloads => &payloads,
            },
        );
        assert!(result.is_ok(), "Template render must succeed: {:?}", result);
        let html = result.unwrap();
        assert!(html.contains("SECURITY RESEARCH CANARY"));
        assert!(html.contains("warning-banner"));
        assert!(html.contains("Notice:"));
        assert!(html.contains("honeyprompt.dev"));
    }

    #[test]
    fn test_render_template_robots() {
        let result = render_template("robots.txt.jinja", context! {});
        assert!(result.is_ok());
        let txt = result.unwrap();
        assert!(txt.contains("GPTBot"));
        assert!(txt.contains("ClaudeBot"));
    }

    #[test]
    fn test_render_template_ai_txt() {
        let result = render_template(
            "ai.txt.jinja",
            context! {
                page_title => "Test Page",
                callback_base_url => "http://localhost:8080",
            },
        );
        assert!(result.is_ok());
        let txt = result.unwrap();
        assert!(txt.contains("Disallow: Scraping"));
    }

    #[test]
    fn test_generate_writes_output_files() {
        let dir = tempfile::tempdir().expect("tempdir must create");
        let config = Config::default();
        let conn = test_conn();

        generate(&config, &conn, dir.path()).expect("generate must succeed");

        assert!(dir.path().join("output/index.html").exists());
        assert!(dir.path().join("output/robots.txt").exists());
        assert!(dir.path().join("output/ai.txt").exists());
        assert!(dir.path().join("output/callback-map.json").exists());
        assert!(dir.path().join("output/.well-known/security.txt").exists());
    }

    // ---- Phase 13 Plan 03: T4/T5 generator rendering + seed JSON-LD emission ----

    fn test_config_with_tiers(tiers: Vec<u8>) -> Config {
        Config {
            tiers,
            ..Config::default()
        }
    }

    #[test]
    fn test_tier4_renders_with_b64_base() {
        let dir = tempfile::tempdir().expect("tempdir must create");
        let config = test_config_with_tiers(vec![4]);
        let conn = test_conn();

        generate(&config, &conn, dir.path()).expect("generate must succeed");

        let html = std::fs::read_to_string(dir.path().join("output/index.html"))
            .expect("index.html must exist");
        assert!(
            html.contains("/cb/v4/"),
            "index.html must contain /cb/v4/ route"
        );
        assert!(
            !html.contains("{callback_url_b64_base}"),
            "placeholder must be substituted, not left raw"
        );
    }

    #[test]
    fn test_tier5_renders_with_proof_base() {
        let dir = tempfile::tempdir().expect("tempdir must create");
        let config = test_config_with_tiers(vec![5]);
        let conn = test_conn();

        generate(&config, &conn, dir.path()).expect("generate must succeed");

        let html = std::fs::read_to_string(dir.path().join("output/index.html"))
            .expect("index.html must exist");
        assert!(
            html.contains("/cb/v5/"),
            "index.html must contain /cb/v5/ route"
        );
        assert!(
            !html.contains("{callback_url_proof_base}"),
            "placeholder must be substituted, not left raw"
        );
    }

    #[test]
    fn test_t5_seed_json_ld_emission() {
        let dir = tempfile::tempdir().expect("tempdir must create");
        let config = test_config_with_tiers(vec![5]);
        let conn = test_conn();

        generate(&config, &conn, dir.path()).expect("generate must succeed");

        let html = std::fs::read_to_string(dir.path().join("output/index.html"))
            .expect("index.html must exist");

        // D-13-05: one block per active T5 payload. tier5.toml ships 3 payloads.
        let block_count = html.matches("\"verification_seed\"").count();
        let t5_payload_count = crate::catalog::load_for_tiers(&[5])
            .expect("tier5 catalog must load")
            .len();
        assert_eq!(
            block_count, t5_payload_count,
            "must emit one seed JSON-LD block per active T5 payload (D-13-05)"
        );

        // Each emitted seed must equal nonce::derive_seed(nonce). Load nonce_mappings
        // from callback-map.json written by generate().
        let map_json = std::fs::read_to_string(dir.path().join("output/callback-map.json"))
            .expect("callback-map.json must exist");
        let mappings: Vec<serde_json::Value> =
            serde_json::from_str(&map_json).expect("callback-map.json must be valid JSON");
        let t5_nonces: Vec<String> = mappings
            .iter()
            .filter(|m| {
                m.get("tier").and_then(|t| t.as_str()) == Some("Tier5")
                    || m.get("tier").and_then(|t| t.as_u64()) == Some(5)
            })
            .filter_map(|m| m.get("nonce").and_then(|n| n.as_str()).map(String::from))
            .collect();
        assert_eq!(
            t5_nonces.len(),
            t5_payload_count,
            "callback-map.json must contain T5 nonces"
        );

        for n in &t5_nonces {
            let expected_seed = crate::nonce::derive_seed(n).expect("valid nonce");
            let fragment = format!("\"verification_seed\":{}", expected_seed);
            assert!(
                html.contains(&fragment),
                "HTML must contain verification_seed={} for nonce {}",
                expected_seed,
                n
            );
            let nonce_fragment = format!("\"nonce\":\"{}\"", n);
            assert!(
                html.contains(&nonce_fragment),
                "HTML must contain nonce={} in a JSON-LD seed block",
                n
            );
        }
    }

    #[test]
    fn test_no_seed_block_when_t5_filtered_out() {
        let dir = tempfile::tempdir().expect("tempdir must create");
        let config = test_config_with_tiers(vec![1, 2, 3]);
        let conn = test_conn();

        generate(&config, &conn, dir.path()).expect("generate must succeed");

        let html = std::fs::read_to_string(dir.path().join("output/index.html"))
            .expect("index.html must exist");
        assert!(
            !html.contains("verification_seed"),
            "no seed JSON-LD when no T5 payloads active (D-13-05 conditional)"
        );
    }
}
