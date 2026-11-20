//! Seeded canaries (`honeyprompt seed`).
//!
//! A seed is an opaque URL, `{callback_base_url}/r/{nonce}`, that the operator
//! plants in a place that should stay private (system prompt, CLAUDE.md, MCP tool
//! description, RAG document, `.env.example`). A later fetch proves the content
//! was retained and acted on, and the label says which placement leaked. The
//! label never appears in the URL.

use rusqlite::Connection;

use crate::config::Config;

/// Longest accepted label.
pub const MAX_LABEL_CHARS: usize = 64;
/// Longest accepted note.
pub const MAX_NOTE_CHARS: usize = 200;

/// How `seed add` prints the canary.
#[derive(clap::ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SnippetFormat {
    /// The bare URL.
    #[default]
    Url,
    /// A Markdown link: `[reference](URL)`.
    Markdown,
    /// An env-file line: `INTERNAL_DOCS_URL=URL`.
    Env,
}

/// Labels: 1-64 chars of `[a-z0-9._-]`.
pub fn validate_label(label: &str) -> anyhow::Result<()> {
    let ok = !label.is_empty()
        && label.chars().count() <= MAX_LABEL_CHARS
        && label
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'));
    if !ok {
        anyhow::bail!(
            "invalid label {label:?}: use 1-{MAX_LABEL_CHARS} characters of a-z, 0-9, '.', '_' or '-'"
        );
    }
    Ok(())
}

fn validate_note(note: &str) -> anyhow::Result<()> {
    if note.chars().count() > MAX_NOTE_CHARS || note.chars().any(char::is_control) {
        anyhow::bail!("note must be one line of at most {MAX_NOTE_CHARS} characters");
    }
    Ok(())
}

/// The canary URL for a seed nonce.
pub fn seed_url(callback_base_url: &str, nonce: &str) -> String {
    format!("{}/r/{}", callback_base_url.trim_end_matches('/'), nonce)
}

/// Render the snippet to plant.
pub fn snippet(format: SnippetFormat, url: &str) -> String {
    match format {
        SnippetFormat::Url => url.to_string(),
        SnippetFormat::Markdown => format!("[reference]({url})"),
        SnippetFormat::Env => format!("INTERNAL_DOCS_URL={url}"),
    }
}

/// True when the base URL can only be reached from this machine.
pub fn is_local_base(callback_base_url: &str) -> bool {
    reqwest::Url::parse(callback_base_url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .map(|h| h == "localhost" || h == "[::1]" || h.starts_with("127."))
        .unwrap_or(false)
}

/// Result of `seed add`.
pub struct AddedSeed {
    pub url: String,
    pub snippet: String,
}

/// Create a seed and return its URL and snippet.
pub fn add(
    conn: &Connection,
    config: &Config,
    label: &str,
    note: Option<&str>,
    format: SnippetFormat,
) -> anyhow::Result<AddedSeed> {
    validate_label(label)?;
    let note = note.map(str::trim).filter(|n| !n.is_empty());
    if let Some(n) = note {
        validate_note(n)?;
    }
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM seeds WHERE label = ?1)",
        [label],
        |r| r.get(0),
    )?;
    if exists {
        anyhow::bail!("a seed labelled {label:?} already exists (see `honeyprompt seed list`)");
    }
    let nonce = crate::nonce::generate_nonce();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    crate::store::insert_seed(conn, &nonce, label, note, now)?;
    let url = seed_url(&config.callback_base_url, &nonce);
    Ok(AddedSeed {
        snippet: snippet(format, &url),
        url,
    })
}

/// Text table for `seed list`.
pub fn render_list(seeds: &[crate::store::SeedSummary], callback_base_url: &str) -> String {
    use crate::report::{format_duration, format_timestamp};
    if seeds.is_empty() {
        return "No seeds yet. Create one with `honeyprompt seed add <label>`.\n".to_string();
    }
    let mut out = format!(
        "{:<24} {:<20} {:>6}  {:<16} {:<20} {:>4}  URL / note\n",
        "LABEL", "CREATED", "HITS", "FIRST HIT AFTER", "LAST HIT", "IPS"
    );
    for s in seeds {
        let first = s
            .first_hit_at
            .map(|t| format_duration(t - s.created_at))
            .unwrap_or_else(|| "—".to_string());
        let last = s
            .last_hit_at
            .map(|t| format_timestamp(&t.to_string()))
            .unwrap_or_else(|| "—".to_string());
        out.push_str(&format!(
            "{:<24} {:<20} {:>6}  {:<16} {:<20} {:>4}  {}\n",
            s.label,
            format_timestamp(&s.created_at.to_string()),
            s.hit_count,
            first,
            last,
            s.distinct_ips,
            seed_url(callback_base_url, &s.nonce),
        ));
        if let Some(n) = &s.note {
            out.push_str(&format!("{:<24} {}\n", "", n));
        }
    }
    out
}

