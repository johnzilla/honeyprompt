use anyhow::Context;
use minijinja::{context, Environment};
use rust_embed::RustEmbed;
use rusqlite::Connection;
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
    let file = Templates::get(name)
        .with_context(|| format!("Embedded template not found: {}", name))?;
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
    let payloads = catalog::load_for_tiers(&config.tiers)
        .context("Failed to load payload catalog")?;

    let mut rendered_payloads: Vec<RenderedPayload> = Vec::new();
    let mut nonce_mappings: Vec<NonceMapping> = Vec::new();

    for payload in &payloads {
        let embedding_loc = payload.embedding_location.to_string();
        let tier_num: u8 = payload.tier.into();

        match payload.tier {
            Tier::Tier1 => {
                let nonce = nonce::generate_nonce();
                let callback_url = format!("{}/cb/{}", config.callback_base_url, nonce);
                let rendered = payload.instruction.replace("{callback_url}", &callback_url);

                store::insert_nonce(conn, &nonce, tier_num, &payload.id, &embedding_loc)
                    .with_context(|| format!("Failed to insert nonce for payload {}", payload.id))?;

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
                let callback_url_a = format!("{}/cb/{}", config.callback_base_url, nonce_a);
                let callback_url_b = format!("{}/cb/{}", config.callback_base_url, nonce_b);

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
                let callback_url_base = format!("{}/cb/{}", config.callback_base_url, nonce);
                let rendered = payload
                    .instruction
                    .replace("{callback_url_base}", &callback_url_base);

                store::insert_nonce(conn, &nonce, tier_num, &payload.id, &embedding_loc)
                    .with_context(|| format!("Failed to insert nonce for payload {}", payload.id))?;

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
        }
    }

    // Render templates
    let html = render_template(
        "index.html.jinja",
        context! {
            page_title => &config.page_title,
            payloads => &rendered_payloads,
        },
    )
    .context("Failed to render index.html")?;

    let robots = render_template("robots.txt.jinja", context! {})
        .context("Failed to render robots.txt")?;

    let ai_txt = render_template(
        "ai.txt.jinja",
        context! {
            page_title => &config.page_title,
            callback_base_url => &config.callback_base_url,
        },
    )
    .context("Failed to render ai.txt")?;

    let callback_map_json = serde_json::to_string_pretty(&nonce_mappings)
        .context("Failed to serialize callback-map.json")?;

    // Write output files
    let output_dir = project_path.join("output");
    std::fs::create_dir_all(&output_dir).context("Failed to create output/ directory")?;

    std::fs::write(output_dir.join("index.html"), html)
        .context("Failed to write output/index.html")?;
    std::fs::write(output_dir.join("robots.txt"), robots)
        .context("Failed to write output/robots.txt")?;
    std::fs::write(output_dir.join("ai.txt"), ai_txt)
        .context("Failed to write output/ai.txt")?;
    std::fs::write(output_dir.join("callback-map.json"), callback_map_json)
        .context("Failed to write output/callback-map.json")?;

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
    }
}