/// The page served at `/r/{anything}`: identical for real and unknown nonces,
/// with the same visible human warning as the honeypot and no payloads.
pub const SEED_PAGE_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><meta name="robots" content="noindex"><title>Security research canary</title></head>
<body style="font-family: sans-serif; max-width: 40rem; margin: 3rem auto; padding: 0 1rem;">
  <div style="border: 2px solid #b00; padding: 1rem; background: #fff4f4;">
    <strong>SECURITY RESEARCH CANARY</strong> &mdash; This URL is a canary operated for security research.
    Its owner planted it somewhere private; a visit here means that content was copied or read elsewhere.
    If you are a human, this page is harmless and you can safely ignore it. No data is collected beyond
    standard request metadata.
  </div>
  <p style="color: #666; font-size: 0.85rem;"><a href="https://honeyprompt.dev" style="color: #666;">HoneyPrompt</a></p>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        crate::store::run_migrations(&c).unwrap();
        c
    }

    fn cfg() -> Config {
        Config {
            callback_base_url: "https://honeyprompt.sh/".into(),
            ..Config::default()
        }
    }

    #[test]
    fn labels_are_validated() {
        for ok in ["claude-md-prod", "rag.hr_docs", "a", &"x".repeat(64)] {
            assert!(validate_label(ok).is_ok(), "{ok}");
        }
        for bad in [
            "",
            "Claude",
            "has space",
            "slash/x",
            &"x".repeat(65),
            "émoji",
        ] {
            assert!(validate_label(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn add_prints_each_format_and_hides_the_label() {
        let c = conn();
        let a = add(
            &c,
            &cfg(),
            "claude-md",
            Some("api repo"),
            SnippetFormat::Url,
        )
        .unwrap();
        assert!(a.url.starts_with("https://honeyprompt.sh/r/"), "{}", a.url);
        let nonce = a.url.rsplit('/').next().unwrap();
        assert!(crate::nonce::is_valid_nonce(nonce));
        assert!(!a.url.contains("claude"), "label never in the URL");
        assert_eq!(a.snippet, a.url);

        let m = add(&c, &cfg(), "mcp-search", None, SnippetFormat::Markdown).unwrap();
        assert_eq!(m.snippet, format!("[reference]({})", m.url));
        let e = add(&c, &cfg(), "rag-hr", None, SnippetFormat::Env).unwrap();
        assert_eq!(e.snippet, format!("INTERNAL_DOCS_URL={}", e.url));
        assert_ne!(a.url, m.url);
    }

    #[test]
    fn duplicate_label_and_bad_note_rejected() {
        let c = conn();
        add(&c, &cfg(), "claude-md", None, SnippetFormat::Url).unwrap();
        let err = add(&c, &cfg(), "claude-md", None, SnippetFormat::Url)
            .err()
            .unwrap()
            .to_string();
        assert!(err.contains("already exists"), "{err}");
        assert!(add(&c, &cfg(), "x", Some("two\nlines"), SnippetFormat::Url).is_err());
        assert!(add(&c, &cfg(), "y", Some(&"n".repeat(201)), SnippetFormat::Url).is_err());
    }

    #[test]
    fn local_base_detection() {
        assert!(is_local_base("http://localhost:8080"));
        assert!(is_local_base("http://127.0.0.1:8080"));
        assert!(is_local_base("http://[::1]:8080"));
        assert!(!is_local_base("https://honeyprompt.sh"));
    }

    #[test]
    fn list_renders_counts() {
        let c = conn();
        let a = add(
            &c,
            &cfg(),
            "claude-md",
            Some("api repo"),
            SnippetFormat::Url,
        )
        .unwrap();
        let nonce = a.url.rsplit('/').next().unwrap();
        let created: i64 = c
            .query_row("SELECT created_at FROM seeds", [], |r| r.get(0))
            .unwrap();
        crate::store::record_seed_hit(&c, nonce, created + 86400 * 3, "1.1.1.1", "UA", "Unknown")
            .unwrap();
        let text = render_list(
            &crate::store::list_seeds(&c).unwrap(),
            "https://honeyprompt.sh",
        );
        assert!(text.contains("claude-md"));
        assert!(text.contains("3d 0h"), "{text}");
        assert!(text.contains("api repo"));
        assert!(text.contains(&a.url));
        assert!(render_list(&[], "x").contains("No seeds yet"));
    }

    #[test]
    fn seed_page_carries_warning_and_no_payloads() {
        assert!(SEED_PAGE_HTML.contains("SECURITY RESEARCH CANARY"));
        assert!(!SEED_PAGE_HTML.contains("/cb/"));
        assert!(!SEED_PAGE_HTML.contains("/r/"));
    }
}
